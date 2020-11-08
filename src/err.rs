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

#[derive(Debug)]
pub enum ConfigErrorKind {
  TomlDecodeError(toml::de::Error),
  RootNotExists(String),
  RootNotDir(String),
  IoError(std::io::Error),
}

impl From<toml::de::Error> for ConfigErrorKind {
  fn from(e: toml::de::Error) -> Self { Self::TomlDecodeError(e) }
}

impl From<std::io::Error> for ConfigErrorKind {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}

#[derive(Debug)]
pub enum VersionsErrorKind {
  FileNotExists(String),
  FileNotReadable(String),
  IoError(std::io::Error),
  SerdeJsonError(serde_json::Error),
}

impl From<std::io::Error> for VersionsErrorKind {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
impl From<serde_json::Error> for VersionsErrorKind {
  fn from(e: serde_json::Error) -> Self { Self::SerdeJsonError(e) }
}
