//! rip_intruder
//!
//! This program is intended to be a viable alternative to Burp Suite's Intruder.
//! Eventually implementing all of its most relevant features. In its current
//! state the only supported attack type is the "Battering Ram" attack type,
//! where using a single set of payloads it places the same payload at all
//! defined payload positions.
//!
//! ```plaintext
//! Usage: rip_intruder [OPTIONS] <REQ_F> <PASS_F>
//!Arguments:
//!<REQ_F>   Path to request template file
//!<PASS_F>  Path to password file
//!
//!Options:
//!-c, --concurrent-requests <CONCURRENT_REQUESTS>  Number of concurrent requests [default: 1]
//!-h, --help                                       Print help information
//!-V, --version                                    Print version information
//! ```
//!

use crate::intruder::Intruder;
use anyhow::Result;
use std::fs::File;
mod request_template;
use clap::Parser;
mod intruder;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to request template file
    #[arg(index = 1, value_hint = clap::ValueHint::FilePath)]
    req_f: std::path::PathBuf,

    /// Path to password file
    #[arg(index = 2, value_hint = clap::ValueHint::FilePath)]
    pass_f: std::path::PathBuf,

    /// Number of concurrent requests
    #[arg(short, long, default_value_t = 1)]
    concurrent_requests: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let intruder = Intruder::new(File::open(&args.req_f)?, args.concurrent_requests)?;

    let pass_file = File::open(&args.pass_f)?;
    let pw = intruder.bruteforce(pass_file).await?;
    println!("{:}", pw);

    Ok(())
}
