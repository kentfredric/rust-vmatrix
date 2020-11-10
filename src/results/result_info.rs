use super::result_type::ResultType;

pub type Version = String;
pub type RustcList = Vec<Version>;

use serde_derive::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct ResultInfo {
  #[serde(rename = "crate")]
  crate_name: String,
  num:        Version,
  rustc_fail: RustcList,
  rustc_pass: RustcList,
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
  pub fn version_cmp<V>(&self, version: V) -> std::cmp::Ordering
  where
    V: Into<String>,
  {
    self.num.cmp(&version.into())
  }

  #[inline(always)]
  pub fn version(&self) -> &String { &self.num }
}
