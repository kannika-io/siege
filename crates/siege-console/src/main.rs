mod components;
mod layouts;
mod pages;
mod sse;
mod state;

use layouts::default::Layout;
use pages::topics::topic_detail::TopicDetailPanel;
use pages::topics::topic_list::TopicList;
use dioxus::prelude::*;
use siege_api_client::TopicDetailResource;
use state::{AppState, Theme};

const API_URL: &str = "http://localhost:8080";

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let topics = use_signal(Vec::new);
    let selected_topic = use_signal(|| None::<TopicDetailResource>);
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
            div { class: "flex flex-1 overflow-hidden",
                div { class: if selected.is_some() { "w-1/2 flex flex-col overflow-hidden" } else { "flex-1 flex flex-col overflow-hidden" },
                    TopicList {}
                }
                if let Some(ref detail) = selected {
                    div { class: "w-1/2 border-l border-border overflow-y-auto",
                        TopicDetailPanel { detail: detail.clone() }
                    }
                }
            }
        }
    }
}
