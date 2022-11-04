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
//!
//! Arguments:
//!   <REQ_F>   Path to request template file
//!   <PASS_F>  Path to password file
//!
//! Options:
//!   -c, --concurrent-requests <CONCURRENT_REQUESTS>
//!           Number of concurrent requests [default: 1]
//!   -p, --pattern <PATTERN>
//!           Regex pattern [default: §§]
//!       --hit-type <HIT_TYPE>
//!           What is considered a hit [default: ok] [possible values: ok, all]
//!   -o <OF>
//!           Output to file
//!   -s <STOP>
//!           Stop after n hits, -1 to try all provided words [default: 1]
//!       --format <OUT_FORMAT>
//!           Output format [default: csv] [possible values: csv, jsonl]
//!   -h, --help
//!           Print help information
//!   -V, --version
//!           Print version information
//! ```
mod arg_types;
mod output;

use intruder::intruder::Intruder;
use intruder::intruder::IntruderConfig;
use output::Cli;
use arg_types::{HitType, OutputFormat};
use anyhow::Result;
use clap::Parser;
use output::CliConfig;
use std::io::stderr;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to request template file
    #[arg(index = 1, value_hint = clap::ValueHint::FilePath)]
    req_f: std::path::PathBuf,

    /// Path to password file
    #[arg(index = 2, value_hint = clap::ValueHint::FilePath)]
    pass_f: std::path::PathBuf,

    /// Number of concurrent requests
    #[arg(short, long, default_value_t = 1)]
    concurrent_requests: usize,

    /// Regex pattern
    #[arg(short, long, default_value_t = str::to_string("§§"))]
    pattern: String,

    /// What is considered a hit
    #[arg(long, value_enum, default_value_t = HitType::Ok)]
    hit_type: HitType,

    /// Output to file
    #[arg(short)]
    out_file: Option<PathBuf>,

    /// Stop after n hits, -1 to try all provided words
    #[arg(short, default_value_t = 1, allow_hyphen_values = true)]
    stop: isize,

    /// Output format
    #[arg(short = 'f', long = "format", value_enum, default_value_t = OutputFormat::Csv)]
    out_format: OutputFormat,
}

fn get_configs(args: Args) -> (CliConfig, IntruderConfig){
    let cliconfig = CliConfig{
        out_format: args.out_format,
        out_file: args.out_file,
        hit_type: args.hit_type,
        stop: args.stop
    };

    let intruderconfig = IntruderConfig {
        req_f: args.req_f,
        pass_f: args.pass_f,
        pattern: args.pattern,
        concurrent_requests: args.concurrent_requests
    };

    (cliconfig, intruderconfig)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let (cliconfig, intruderconfig) = get_configs(args);

    let cli = Cli::new(cliconfig);
    let intruder = Intruder::new(intruderconfig)?;
    let errors = cli.run(intruder).await?;
    if errors.len() > 0 {
        writeln!(
            stderr(),
            "These payloads were not sent successfully: {:?}",
            errors
        )?;
    }

    Ok(())
}
