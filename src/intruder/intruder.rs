//! Intruder
use crate::intruder::request_template::{ReqTemplateFile, RequestTemplate};
use crate::Args;

use anyhow::{anyhow, Context, Result};

use futures::{stream, Stream, StreamExt};

use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

/// Struct for managing the bruteforcing process
///
/// The Intruder struct stores the [RequestTemplate] for creating new requests, the client for sending said
/// requests and any configuration parameters relevant to the bruteforcing process.
pub struct Intruder {
    client: Client<HttpConnector>,
    req_templ: RequestTemplate,
    pub(crate) config: Args,
}

impl Intruder {
    /// Create new Intruder
    pub(crate) fn new(config: Args) -> Result<Self> {
        Ok(Intruder {
            client: Client::new(),
            req_templ: RequestTemplate::try_from(ReqTemplateFile::new(
                File::open(&config.req_f)?,
                &config.pattern,
            )?)?,
            config,
        })
    }

    /// Send a single request, returns a tuple containg the response and the payload
    async fn send_req(
        &self,
        req: Request<Body>,
        payload: String,
    ) -> Result<(Response<Body>, String)> {
        let resp = match self.client.request(req).await.context(payload.clone()) {
            Ok(out) => out,
            Err(_) => return Err(anyhow!(payload)),
        };
        Ok((resp, payload))
    }

    fn get_reqs<T>(&self, payloads: T) -> impl Iterator<Item = (Result<Request<Body>>, String)> + '_
    where
        T: IntoIterator<Item = String> + 'static,
    {
        payloads
            .into_iter()
            .map(|payload| (self.req_templ.replace_then_request(&payload), payload))
            .filter(|req| req.0.is_ok())
    }

    pub(crate) fn get_payload_buffer(&self) -> impl Iterator<Item = String> {
        // Iterator for the passwords with any errors filtered out
        BufReader::new(File::open(&self.config.pass_f).unwrap())
            .lines()
            .filter_map(|payload| payload.ok())
    }

    /// Creates a stream for asynchronously iterating over the responses for the bruteforce attempt
    pub(crate) async fn bruteforce<T>(
        &self,
        payloads: T,
    ) -> Result<impl Stream<Item = Result<(Response<Body>, String)>> + '_>
    where
        T: IntoIterator<Item = String> + 'static,
    {
        let reqs = self.get_reqs(payloads);

        let mut futures = vec![];
        for (req, payload) in reqs {
            futures.push(self.send_req(req?, payload));
        }

        Ok(stream::iter(futures).buffer_unordered(self.config.concurrent_requests))
    }
}
