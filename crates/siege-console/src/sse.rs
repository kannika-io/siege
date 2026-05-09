use dioxus::prelude::*;
use siege_api_client::SseEvent;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::EventSource;

use crate::state::AppState;

pub fn use_sse_subscription() {
    let state = use_context::<AppState>();

    use_hook(|| {
        let api_url = state.api_url;
        let mut topics = state.topics;

        let url = format!("{api_url}/api/events");
        let es = EventSource::new(&url).expect("failed to connect SSE");

        let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            let Some(data) = e.data().as_string() else {
                return;
            };
            let Ok(event) = serde_json::from_str::<SseEvent>(&data) else {
                return;
            };
            match event {
                SseEvent::TopicsSnapshot { topics: t } => {
                    topics.set(t);
                }
                SseEvent::TopicCreated { topic } => {
                    topics.write().push(topic);
                }
                SseEvent::TopicDeleted { name } => {
                    topics.write().retain(|t| t.name != name);
                }
            }
        }) as Box<dyn FnMut(_)>);

        es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
    });
}
