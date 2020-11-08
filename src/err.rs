use super::{config::ConfigErrorKind, versions::VersionsErrorKind};

pub enum ErrorKind {
  ConfigError(ConfigErrorKind),
  VersionsError(VersionsErrorKind),
}

impl From<ConfigErrorKind> for ErrorKind {
  fn from(e: ConfigErrorKind) -> Self { ErrorKind::ConfigError(e) }
}
impl From<VersionsErrorKind> for ErrorKind {
  fn from(e: VersionsErrorKind) -> Self { ErrorKind::VersionsError(e) }
}
