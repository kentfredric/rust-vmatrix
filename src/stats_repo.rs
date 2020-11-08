use std::path::PathBuf;

pub struct StatsRepo {
  root: PathBuf,
}

pub fn from_config(c: super::config::Config) -> StatsRepo { StatsRepo { root: c.stats_repo.root } }

impl StatsRepo {
  pub fn crate_versions_path<C>(&self, my_crate: C) -> PathBuf
  where
    C: AsRef<str>,
  {
    self.root.join(my_crate.as_ref()).join("versions.json")
  }
}
