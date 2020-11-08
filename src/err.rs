use super::versions::VersionsErrorKind;

pub enum ErrorKind {
  ConfigError(super::config::Error),
  VersionsError(VersionsErrorKind),
}

impl From<super::config::Error> for ErrorKind {
  fn from(e: super::config::Error) -> Self { ErrorKind::ConfigError(e) }
}
impl From<VersionsErrorKind> for ErrorKind {
  fn from(e: VersionsErrorKind) -> Self { ErrorKind::VersionsError(e) }
}
