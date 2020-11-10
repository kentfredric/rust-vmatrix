use vmatrix::config;

fn main() {
  let cfg = config::from_file("./vmatrix.toml").unwrap();

  dbg!(cfg.targets.unwrap().rustc);
}
