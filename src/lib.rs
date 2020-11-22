pub mod config;
pub mod pages;
pub mod results;
pub mod stats_repo;
pub mod stats_repo_cache;
pub mod versions;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Configuration Error: {0}")]
  ConfigError(#[from] config::Error),
  #[error("Result Data Error: {0}")]
  ResultsError(#[from] results::Error),
  #[error("Version Data Error: {0}")]
  VersionsError(#[from] versions::Error),
  #[error("Error in Stats Repo: {0}")]
  StatsRepoError(#[from] stats_repo::Error),
}
