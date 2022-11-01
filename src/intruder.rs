use super::request_template::RequestTemplate;
use anyhow::{anyhow, Context, Result};

use futures::{stream, StreamExt};

use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use indicatif::ProgressBar;

pub(crate) struct Intruder {
    pub(crate) client: Client<HttpConnector>,
    pub(crate) req_templ: RequestTemplate,
    concurrent_reqs: usize
}

impl Intruder {
    pub(crate) fn new(req_file: File, concurrent_reqs: usize) -> Result<Self> {
        Ok(Intruder {
            client: Client::new(),
            req_templ: RequestTemplate::from_file(req_file)?,
            concurrent_reqs
        })
    }

    pub(crate) async fn send_req(
        &self,
        req: (Request<Body>, String),
    ) -> Result<(Response<Body>, String)> {
        let resp = match self.client.request(req.0).await.context(req.1.clone()) {
            Ok(out) => out,
            Err(_) => return Err(anyhow!(req.1)),
        };
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

        let bar = ProgressBar::new(futures.len() as u64);

        let mut conc = self.concurrent_reqs;
        let mut futures = stream::iter(futures).buffer_unordered(conc);
        let mut errors: Vec<String> = vec![];

        // do while errors is not empty
        while {
            while let Some(res) = futures.next().await {
                let (resp, pw);
                match res {
                    Ok(result) => {
                        bar.inc(1);
                        (resp, pw) = result;
                    }
                    Err(err) => {
                        errors.push(err.to_string());
                        continue;
                        //
                    }
                };
                if resp.status() == 200 {
                    return Ok(pw);
                }
            }
            !errors.is_empty()
        } {
            // If errors is not empty either the request is malformed or you are requesting too fast
            if bar.position() == 0 {
                return Err(anyhow!("Your requests are not getting through"));
            }
            let mut new_futures = vec![];
            let reqs = errors
                .into_iter()
                .map(|pw| self.req_templ.replace_then_request(pw))
                .filter_map(|req| req.ok());
            for req in reqs {
                new_futures.push(self.send_req(req))
            }
            conc = conc/2;
            if conc == 0 {
                break;
            }
            futures = stream::iter(new_futures).buffer_unordered(conc);
            errors = vec![];
        }

        Err(anyhow!("Password not found"))
    }
}
