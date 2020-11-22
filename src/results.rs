mod result_info;
mod result_list;
mod result_type;

use std::path::PathBuf;

pub use self::{result_info::ResultInfo, result_list::ResultList, result_type::ResultType};

pub fn from_str<N>(content: N) -> Result<ResultList, ResultsError>
where
  N: AsRef<str>,
{
  use serde_json::from_str;
  Ok(from_str(content.as_ref())?)
}
pub fn from_file<N>(file: N) -> Result<ResultList, ResultsError>
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
pub enum ResultsError {
  #[error("Error reading Result JSON data: {0}")]
  IoError(#[from] std::io::Error),
  #[error("Error reading Result JSON file: {0}")]
  SerdeJsonError(#[from] serde_json::Error),
}
