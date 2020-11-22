fn main() {
  use std::path::Path;
  let cfg = vmatrix::Config::from_path(Path::new("./vmatrix.toml")).unwrap();
  let repo = vmatrix::stats_repo::from_config(cfg);
  repo.crate_names_iterator();
}
