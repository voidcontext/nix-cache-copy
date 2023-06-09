use std::collections::HashMap;

use async_trait::async_trait;
use clap::ValueEnum;
use serde::Deserialize;
use tokio::process::Command;

use crate::{DrvFile, Error, StorePath};

#[async_trait]
pub trait CopyCommand {
    async fn store_path(&self, path: &StorePath) -> anyhow::Result<()>;
    async fn drv_output(&self, drv: &DrvFile) -> anyhow::Result<()>;
    fn to(&self) -> String;
}

#[derive(Clone)]
pub struct CliProcess {
    dry_run: bool,
    to: String,
    compression: Compression,
    secret_key: String,
}

impl CliProcess {
    pub fn new(dry_run: bool, to: &str, compression: &Compression, secret_key: &str) -> Self {
        Self {
            dry_run,
            to: to.to_string(),
            compression: (*compression).clone(),
            secret_key: secret_key.to_string(),
        }
    }
}

#[derive(Clone, ValueEnum, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Compression {
    None,
    Xz,
    Bzip2,
    Gzip,
    Zstd,
}

type Derivation = HashMap<String, DerivationInfo>;

#[derive(Debug, Deserialize)]
struct DerivationInfo {
    outputs: HashMap<String, DerivationOutput>,
}

#[derive(Debug, Deserialize)]
struct DerivationOutput {
    path: StorePath,
}

#[async_trait]
impl CopyCommand for CliProcess {
    async fn store_path(&self, path: &StorePath) -> anyhow::Result<()> {
        println!("copying path: {path:?}");

        if !self.dry_run {
            let mut child = Command::new("nix")
                .args([
                    "copy",
                    "--to",
                    format!(
                        "{}?compression={}&secret-key={}",
                        self.to, self.compression, self.secret_key
                    )
                    .as_str(),
                    path,
                ])
                .spawn()?;

            child.wait().await?;
        }

        Ok(())
    }

    async fn drv_output(&self, drv: &DrvFile) -> anyhow::Result<()> {
        println!("copying derivation output: {drv:?}");

        if self.dry_run {
            Ok(())
        } else {
            let output = Command::new("nix")
                .args(["show-derivation", drv])
                .output()
                .await?;

            let output_str = std::str::from_utf8(&output.stdout)?;
            let derivation: Derivation = serde_json::from_str(output_str)?;

            let store_path = &derivation
                .get(&drv.to_string())
                .ok_or_else(|| Error::new("Coulnd't find derivation in the file"))?
                .outputs
                .get("out")
                .ok_or_else(|| Error::new("Couldn't find 'out' attr of derivation"))?
                .path;

            self.store_path(store_path).await
        }
    }

    fn to(&self) -> String {
        self.to.clone()
    }
}
