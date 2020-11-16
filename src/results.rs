mod result_info;
mod result_list;
mod result_type;

use std::path::PathBuf;

pub use self::{result_info::ResultInfo, result_list::ResultList, result_type::ResultType};

pub fn from_str<N>(content: N) -> Result<ResultList, self::Error>
where
  N: AsRef<str>,
{
  use serde_json::from_str;
  Ok(from_str(content.as_ref())?)
}
pub fn from_file<N>(file: N) -> Result<ResultList, self::Error>
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

impl std::fmt::Display for Error {
  fn fmt(&self, fmter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    match self {
      | Self::SerdeJsonError(e) => {
        write!(fmter, "Error reading Result JSON data: {}", e)
      },
      | Self::IoError(e) => {
        write!(fmter, "Error reading Result JSON file: {}", e)
      },
    }
  }
}
impl std::error::Error for Error {}
