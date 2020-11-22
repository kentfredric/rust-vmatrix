mod config;

pub mod pages;
pub mod results;
pub mod stats_repo;
pub mod stats_repo_cache;
pub mod versions;

pub fn config_from_str<N>(content: N) -> Result<config::Config, self::Error>
where
  N: AsRef<str>,
{
  Ok(config::from_str(content)?)
}

pub fn config_from_file<N>(file: N) -> Result<config::Config, self::Error>
where
  N: Into<std::path::PathBuf>,
{
  Ok(config::from_file(file)?)
}

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
