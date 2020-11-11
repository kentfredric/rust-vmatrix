use std::path::PathBuf;

mod config_data;

pub use self::config_data::Config;

pub fn from_str<N>(content: N) -> Result<Config, self::Error>
where
  N: AsRef<str>,
{
  use toml::from_str;
  Ok(from_str(content.as_ref())?)
}

pub fn from_file<N>(file: N) -> Result<Config, self::Error>
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

#[derive(Debug)]
pub enum Error {
  TomlDecodeError(toml::de::Error),
  IoError(std::io::Error),
}

impl From<toml::de::Error> for Error {
  fn from(e: toml::de::Error) -> Self { Self::TomlDecodeError(e) }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
