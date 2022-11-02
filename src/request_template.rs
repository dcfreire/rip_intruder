//! Request Templates
//!
//! This module houses the structures for creating new requests from a template request.
use anyhow::{anyhow, Result, Error};

use hyper::header::HeaderName;
use hyper::http::header::HeaderValue;
use hyper::{Body, Request, Uri, Version, HeaderMap, Method};

use itertools::Itertools;

use std::fs::File;
use std::io::{BufReader, prelude::*};


/// Represents the components of a request for recreating the [Request] object
///
/// This struct is useful since the [Request] object is not Clone.
/// (TODO: Add Extensions)
pub struct RequestComponents {
    head: HeaderMap<HeaderValue>,
    uri: Uri,
    version: Version,
    body: String,
    method: Method
}

impl RequestComponents {
    /// Create a new empty [RequestComponents].
    fn new() -> Self {
        RequestComponents { head: HeaderMap::new(), uri: Uri::from_static("/"), version: Version::HTTP_11, body: "".to_string(), method: Method::GET }
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
pub(crate) struct RequestTemplate {
    pub(crate) req: RequestComponents,
    pub(crate) marked: Vec<Part>,
    pub(crate) pattern: String,
}

/// Either a element in the header is marked, or an element in the body.
pub(crate) enum Part {
    Body(String),
    Header(String),
}


/// Trait for creating a RequestTemplate from a file. (TODO: Implement TryFrom for other types)
impl TryFrom<File> for RequestTemplate {
    type Error = Error;
    fn try_from(req_file: File) -> Result<Self, Self::Error> {
        let pattern = "§§";

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
            _ => Version::HTTP_11
        };
        req.method = Method::try_from(method)?;

        for header in lines.by_ref() {
            let header = header?.trim().to_string();
            if header.is_empty() {
                break;
            }
            if header.contains(pattern) {
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
        if body.contains(pattern) {
            marked.push(Part::Body(body));
        }

        Ok(Self {
            req: req,
            marked,
            pattern: pattern.to_string(),
        })
    }

}



impl RequestTemplate {
    /// Replace the marked Parts with pw and build a new Request from them.
    pub(crate) fn replace_then_request(&self, pw: &str) -> Result<Request<Body>> {
        let mut body: &String = &self.req.body;
        let mut req = Request::builder()
            .version(self.req.version)
            .method(self.req.method.clone())
            .uri(self.req.uri.clone());
        let headers = req.headers_mut().ok_or(anyhow!("Builder has error"))?;
        headers.clone_from(&self.req.head);
        for part in &self.marked {
            match part {
                Part::Header(header) => {
                    let (key, value) = header
                        .split(':')
                        .map(str::trim)
                        .map(|s| s.replace(&self.pattern, &pw))
                        .next_tuple()
                        .ok_or(anyhow!("Invalid Header"))?;
                    req = req.header(key, value);
                }
                Part::Body(bd) => {
                    body = bd;
                }
            }
        }
        Ok(req.body(Body::from(body.replace(&self.pattern, &pw)))?)
    }
}
