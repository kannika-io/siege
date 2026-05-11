mod components;
mod layouts;
mod pages;
mod sse;
mod state;

use components::ui::toast::{ToastContainer, Toaster};
use layouts::default::Layout;
use pages::topics::TopicsPage;
use dioxus::prelude::*;
use state::{AppState, Theme};

const API_URL: &str = "http://localhost:8080";

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
        Layout {
            TopicsPage {}
        }
        ToastContainer {}
    }
}
