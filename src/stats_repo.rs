use std::{
  ffi::OsString,
  path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
  RootNotDir(PathBuf),
  RootIoError(PathBuf, std::io::Error),
  CrateNotDir(PathBuf),
  CrateIoError(PathBuf, std::io::Error),
  FileNotReadable(PathBuf),
  FileIoError(PathBuf, std::io::Error),
  NotUnicode(OsString),
  IoError(std::io::Error),
  ResultsError(super::results::Error),
  VersionsError(super::versions::Error),
}

pub struct StatsRepo {
  root: PathBuf,
}

pub fn from_config(c: super::config::Config) -> StatsRepo { StatsRepo { root: c.stats_repo.root } }

impl StatsRepo {
  pub fn root(&self) -> Result<PathBuf, Error> {
    let meta = self.root.metadata().map_err(|e| Error::RootIoError(self.root.to_owned(), e))?;
    if !meta.is_dir() {
      return Err(Error::RootNotDir(self.root.to_owned()));
    }
    Ok(self.root.to_owned())
  }

  pub fn crate_names(&self) -> Result<Vec<OsString>, Error> {
    let mut x = Vec::new();
    for entry in std::fs::read_dir(self.root()?)? {
      let direntry = entry?;
      let ent = direntry.file_name();
      let spth = ent.to_owned().into_string().map_err(Error::NotUnicode)?;
      if spth.starts_with('.') {
        continue;
      }
      if direntry.file_type()?.is_dir() {
        x.push(ent)
      }
    }
    x.sort_unstable();
    Ok(x)
  }

  pub fn crate_path<C>(&self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<Path>,
  {
    let path = self.root()?.join(my_crate.as_ref());
    let meta = path.metadata().map_err(|e| Error::CrateIoError(path.to_owned(), e))?;
    if !meta.is_dir() {
      return Err(Error::CrateNotDir(path));
    }
    Ok(path)
  }

  pub fn crate_file_path<C, F>(&self, my_crate: C, file: F) -> Result<PathBuf, Error>
  where
    C: AsRef<Path>,
    F: AsRef<Path>,
  {
    let path = self.crate_path(my_crate)?.join(file.as_ref());
    let meta = path.metadata().map_err(|e| Error::FileIoError(path.to_owned(), e))?;
    if meta.is_dir() {
      return Err(Error::FileNotReadable(path));
    }
    Ok(path)
  }

  pub fn crate_versions_path<C>(&self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<Path>,
  {
    self.crate_file_path(my_crate, "versions.json")
  }

  pub fn crate_results_path<C>(&self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<Path>,
  {
    self.crate_file_path(my_crate, "results.json")
  }

  pub fn crate_index_path<C>(&self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<Path>,
  {
    self.crate_file_path(my_crate, "index.html")
  }

  pub fn crate_versions<C>(&self, my_crate: C) -> Result<super::versions::VersionList, Error>
  where
    C: AsRef<Path>,
  {
    Ok(super::versions::from_file(self.crate_versions_path(my_crate)?)?)
  }

  pub fn crate_results<C>(&self, my_crate: C) -> Result<super::results::ResultList, Error>
  where
    C: AsRef<Path>,
  {
    Ok(super::results::from_file(self.crate_results_path(my_crate)?)?)
  }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
impl From<super::results::Error> for Error {
  fn from(e: super::results::Error) -> Self { Self::ResultsError(e) }
}
impl From<super::versions::Error> for Error {
  fn from(e: super::versions::Error) -> Self { Self::VersionsError(e) }
}
