use serde_derive::{Deserialize, Serialize};
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum ResultsError {
  #[error("Error reading Result JSON data: {0}")]
  IoError(#[from] std::io::Error),
  #[error("Error reading Result JSON file: {0}")]
  SerdeJsonError(#[from] serde_json::Error),
}

pub(crate) type ResultListInner = Vec<ResultInfo>;
type Version = String;
type RustcList = Vec<Version>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct ResultList(ResultListInner);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultInfo {
  #[serde(rename = "crate")]
  crate_name: String,
  num:        Version,
  rustc_fail: RustcList,
  rustc_pass: RustcList,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ResultType {
  Pass,
  Fail,
  Unknown,
}

impl ResultList {
  pub fn from_path(path: &Path) -> Result<Self, ResultsError> { std::fs::read_to_string(path)?.parse() }

  pub fn rustc_result<V, R>(&self, version: V, rustc: R) -> ResultType
  where
    R: Into<String>,
    V: Into<String>,
  {
    let v = version.into();
    let r = rustc.into();

    if let Ok(vrec) = self.0.binary_search_by(|probe| probe.version_cmp(&v)) {
      self.0[vrec].rustc_result(&r)
    } else {
      ResultType::Unknown
    }
  }
}

impl std::str::FromStr for ResultList {
  type Err = ResultsError;

  fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(serde_json::from_str(s)?) }
}

impl std::ops::Deref for ResultList {
  type Target = ResultListInner;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl ResultInfo {
  #[inline(always)]
  pub fn rustc_result<R>(&self, rustc: R) -> ResultType
  where
    R: Into<String>,
  {
    let r = rustc.into();
    if self.rustc_fail.contains(&r) {
      return ResultType::Fail;
    }
    if self.rustc_pass.contains(&r) {
      return ResultType::Pass;
    }
    ResultType::Unknown
  }

  #[inline(always)]
  fn version_cmp<V>(&self, version: V) -> std::cmp::Ordering
  where
    V: Into<String>,
  {
    self.num.cmp(&version.into())
  }

  #[inline(always)]
  pub fn version(&self) -> &String { &self.num }
}
