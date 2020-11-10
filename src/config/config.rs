use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
  pub stats_repo: StatsRepoConfig,
}
#[derive(Deserialize, Debug)]
pub struct StatsRepoConfig {
  pub root: PathBuf,
}
