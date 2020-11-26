use std::{collections::HashMap, path::PathBuf};

/// A Caching wrapper around [`super::StatsRepo`]
#[derive(Debug)]
pub struct StatsRepoCache<'a> {
  repo:           &'a super::stats_repo::StatsRepo,
  crate_names:    Option<Vec<String>>,
  crate_paths:    HashMap<String, PathBuf>,
  crate_versions: HashMap<String, Option<super::VersionList>>,
  crate_results:  HashMap<String, Option<super::ResultList>>,
}

impl StatsRepoCache<'_> {
  /// Construct a Caching wrapper around `repo:`[`super::StatsRepo`]
  pub fn for_repo(repo: &'_ super::StatsRepo) -> StatsRepoCache<'_> {
    StatsRepoCache {
      repo,
      crate_names: None,
      crate_paths: HashMap::new(),
      crate_versions: HashMap::new(),
      crate_results: HashMap::new(),
    }
  }

  /// Returns returns the configured rust targets from `repo`
  pub fn rustcs(&self) -> Vec<String> { self.repo.rustcs().to_vec() }

  /// Returns a (possibly cached) [`Vec`] of crate names.
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

  /// Returns a (possibly cached) path to the named crate
  pub fn crate_path(&mut self, my_crate: &str) -> Result<PathBuf, StatsRepoCacheError> {
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

  /// Returns a (possibly cached) [`super::VersionList`] for the
  /// named crate
  pub fn crate_versions(&mut self, my_crate: &str) -> Result<super::VersionList, StatsRepoCacheError> {
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

  /// Returns a (possibly cached) [`super::ResultList`]
  pub fn crate_results(&mut self, my_crate: &str) -> Result<super::ResultList, StatsRepoCacheError> {
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

/// Errors from [`StatsRepoCache`]
#[derive(thiserror::Error, Debug)]
pub enum StatsRepoCacheError {
  /// Errors from [`std`] calls
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  /// A cached lookup failure for a given `ResultList` but without
  /// the original error context
  #[error("No result found for {0}")]
  ResultNotExists(String),
  /// A cached lookup failure for a given `VersionList` but without
  /// original error context
  #[error("No version data for {0}")]
  VersionNotExists(String),
  /// An underlying error from [`super::StatsRepo`]
  #[error("Stats Repository error: {0}")]
  StatsRepoError(#[from] super::StatsRepoError),
}
