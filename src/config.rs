use std::path::{Path, PathBuf};

/// Container for decoded configuration
///
/// # Read from a TOML file
/// ```no_run
/// # use std::path::Path;
/// # use vmatrix::{Config,ConfigError};
/// # fn main() -> Result<(),ConfigError> {
/// let c = Config::from_path(Path::new("./vmatrix.toml"))?;
/// # Ok(())
/// # }
/// ```
///
/// # Parse from a TOML string
/// ```
/// # use std::path::{Path,PathBuf};
/// # use vmatrix::{Config,ConfigError};
/// # fn main() -> Result<(),ConfigError> {
/// let c: Config = r#"
///     [stats_repo]
///     root = "/some/path"
///
///     [targets]
///     rustc = ["1.0.0", "1.1.0"]
/// "#
/// .parse()?;
/// assert_eq!(c.root(), &PathBuf::from("/some/path"));
/// assert_eq!(c.rustc(), Some(&vec!["1.0.0".to_string(), "1.1.0".to_string()]));
/// # Ok(())
/// # }
/// ```

#[derive(serde_derive::Deserialize, Debug)]
pub struct Config {
  stats_repo: StatsRepoConfig,
  targets:    Option<TargetsConfig>,
}

/// Errors in configuration loading/decoding
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
  /// Errors returned by [`toml::from_str`] decode failures
  #[error("Error reading TOML data: {0}")]
  DecodeTomlError(#[from] toml::de::Error),
  /// Errors returned from [`std`] IO calls for paths
  #[error("Error reading Config TOML file: {0}")]
  IoError(#[from] std::io::Error),
}

impl Config {
  /// Returns configured repository root
  pub fn root(&self) -> &PathBuf { &self.stats_repo.root }

  /// Returns a list of configured rust targets
  pub fn rustc(&self) -> Option<&Vec<String>> { self.targets.as_ref().and_then(|x| x.rustc.as_ref()) }

  /// Read a [`Config`] from a TOML file by path
  ///
  /// # Errors
  ///
  /// * If an IO error occurs reading the specified path, a
  ///   [`ConfigError::IoError`] will be
  /// returned.
  ///
  /// * If a parse error occurs while reading string content as TOML,
  ///   a
  /// [`ConfigError::DecodeTomlError`] will be returned.
  pub fn from_path(path: &Path) -> Result<Self, ConfigError> { std::fs::read_to_string(path)?.parse() }
}

impl std::str::FromStr for Config {
  type Err = ConfigError;

  /// Parse a [`Config`] from a TOML string
  ///
  /// # Errors
  ///
  /// * If a parse error occurs while reading a string as TOML, a
  ///   [`ConfigError::DecodeTomlError`]
  /// will be retunred.
  fn from_str(s: &str) -> Result<Self, Self::Err> { toml::from_str(s).map_err(Into::into) }
}

#[derive(serde_derive::Deserialize, Debug)]
struct StatsRepoConfig {
  root: PathBuf,
}

#[derive(serde_derive::Deserialize, Debug)]
struct TargetsConfig {
  rustc: Option<Vec<String>>,
}
