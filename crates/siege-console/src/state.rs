use dioxus::prelude::*;
use siege_api_client::SiegeClient;
use siege_core::{Topic, TopicDetail};

#[derive(Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn next(self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::System,
            Theme::System => Theme::Light,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::System => "System",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Theme::Light => "",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub topics: Signal<Vec<Topic>>,
    pub selected_topic: Signal<Option<TopicDetail>>,
    pub theme: Signal<Theme>,
    pub api_url: &'static str,
}

impl AppState {
    pub fn client(&self) -> SiegeClient {
        SiegeClient::new(self.api_url)
    }
}
