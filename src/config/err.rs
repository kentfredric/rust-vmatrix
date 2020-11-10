#[derive(Debug)]
pub enum Error {
  TomlDecodeError(toml::de::Error),
  IoError(std::io::Error),
}

impl From<toml::de::Error> for Error {
  fn from(e: toml::de::Error) -> Self { Self::TomlDecodeError(e) }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
