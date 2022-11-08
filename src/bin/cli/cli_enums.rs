use clap::ValueEnum;
use intruder::request_template::AttackType;

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

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum AttackTypeCli{
    Sniper,
    BatteringRam,
    Pitchfork,
    ClusterBomb
}

impl Into<AttackType> for AttackTypeCli {
    fn into(self) -> AttackType {
        match self {
            Self::Sniper => AttackType::Sniper,
            Self::BatteringRam => AttackType::BatteringRam,
            Self::Pitchfork => AttackType::Pitchfork,
            Self::ClusterBomb => AttackType::ClusterBomb,
        }
    }
}