use std::path::PathBuf;

pub(super) fn from_str<N>(content: N) -> Result<crate::config_data::Config, Error>
where
  N: AsRef<str>,
{
  use toml::from_str;
  Ok(from_str(content.as_ref())?)
}

pub(super) fn from_file<N>(file: N) -> Result<crate::config_data::Config, Error>
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

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Error reading TOML data: {0}")]
  TomlDecodeError(#[from] toml::de::Error),
  #[error("Error reading Config TOML file: {0}")]
  IoError(#[from] std::io::Error),
}
