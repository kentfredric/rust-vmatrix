use std::path::PathBuf;

/// Errors from [`StatsRepo`]
#[derive(thiserror::Error, Debug)]
pub enum StatsRepoError {
  /// Errors from [`std`] in directory handling
  #[error("IO Error in Stats Directory: {0}")]
  IoError(#[from] std::io::Error),
  /// Errors from [`super::CrateDir`]
  #[error("Error mapping to/from crate directory stats layout: {0}")]
  CrateDirError(#[from] super::CrateDirError),
  /// Errors from [`super::ResultList`] and friends
  #[error("Error loading results: {0}")]
  ResultsError(#[from] super::ResultsError),
  /// Errors from [`super::VersionList`] and friends
  #[error("Error loading versions: {0}")]
  VersionsError(#[from] super::VersionsError),
}

/// Higher Level General API for accessing state about crate testing
#[derive(Debug)]
pub struct StatsRepo {
  crate_dir: super::CrateDir,
  rustcs:    Vec<String>,
}

impl StatsRepo {
  /// Construct a [`StatsRepo`] using configuration found in a
  /// [`super::Config`]
  pub fn from_config(c: crate::Config) -> Self {
    StatsRepo {
      rustcs:    c.rustc().map(|x| x.to_vec()).unwrap_or_else(Vec::new),
      crate_dir: super::CrateDir::new(c.root(), "crates-"),
    }
  }
  /// Return the configured list of Rust targets
  pub fn rustcs(&self) -> &Vec<String> { &self.rustcs }

  /// Return an iterator that traverses all crate names in the repo
  pub fn crate_names_iterator(&self) -> impl Iterator<Item = Result<String, super::CrateDirError>> + '_ {
    self.crate_dir.crate_ids()
  }

  /// Returns a sorted [`Vec`] of all crate names in the repo, or
  /// returning an Err() when an unhandlable error comes through
  /// the [`Iterator`]
  pub fn crate_names(&self) -> Result<Vec<String>, StatsRepoError> {
    use super::CrateDirError;
    self
      .crate_names_iterator()
      .filter(|e| !matches!(e, Err(CrateDirError::NonSection(..))))
      .collect::<Result<Vec<String>, CrateDirError>>()
      .map(|mut i| {
        i.sort_unstable();
        i
      })
      .map_err(Into::into)
  }

  /// Return the path to named crate
  pub fn crate_path(&self, my_crate: &str) -> Result<PathBuf, StatsRepoError> {
    self.crate_dir.abs_path_to(my_crate).map_err(Into::into)
  }

  /// Return the path to a `file` for the named crate
  pub fn crate_file_path(&self, my_crate: &str, file: &str) -> Result<PathBuf, StatsRepoError> {
    self.crate_dir.abs_path_to_file(my_crate, file).map_err(Into::into)
  }

  /// Return the path to a `versions.json` for the named crate
  pub fn crate_versions_path(&self, my_crate: &str) -> Result<PathBuf, StatsRepoError> {
    self.crate_file_path(my_crate, "versions.json")
  }

  /// Return the path to a `results.json` for the named crate
  pub fn crate_results_path(&self, my_crate: &str) -> Result<PathBuf, StatsRepoError> {
    self.crate_file_path(my_crate, "results.json")
  }

  /// Return the path to an `index.html` for the named crate
  pub fn crate_index_path(&self, my_crate: &str) -> Result<PathBuf, StatsRepoError> {
    self.crate_file_path(my_crate, "index.html")
  }

  /// Return a [`super::VersionList`] for the named crate
  pub fn crate_versions(&self, my_crate: &str) -> Result<super::VersionList, StatsRepoError> {
    Ok(super::VersionList::from_path(&self.crate_versions_path(my_crate)?)?)
  }

  /// Return a [`super::ResultList`] for the named crate
  pub fn crate_results(&self, my_crate: &str) -> Result<super::ResultList, StatsRepoError> {
    Ok(super::ResultList::from_path(&self.crate_results_path(my_crate)?)?)
  }
}

impl AsRef<StatsRepo> for StatsRepo {
  fn as_ref(&self) -> &Self { &self }
}
