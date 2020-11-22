use std::path::{Path, PathBuf};

#[derive(serde_derive::Deserialize, Debug)]
pub struct Config {
  stats_repo: StatsRepoConfig,
  targets:    Option<TargetsConfig>,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
  #[error("Error reading TOML data: {0}")]
  TomlDecodeError(#[from] toml::de::Error),
  #[error("Error reading Config TOML file: {0}")]
  IoError(#[from] std::io::Error),
}

impl Config {
  pub fn root(&self) -> &PathBuf { &self.stats_repo.root }

  pub fn rustc(&self) -> Option<&Vec<String>> { self.targets.as_ref().and_then(|x| x.rustc.as_ref()) }

  pub fn from_path(path: &Path) -> Result<Self, ConfigError> { std::fs::read_to_string(path)?.parse() }
}

impl std::str::FromStr for Config {
  type Err = ConfigError;

  fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(toml::from_str(s)?) }
}

#[derive(serde_derive::Deserialize, Debug)]
struct StatsRepoConfig {
  root: PathBuf,
}

#[derive(serde_derive::Deserialize, Debug)]
struct TargetsConfig {
  rustc: Option<Vec<String>>,
}
