use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
  RootNotExists(PathBuf),
  RootNotDir(PathBuf),
  FileNotExists(PathBuf),
  FileNotReadable(PathBuf),
  IoError(std::io::Error),
}

pub struct StatsRepo {
  root: PathBuf,
}

pub fn from_config(c: super::config::Config) -> StatsRepo { StatsRepo { root: c.stats_repo.root } }

impl StatsRepo {
  pub fn root(&self) -> Result<PathBuf, Error> {
    if !self.root.exists() {
      return Err(Error::RootNotExists(self.root.to_owned()));
    }
    if !self.root.is_dir() {
      return Err(Error::RootNotDir(self.root.to_owned()));
    }
    Ok(self.root.to_owned())
  }

  pub fn crates(&self) -> Result<Vec<String>, Error> { todo!() }

  pub fn crate_versions_path<C>(&self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<str>,
  {
    let path = self.root()?.join(my_crate.as_ref()).join("versions.json");
    if !path.exists() {
      return Err(Error::FileNotExists(path));
    }
    if path.is_dir() {
      return Err(Error::FileNotReadable(path));
    }
    Ok(path)
  }

  pub fn crate_versions<C>(&self, _my_crate: C) -> Result<Vec<String>, Error>
  where
    C: AsRef<str>,
  {
    todo!()
  }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
