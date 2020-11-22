use std::ops::Deref;
use vmatrix::stats_repo_cache;

use vmatrix::ResultType;

#[derive(Debug)]
struct ResultBlock {
  result: ResultType,
  min:    String,
  max:    String,
}

fn main() {
  use std::path::Path;
  let cfg = vmatrix::Config::from_path(Path::new("./vmatrix.toml")).unwrap();
  let repo = vmatrix::StatsRepo::from_config(cfg);
  let mut cache = stats_repo_cache::for_repo(&repo);
  for rustc in cache.rustcs() {
    for s in cache.crate_names().unwrap() {
      if let Ok(results) = cache.crate_results(&s) {
        let mut bundles = Vec::new();
        let mut last_state = Option::None;
        let mut block_min = Option::None;
        let mut block_max = Option::None;
        let mut saw_pass = false;

        for v in results.deref() {
          let my_result = v.rustc_result(&rustc);
          if ResultType::Pass == my_result {
            saw_pass = true;
          }
          if last_state.clone().is_none() {
            last_state.replace(my_result);
            block_min.replace(v.version());
            block_max.replace(v.version());
          } else if last_state.clone().unwrap().eq(&my_result) {
            block_max.replace(v.version());
          } else {
            bundles.push(ResultBlock {
              result: last_state.clone().unwrap(),
              min:    block_min.unwrap().clone(),
              max:    block_max.unwrap().clone(),
            });
            last_state.replace(my_result);
            block_min.replace(v.version());
            block_max.replace(v.version());
          }
        }
        bundles.push(ResultBlock {
          result: last_state.clone().unwrap(),
          min:    block_min.unwrap().clone(),
          max:    block_max.unwrap().clone(),
        });
        if !saw_pass {
          continue;
        }
        print!("rust \x1B[33m{}\x1B[0m  {:#?} ", rustc, s);
        for bundle in bundles {
          let range = if bundle.min.eq(&bundle.max) { bundle.min } else { format!("{} -> {}", bundle.min, bundle.max) };
          match bundle.result {
            | ResultType::Pass => print!("\x1B[32m{}\x1B[0m ", range),
            | ResultType::Fail => print!("\x1B[31m{}\x1B[0m ", range),
            | ResultType::Unknown => print!("{} ", range),
          }
        }
        println!()
      }
    }
  }
}
