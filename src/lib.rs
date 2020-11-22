mod config;
mod cratedir;

pub mod pages;
pub mod results;
mod stats_repo;
pub mod stats_repo_cache;
mod versions;

pub use config::{Config, ConfigError};
pub use cratedir::CrateDirError;
pub use stats_repo::{StatsRepo, StatsRepoError};
pub use versions::VersionsError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Configuration Error: {0}")]
  ConfigError(#[from] ConfigError),
  #[error("Result Data Error: {0}")]
  ResultsError(#[from] results::Error),
  #[error("Version Data Error: {0}")]
  VersionsError(#[from] VersionsError),
  #[error("Error in Stats Repo: {0}")]
  StatsRepoError(#[from] StatsRepoError),
}
