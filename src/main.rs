use std::io::{self, BufRead};

mod bootstrap;
mod parser;
mod worker;

#[tokio::main]
async fn main() {
    let stdin = io::stdin();

    bootstrap::run(stdin.lock().lines()).await;
}
