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

    fn get_compose_config_file_path(name: String) -> Result<String> {
        let output = Command::new("docker")
            .arg("inspect")
            .arg(&name)
            .arg("--format")
            .arg("'{{ index .Config.Labels \"com.docker.compose.project.config_files\" }}'")
            .output()?;

        let output = String::from_utf8(output.stdout)?;

        if output.is_empty() {
            return Err(anyhow::anyhow!("no compose file found"));
        }

        let output = output.replace("'", "");
        let output = output.trim(); // remove newline

        Ok(output.into())
    }

    fn get_compose_dir(name: String) -> Result<String> {
        let output = Self::get_compose_config_file_path(name)?;

        let output = output
            .split("/")
            .map(|val| val.to_string())
            .collect::<Vec<String>>();

        let output = (&output[0..output.len() - 1]).join("/");

        Ok(output)
    }

    async fn execute_command(
        name: &str,
        mut cmd: tokio::process::Child,
        tx: &broadcast::Sender<ContainerSseEvent>,
    ) -> Result<()> {
        let stdout = cmd
            .stdout
            .take()
            .context("execute_command: stdout take error")?;
        let stderr = cmd
            .stderr
            .take()
            .context("execute_command: stderr take error")?;

        let stdout = LinesStream::new(BufReader::new(stdout).lines());
        let stderr = LinesStream::new(BufReader::new(stderr).lines());

        let mut merged = StreamExt::merge(stdout, stderr);

        while let Some(line) = merged.next().await {
            let line = line?;
            let evt = ContainerSseEvent {
                event: name.into(),
                data: format!("{}\n", line),
            };
            tx.send(evt).context("execute_command: stdout send error")?;
        }

        Ok(())
    }

    /// docker compose pull
    /// broadcast the stdout and stderr results to the sender
    pub async fn pull(name: String, tx: &broadcast::Sender<ContainerSseEvent>) -> Result<()> {
        let dir = Self::get_compose_dir(name.clone())?;

        tx.send(ContainerSseEvent {
            event: name.clone(),
            data: "docker compose pull\n".into(),
        })
        .context("pull: stdout send error")?;

        let cmd = tokio::process::Command::new("docker")
            .arg("compose")
            .arg("pull")
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Self::execute_command(&name, cmd, tx).await?;

        Ok(())
    }

    // docker compose pull && docker compose up -d
    pub async fn update(name: String, tx: &broadcast::Sender<ContainerSseEvent>) -> Result<()> {
        let dir = Self::get_compose_dir(name.clone())?;

        tx.send(ContainerSseEvent {
            event: name.clone(),
            data: "docker compose down\n".into(),
        })
        .context("update: stdout send error")?;

        let down = tokio::process::Command::new("docker")
            .arg("compose")
            .arg("down")
            .current_dir(&dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Self::execute_command(&name, down, tx).await?;

        tx.send(ContainerSseEvent {
            event: name.clone(),
            data: "\ndocker compose up -d\n".into(),
        })
        .context("update: stdout send error")?;

        let up = tokio::process::Command::new("docker")
            .arg("compose")
            .arg("up")
            .arg("-d")
            .current_dir(&dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Self::execute_command(&name, up, tx).await?;

        Ok(())
    }

    pub async fn get_config(name: String, tx: &broadcast::Sender<ContainerSseEvent>) -> Result<()> {
        let config_file_path = Self::get_compose_config_file_path(name.clone())?;

        let file = tokio::fs::File::open(config_file_path).await?;
        let reader = BufReader::new(file);

        let mut lines = reader.lines();
        let mut output: Vec<String> = vec![];

        while let Some(line) = lines.next_line().await? {
            output.push(line);
        }

        tx.send(ContainerSseEvent {
            event: name.clone(),
            data: output.join("\n"),
        })
        .context("get_config: stdout send error")?;

        Ok(())
    }
}
