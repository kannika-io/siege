use dioxus::prelude::*;
use siege_core::Topic;

use crate::state::AppState;

#[component]
pub fn TopicList() -> Element {
    let state = use_context::<AppState>();
    let topics = state.topics;

    rsx! {
        h2 { class: "font-medieval text-gold text-2xl", "The Kingdom" }
        if topics().is_empty() {
            div { class: "text-center py-12 text-parchment-dim italic",
                "Discovering topics..."
            }
        } else {
            div { class: "grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] gap-6 mt-6",
                for topic in topics() {
                    TopicCard { topic }
                }
            }
        }
    }
}

#[component]
fn TopicCard(topic: Topic) -> Element {
    let mut state = use_context::<AppState>();
    let name = topic.name.clone();

    rsx! {
        div {
            class: "bg-stone-700 border border-stone-600 rounded-lg p-6 cursor-pointer transition-all duration-200 hover:border-gold hover:-translate-y-0.5 hover:shadow-[0_4px_12px_rgba(201,168,76,0.15)]",
            onclick: move |_| {
                let client = state.client();
                let name = name.clone();
                spawn(async move {
                    if let Ok(detail) = client.get_topic(&name).await {
                        state.selected_topic.set(Some(detail));
                    }
                });
            },
            h3 { class: "font-medieval text-gold text-xl mb-3", "{topic.name}" }
            div { class: "text-parchment-dim text-sm flex gap-4",
                span { "Partitions: {topic.partitions}" }
                span { "RF: {topic.replication_factor}" }
            }
        }
    }
}
