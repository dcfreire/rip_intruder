//! Intruder
use anyhow::{anyhow, Context, Result};

use futures::{stream, Stream, StreamExt};

use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use crate::request_template::{AttackType, ReqTemplateFile, RequestTemplate};

pub struct IntruderConfig {
    pub req_f: PathBuf,
    pub pass_f: PathBuf,
    pub pattern: String,
    pub concurrent_requests: usize,
    pub attack_type: AttackType,
}

/// Struct for managing the bruteforcing process
///
/// The Intruder struct stores the [RequestTemplate] for creating new requests, the client for sending said
/// requests and any configuration parameters relevant to the bruteforcing process.
pub struct Intruder {
    client: Client<HttpConnector>,
    req_templ: RequestTemplate,
    pub config: IntruderConfig,
}

impl Intruder {
    /// Create new Intruder
    pub fn new(config: IntruderConfig) -> Result<Self> {
        Ok(Intruder {
            client: Client::new(),
            req_templ: RequestTemplate::try_from(ReqTemplateFile::new(
                File::open(&config.req_f)?,
                &config.pattern,
                config.attack_type,
            )?)?,
            config,
        })
    }

    /// Send a single request, returns a tuple containg the response and the payload
    async fn send_reqs(
        &self,
        reqs: Vec<Request<Body>>,
        payload: String,
    ) -> Result<(Vec<Response<Body>>, String)> {
        let mut resps = vec![];
        for req in reqs {
            match self.client.request(req).await.context(payload.clone()) {
                Ok(out) => resps.push(out),
                Err(_) => return Err(anyhow!(payload)),
            };
        }
        Ok((resps, payload))
    }

    fn get_reqs<T>(
        &self,
        payloads: T,
    ) -> impl Iterator<Item = (Result<Vec<Request<Body>>>, String)> + '_
    where
        T: IntoIterator<Item = String> + 'static,
    {
        payloads
            .into_iter()
            .map(|payload| (self.req_templ.replace_then_request(&payload), payload))
            .filter(|req| req.0.is_ok())
    }

    pub fn get_payload_buffer(&self) -> impl Iterator<Item = String> {
        // Iterator for the passwords with any errors filtered out
        BufReader::new(File::open(&self.config.pass_f).unwrap())
            .lines()
            .filter_map(|payload| payload.ok())
    }

    /// Creates a stream for asynchronously iterating over the responses for the provided payloads
    pub async fn bruteforce<T>(
        &self,
        payloads: T,
    ) -> Result<impl Stream<Item = Result<(Vec<Response<Body>>, String)>> + '_>
    where
        T: IntoIterator<Item = String> + 'static,
    {
        let futures = self
            .get_reqs(payloads)
            .map(|(req, payload)| self.send_reqs(req.unwrap(), payload));
        Ok(stream::iter(futures).buffer_unordered(self.config.concurrent_requests))
    }
}
