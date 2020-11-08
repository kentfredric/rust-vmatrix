use vmatrix::{config, versions};

fn main() {
  let cfg = config::from_file("./vmatrix.toml").unwrap();
  let root = cfg.stats_repo.root;
  let jsfile = root.join("time/versions.json");
  let versions = versions::from_file(jsfile);
  dbg!(versions);
}
