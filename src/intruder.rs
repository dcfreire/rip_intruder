use super::request_template::RequestTemplate;
use anyhow::{anyhow, Result};

use futures::stream::FuturesUnordered;
use futures::{stream, StreamExt};

use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub(crate) struct Intruder {
    pub(crate) client: Client<HttpConnector>,
    pub(crate) req_templ: RequestTemplate,
}

impl Intruder {
    pub(crate) fn new(req_file: File) -> Result<Self> {
        Ok(Intruder {
            client: Client::new(),
            req_templ: RequestTemplate::from_file(req_file)?,
        })
    }

    pub(crate) async fn send_req(
        &self,
        req: (Request<Body>, String),
    ) -> Result<(Response<Body>, String)> {
        let resp = self.client.request(req.0).await?;
        Ok((resp, req.1))
    }

    pub(crate) async fn bruteforce(&self, pass_file: File) -> Result<String> {
        let passwords = BufReader::new(pass_file).lines();

        let reqs = passwords
            .filter_map(|pw| pw.ok())
            .map(|pw| self.req_templ.replace_then_request(pw))
            .filter_map(|req| req.ok());

        let mut futures = vec![];
        for req in reqs {
            futures.push(self.send_req(req))
        }
        let mut futures = stream::iter(futures).buffer_unordered(5);
        while let Some(resp) = futures.next().await {
            let (resp, pw) = resp?;
            if resp.status() == 200 {
                return Ok(pw);
            }
        }

        Err(anyhow!("Password not found"))
    }
}
