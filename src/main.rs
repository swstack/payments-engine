mod payments;
mod ingestion;
mod errors;

use std::env;

fn main() {
    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);
}
