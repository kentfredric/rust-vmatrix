#[derive(Debug)]
pub enum Error {
  ConfigError(super::config::Error),
  ResultsError(super::results::Error),
  VersionsError(super::versions::Error),
}

impl From<super::config::Error> for Error {
  fn from(e: super::config::Error) -> Self { Self::ConfigError(e) }
}

impl From<super::results::Error> for Error {
  fn from(e: super::results::Error) -> Self { Self::ResultsError(e) }
}

impl From<super::versions::Error> for Error {
  fn from(e: super::versions::Error) -> Self { Self::VersionsError(e) }
}
