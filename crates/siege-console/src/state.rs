use dioxus::prelude::*;
use siege_api_client::SiegeClient;
use siege_core::{Topic, TopicDetail};

#[derive(Clone, Copy)]
pub struct AppState {
    pub topics: Signal<Vec<Topic>>,
    pub selected_topic: Signal<Option<TopicDetail>>,
    pub api_url: &'static str,
}

impl AppState {
    pub fn client(&self) -> SiegeClient {
        SiegeClient::new(self.api_url)
    }
}
