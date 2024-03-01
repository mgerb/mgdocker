use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use tokio::sync::broadcast;

use crate::{model::SseEvent, util};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Image {
    #[serde(alias = "Containers")]
    pub containers: String,
    #[serde(alias = "CreatedAt")]
    pub created_at: String,
    #[serde(alias = "CreatedSince")]
    pub created_since: String,
    #[serde(alias = "Digest")]
    pub digest: String,
    #[serde(alias = "ID")]
    pub id: String,
    #[serde(alias = "Repository")]
    pub repository: String,
    #[serde(alias = "SharedSize")]
    pub shared_size: String,
    #[serde(alias = "Size")]
    pub size: String,
    #[serde(alias = "Tag")]
    pub tag: String,
    #[serde(alias = "UniqueSize")]
    pub unique_size: String,
    #[serde(alias = "VirtualSize")]
    pub virtual_size: String,
}

impl Image {
    pub fn get_all() -> Result<Vec<Image>> {
        let output = Command::new("docker")
            .arg("images")
            .arg("--all")
            .arg("--no-trunc")
            .arg("--format")
            .arg("'{{json .}}'")
            .output()?;

        let output = String::from_utf8(output.stdout)?;

        let mut output: Vec<Image> = output
            .trim()
            .trim_matches('\'')
            .split("\n")
            .map(|val| val.trim().trim_matches('\''))
            .filter(|val| !val.is_empty())
            .map(|val| serde_json::from_str::<Image>(val))
            .filter_map(|val| val.ok())
            .collect::<Vec<_>>();

        output.sort_by(|a, b| a.repository.cmp(&b.repository));

        Ok(output)
    }

    pub async fn prune(event_name: &str, tx: &broadcast::Sender<SseEvent>) -> Result<()> {
        tx.send(SseEvent {
            event: event_name.into(),
            data: "docker image prune --all --force\n".into(),
        })
        .context("update: stdout send error")?;

        let cmd = tokio::process::Command::new("docker")
            .arg("image")
            .arg("prune")
            .arg("--all")
            .arg("--force")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        util::execute_command(event_name, cmd, tx).await?;

        Ok(())
    }
}
