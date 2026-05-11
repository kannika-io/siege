use dioxus::prelude::*;
use siege_api_client::TopicResource;

use super::topic_pills::TopicPills;
use crate::state::{AppState, TopicsState};

#[component]
pub fn TopicList() -> Element {
    let topics_state = use_context::<TopicsState>();
    let topics = topics_state.list;

    rsx! {
        div { class: "flex items-center justify-between px-6 py-4 border-b border-border",
            div { class: "flex items-center gap-3",
                h1 { class: "text-sm font-semibold", "Topics" }
                if !topics().is_empty() {
                    span { class: "text-xs text-muted-foreground bg-muted px-1.5 py-0.5 rounded",
                        "{topics().len()}"
                    }
                }
            }
        }

        if topics().is_empty() {
            div { class: "flex-1 flex items-center justify-center text-muted-foreground text-sm",
                "Waiting for topics\u{2026}"
            }
        } else {
            div { class: "flex-1 overflow-y-auto",
                for topic in topics() {
                    TopicRow { topic }
                }
            }
        }
    }
}

#[component]
fn TopicRow(topic: TopicResource) -> Element {
    let app = use_context::<AppState>();
    let mut topics_state = use_context::<TopicsState>();
    let name = topic.name.clone();

    rsx! {
        div {
            class: "flex items-center justify-between px-6 py-3 border-b border-border cursor-pointer hover:bg-surface-hover transition-colors",
            onclick: move |_| {
                let client = app.client();
                let name = name.clone();
                spawn(async move {
                    if let Ok(detail) = client.get_topic(&name).await {
                        topics_state.selected.set(Some(detail));
                    }
                });
            },
            span { class: "text-sm font-medium", "{topic.name}" }
            TopicPills { partitions: topic.partitions, replication_factor: topic.replication_factor, config: topic.config.clone() }
        }
    }
}
