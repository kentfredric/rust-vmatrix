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
}

impl From<config::Error> for Error {
  fn from(e: config::Error) -> Self { Self::ConfigError(e) }
}

impl From<results::Error> for Error {
  fn from(e: results::Error) -> Self { Self::ResultsError(e) }
}

impl From<versions::Error> for Error {
  fn from(e: versions::Error) -> Self { Self::VersionsError(e) }
}
