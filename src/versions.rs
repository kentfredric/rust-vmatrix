mod version_list;

use std::path::PathBuf;
pub use version_list::VersionList;

pub fn from_str<N>(content: N) -> Result<VersionList, self::Error>
where
  N: AsRef<str>,
{
  use serde_json::from_str;
  Ok(from_str(content.as_ref())?)
}
pub fn from_file<N>(file: N) -> Result<VersionList, self::Error>
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
  #[error("Error reading Versions JSON data: {0}")]
  IoError(#[from] std::io::Error),
  #[error("Error reading Versions JSON file: {0}")]
  SerdeJsonError(#[from] serde_json::Error),
}
