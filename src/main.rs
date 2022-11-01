use anyhow::Result;
use std::env::args;
use std::fs::File;
use crate::intruder::Intruder;
mod request_template;

mod intruder;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = args().skip(1).collect();
    if args.len() != 2 {
        std::process::exit(1);
    }

    let pass_file = File::open(&args[1])?;
    let intruder = Intruder::new(File::open(&args[0])?)?;

    let pw = intruder.bruteforce(pass_file).await?;
    println!("{:}", pw);

    Ok(())
}
