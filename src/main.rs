extern crate dotenv;
extern crate failure;
use dotenv::dotenv;

pub mod file;
fn main() {
    dotenv().ok();
    match file::new() {
        Ok(mut file_cutter) => file_cutter.run(),
        Err(err) => println!("{:?}", err),
    }
}
