use std::{
    ffi::OsStr,
    io::{self, BufRead},
};

use clap::Parser;
use nix::Compression;
use nova::newtype;

mod bootstrap;
mod nix;
mod parser;
mod worker;

#[newtype(serde, new, borrow = "str", derive(Debug, PartialEq, Clone))]
pub type StorePath = String;

#[newtype(new, borrow = "str", derive(Debug, PartialEq, Clone))]
pub type DrvFile = String;

#[newtype(new, borrow = "str", derive(Debug, PartialEq, Clone))]
pub type BinaryCache = String;

impl AsRef<OsStr> for DrvFile {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self)
    }
}

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: &str) -> Self {
        Error {
            message: String::from(message),
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Parser error: {}", self.message))
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(long, short = 't')]
    to: String,

    #[clap(long, default_value_t = false)]
    dry_run: bool,

    #[clap(long, default_value_t = Compression::None)]
    compression: Compression,

    #[clap(long, short = 'k')]
    secret_key: String,

    #[clap(long, default_value_t = false)]
    skip_cached: bool,
}

#[tokio::main]
async fn main() {
    let stdin = io::stdin();

    let cli = Cli::parse();

    bootstrap::run(
        stdin.lock().lines(),
        nix::CliProcess::new(cli.dry_run, &cli.to, &cli.compression, &cli.secret_key),
        cli.skip_cached,
    )
    .await;
}
