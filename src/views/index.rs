use leptos::{component, view, IntoView};

use crate::views::app::AppComponent;

#[component]
pub fn IndexComponent() -> impl IntoView {
    view! {
        <html>
            <head>
                <title>"mgdocker"</title>
                <link rel="stylesheet" href="https://cdn.simplecss.org/simple.min.css"/>
                <link rel="stylesheet" href="/index.css"/>
                <script src="https://unpkg.com/htmx.org@1.9.10"></script>
                <script src="https://unpkg.com/htmx.org/dist/ext/sse.js"></script>
            </head>
            <body>
                <AppComponent />
            </body>
        </html>
    }
}
