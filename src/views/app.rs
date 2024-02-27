use leptos::{component, view, IntoView};

#[component]
pub fn AppComponent() -> impl IntoView {
    view! {
        <header style="margin-bottom:1rem">
            <h1>mgdocker</h1>
        </header>
        <div style="word-break:break-word" hx-get="/containers" hx-trigger="load">
        </div>
    }
}
