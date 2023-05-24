use std::collections::HashMap;

use async_trait::async_trait;
use serde::Deserialize;
use tokio::process::Command;

use crate::{DrvFile, StorePath};

#[async_trait]
pub trait Cli {
    async fn copy_store_path(&self, path: &StorePath) -> anyhow::Result<()>;
    async fn copy_drv_output(&self, drv: &DrvFile) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct CliProcess {
    dry_run: bool,
}

impl CliProcess {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }
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
impl Cli for CliProcess {
    async fn copy_store_path(&self, path: &StorePath) -> anyhow::Result<()> {
        println!("copying path: {path:?}");

        if !self.dry_run {
            let mut child = Command::new("nix")
                .args(["copy", "--to", "file:///tmp/store", path])
                .spawn()
                .unwrap();

            child.wait().await.unwrap();
        }

        Ok(())
    }

    async fn copy_drv_output(&self, drv: &DrvFile) -> anyhow::Result<()> {
        println!("copying derivation output: {drv:?}");

        if self.dry_run {
            Ok(())
        } else {
            let output = Command::new("nix")
                .args(["show-derivation", drv])
                .output()
                .await?;

            let output_str = std::str::from_utf8(&output.stdout)?;

            println!("Show derivation output: {output_str:?}");
            let derivation: Derivation = serde_json::from_str(output_str)?;

            println!("Derivaton output to copy: {derivation:?}");

            let store_path = &derivation
                .get(&drv.to_string())
                .unwrap()
                .outputs
                .get("out")
                .unwrap()
                .path;

            self.copy_store_path(store_path).await
        }
    }
}
