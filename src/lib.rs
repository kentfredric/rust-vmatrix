mod config;
mod cratedir;

pub mod pages;
pub mod results;
pub mod stats_repo;
pub mod stats_repo_cache;
pub mod versions;

pub use config::{Config, ConfigError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Configuration Error: {0}")]
  ConfigError(#[from] ConfigError),
  #[error("Result Data Error: {0}")]
  ResultsError(#[from] results::Error),
  #[error("Version Data Error: {0}")]
  VersionsError(#[from] versions::Error),
  #[error("Error in Stats Repo: {0}")]
  StatsRepoError(#[from] stats_repo::Error),
}
