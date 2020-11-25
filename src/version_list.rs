use std::path::Path;

/// Errors from [`VersionList`] and friends
#[derive(thiserror::Error, Debug)]
pub enum VersionsError {
  /// Errors returned from [`std`] IO calls for paths
  #[error("Error reading Versions JSON data: {0}")]
  IoError(#[from] std::io::Error),
  /// Errors from [`serde_json::from_str`] decode failures
  #[error("Error reading Versions JSON file: {0}")]
  SerdeJsonError(#[from] serde_json::Error),
}
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
type AuditActions = Vec<AuditAction>;
type Dependencies = Vec<Dependency>;
type FeatureReqs = Vec<String>;
type Features = HashMap<String, FeatureReqs>;
type License = String;
type Links = HashMap<String, ApiUrl>;
type TimeStamp = String;
type ApiUrl = String;
type Url = String;
type Version = String;
type VersionListInner = Vec<VersionInfo>;

/// A collection of [`VersionInfo`] records for a given `crate`
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct VersionList(VersionListInner);

/// Published metadata for a given `crate` version.
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
struct AuditAction {
  action: String,
  time:   TimeStamp,
  user:   User,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AuthorInfo {}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Dependency {
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
struct User {
  avatar: Url,
  id:     u64,
  login:  String,
  name:   Option<String>,
  url:    Url,
}

impl VersionList {
  /// Parse the file at `path:`[`Path`] as a `json` file and yield a
  /// [`VersionList`]
  pub fn from_path(path: &Path) -> Result<Self, VersionsError> { std::fs::read_to_string(path)?.parse() }
}
impl std::str::FromStr for VersionList {
  type Err = VersionsError;

  fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(serde_json::from_str(s)?) }
}
impl std::ops::Deref for VersionList {
  type Target = VersionListInner;

  fn deref(&self) -> &Self::Target { &self.0 }
}
