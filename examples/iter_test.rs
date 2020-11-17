use vmatrix::{config, stats_repo};

fn main() {
  let cfg = config::from_file("./vmatrix.toml").unwrap();
  let repo = stats_repo::from_config(cfg);
  repo.crate_names_iterator();
}
