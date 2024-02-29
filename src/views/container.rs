use leptos::*;

use crate::{container::Container, model::SseTask};

#[component]
pub fn ContainerComponent(c: Container) -> impl IntoView {
    let pull_url = format!("/containers/{}/{}", c.names, SseTask::Pull);
    let update_url = format!("/containers/{}/{}", c.names, SseTask::Update);
    let config_url = format!("/containers/{}/{}", c.names, SseTask::GetConfig);
    view! {
        <details>
            <summary>
                {c.names}
            </summary>
            <div style="display:flex;gap:0.5rem">
                <button
                    hx-get=pull_url
                    hx-swap="innerHTML"
                    hx-target="next #task_container"
                    title="docker compose pull"
                    hx-indicator="next #loader"
                >
                    "Pull"
                </button>
                <button
                    hx-get=update_url
                    hx-swap="innerHTML"
                    hx-target="next #task_container"
                    hx-indicator="next #loader"
                    title="docker compose down && docker compose up -d"
                >
                    "Update"
                </button>
                <button
                    hx-get=config_url
                    hx-swap="innerHTML"
                    hx-target="next #task_container"
                    hx-indicator="next #loader"
                >
                    "View Config"
                </button>
            </div>
            <div id="loader" class="htmx-indicator">"Loading..."</div>
            <div id="task_container"></div>
            <div><b>"id: "</b>{c.id}</div>
            <div><b>"image: "</b> {c.image}</div>
            <div><b>"status: "</b> {c.status}</div>
            <div><b>"state: "</b> {c.state}</div>
            <div><b>"ports: "</b> {c.ports}</div>
            <div><b>"created at: "</b> {c.created_at}</div>
            <div><b>"running for: "</b> {c.running_for}</div>
            <div><b>"size: "</b> {c.size}</div>
            <div><b>"mounts: "</b> {c.mounts}</div>
            <div><b>"networks: "</b> {c.networks}</div>
            <div><b>"local volumes: "</b> {c.local_volumes}</div>
            <div><b>"labels: "</b> {c.labels}</div>
        </details>
    }
}

#[component]
pub fn ContainerListComponent(containers: Vec<Container>) -> impl IntoView {
    let (containers, _) = create_signal::<Vec<Container>>(containers);

    view! {
        <For
            each=move || containers.get()
            key=|c| c.id.clone()
            children=move |c: Container| {
                view! {
                    <ContainerComponent c=c />
                }
            }
        />
    }
}

#[component]
pub fn ContainerSseResultsComponent(name: String, task: SseTask) -> impl IntoView {
    let sse_connect = format!("/containers/sse/{}/{}", name.clone(), task);
    view! {
        <pre id=name.clone() style="max-height:20rem;overflow:auto;"
             hx-on:htmx:after-settle="this.scrollTo(0, this.scrollHeight);"
            >
            <code
                hx-ext="sse"
                sse-connect=sse_connect
                sse-swap=name.clone()
                hx-swap="beforeend"
            ></code>
        </pre>
    }
}
