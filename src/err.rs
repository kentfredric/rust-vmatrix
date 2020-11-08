pub enum ErrorKind {
  ConfigError(super::config::Error),
  VersionsError(super::versions::Error),
}

impl From<super::config::Error> for ErrorKind {
  fn from(e: super::config::Error) -> Self { ErrorKind::ConfigError(e) }
}
impl From<super::versions::Error> for ErrorKind {
  fn from(e: super::versions::Error) -> Self { ErrorKind::VersionsError(e) }
}
