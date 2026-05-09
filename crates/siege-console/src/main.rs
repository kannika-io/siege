mod components;
mod sse;
mod state;

use components::layout::Layout;
use components::topic_detail::TopicDetailPanel;
use components::topic_list::TopicList;
use dioxus::prelude::*;
use siege_core::TopicDetail;
use state::{AppState, Theme};

const API_URL: &str = "http://localhost:8080";

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let topics = use_signal(Vec::new);
    let selected_topic = use_signal(|| None::<TopicDetail>);
    let theme = use_signal(|| Theme::Dark);

    use_context_provider(|| AppState {
        topics,
        selected_topic,
        theme,
        api_url: API_URL,
    });

    sse::use_sse_subscription();

    let selected = (use_context::<AppState>().selected_topic)();

    rsx! {
        Layout {
            TopicList {}
            if let Some(detail) = selected {
                TopicDetailPanel { detail }
            }
        }
    }
}
