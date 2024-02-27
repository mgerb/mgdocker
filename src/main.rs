mod container;
mod util;
mod views;

use axum::{extract::Path, response::Html, routing::get};
use container::Container;
use leptos::*;

use util::AppError;
use views::{
    container::{ContainerListComponent, ContainerListComponentProps},
    index::IndexComponent,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let app = axum::Router::new()
        .route("/containers", get(get_containers))
        .route("/containers/update/:id", get(get_containers_update))
        .route("/", get(get_index));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2999").await?;

    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_containers_update(Path(id): Path<String>) -> Result<Html<String>, AppError> {
    let (stdout, stderr) = Container::update(id)?;

    let show_stdout = !stdout.is_empty();
    let show_stderr = !stderr.is_empty();
    let view = ssr::render_to_string(move || {
        view! {
            <Show when=move || show_stdout>
                <pre>
                    <code>
                    "stdout:\n"
                    {stdout.clone()}
                    </code>
                </pre>
            </Show>
            <Show when=move || show_stderr>
                <pre>
                    <code>
                    "stderr:\n"
                    {stderr.clone()}
                    </code>
                </pre>
            </Show>
        }
    });

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
