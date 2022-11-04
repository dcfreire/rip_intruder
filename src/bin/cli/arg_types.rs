use clap::ValueEnum;

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum HitType {
    Ok,
    All,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum OutputFormat {
    Csv,
    Jsonl,
}
