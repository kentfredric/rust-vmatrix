type EResult = Result<(), Box<dyn std::error::Error>>;

fn from_file() -> EResult {
  use std::path::Path;
  let cfg = vmatrix::Config::from_path(Path::new("./vmatrix.toml"))?;
  eprintln!("{:#?}", cfg);
  eprintln!("root() {:#?}", cfg.root());
  eprintln!("rustc() {:#?}", cfg.rustc());
  Ok(())
}
fn from_str() -> EResult {
  let cfg: vmatrix::Config = r#"
    [stats_repo]
    root = "/home/kent/rust/vmatrix/"

    [targets]
    rustc = [ "1.46.0", "1.47.0" ]
  "#
  .parse()?;
  eprintln!("{:#?}", cfg);
  eprintln!("root() {:#?}", cfg.root());
  eprintln!("rustc() {:#?}", cfg.rustc());
  Ok(())
}

fn main() -> EResult {
  from_file()?;
  from_str()?;
  Ok(())
}
