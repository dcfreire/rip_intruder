//! Intruder
//! (TODO: MAKE CONFIG TYPE FOR INTRUDER)

use crate::request_template::ReqTemplateFile;

use super::request_template::RequestTemplate;
use anyhow::{anyhow, Context, Result};

use futures::{stream, StreamExt};

use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use indicatif::{ProgressBar, ProgressStyle};


/// Object for managing the bruteforcing process
///
/// The Intruder object stores the [RequestTemplate] for creating new requests, the client for sending said
/// requests and any configuration parameters relevant to the bruteforcing process.
pub(crate) struct Intruder {
    pub(crate) client: Client<HttpConnector>,
    pub(crate) req_templ: RequestTemplate,
    concurrent_reqs: usize
}

impl Intruder {
    /// Create new Intruder
    pub(crate) fn new(req_file: File, concurrent_reqs: usize, pattern: String) -> Result<Self> {
        Ok(Intruder {
            client: Client::new(),
            req_templ: RequestTemplate::try_from(ReqTemplateFile::new(req_file, pattern)?)?,
            concurrent_reqs
        })
    }

    /// Send a single request composed of (Request, Password)
    pub(crate) async fn send_req(
        &self,
        req: Request<Body>,
        pw: String
    ) -> Result<(Response<Body>, String)> {
        let resp = match self.client.request(req).await.context(pw.clone()) {
            Ok(out) => out,
            Err(_) => return Err(anyhow!(pw)),
        };
        Ok((resp, pw))
    }

    /// Start the bruteforcing process
    ///
    /// This function will take the pass_file, create a separate request for each
    /// line in it. If some of the requests don't go through it will retry with a lower
    /// concurrency (TODO: Add an option to enable/disable this, and options for the number
    /// of retries).
    pub(crate) async fn bruteforce(&self, pass_file: File) -> Result<String> {
        let passwords = BufReader::new(pass_file).lines();

        let reqs = passwords
            .filter_map(|pw| pw.ok())
            .map(|pw| (self.req_templ.replace_then_request(&pw), pw))
            .filter(|req| req.0.is_ok());

        let mut futures = vec![];
        for (req, pw) in reqs {
            futures.push(self.send_req(req?, pw));
        }

        let bar = ProgressBar::new(futures.len() as u64);
        bar.set_style(ProgressStyle::with_template("{msg} {spinner}\n[{elapsed_precise}] {wide_bar} {pos}/{len}\nReq/sec: {per_sec}\nETA: {eta}")?);

        let mut conc = self.concurrent_reqs;
        let mut futures = stream::iter(futures).buffer_unordered(conc);
        let mut errors: Vec<String> = vec![];
        bar.set_message("Sending requests");
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
            bar.set_message(format!("Completed with {} errors", errors.len()));

            // If errors is not empty either the request is malformed or you are requesting too fast
            if bar.position() == 0 {
                return Err(anyhow!("Your requests are not getting through"));
            }

            conc = conc/2;
            if conc == 0 {
                bar.finish();
                return Err(anyhow!("Search aborted"));
            }

            bar.set_message(format!("Completed with {} errors, retrying with half ({}) the concurrency", errors.len(), conc));
            bar.set_length(errors.len() as u64);
            bar.set_position(0);
            let mut new_futures = vec![];

            let reqs = errors
                .into_iter()
                .map(|pw| (self.req_templ.replace_then_request(&pw), pw))
                .filter(|req| req.0.is_ok());

            for (req, pw) in reqs {
                new_futures.push(self.send_req(req?, pw));
            }

            futures = stream::iter(new_futures).buffer_unordered(conc);
            errors = vec![];
        }
        bar.finish();
        Err(anyhow!("Password not found"))
    }
}
