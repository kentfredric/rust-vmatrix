use std::{collections::HashMap, path::PathBuf};

pub struct StatsRepoCache<'a> {
  repo:           &'a super::stats_repo::StatsRepo,
  crate_names:    Option<Vec<String>>,
  crate_paths:    HashMap<String, PathBuf>,
  crate_versions: HashMap<String, Option<super::versions::VersionList>>,
  crate_results:  HashMap<String, Option<super::results::ResultList>>,
}

impl StatsRepoCache<'_> {
  pub fn for_repo(repo: &'_ super::StatsRepo) -> StatsRepoCache {
    StatsRepoCache {
      repo,
      crate_names: Option::None,
      crate_paths: HashMap::new(),
      crate_versions: HashMap::new(),
      crate_results: HashMap::new(),
    }
  }

  pub fn root(&self) -> Result<PathBuf, StatsRepoCacheError> { Ok(self.repo.root()?) }

  pub fn rustcs(&self) -> Vec<String> { self.repo.rustcs().to_vec() }

  pub fn crate_names(&mut self) -> Result<Vec<String>, StatsRepoCacheError> {
    match &self.crate_names {
      | None => {
        let names = self.repo.crate_names()?;
        self.crate_names.replace(names.to_owned());
        Ok(names)
      },
      | Some(cached) => Ok(cached.to_vec()),
    }
  }

  pub fn crate_path<C>(&mut self, my_crate: C) -> Result<PathBuf, StatsRepoCacheError>
  where
    C: AsRef<str>,
  {
    let my_crate = my_crate.as_ref();
    use std::collections::hash_map::Entry;

    match self.crate_paths.entry(my_crate.to_string()) {
      | Entry::Occupied(e) => Ok(e.get().to_path_buf()),
      | Entry::Vacant(e) => {
        match self.repo.crate_path(my_crate) {
          | Ok(p) => {
            e.insert(p.to_owned());
            Ok(p)
          },
          | Err(err) => Err(err.into()),
        }
      },
    }
  }

  pub fn crate_versions<C>(&mut self, my_crate: C) -> Result<super::versions::VersionList, StatsRepoCacheError>
  where
    C: AsRef<str>,
  {
    let my_crate = my_crate.as_ref();

    use super::{StatsRepoError::VersionsError, VersionsError::IoError as VersionsIoError};
    use std::{collections::hash_map::Entry, io::ErrorKind::NotFound};

    match self.crate_versions.entry(my_crate.to_string()) {
      | Entry::Occupied(e) => {
        match e.get() {
          | Some(result) => Ok(result.to_owned()),
          | None => Err(StatsRepoCacheError::VersionNotExists(my_crate.to_string())),
        }
      },
      | Entry::Vacant(e) => {
        match self.repo.crate_versions(my_crate) {
          | Ok(result) => {
            e.insert(Some(result.to_owned()));
            Ok(result)
          },
          | Err(VersionsError(VersionsIoError(err))) if err.kind() == NotFound => {
            e.insert(None);
            Err(err.into())
          },
          | Err(err) => Err(err.into()),
        }
      },
    }
  }

  pub fn crate_results<C>(&mut self, my_crate: C) -> Result<super::results::ResultList, StatsRepoCacheError>
  where
    C: AsRef<str>,
  {
    let my_crate = my_crate.as_ref();

    use super::{ResultsError::IoError as ResultsIoError, StatsRepoError::ResultsError};
    use std::{collections::hash_map::Entry, io::ErrorKind::NotFound};

    match self.crate_results.entry(my_crate.to_string()) {
      | Entry::Occupied(e) => {
        match e.get() {
          | Some(result) => Ok(result.to_owned()),
          | None => Err(StatsRepoCacheError::ResultNotExists(my_crate.to_string())),
        }
      },
      | Entry::Vacant(e) => {
        match self.repo.crate_results(my_crate) {
          | Ok(r) => {
            e.insert(Some(r.to_owned()));
            Ok(r)
          },
          | Err(ResultsError(ResultsIoError(err))) if err.kind() == NotFound => {
            e.insert(None);
            Err(err.into())
          },
          | Err(err) => Err(err.into()),
        }
      },
    }
  }
}

#[derive(thiserror::Error, Debug)]
pub enum StatsRepoCacheError {
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error("No result found for {0}")]
  ResultNotExists(String),
  #[error("No version data for {0}")]
  VersionNotExists(String),
  #[error("Stats Repository error: {0}")]
  StatsRepoError(#[from] super::StatsRepoError),
}
