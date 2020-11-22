fn main() {
  use std::path::Path;
  let cfg = vmatrix::Config::from_path(Path::new("./vmatrix.toml")).unwrap();

  dbg!(cfg.rustc().unwrap());
}
