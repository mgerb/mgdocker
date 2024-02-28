use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::{wrappers::LinesStream, StreamExt};

use anyhow::{Context, Result};
use tokio::sync::broadcast;

use crate::ContainerSseEvent;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Container {
    #[serde(alias = "ID")]
    pub id: String,
    #[serde(alias = "Image")]
    pub image: String,
    #[serde(alias = "Command")]
    pub command: String,
    #[serde(alias = "CreatedAt")]
    pub created_at: String,
    #[serde(alias = "RunningFor")]
    pub running_for: String,
    #[serde(alias = "Ports")]
    pub ports: String,
    #[serde(alias = "Status")]
    pub status: String,
    #[serde(alias = "Size")]
    pub size: String,
    #[serde(alias = "Names")]
    pub names: String,
    #[serde(alias = "Labels")]
    pub labels: String,
    #[serde(alias = "Mounts")]
    pub mounts: String,
    #[serde(alias = "Networks")]
    pub networks: String,
    #[serde(alias = "State")]
    pub state: String,
    #[serde(alias = "LocalVolumes")]
    pub local_volumes: String,
}

impl Container {
    pub fn get_all() -> Result<Vec<Container>> {
        let output = Command::new("docker")
            .arg("ps")
            .arg("--all")
            .arg("--no-trunc")
            .arg("--format")
            .arg("'{{json .}}'")
            .output()?;

        let output = String::from_utf8(output.stdout)?;

        let mut output: Vec<Container> = output
            .split("\n")
            .filter(|val| !val.is_empty())
            .map(|val| val.trim_matches('\''))
            .map(|val| serde_json::from_str::<Container>(val))
            .filter_map(|val| val.ok())
            .filter(move |val| {
                val.labels
                    .contains("com.docker.compose.project.config_files")
            })
            .collect::<Vec<_>>();

        output.sort_by(|a, b| a.names.cmp(&b.names));

        Ok(output)
    }

    pub async fn update(id: String, tx: &broadcast::Sender<ContainerSseEvent>) -> Result<()> {
        let output = Command::new("docker")
            .arg("inspect")
            .arg(&id)
            .arg("--format")
            .arg("'{{ index .Config.Labels \"com.docker.compose.project.config_files\" }}'")
            .output()?;

        let output = String::from_utf8(output.stdout)?;

        let output = output.trim_matches('\'');

        let output = output
            .split("/")
            .map(|val| val.to_string())
            .collect::<Vec<String>>();

        let output = (&output[0..output.len() - 1]).join("/");

        tx.send(ContainerSseEvent {
            event: id.clone(),
            data: "docker compose pull\n".into(),
        })
        .context("stdout send error")?;

        let mut update = tokio::process::Command::new("docker")
            .arg("compose")
            .arg("pull")
            .current_dir(output)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = update.stdout.take().context("stdout take error")?;
        let stderr = update.stderr.take().context("stderr take error")?;

        let stdout = LinesStream::new(BufReader::new(stdout).lines());
        let stderr = LinesStream::new(BufReader::new(stderr).lines());

        let mut merged = StreamExt::merge(stdout, stderr);

        while let Some(line) = merged.next().await {
            let line = line?;
            let evt = ContainerSseEvent {
                event: id.clone(),
                data: format!("{}\n", line),
            };
            tx.send(evt).context("stdout send error")?;
        }

        // NOTE: for testing purposes
        // for _ in 0..100 {
        //     let evt = ContainerSseEvent {
        //         event: id.clone(),
        //         data: "test 123\n".into(),
        //     };
        //     tx.send(evt).context("stdout send error")?;
        //     tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        // }

        Ok(())
    }
}
