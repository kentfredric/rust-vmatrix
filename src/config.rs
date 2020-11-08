use serde_derive::Deserialize;
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Debug)]
pub enum Error {
  TomlDecodeError(toml::de::Error),
  IoError(std::io::Error),
}

#[derive(Deserialize, Debug)]
pub struct Config {
  pub stats_repo: StatsRepoConfig,
}

#[derive(Deserialize, Debug)]
pub struct StatsRepoConfig {
  pub root: PathBuf,
}

pub fn from_str<N>(content: N) -> Result<Config, Error>
where
  N: AsRef<str>,
{
  use toml::from_str;
  Ok(from_str(content.as_ref())?)
}

pub fn from_file<N>(file: N) -> Result<Config, Error>
where
  N: Into<PathBuf>,
{
  let path = file.into();
  let mut file = File::open(path)?;
  let mut contents = String::new();

  file.read_to_string(&mut contents)?;
  from_str(contents)
}

impl From<toml::de::Error> for Error {
  fn from(e: toml::de::Error) -> Self { Self::TomlDecodeError(e) }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
