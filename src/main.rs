use std::{
    ffi::OsStr,
    io::{self, BufRead},
};

use clap::Parser;
use nova::newtype;

mod bootstrap;
mod nix;
mod parser;
mod worker;

#[newtype(serde, new, borrow = "str", derive(Debug, PartialEq, Clone))]
pub type StorePath = String;

#[newtype(new, borrow = "str", derive(Debug, PartialEq, Clone))]
pub type DrvFile = String;

impl AsRef<OsStr> for DrvFile {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self)
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(long)]
    to: String,
}

#[tokio::main]
async fn main() {
    let stdin = io::stdin();

    let cli = Cli::parse();

    bootstrap::run(
        stdin.lock().lines(),
        nix::CliProcess::new(true),
        cli.to.clone(),
    )
    .await;
}
