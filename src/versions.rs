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

#[derive(Debug)]
pub enum Error {
  IoError(std::io::Error),
  SerdeJsonError(serde_json::Error),
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Self { Self::SerdeJsonError(e) }
}
