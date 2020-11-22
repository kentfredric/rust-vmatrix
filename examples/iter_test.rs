use vmatrix::stats_repo;

fn main() {
  let cfg = vmatrix::config_from_file("./vmatrix.toml").unwrap();
  let repo = stats_repo::from_config(cfg);
  repo.crate_names_iterator();
}
