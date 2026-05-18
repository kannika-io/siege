use dioxus::prelude::*;
use siege_api_client::SseEvent;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::EventSource;

use crate::state::{AppState, TopicsState};

pub fn use_sse_subscription() {
    let app = use_context::<AppState>();
    let mut topics_state = use_context::<TopicsState>();

    use_hook(|| {
        let api_url = app.api_url;

        let url = format!("{api_url}/api/events");
        let Ok(es) = EventSource::new(&url) else {
            return;
        };

        let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            let Some(data) = e.data().as_string() else {
                return;
            };
            let Ok(event) = serde_json::from_str::<SseEvent>(&data) else {
                return;
            };
            match event {
                SseEvent::TopicsSnapshot { topics } => {
                    topics_state.set_topics(topics);
                }
                SseEvent::TopicCreated { topic }
                | SseEvent::TopicUpdated { topic }
                | SseEvent::ChaosRetentionLowered { topic }
                | SseEvent::ChaosCleanupPolicyFlipped { topic }
                | SseEvent::ChaosPartitionsIncreased { topic } => {
                    topics_state.upsert_topic(topic);
                }
                SseEvent::TopicDeleted { name } | SseEvent::ChaosTopicDeleted { topic: name } => {
                    topics_state.remove_topic(&name);
                }
                SseEvent::TopicsSeeded { topics } => {
                    for t in topics {
                        topics_state.upsert_topic(t);
                    }
                }
                SseEvent::ChaosPoisonPillsSent { .. }
                | SseEvent::ChaosSchemaBreakSent { .. }
                | SseEvent::SeedProgress { .. } => {}
            }
        }) as Box<dyn FnMut(_)>);

        es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
    });
}
