pub mod config;
pub mod pages;
pub mod results;
pub mod stats_repo;
pub mod versions;

#[derive(Debug)]
pub enum Error {
  ConfigError(config::Error),
  ResultsError(results::Error),
  VersionsError(versions::Error),
  StatsRepoError(stats_repo::Error),
}

impl From<config::Error> for Error {
  fn from(e: config::Error) -> Self { Self::ConfigError(e) }
}

impl From<results::Error> for Error {
  fn from(e: results::Error) -> Self { Self::ResultsError(e) }
}

impl From<stats_repo::Error> for Error {
  fn from(e: stats_repo::Error) -> Self { Self::StatsRepoError(e) }
}

impl From<versions::Error> for Error {
  fn from(e: versions::Error) -> Self { Self::VersionsError(e) }
}

impl std::fmt::Display for Error {
  fn fmt(&self, fmter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    match self {
      | Self::ConfigError(e) => write!(fmter, "Configuration Error: {}", e),
      | Self::ResultsError(e) => write!(fmter, "Result Data Error: {}", e),
      | Self::StatsRepoError(e) => write!(fmter, "Error in Stats Repo: {}", e),
      | Self::VersionsError(e) => write!(fmter, "Version Data Error: {}", e),
    }
  }
}
impl std::error::Error for Error {}
