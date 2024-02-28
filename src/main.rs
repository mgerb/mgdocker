mod container;
mod util;
mod views;

use std::sync::Arc;
use tokio::sync::broadcast;

use axum::{
    extract::{Path, State},
    response::{sse::Event, Html, Sse},
    routing::get,
};
use container::Container;
use futures::stream::Stream;
use leptos::*;

use util::AppError;
use views::{
    container::{
        ContainerListComponent, ContainerListComponentProps, ContainerUpdateComponent,
        ContainerUpdateComponentProps,
    },
    index::IndexComponent,
};

#[derive(Debug, Clone, serde::Deserialize)]
struct ContainerSseEvent {
    event: String,
    data: String,
}

struct AppState {
    pub tx: broadcast::Sender<ContainerSseEvent>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (tx, _) = broadcast::channel::<ContainerSseEvent>(1000);
    let app_state = Arc::new(AppState { tx });

    let app = axum::Router::new()
        .route(
            "/index.css",
            get(|| async {
                (
                    [("content-type", "text/css")],
                    include_str!("./styles/index.css"),
                )
            }),
        )
        .route("/sse/:id", get(sse_handler))
        .route("/containers", get(get_containers))
        .route("/containers/update/:id", get(get_containers_update))
        .route("/", get(get_index))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2999").await?;

    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_containers_update(Path(id): Path<String>) -> Result<Html<String>, AppError> {
    let props = ContainerUpdateComponentProps { id };
    let view = ssr::render_to_string(|| ContainerUpdateComponent(props));
    Ok(Html(view.into()))
}

async fn get_containers() -> Result<Html<String>, AppError> {
    let containers = Container::get_all()?;
    let containers = ContainerListComponentProps { containers };
    let view = ssr::render_to_string(|| ContainerListComponent(containers));
    Ok(Html(view.into()))
}

async fn get_index() -> Result<Html<String>, AppError> {
    let view = ssr::render_to_string(|| IndexComponent());
    // add doctype here because leptos strips it
    Ok(Html(format!("<!DOCTYPE html>{}", view.to_string())))
}

async fn sse_handler(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, anyhow::Error>>>, AppError> {
    let mut rx = app_state.tx.subscribe();

    let stream = async_stream::stream! {
        loop {

            match rx.recv().await {
                Ok(evt) => {
                    let res: Result<_, anyhow::Error> = Ok(Event::default().data(evt.data).event(evt.event));
                    yield res;
                }
                Err(e) => {
                    tracing::error!("error: {}", e);
                    let res: Result<_, anyhow::Error> = Ok(Event::default().data("close").event("CloseEvent"));
                    yield res;
                    break;
                },
            }
        }
    };

    let app_state = app_state.clone();

    tokio::spawn(async move {
        let _ = Container::update(id, &app_state.tx).await;
    });

    return Ok(Sse::new(stream));
}
