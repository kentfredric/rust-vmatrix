pub mod config;
pub mod err;
pub mod pages;
pub mod results;
pub mod stats_repo;
pub mod versions;

#[cfg(test)]

mod tests {

  #[test]

  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
