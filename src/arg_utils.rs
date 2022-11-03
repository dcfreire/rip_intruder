use clap::ValueEnum;

#[derive(Copy, Clone, ValueEnum, Debug)]
pub(crate) enum HitType {
    Ok,
    All,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub(crate) enum OutputFormat {
    Csv,
    Jsonl,
}
