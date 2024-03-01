use leptos::*;

use crate::{image::Image, model::SseTask};

#[component]
pub fn ImagesComponent(images: Vec<Image>) -> impl IntoView {
    let images = images
        .iter()
        .map(move |image| {
            view! {
                <tr>
                    <td>{image.repository.clone()}</td>
                    <td>{image.tag.clone()}</td>
                    <td>{image.created_since.clone()}</td>
                    <td>{image.size.clone()}</td>
                </tr>
            }
        })
        .collect::<Vec<_>>();

    let prune_url = format!(
        "/components/shared/sse/{}/{}",
        SseTask::PruneImages,
        SseTask::PruneImages
    );
    view! {
        <button
            hx-get=prune_url
            hx-swap="innerHTML"
            hx-target="next #image_task_container"
            title="docker image prune --all --force"
            hx-indicator="next .loader"
        >
            "Prune"
        </button>
        <div class="loader htmx-indicator">"Loading..."</div>
        <div id="image_task_container"></div>
        <table style="width:100%">
            <thead>
                <tr>
                    <th>Repository</th>
                    <th>Tag</th>
                    <th>Created Since</th>
                    <th>Size</th>
                </tr>
            </thead>
            <tbody>
                {images}
            </tbody>
        </table>
    }
}
