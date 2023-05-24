use std::{
    ffi::OsStr,
    io::{self, BufRead},
};

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

#[tokio::main]
async fn main() {
    let stdin = io::stdin();

    bootstrap::run(stdin.lock().lines(), nix::CliProcess::new(true)).await;
}
