use vmatrix::config;

fn main() {
  let cfg = config::from_file("./vmatrix.toml").unwrap();

  dbg!(cfg.stats_repo.root_path().unwrap());
}
