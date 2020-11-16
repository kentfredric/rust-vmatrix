use std::{collections::HashMap, path::PathBuf};

pub struct StatsRepoCache<'a> {
  repo:           &'a super::stats_repo::StatsRepo,
  crate_names:    Option<Vec<String>>,
  crate_paths:    HashMap<String, PathBuf>,
  crate_versions: HashMap<String, super::versions::VersionList>,
  crate_results:  HashMap<String, super::results::ResultList>,
}

pub fn for_repo(repo: &'_ super::stats_repo::StatsRepo) -> StatsRepoCache {
  StatsRepoCache {
    repo,
    crate_names: Option::None,
    crate_paths: HashMap::new(),
    crate_versions: HashMap::new(),
    crate_results: HashMap::new(),
  }
}

impl StatsRepoCache<'_> {
  pub fn root(&self) -> Result<PathBuf, Error> { Ok(self.repo.root()?) }

  pub fn rustcs(&self) -> Vec<String> { self.repo.rustcs().to_vec() }

  pub fn crate_names(&mut self) -> Result<Vec<String>, Error> {
    if self.crate_names.is_none() {
      self.crate_names.replace(self.repo.crate_names()?);
    }
    if let Some(cached) = &self.crate_names {
      Ok(cached.to_vec())
    } else {
      Err(Error::ProvisionFail("cache_names".to_string()))
    }
  }

  pub fn crate_path<C>(&mut self, my_crate: C) -> Result<PathBuf, Error>
  where
    C: AsRef<str>,
  {
    if !self.crate_paths.contains_key(my_crate.as_ref()) {
      self.crate_paths.insert(my_crate.as_ref().into(), self.repo.crate_path(my_crate.as_ref())?);
    }
    if let Some(cached) = &self.crate_paths.get(my_crate.as_ref()) {
      Ok(cached.to_path_buf())
    } else {
      Err(Error::ProvisionFail("crate_path".to_string()))
    }
  }

  pub fn crate_versions<C>(&mut self, my_crate: C) -> Result<super::versions::VersionList, Error>
  where
    C: AsRef<str>,
  {
    if !self.crate_versions.contains_key(my_crate.as_ref()) {
      self.crate_versions.insert(my_crate.as_ref().into(), self.repo.crate_versions(my_crate.as_ref())?);
    }
    if let Some(cached) = &self.crate_versions.get(my_crate.as_ref()) {
      Ok(cached.to_vec())
    } else {
      Err(Error::ProvisionFail("crate_versions".to_string()))
    }
  }

  pub fn crate_results<C>(&mut self, my_crate: C) -> Result<super::results::ResultList, Error>
  where
    C: AsRef<str>,
  {
    if !self.crate_results.contains_key(my_crate.as_ref()) {
      self.crate_results.insert(my_crate.as_ref().into(), self.repo.crate_results(my_crate.as_ref())?);
    }
    if let Some(cached) = &self.crate_results.get(my_crate.as_ref()) {
      Ok(cached.to_owned().to_owned())
    } else {
      Err(Error::ProvisionFail("crate_results".to_string()))
    }
  }
}

#[derive(Debug)]
pub enum Error {
  ProvisionFail(String),
  StatsRepoError(super::stats_repo::Error),
}

impl From<super::stats_repo::Error> for Error {
  fn from(e: super::stats_repo::Error) -> Self { Self::StatsRepoError(e) }
}
