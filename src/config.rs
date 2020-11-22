use std::path::PathBuf;

#[derive(serde_derive::Deserialize, Debug)]
pub struct Config {
  stats_repo: StatsRepoConfig,
  targets:    Option<TargetsConfig>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Error reading TOML data: {0}")]
  TomlDecodeError(#[from] toml::de::Error),
  #[error("Error reading Config TOML file: {0}")]
  IoError(#[from] std::io::Error),
}

pub(super) fn from_str<N>(content: N) -> Result<Config, Error>
where
  N: AsRef<str>,
{
  use toml::from_str;
  Ok(from_str(content.as_ref())?)
}

pub(super) fn from_file<N>(file: N) -> Result<Config, Error>
where
  N: Into<PathBuf>,
{
  use std::{fs::File, io::Read};

  let path = file.into();
  let mut file = File::open(path)?;
  let mut contents = String::new();

  file.read_to_string(&mut contents)?;
  from_str(contents)
}

impl Config {
  pub fn root(&self) -> PathBuf { self.stats_repo.root.to_owned() }

  pub fn rustc(&self) -> Option<Vec<String>> { self.targets.as_ref().and_then(|x| x.rustc.to_owned()) }
}
#[derive(serde_derive::Deserialize, Debug)]
struct StatsRepoConfig {
  root: PathBuf,
}

#[derive(serde_derive::Deserialize, Debug)]
struct TargetsConfig {
  rustc: Option<Vec<String>>,
}
