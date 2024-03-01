use tokio::io::{AsyncBufReadExt, BufReader};

use anyhow::{Context, Result};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::LinesStream, StreamExt};

use crate::model::SseEvent;

pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

/// Execute a tokio command and broadast the output line by line to the sender channel
pub async fn execute_command(
    event_name: &str,
    mut cmd: tokio::process::Child,
    tx: &broadcast::Sender<SseEvent>,
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
        let evt = SseEvent {
            event: event_name.into(),
            data: format!("{}\n", line),
        };
        tx.send(evt).context("execute_command: stdout send error")?;
    }

    Ok(())
}
