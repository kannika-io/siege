mod app;
mod chaos_action;
mod components;
mod layouts;
mod pages;
mod routes;
mod sse;
mod state;

use components::ui::toast::{ToastContainer, Toaster};
use dioxus::prelude::*;
use routes::Route;
use state::{AppState, Theme};

const API_URL: &str = "http://localhost:51363";

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let theme = use_signal(Theme::load);

    use_context_provider(|| AppState {
        theme,
        api_url: API_URL,
    });
    use_context_provider(Toaster::new);

    rsx! {
        Router::<Route> {}
        ToastContainer {}
    }
}
