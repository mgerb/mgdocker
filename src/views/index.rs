use leptos::{component, view, IntoView};

use crate::views::app::AppComponent;

#[component]
pub fn IndexComponent() -> impl IntoView {
    view! {
        <html>
            <head>
                <title>"mgdocker"</title>
                <link rel="stylesheet" href="https://cdn.simplecss.org/simple.min.css"/>
                <script src="https://unpkg.com/htmx.org@1.9.10"></script>
            </head>
            <body>
                <AppComponent />
            </body>
        </html>
    }
}
