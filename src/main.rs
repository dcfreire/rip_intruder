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
