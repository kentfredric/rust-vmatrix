use vmatrix::{config, stats_repo, versions};

fn main() {
  let cfg = config::from_file("./vmatrix.toml").unwrap();
  let repo = stats_repo::from_config(cfg);
  let jsfile = repo.crate_versions_path("time");
  let versions = versions::from_file(jsfile);
}
