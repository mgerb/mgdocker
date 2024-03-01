use leptos::{component, view, IntoView};

use crate::model::AppPage;

#[component]
pub fn AppComponent(app_page: AppPage) -> impl IntoView {
    let ap = app_page.clone();
    let index_link = view! {
        <a href="/" class={move || if ap == AppPage::Index {"current"} else {""}}>Containers</a>
    };

    let ap = app_page.clone();
    let images_link = view! {
        <a href="/images" class={move || if ap == AppPage::Images {"current"} else {""}}>Images</a>
    };

    view! {
        <header style="margin-bottom:1rem">
            <h1>mgdocker</h1>
            <nav>
                {index_link}
                {images_link}
            </nav>
        </header>
        {match app_page {
            AppPage::Index => view! {
                <div style="word-break:break-word" hx-get="/components/containers" hx-trigger="load"></div>
            },
            AppPage::Images => view! {
                <div style="word-break:break-word" hx-get="/components/images" hx-trigger="load"></div>
            },
        }}
    }
}
