use std::{
    convert,
    fmt::{self, Display, Formatter},
};

use tokio::sync::broadcast;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContainerSseEvent {
    pub event: String,
    pub data: String,
}

pub struct AppState {
    pub tx: broadcast::Sender<ContainerSseEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SseTask {
    Update,
    Pull,
    GetConfig,
}

impl SseTask {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Update => "update",
            Self::Pull => "pull",
            Self::GetConfig => "get_config",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "update" => Some(Self::Update),
            "pull" => Some(Self::Pull),
            "get_config" => Some(Self::GetConfig),
            _ => None,
        }
    }
}

impl convert::From<SseTask> for String {
    fn from(task: SseTask) -> Self {
        task.to_str().to_string()
    }
}

impl Display for SseTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
