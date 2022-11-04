

use anyhow::Result;


use hyper::body;
use hyper::{Body, Response, StatusCode};

use indicatif::{ProgressBar, ProgressStyle};

use intruder::intruder::Intruder;
use serde_json::{json, Value};

use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;
use futures::StreamExt;

use crate::arg_types::{OutputFormat, HitType};

pub struct CliConfig {
    pub out_format: OutputFormat,
    pub out_file: Option<PathBuf>,
    pub hit_type: HitType,
    pub stop: isize
}

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

/// Represents one line of the output
pub(crate) struct OutLine {
    status: StatusCode,
    body: Body,
    payload: String,
    idx: usize,
}

/// Represents where the output is going to be written to
pub(crate) enum Writer<'a> {
    File(Box<dyn Write>),
    Bar(&'a ProgressBar),
}

enum Out {
    Json(Value),
    Msg(String),
}

impl Display for Out {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Out::Json(json) => json.fmt(f),
            Out::Msg(msg) => msg.fmt(f),
        }
    }
}

impl OutLine {
    pub(crate) async fn new(resp: Response<Body>, payload: String, idx: usize) -> Result<Self> {
        Ok(Self {
            status: resp.status(),
            body: resp.into_body(),
            payload,
            idx,
        })
    }

    fn output_bar(out: Out, bar: &ProgressBar) -> Result<()> {
        Ok(bar.println(out.to_string()))
    }

    /// Writes output, consuming self in the process.
    pub(crate) async fn output<'a>(self, config: &CliConfig, writer: &mut Writer<'_>) -> Result<()> {
        let out = self.create_output(config).await?;
        match writer {
            Writer::File(wr) => Self::output_file(out, wr).await,
            Writer::Bar(bar) => Self::output_bar(out, bar),
        }
    }

    async fn create_output(self, config: &CliConfig) -> Result<Out> {
        match config.out_format {
            OutputFormat::Csv => Ok(Out::Msg(format!(
                "{:}, {:}, {:}",
                self.idx, self.payload, self.status
            ))),
            OutputFormat::Jsonl => {
                let body = String::from_utf8(body::to_bytes(self.body).await?.to_vec())?;
                let out = json!({
                    "Payload": self.payload,
                    "Status": self.status.as_u16(),
                    "Body": body
                    }
                );
                Ok(Out::Json(out))
            }
        }
    }

    async fn output_file(out: Out, writer: &mut Box<dyn Write>) -> Result<()> {
        writeln!(writer, "{:}", out)?;
        Ok(())
    }
}

pub struct Cli {
    hit_d: Hit,
    bar: ProgressBar,
    config: CliConfig
}

impl Cli {
    pub fn new(config: CliConfig) -> Self {
        let bar = ProgressBar::new(0);
        bar.set_style(ProgressStyle::with_template("{msg} {spinner}\n[{elapsed_precise}] {wide_bar} {pos}/{len}\nReq/sec: {per_sec}\nETA: {eta}").unwrap());

        Self {
            bar,
            hit_d: Hit::new(config.hit_type),
            config
        }
    }

    pub async fn run(&self, intr: Intruder) -> Result<Vec<String>> {
        let mut writer = match &self.config.out_file {
            Some(path) => Writer::File(Box::new(
                OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)?,
            )),
            None => Writer::Bar(&self.bar),
        };
        let payloads: Vec<String> = intr.get_payload_buffer().collect();
        self.bar.set_length(payloads.len() as u64);

        let mut responses = intr.bruteforce(payloads).await?;
        let mut hits = 0;
        let mut errors = vec![];

        while let Some(resp_pay) = responses.next().await {
            let (resp, payload);
            match resp_pay {
                Ok(result) => {
                    self.bar.inc(1);
                    (resp, payload) = result;
                }
                Err(payload) => {
                    errors.push(payload.to_string());
                    continue;
                }
            }

            if self.hit_d.is_hit(&resp) {
                hits += 1;
                OutLine::new(resp, payload, hits)
                    .await?
                    .output(&self.config, &mut writer)
                    .await?;
            }
            if hits as isize == self.config.stop {
                break;
            }
        }
        Ok(errors)
    }
}
