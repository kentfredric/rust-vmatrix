use std::{
  collections::{self, HashMap},
  path::PathBuf,
};

pub struct StatsRepoCache<'a> {
  repo:           &'a super::stats_repo::StatsRepo,
  crate_names:    Option<Vec<String>>,
  crate_paths:    HashMap<String, PathBuf>,
  crate_versions: HashMap<String, super::versions::VersionList>,
  crate_results:  HashMap<String, Option<super::results::ResultList>>,
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
    let my_crate = my_crate.as_ref();
    let repo = self.repo;
    let crate_results = &mut self.crate_results;

    match crate_results.entry(my_crate.to_string()) {
      | collections::hash_map::Entry::Occupied(e) => {
        match e.get() {
          | Some(result) => Ok(result.to_owned()),
          | None => Err(Error::ResultNotExists(my_crate.to_string())),
        }
      },
      | collections::hash_map::Entry::Vacant(e) => {
        match repo.crate_results(my_crate) {
          | Ok(r) => {
            e.insert(Some(r.to_owned()));
            Ok(r)
          },
          | Err(super::stats_repo::Error::ResultsError(super::results::Error::IoError(err)))
            if err.kind() == std::io::ErrorKind::NotFound =>
          {
            e.insert(None);
            Err(err.into())
          }
          | Err(err) => Err(err.into()),
        }
      },
    }
  }
}

#[derive(Debug)]
pub enum Error {
  IoError(std::io::Error),
  ProvisionFail(String),
  ResultNotExists(String),
  StatsRepoError(super::stats_repo::Error),
}

impl From<super::stats_repo::Error> for Error {
  fn from(e: super::stats_repo::Error) -> Self { Self::StatsRepoError(e) }
}
impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}
