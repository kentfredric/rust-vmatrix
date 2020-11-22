use serde_derive::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
  stats_repo: StatsRepoConfig,
  targets:    Option<TargetsConfig>,
}

impl Config {
  pub fn root(&self) -> PathBuf { self.stats_repo.root.to_owned() }

  pub fn rustc(&self) -> Option<Vec<String>> { self.targets.as_ref().and_then(|x| x.rustc.to_owned()) }
}
#[derive(Deserialize, Debug)]
struct StatsRepoConfig {
  root: PathBuf,
}

#[derive(Deserialize, Debug)]
struct TargetsConfig {
  rustc: Option<Vec<String>>,
}
