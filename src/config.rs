use std::path::PathBuf;

mod config;
mod err;
pub use self::{config::Config, err::Error};

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
  use std::{fs::File, io::Read};

  let path = file.into();
  let mut file = File::open(path)?;
  let mut contents = String::new();

  file.read_to_string(&mut contents)?;
  from_str(contents)
}
