use leptos::*;

use crate::model::SseTask;

#[component]
pub fn SseResultsComponent(name: String, task: SseTask) -> impl IntoView {
    let sse_connect = format!("/components/shared/sse/connect/{}/{}", name.clone(), task);
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
