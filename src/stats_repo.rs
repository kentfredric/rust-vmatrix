use std::path::{Path, PathBuf};

mod cratedir;

#[derive(Debug)]
pub enum Error {
  IoError(std::io::Error),
  CrateDirError(cratedir::Error),
  ResultsError(super::results::Error),
  VersionsError(super::versions::Error),
}

pub struct StatsRepo {
  root:   PathBuf,
  rustcs: Vec<String>,
}

pub fn from_config(c: super::config::Config) -> StatsRepo {
  StatsRepo {
    root:   c.stats_repo.root,
    rustcs: c.targets.map(|t| t.rustc.unwrap_or_else(Vec::new)).unwrap_or_else(Vec::new),
  }
}

impl StatsRepo {
  pub fn root(&self) -> Result<PathBuf, Error> { Ok(self.root.to_owned()) }

  pub fn rustcs(&self) -> &Vec<String> { &self.rustcs }

  pub fn crate_names(&self) -> Result<Vec<String>, Error> {
    let mut x = Vec::new();
    let root = self.root()?;
    for suffix in cratedir::sections_in(&root, "crates-")? {
      let section = root.join(format!("crates-{}", suffix));
      for slug in cratedir::subsections_in(&section, &suffix)? {
        let subsection = section.join(&slug);
        for member in cratedir::crates_in(&subsection, &slug)? {
          x.push(member);
        }
      }
    }
    x.sort_unstable();
    Ok(x)
  }

  pub fn crate_path<C>(&self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<str>,
  {
    Ok(cratedir::crate_path(self.root()?, "crates-", my_crate)?)
  }

  pub fn crate_file_path<C, F>(&self, my_crate: C, file: F) -> Result<PathBuf, Error>
  where
    C: AsRef<str>,
    F: AsRef<Path>,
  {
    Ok(self.crate_path(my_crate)?.join(file.as_ref()))
  }

  pub fn crate_versions_path<C>(&self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<str>,
  {
    self.crate_file_path(my_crate, "versions.json")
  }

  pub fn crate_results_path<C>(&self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<str>,
  {
    self.crate_file_path(my_crate, "results.json")
  }

  pub fn crate_index_path<C>(&self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<str>,
  {
    self.crate_file_path(my_crate, "index.html")
  }

  pub fn crate_versions<C>(&self, my_crate: C) -> Result<super::versions::VersionList, Error>
  where
    C: AsRef<str>,
  {
    Ok(super::versions::from_file(self.crate_versions_path(my_crate)?)?)
  }

  pub fn crate_results<C>(&self, my_crate: C) -> Result<super::results::ResultList, Error>
  where
    C: AsRef<str>,
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
impl From<cratedir::Error> for Error {
  fn from(e: cratedir::Error) -> Self { Self::CrateDirError(e) }
}

impl std::fmt::Display for Error {
  fn fmt(&self, fmter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    match self {
      | Self::ResultsError(e) => write!(fmter, "Error loading results: {}", e),
      | Self::VersionsError(e) => {
        write!(fmter, "Error loading versions: {}", e)
      },
      | Self::CrateDirError(e) => {
        write!(fmter, "Error mapping to/from crate directory stats layout: {}", e)
      },
      | Self::IoError(e) => write!(fmter, "IO Error in Stats Directory: {}", e),
    }
  }
}
impl std::error::Error for Error {}
