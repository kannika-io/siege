use dioxus::prelude::*;
use siege_api_client::SiegeClient;
use siege_api_client::{TopicDetailResource, TopicResource};

#[derive(Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

const STORAGE_KEY: &str = "siege-theme";

impl Theme {
    pub fn load() -> Self {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
            .map(|v| match v.as_str() {
                "light" => Theme::Light,
                "dark" => Theme::Dark,
                _ => Theme::System,
            })
            .unwrap_or(Theme::Dark)
    }

    pub fn save(self) {
        let value = match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        };
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            let _ = storage.set_item(STORAGE_KEY, value);
        }
    }

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
    pub theme: Signal<Theme>,
    pub api_url: &'static str,
}

impl AppState {
    pub fn client(&self) -> SiegeClient {
        SiegeClient::new(self.api_url)
    }
}

#[derive(Clone, Copy)]
pub struct TopicsState {
    pub list: Signal<Vec<TopicResource>>,
    pub selected: Signal<Option<TopicDetailResource>>,
}
