use crate::{Args, OutputFormat};
use anyhow::Result;
use hyper::body;
use hyper::{Body, Response, StatusCode};
use indicatif::ProgressBar;
use serde_json::{json, Value};
use std::fmt::Display;
use std::io::prelude::*;

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

    pub(crate) async fn output<'a>(self, config: &Args, writer: &mut Writer<'_>) -> Result<()> {
        let out = self.create_output(config).await?;
        match writer {
            Writer::File(wr) => Self::output_file(out, wr).await,
            Writer::Bar(bar) => Self::output_bar(out, bar),
        }
    }

    async fn create_output(self, config: &Args) -> Result<Out> {
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
