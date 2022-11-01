use anyhow::{anyhow, Result};
use hyper::{Body, Client, Request, Uri, Version};
use itertools::Itertools;
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn parse_req_file(req_file: &str, sub: &str) -> Result<Request<Body>> {
    let mut lines = req_file.lines();
    let first_line = lines.next().unwrap();
    let (method, uri, httpver) = first_line.split(' ').next_tuple().unwrap();
    let mut req = Request::builder()
        .version(match httpver {
            "HTTP/0.9" => Version::HTTP_09,
            "HTTP/1" => Version::HTTP_10,
            "HTTP/2" => Version::HTTP_2,
            "HTTP/3" => Version::HTTP_3,
            _ => Version::HTTP_11,
        })
        .method(method);

    for header in lines.by_ref() {
        let header = header.trim().to_string();
        if header.is_empty() {
            break;
        }

        let (key, value) = header.split(':').map(str::trim).next_tuple().unwrap();

        if key == "Host" {
            let uri = Uri::builder()
                .scheme("http")
                .authority(value)
                .path_and_query(uri)
                .build()?;
            req = req.uri(uri);
        }
        if key == "Content-Length" {
            continue;
        }
        req = req.header(key, value)
    }

    if req.uri_ref() == None {
        req = req.uri(uri);
    }

    let body = lines.map(|s| s.replace("§§", &sub)).join("");
    Ok(req.body(Body::from(body))?)
}

async fn bruteforce(pass_file: File, req_file: String) -> Result<String> {
    let passwords = BufReader::new(pass_file).lines();
    let req_file = std::fs::read_to_string(req_file)?;

    let client = Client::new();

    let reqs = passwords
        .filter_map(|pw| pw.ok())
        .map(|pw| (parse_req_file(&req_file, &pw), pw))
        .filter(|req| req.0.is_ok());

    for (req, pw) in reqs {
        let resp = client.request(req.unwrap()).await?;

        if resp.status() == 200 {
            return Ok(pw);
        }
    }
    Err(anyhow!("Password not found"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = args().skip(1).collect();
    if args.len() != 2 {
        std::process::exit(1);
    }

    let pass_file = File::open(&args[1])?;

    let res = bruteforce(pass_file, args[0].to_owned()).await?;
    println!("{:?}", res);
    Ok(())
}
