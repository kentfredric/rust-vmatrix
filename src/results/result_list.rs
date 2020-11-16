use super::{result_info::ResultInfo, result_type::ResultType};
use serde_derive::{Deserialize, Serialize};

pub type ResultListInner = Vec<ResultInfo>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct ResultList(ResultListInner);

impl ResultList {
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
impl std::ops::Deref for ResultList {
  type Target = ResultListInner;

  fn deref(&self) -> &Self::Target { &self.0 }
}
