use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
  pub stats_repo: StatsRepoConfig,
  pub targets:    Option<TargetsConfig>,
}
#[derive(Deserialize, Debug)]
pub struct StatsRepoConfig {
  pub root: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct TargetsConfig {
  pub rustc: Option<Vec<String>>,
}
