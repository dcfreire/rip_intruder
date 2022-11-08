//! Request Templates
//!
//! This module houses the structures for creating new requests from a template request.
use anyhow::{anyhow, Error, Result};

use hyper::header::HeaderName;
use hyper::http::header::HeaderValue;
use hyper::http::request::Builder;
use hyper::{Body, HeaderMap, Method, Request, Uri, Version};

use itertools::Itertools;
use regex::Regex;

use std::fs::File;
use std::io::{prelude::*, BufReader};

#[derive(Copy, Clone, Debug)]
pub enum AttackType {
    Sniper,
    BatteringRam,
    Pitchfork,
    ClusterBomb
}

/// Represents the components of a request for recreating the [Request] object
///
/// This struct is useful since the [Request] object is not Clone.
/// (TODO: Add Extensions)
pub struct RequestComponents {
    head: HeaderMap<HeaderValue>,
    uri: Uri,
    version: Version,
    body: String,
    method: Method,
}

impl RequestComponents {
    /// Create a new empty [RequestComponents].
    fn new() -> Self {
        RequestComponents {
            head: HeaderMap::new(),
            uri: Uri::from_static("/"),
            version: Version::HTTP_11,
            body: "".to_string(),
            method: Method::GET,
        }
    }

    /// Insert a header into head.
    fn insert_header(&mut self, key: String, value: String) -> Result<()> {
        self.head.insert(
            <HeaderName as TryFrom<String>>::try_from(key)?,
            <HeaderValue as TryFrom<String>>::try_from(value)?,
        );
        Ok(())
    }
}

/// Stores the template
///
/// This struct stores the known RequestComponents, the pattern for identifying what components
/// are not known (marked) and have to be modified before building a new request, and the
/// marked [Part]s themselves.
pub struct RequestTemplate {
    pub req: RequestComponents,
    pub marked: Vec<Part>,
    pub pattern: Regex,
    pub attack_type: AttackType
}

/// Either a element in the header is marked, or an element in the body.
pub enum Part {
    Body(String),
    Header(String),
}

/// Trait for creating a RequestTemplate from a file. (TODO: Implement TryFrom for other types)
impl TryFrom<ReqTemplateFile> for RequestTemplate {
    type Error = Error;
    fn try_from(req_templ: ReqTemplateFile) -> Result<Self, Self::Error> {
        let pattern = req_templ.pattern;
        let req_file = req_templ.file;
        let mut lines = BufReader::new(req_file).lines();
        let request_line = lines.next().ok_or(anyhow!("File is empty"))??;
        let (method, uri, httpver) = request_line
            .split(' ')
            .next_tuple()
            .ok_or(anyhow!("Invalid Request Line"))?;

        let mut marked = Vec::new();

        let mut req = RequestComponents::new();
        req.version = match httpver {
            "HTTP/0.9" => Version::HTTP_09,
            "HTTP/1" => Version::HTTP_10,
            "HTTP/2" => Version::HTTP_2,
            "HTTP/3" => Version::HTTP_3,
            _ => Version::HTTP_11,
        };
        req.method = Method::try_from(method)?;

        for header in lines.by_ref() {
            let header = header?.trim().to_string();
            if header.is_empty() {
                break;
            }
            if pattern.is_match(&header) {
                marked.push(Part::Header(header));
                continue;
            }

            let (key, value) = header
                .split(':')
                .map(str::trim)
                .next_tuple()
                .ok_or(anyhow!("Invalid Header"))?;

            if key == "Host" {
                let uri = Uri::builder()
                    .scheme("http")
                    .authority(value)
                    .path_and_query(uri)
                    .build()?;
                req.uri = uri;
            }

            if key == "Content-Length" {
                continue;
            }
            req.insert_header(key.to_owned(), value.to_owned())?;
        }

        let body = lines.filter_map(|l| l.ok()).join("");
        if pattern.is_match(&body) {
            marked.push(Part::Body(body));
        }

        Ok(Self {
            req: req,
            marked,
            pattern: pattern,
            attack_type: req_templ.attack_type
        })
    }
}

impl TryFrom<File> for RequestTemplate {
    type Error = Error;
    fn try_from(req_file: File) -> Result<Self, Self::Error> {
        RequestTemplate::try_from(ReqTemplateFile {
            file: req_file,
            pattern: Regex::new("§§")?,
            attack_type: AttackType::BatteringRam
        })
    }
}

/// Represents a request template file
/// (TODO: Make a more generic type for inputted templates)
pub struct ReqTemplateFile {
    file: File,
    pattern: Regex,
    attack_type: AttackType,
}

impl ReqTemplateFile {
    pub fn new(file: File, pattern: &str, attack_type: AttackType) -> Result<Self> {
        Ok(Self {
            file,
            pattern: Regex::new(pattern)?,
            attack_type
        })
    }
}

impl RequestTemplate {
    fn battering_ram(&self, pw: &str, mut req: Builder) -> Result<Vec<Request<Body>>> {
        let mut body: &String = &self.req.body;
        for part in &self.marked {
            match part {
                Part::Header(header) => {
                    let (key, value) = header
                        .split(':')
                        .map(str::trim)
                        .map(|s| self.pattern.replace_all(s, pw))
                        .next_tuple()
                        .ok_or(anyhow!("Invalid Header"))?;

                    req = req.header(key.to_string(), value.to_string());
                }
                Part::Body(bd) => {
                    body = bd;
                }
            }
        }
        Ok(vec![req.body(Body::from(self.pattern.replace_all(body, pw).to_string()))?])
    }

    fn cluster_bomb(&self, pw: &str, mut req: Builder) -> Result<Vec<Request<Body>>> {
        Err(anyhow!("Not Implemented"))
    }

    fn pitchfork(&self, pw: &str, mut req: Builder) -> Result<Vec<Request<Body>>> {
        Err(anyhow!("Not Implemented"))
    }

    fn sniper(&self, pw: &str, mut req: Builder) -> Result<Vec<Request<Body>>> {
        Err(anyhow!("Not Implemented"))
    }

    /// Replace the marked Parts with pw and build a new Request from them.
    pub fn replace_then_request(&self, pw: &str) -> Result<Vec<Request<Body>>> {
        let mut req = Request::builder()
            .version(self.req.version)
            .method(self.req.method.clone())
            .uri(self.req.uri.clone());
        let headers = req.headers_mut().ok_or(anyhow!("Builder has error"))?;
        headers.clone_from(&self.req.head);
        match self.attack_type {
            AttackType::BatteringRam => self.battering_ram(pw, req),
            AttackType::ClusterBomb => self.cluster_bomb(pw, req),
            AttackType::Pitchfork => self.pitchfork(pw, req),
            AttackType::Sniper => self.sniper(pw, req)
        }
    }
}
