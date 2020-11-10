use std::io;

#[derive(Debug)]
pub enum Error {
  IoError(io::Error),
  SerdeJsonError(serde_json::Error),
}

impl From<io::Error> for Error {
  fn from(e: io::Error) -> Self { Self::IoError(e) }
}
impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Self { Self::SerdeJsonError(e) }
}
