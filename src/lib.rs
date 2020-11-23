mod config;
mod cratedir;

pub mod pages;
mod result_list;
mod stats_repo;
mod stats_repo_cache;
mod version_list;

pub use config::{Config, ConfigError};
pub use cratedir::CrateDirError;
pub use result_list::{ResultInfo, ResultList, ResultType, ResultsError};
pub use stats_repo::{StatsRepo, StatsRepoError};
pub use stats_repo_cache::{StatsRepoCache, StatsRepoCacheError};
pub use version_list::{VersionInfo, VersionList, VersionsError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Configuration Error: {0}")]
  ConfigError(#[from] ConfigError),
  #[error("Result Data Error: {0}")]
  ResultsError(#[from] ResultsError),
  #[error("Version Data Error: {0}")]
  VersionsError(#[from] VersionsError),
  #[error("Error in Stats Repo: {0}")]
  StatsRepoError(#[from] StatsRepoError),
  #[error("Error in Stats Repo Cache: {0}")]
  StatsRepoCacheError(#[from] StatsRepoCacheError),
}
