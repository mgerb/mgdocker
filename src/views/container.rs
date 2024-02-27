use leptos::*;

use crate::container::Container;

#[component]
pub fn ContainerComponent(c: Container) -> impl IntoView {
    let update_url = format!("/containers/update/{}", c.id);
    view! {
        <details>
            <summary>
                {c.names}
            </summary>
            <button hx-get={update_url} hx-swap="innerHTML" hx-target="next #update_results" hx-indicator="">Update</button>
            <div id="loading" style="display:none">"Loading..."</div>
            <div id="update_results"></div>
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
                    <ContainerComponent c={c} />
                }
            }
        />
    }
}
