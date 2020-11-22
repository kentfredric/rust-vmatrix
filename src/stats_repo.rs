use either::Either;
use std::{
  iter,
  path::{Path, PathBuf},
};

mod cratedir;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("IO Error in Stats Directory: {0}")]
  IoError(#[from] std::io::Error),
  #[error("Error mapping to/from crate directory stats layout: {0}")]
  CrateDirError(#[from] cratedir::Error),
  #[error("Error loading results: {0}")]
  ResultsError(#[from] super::results::Error),
  #[error("Error loading versions: {0}")]
  VersionsError(#[from] super::versions::Error),
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

  pub fn crate_names_iterator(&self) -> Box<dyn Iterator<Item = Result<String, cratedir::Error>>> {
    let root = self.root.to_owned();

    Box::new(cratedir::SectionIterator::new(root.to_owned(), "crates-").flat_map(move |section| {
      match section {
        | Err(e) => Either::Left(iter::once(Err(e))),
        | Ok(section_name) => {
          let sec = root.to_owned().join(format!("crates-{}", &section_name));
          Either::Right(cratedir::SubSectionIterator::new(&sec, &section_name).flat_map(move |subsection| {
            match subsection {
              | Err(e) => Either::Left(iter::once(Err(e))),
              | Ok(subsection_name) => {
                let subsec = sec.join(&subsection_name);
                Either::Right(cratedir::CrateIterator::new(subsec, &subsection_name))
              },
            }
          }))
        },
      }
    }))
  }

  pub fn crate_names(&self) -> Result<Vec<String>, Error> {
    Ok(
      self
        .crate_names_iterator()
        .filter(|e| !matches!(e, Err(cratedir::Error::NonSection(..))))
        .collect::<Result<Vec<String>, cratedir::Error>>()
        .map(|mut i| {
          i.sort_unstable();
          i
        })?,
    )
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

impl AsRef<StatsRepo> for StatsRepo {
  fn as_ref(&self) -> &Self { &self }
}
