//! A collection of tools for managing testing of multiple versions
//! of rust crates against multiple rusts

mod config;
mod crate_dir;

mod result_list;
mod stats_repo;
mod stats_repo_cache;
mod version_list;

pub use config::{Config, ConfigError};
pub use crate_dir::{CrateDir, CrateDirError};
pub use result_list::{ResultInfo, ResultList, ResultType, ResultsError};
pub use stats_repo::{StatsRepo, StatsRepoError};
pub use stats_repo_cache::{StatsRepoCache, StatsRepoCacheError};
pub use version_list::{VersionInfo, VersionList, VersionsError};

/// Global Collector Error type for all internal error kinds
#[derive(thiserror::Error, Debug)]
pub enum Error {
  /// Errors from [`Config`]
  #[error("Configuration Error: {0}")]
  ConfigError(#[from] ConfigError),
  /// Errors from [`ResultList`] and friends
  #[error("Result Data Error: {0}")]
  ResultsError(#[from] ResultsError),
  /// Errors from [`VersionList`] and friends
  #[error("Version Data Error: {0}")]
  VersionsError(#[from] VersionsError),
  /// Errors from [`StatsRepo`]
  #[error("Error in Stats Repo: {0}")]
  StatsRepoError(#[from] StatsRepoError),
  /// Errors from [`StatsRepoCache`]
  #[error("Error in Stats Repo Cache: {0}")]
  StatsRepoCacheError(#[from] StatsRepoCacheError),
}
