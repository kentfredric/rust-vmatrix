use serde_derive::Deserialize;
use std::{
  fs::File,
  io::Read,
  path::{Path, PathBuf},
};

#[derive(Deserialize, Debug)]

pub struct Config {
  pub stats_repo: StatsRepoConfig,
}

#[derive(Deserialize, Debug)]

pub struct StatsRepoConfig {
  pub root: String,
}

pub fn from_str<N>(content: N) -> Result<Config, ConfigErrorKind>
where
  N: AsRef<str>,
{
  use toml::from_str;

  Ok(from_str(content.as_ref())?)
}

pub fn from_file<N>(file: N) -> Result<Config, ConfigErrorKind>
where
  N: Into<PathBuf>,
{
  let path = file.into();

  let mut file = File::open(path)?;

  let mut contents = String::new();

  file.read_to_string(&mut contents)?;

  from_str(contents)
}

#[derive(Debug)]
pub enum ConfigErrorKind {
  TomlDecodeError(toml::de::Error),
  RootNotExists(String),
  RootNotDir(String),
  IoError(std::io::Error),
}

impl From<toml::de::Error> for ConfigErrorKind {
  fn from(e: toml::de::Error) -> Self { Self::TomlDecodeError(e) }
}

impl From<std::io::Error> for ConfigErrorKind {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}

impl StatsRepoConfig {
  pub fn root_path(&self) -> Result<&Path, ConfigErrorKind> {
    let p = Path::new(&self.root);

    if !p.exists() {
      Err(ConfigErrorKind::RootNotExists(self.root.to_owned()))
    } else if !p.is_dir() {
      Err(ConfigErrorKind::RootNotDir(self.root.to_owned()))
    } else {
      Ok(p)
    }
  }
}
