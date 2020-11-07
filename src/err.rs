pub enum ErrorKind {
  ConfigError(ConfigErrorKind),
}

impl From<ConfigErrorKind> for ErrorKind {
  fn from(e: ConfigErrorKind) -> Self { ErrorKind::ConfigError(e) }
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
