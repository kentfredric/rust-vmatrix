use std::{
  ffi::OsString,
  path::{Path, PathBuf},
};

mod cratedir;

#[derive(Debug)]
pub enum Error {
  NotUnicode(OsString),
  CrateNameTooShort(PathBuf),
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

  pub fn crate_names(&self) -> Result<Vec<OsString>, Error> {
    let mut x = Vec::new();
    let root = self.root()?;
    for suffix in cratedir::sections_in(&root, "crates-")? {
      let section = root.join(format!("crates-{}", suffix));
      for slug in cratedir::subsections_in(&section, &suffix)? {
        let subsection = section.join(&slug);
        for member in cratedir::crates_in(&subsection, &slug)? {
          x.push(member.into());
        }
      }
    }
    x.sort_unstable();
    Ok(x)
  }

  pub fn crate_path<C>(&self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<Path>,
  {
    let crate_name = my_crate.as_ref();
    let mut crate_chars = crate_name.to_str().ok_or_else(|| Error::NotUnicode(crate_name.into()))?.chars();
    let first = crate_chars.next().ok_or_else(|| Error::CrateNameTooShort(crate_name.into()))?;
    let nibble = match crate_chars.next() {
      | Some(c) => format!("{}{}", first, c),
      | None => first.to_string(),
    };
    let path = self.root()?.join(format!("crates-{}", first)).join(nibble).join(crate_name);
    Ok(path)
  }

  pub fn crate_file_path<C, F>(&self, my_crate: C, file: F) -> Result<PathBuf, Error>
  where
    C: AsRef<Path>,
    F: AsRef<Path>,
  {
    let path = self.crate_path(my_crate)?.join(file.as_ref());
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
impl From<cratedir::Error> for Error {
  fn from(e: cratedir::Error) -> Self { Self::CrateDirError(e) }
}
