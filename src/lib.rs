mod config_data;
mod config_reader;

pub mod pages;
pub mod results;
pub mod stats_repo;
pub mod stats_repo_cache;
pub mod versions;

pub fn config_from_str<N>(content: N) -> Result<config_data::Config, self::Error>
where
  N: AsRef<str>,
{
  Ok(config_reader::from_str(content)?)
}

pub fn config_from_file<N>(file: N) -> Result<config_data::Config, self::Error>
where
  N: Into<std::path::PathBuf>,
{
  Ok(config_reader::from_file(file)?)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Configuration Reading Error: {0}")]
  ConfigReaderError(#[from] config_reader::Error),
  #[error("Result Data Error: {0}")]
  ResultsError(#[from] results::Error),
  #[error("Version Data Error: {0}")]
  VersionsError(#[from] versions::Error),
  #[error("Error in Stats Repo: {0}")]
  StatsRepoError(#[from] stats_repo::Error),
}
