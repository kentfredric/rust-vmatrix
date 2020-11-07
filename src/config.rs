use super::err::ConfigErrorKind;
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
