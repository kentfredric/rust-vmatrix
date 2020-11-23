use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum VersionsError {
  #[error("Error reading Versions JSON data: {0}")]
  IoError(#[from] std::io::Error),
  #[error("Error reading Versions JSON file: {0}")]
  SerdeJsonError(#[from] serde_json::Error),
}
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
pub type AuditActions = Vec<AuditAction>;
pub type Dependencies = Vec<Dependency>;
pub type FeatureReqs = Vec<String>;
pub type Features = HashMap<String, FeatureReqs>;
pub type License = String;
pub type Links = HashMap<String, ApiUrl>;
pub type TimeStamp = String;
pub type ApiUrl = String;
pub type Url = String;
pub type Version = String;
pub type VersionList = Vec<VersionInfo>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionInfo {
  audit_actions: AuditActions,
  authors:       AuthorInfo,
  #[serde(rename = "crate")]
  crate_name:    String,
  crate_size:    Option<u64>,
  created_at:    TimeStamp,
  dependencies:  Dependencies,
  dl_path:       ApiUrl,
  downloads:     u64,
  features:      Features,
  id:            u64,
  license:       Option<License>,
  links:         Links,
  num:           Version,
  published_by:  Option<User>,
  readme_path:   ApiUrl,
  updated_at:    String,
  yanked:        bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuditAction {
  action: String,
  time:   TimeStamp,
  user:   User,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorInfo {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependency {
  crate_id:         String,
  default_features: bool,
  downloads:        u64,
  features:         FeatureReqs,
  id:               u64,
  kind:             String,
  optional:         bool,
  req:              String,
  target:           Option<String>,
  version_id:       u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
  avatar: Url,
  id:     u64,
  login:  String,
  name:   Option<String>,
  url:    Url,
}

pub fn from_str<N>(content: N) -> Result<VersionList, VersionsError>
where
  N: AsRef<str>,
{
  use serde_json::from_str;
  Ok(from_str(content.as_ref())?)
}
pub fn from_file<N>(file: N) -> Result<VersionList, VersionsError>
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
