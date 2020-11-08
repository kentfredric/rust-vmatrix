use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

pub fn from_str<N>(content: N) -> Result<VersionList, VersionsErrorKind>
where
  N: AsRef<str>,
{
  use serde_json::from_str;
  Ok(from_str(content.as_ref())?)
}
pub fn from_file<N>(file: N) -> Result<VersionList, VersionsErrorKind>
where
  N: Into<PathBuf>,
{
  let path = file.into();
  let mut file = File::open(path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  from_str(contents)
}

#[derive(Debug)]
pub enum VersionsErrorKind {
  FileNotExists(String),
  FileNotReadable(String),
  IoError(std::io::Error),
  SerdeJsonError(serde_json::Error),
}

impl From<std::io::Error> for VersionsErrorKind {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
impl From<serde_json::Error> for VersionsErrorKind {
  fn from(e: serde_json::Error) -> Self { Self::SerdeJsonError(e) }
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionInfo {
  pub audit_actions: AuditActions,
  pub authors:       AuthorInfo,
  #[serde(rename = "crate")]
  pub crate_name:    String,
  pub crate_size:    Option<u64>,
  pub created_at:    TimeStamp,
  pub dependencies:  Dependencies,
  pub dl_path:       ApiUrl,
  pub downloads:     u64,
  pub features:      Features,
  pub id:            u64,
  pub license:       License,
  pub links:         Links,
  pub num:           Version,
  pub published_by:  Option<User>,
  pub readme_path:   ApiUrl,
  pub updated_at:    String,
  pub yanked:        bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuditAction {
  pub action: String,
  pub time:   TimeStamp,
  pub user:   User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorInfo {}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
  pub avatar: String,
  pub id:     u64,
  pub login:  String,
  pub name:   String,
  pub url:    String,
}
