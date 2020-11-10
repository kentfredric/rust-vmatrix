mod err;
mod version_list;

pub use err::Error;
use std::path::PathBuf;
pub use version_list::VersionList;

pub fn from_str<N>(content: N) -> Result<VersionList, Error>
where
  N: AsRef<str>,
{
  use serde_json::from_str;
  Ok(from_str(content.as_ref())?)
}
pub fn from_file<N>(file: N) -> Result<VersionList, Error>
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
