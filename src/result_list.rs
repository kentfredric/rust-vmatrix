use serde_derive::{Deserialize, Serialize};
use std::path::Path;

/// Error types for ResultList handling issues
#[derive(thiserror::Error, Debug)]
pub enum ResultsError {
  /// Errors returned from [`std`] IO calls for paths
  #[error("Error reading Result JSON data: {0}")]
  IoError(#[from] std::io::Error),
  /// Errors returned by [`serde_json::from_str`] decode failures
  #[error("Error reading Result JSON file: {0}")]
  SerdeJsonError(#[from] serde_json::Error),
}

pub(crate) type ResultListInner = Vec<ResultInfo>;
type Version = String;
type RustcList = Vec<Version>;

/// A collection of build result data for a given crate
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct ResultList(ResultListInner);

/// Build result information for a single version of a given crate
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultInfo {
  #[serde(rename = "crate")]
  crate_name: String,
  num:        Version,
  rustc_fail: RustcList,
  rustc_pass: RustcList,
}

/// Build result for a single version of a given crate on a given
/// rust
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ResultType {
  /// Build Passed
  Pass,
  /// Build Failed
  Fail,
  /// No Known Status on Build
  Unknown,
}

impl ResultList {
  /// Parse the file at `path:`[`Path`] as a `json` file and yeild a
  /// [`ResultList`]
  pub fn from_path(path: &Path) -> Result<Self, ResultsError> { std::fs::read_to_string(path)?.parse() }

  /// Query the ResultList to Asertain the [`ResultType`] of the
  /// given `crate` `version` on a given `rustc`
  pub fn rustc_result(&self, version: &str, rustc: &str) -> ResultType {
    if let Ok(vrec) = self.0.binary_search_by(|probe| probe.version_cmp(version)) {
      self.0[vrec].rustc_result(rustc)
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
  /// Query the result of a given `rustc`
  #[inline(always)]
  pub fn rustc_result(&self, rustc: &str) -> ResultType {
    if self.rustc_fail.contains(&rustc.to_string()) {
      return ResultType::Fail;
    }
    if self.rustc_pass.contains(&rustc.to_string()) {
      return ResultType::Pass;
    }
    ResultType::Unknown
  }

  #[inline(always)]
  fn version_cmp(&self, version: &str) -> std::cmp::Ordering { self.num.cmp(&version.to_string()) }

  /// Returns the crate version this information record is for
  #[inline(always)]
  pub fn version(&self) -> &String { &self.num }
}
