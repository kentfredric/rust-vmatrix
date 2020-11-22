use vmatrix;

fn main() {
  let cfg = vmatrix::config_from_file("./vmatrix.toml").unwrap();

  dbg!(cfg.rustc().unwrap());
}
