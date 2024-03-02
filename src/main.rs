mod args;
mod components;
mod container;
mod image;
mod model;
mod util;

use crate::model::{AppState, SseEvent};
use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::{sse::Event, Html, Sse},
    routing::get,
};
use clap::Parser;
use components::{
    container::{ContainerListComponent, ContainerListComponentProps},
    images::{ImagesComponent, ImagesComponentProps},
    index::{IndexComponent, IndexComponentProps},
    shared::sse::{SseResultsComponent, SseResultsComponentProps},
};
use container::Container;
use futures::stream::Stream;
use image::Image;
use leptos::*;
use model::{AppPage, SseTask};
use std::sync::Arc;
use tokio::sync::broadcast;
use util::AppError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = args::Args::parse();

    tracing_subscriber::fmt::init();

    let (tx, _) = broadcast::channel::<SseEvent>(1000);
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
        .route("/components/containers", get(get_containers))
        .route("/components/shared/sse/:name/:task", get(get_sse_task))
        .route(
            "/components/shared/sse/connect/:name/:task",
            get(sse_connect_handler),
        )
        .route("/components/images", get(get_images))
        .route("/", get(get_index_page))
        .route("/images", get(get_images_page))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.host, args.port)).await?;

    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_containers() -> Result<Html<String>, AppError> {
    let containers = Container::get_all()?;
    let containers = ContainerListComponentProps { containers };
    let view = ssr::render_to_string(|| ContainerListComponent(containers));
    Ok(Html(view.into()))
}

async fn get_images() -> Result<Html<String>, AppError> {
    let images = Image::get_all()?;
    let props = ImagesComponentProps { images };
    let view = ssr::render_to_string(|| ImagesComponent(props));
    Ok(Html(view.into()))
}

async fn get_index_page() -> Result<Html<String>, AppError> {
    let props = IndexComponentProps {
        app_page: AppPage::Index,
    };
    let view = ssr::render_to_string(|| IndexComponent(props));
    Ok(render_index(view.to_string()))
}

async fn get_images_page() -> Result<Html<String>, AppError> {
    let props = IndexComponentProps {
        app_page: AppPage::Images,
    };
    let view = ssr::render_to_string(|| IndexComponent(props));
    Ok(render_index(view.to_string()))
}

fn render_index(view: String) -> Html<String> {
    // add doctype here because leptos strips it
    Html(format!("<!DOCTYPE html>{}", view))
}

async fn get_sse_task(
    Path((name, task)): Path<(String, String)>,
) -> Result<Html<String>, AppError> {
    let props = SseResultsComponentProps {
        name,
        task: SseTask::from_str(&task).context("get_containers_task: invalid task")?,
    };
    let view = ssr::render_to_string(|| SseResultsComponent(props));
    Ok(Html(view.into()))
}

async fn sse_connect_handler(
    State(app_state): State<Arc<AppState>>,
    Path((name, task)): Path<(String, String)>,
) -> Result<Sse<impl Stream<Item = Result<Event, anyhow::Error>>>, AppError> {
    let mut rx = app_state.tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(evt) => {
                    if evt.event == "Done" {
                        break;
                    }
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

    let task = SseTask::from_str(&task);

    match task {
        Some(SseTask::Update) => {
            tokio::spawn(async move {
                match Container::update(name, &app_state.tx).await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("sse_handler update error: {}", e),
                }
            });
        }
        Some(SseTask::Pull) => {
            tokio::spawn(async move {
                match Container::pull(name, &app_state.tx).await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("sse_handler pull error: {}", e),
                }
            });
        }
        Some(SseTask::GetConfig) => {
            tokio::spawn(async move {
                match Container::get_config(name, &app_state.tx).await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("sse_handler get config error: {}", e),
                }
            });
        }
        Some(SseTask::PruneImages) => {
            tokio::spawn(async move {
                match Image::prune(SseTask::PruneImages.to_str(), &app_state.tx).await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("sse_handler prune images error: {}", e),
                }
            });
        }
        None => {
            tracing::error!("error: invalid task in sse handler");
        }
    }

    return Ok(Sse::new(stream));
}
