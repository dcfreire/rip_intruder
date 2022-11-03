//! Intruder
use crate::intruder::request_template::{ReqTemplateFile, RequestTemplate};
use crate::intruder::output::{Writer, OutLine};

use crate::{Args, HitType};

use anyhow::{anyhow, Context, Result};

use futures::{stream, StreamExt};

use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Response};

use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;

use indicatif::{ProgressBar, ProgressStyle};

/// Struct for detecting hits
struct Hit {
    hit_type: HitType,
}

impl Hit {
    fn new(hit_type: HitType) -> Self {
        Self { hit_type }
    }

    fn is_hit(&self, resp: &Response<Body>) -> bool {
        match self.hit_type {
            HitType::Ok => Hit::success_hit(&resp),
            HitType::All => Hit::all_hit(),
        }
    }

    fn all_hit() -> bool {
        true
    }

    fn success_hit(resp: &Response<Body>) -> bool {
        if resp.status() == 200 {
            true
        } else {
            false
        }
    }
}

/// Struct for managing the bruteforcing process
///
/// The Intruder struct stores the [RequestTemplate] for creating new requests, the client for sending said
/// requests and any configuration parameters relevant to the bruteforcing process.
pub(crate) struct Intruder {
    client: Client<HttpConnector>,
    req_templ: RequestTemplate,
    writer: Option<Box<dyn Write>>,
    config: Args,
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
            writer: match &config.of {
                Some(path) => Some(Box::new(
                    OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(path)?,
                )),
                None => Some(Box::new(std::io::stdout())),
            },
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

    /// Start the bruteforcing process
    ///
    /// This function will take the pass_file, create a separate request for each
    /// line in it. If some of the requests don't go through it will retry with a lower
    /// concurrency (TODO: Add an option to enable/disable this, and options for the number
    /// of retries).
    pub(crate) async fn bruteforce(&mut self) -> Result<()> {
        // Take the writer, having ownership of it is required since we borrow self for the futures
        let temp_writer = self.writer.take().unwrap();

        // Create the hit detector
        let hit_d = Hit::new(self.config.hit_type);

        // Iterator for the passwords with any errors filtered out
        let payloads = BufReader::new(File::open(&self.config.pass_f)?)
            .lines()
            .filter_map(|payload| payload.ok());

        let reqs = self.get_reqs(payloads);

        let mut futures = vec![];
        for (req, payload) in reqs {
            futures.push(self.send_req(req?, payload));
        }

        // Progress bar configuration
        let bar = ProgressBar::new(futures.len() as u64);
        bar.set_style(ProgressStyle::with_template("{msg} {spinner}\n[{elapsed_precise}] {wide_bar} {pos}/{len}\nReq/sec: {per_sec}\nETA: {eta}")?);
        bar.set_message("Sending requests");

        // Some variables we'll be using
        let mut conc = self.config.concurrent_requests;
        let mut futures = stream::iter(futures).buffer_unordered(conc);
        let mut errors: Vec<String> = vec![];
        let mut hits = 0;

        let mut writer = match self.config.of {
            Some(_) => Writer::File(temp_writer),
            None => Writer::Bar(&bar),
        };
        // do while errors is not empty
        while {
            while let Some(res) = futures.next().await {
                let (resp, payload);
                match res {
                    Ok(result) => {
                        bar.inc(1);
                        (resp, payload) = result;
                    }
                    Err(err) => {
                        errors.push(err.to_string());
                        continue;
                    }
                };

                if hit_d.is_hit(&resp) {
                    hits += 1;
                    OutLine::new(resp, payload, hits)
                        .await?
                        .output(&self.config, &mut writer)
                        .await?;
                }
                if hits as isize == self.config.stop {
                    return Ok(());
                }
            }
            !errors.is_empty()
        } {
            bar.set_message(format!("Completed with {} errors", errors.len()));

            // If errors is not empty either the request is malformed or you are requesting too fast
            if bar.position() == 0 {
                return Err(anyhow!("Your requests are not getting through"));
            }

            conc = conc / 2;
            if conc == 0 {
                bar.finish();
                return Err(anyhow!("Search aborted"));
            }

            bar.set_message(format!(
                "Completed with {} errors, retrying with half ({}) the concurrency",
                errors.len(),
                conc
            ));
            bar.set_length(errors.len() as u64);
            bar.set_position(0);
            let mut new_futures = vec![];

            let reqs = self.get_reqs(errors);

            for (req, payload) in reqs {
                new_futures.push(self.send_req(req?, payload));
            }

            futures = stream::iter(new_futures).buffer_unordered(conc);
            errors = vec![];
        }
        bar.finish();
        Err(anyhow!("Password not found"))
    }
}
