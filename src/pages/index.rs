use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexPage {
  contributing: ContributingSection,
}

impl IndexPage {
  pub fn new() -> Self {
    IndexPage {
      contributing: ContributingSection {
        todo_time:     "Sun Nov 15 13:58:18 2020".to_string(),
        todo_crates:   1551,
        todo_targets:  1487182,
        low_estimate:  "80.76 days ( 1938.28 hours ( 116296.72 minutes ( 6977803 seconds ) ) )".to_string(),
        high_estimate: "2.83 years ( 1032.77 days ( 24786.37 hours ( 1487182.00 minutes ( 89230920 seconds ) ) ) )"
          .to_string(),
      },
    }
  }
}

#[derive(Template)]
#[template(path = "index/contributing.html")]
pub struct ContributingSection {
  todo_time:     String,
  todo_crates:   u64,
  todo_targets:  u64,
  low_estimate:  String,
  high_estimate: String,
}
