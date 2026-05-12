use dioxus::prelude::*;
use siege_api_client::{SseEvent, TopicResource};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::EventSource;

use crate::state::{AppState, TopicsState};

pub fn use_sse_subscription() {
    let app = use_context::<AppState>();
    let topics_state = use_context::<TopicsState>();

    use_hook(|| {
        let api_url = app.api_url;
        let mut topics = topics_state.list;
        let selected = topics_state.selected;

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
                SseEvent::TopicsSnapshot { topics: t } => {
                    set_topics(topics, t);
                }
                SseEvent::TopicCreated { topic } => {
                    if let Ok(mut w) = topics.try_write() {
                        w.push(topic);
                    }
                }
                SseEvent::TopicUpdated { topic }
                | SseEvent::ChaosRetentionZeroed { topic }
                | SseEvent::ChaosCleanupPolicyFlipped { topic }
                | SseEvent::ChaosPartitionsIncreased { topic } => {
                    update_topic(topics, selected, topic);
                }
                SseEvent::TopicDeleted { name } | SseEvent::ChaosTopicDeleted { topic: name } => {
                    remove_topic(topics, selected, &name);
                }
                SseEvent::TopicsSeeded { topics: seeded } => {
                    if let Ok(mut w) = topics.try_write() {
                        for t in seeded {
                            if !w.iter().any(|existing| existing.name == t.name) {
                                w.push(t);
                            }
                        }
                    }
                }
                SseEvent::ChaosPoisonPillsSent { .. }
                | SseEvent::ChaosSchemaBreakSent { .. } => {}
            }
        }) as Box<dyn FnMut(_)>);

        es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
    });
}

fn set_topics(mut topics: Signal<Vec<TopicResource>>, list: Vec<TopicResource>) {
    if let Ok(mut w) = topics.try_write() {
        *w = list;
    }
}

fn update_topic(
    mut topics: Signal<Vec<TopicResource>>,
    mut selected: Signal<Option<siege_api_client::TopicDetailResource>>,
    topic: TopicResource,
) {
    if let Ok(mut w) = topics.try_write() {
        if let Some(t) = w.iter_mut().find(|t| t.name == topic.name) {
            *t = topic.clone();
        }
    }
    if selected
        .try_read()
        .ok()
        .and_then(|s| s.as_ref().map(|s| s.name == topic.name))
        .unwrap_or(false)
    {
        if let Ok(mut w) = selected.try_write() {
            *w = Some(topic.into());
        }
    }
}

fn remove_topic(
    mut topics: Signal<Vec<TopicResource>>,
    mut selected: Signal<Option<siege_api_client::TopicDetailResource>>,
    name: &str,
) {
    if let Ok(mut w) = topics.try_write() {
        w.retain(|t| t.name != name);
    }
    if selected
        .try_read()
        .ok()
        .and_then(|s| s.as_ref().map(|s| s.name == name))
        .unwrap_or(false)
    {
        if let Ok(mut w) = selected.try_write() {
            *w = None;
        }
    }
}
