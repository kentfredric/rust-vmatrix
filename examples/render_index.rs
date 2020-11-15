use askama::Template;
use vmatrix::pages::index;

fn main() { println!("{}", index::IndexPage::new().render().unwrap()) }
