extern crate dotenv;
use dotenv::dotenv;

pub mod file;
fn main() {
    dotenv().ok();
    file::new().run();
}
