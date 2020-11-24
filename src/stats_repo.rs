use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum StatsRepoError {
  #[error("IO Error in Stats Directory: {0}")]
  IoError(#[from] std::io::Error),
  #[error("Error mapping to/from crate directory stats layout: {0}")]
  CrateDirError(#[from] super::CrateDirError),
  #[error("Error loading results: {0}")]
  ResultsError(#[from] super::ResultsError),
  #[error("Error loading versions: {0}")]
  VersionsError(#[from] super::VersionsError),
}

#[derive(Debug)]
pub struct StatsRepo {
  root:      PathBuf,
  crate_dir: super::CrateDir,
  rustcs:    Vec<String>,
}

impl StatsRepo {
  pub fn from_config(c: crate::Config) -> Self {
    StatsRepo {
      root:      c.root().to_owned(),
      rustcs:    c.rustc().map(|x| x.to_vec()).unwrap_or_else(Vec::new),
      crate_dir: super::CrateDir::new(c.root(), "crates-"),
    }
  }

  pub fn root(&self) -> Result<PathBuf, StatsRepoError> { Ok(self.root.to_owned()) }

  pub fn rustcs(&self) -> &Vec<String> { &self.rustcs }

  pub fn crate_names_iterator(&self) -> impl Iterator<Item = Result<String, super::CrateDirError>> + '_ {
    self.crate_dir.crate_ids()
  }

  pub fn crate_names(&self) -> Result<Vec<String>, StatsRepoError> {
    use super::CrateDirError;
    Ok(
      self
        .crate_names_iterator()
        .filter(|e| !matches!(e, Err(CrateDirError::NonSection(..))))
        .collect::<Result<Vec<String>, CrateDirError>>()
        .map(|mut i| {
          i.sort_unstable();
          i
        })?,
    )
  }

  pub fn crate_path<C>(&self, my_crate: C) -> Result<PathBuf, StatsRepoError>
  where
    C: AsRef<str>,
  {
    Ok(self.crate_dir.abs_path_to(my_crate.as_ref())?)
  }

  pub fn crate_file_path<C, F>(&self, my_crate: C, file: F) -> Result<PathBuf, StatsRepoError>
  where
    C: AsRef<str>,
    F: AsRef<str>,
  {
    Ok(self.crate_dir.abs_path_to_file(my_crate.as_ref(), file.as_ref())?)
  }

  pub fn crate_versions_path<C>(&self, my_crate: C) -> Result<PathBuf, StatsRepoError>
  where
    C: AsRef<str>,
  {
    self.crate_file_path(my_crate, "versions.json")
  }

  pub fn crate_results_path<C>(&self, my_crate: C) -> Result<PathBuf, StatsRepoError>
  where
    C: AsRef<str>,
  {
    self.crate_file_path(my_crate, "results.json")
  }

  pub fn crate_index_path<C>(&self, my_crate: C) -> Result<PathBuf, StatsRepoError>
  where
    C: AsRef<str>,
  {
    self.crate_file_path(my_crate, "index.html")
  }

  pub fn crate_versions<C>(&self, my_crate: C) -> Result<super::VersionList, StatsRepoError>
  where
    C: AsRef<str>,
  {
    Ok(super::VersionList::from_path(&self.crate_versions_path(my_crate)?)?)
  }

  pub fn crate_results<C>(&self, my_crate: C) -> Result<super::ResultList, StatsRepoError>
  where
    C: AsRef<str>,
  {
    Ok(super::ResultList::from_path(&self.crate_results_path(my_crate)?)?)
  }
}

impl AsRef<StatsRepo> for StatsRepo {
  fn as_ref(&self) -> &Self { &self }
}
