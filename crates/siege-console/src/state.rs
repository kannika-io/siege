use std::collections::HashMap;

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

#[derive(Clone, PartialEq)]
pub struct SeedProgress {
    pub records_generated: u32,
    pub total_records: u32,
}

#[derive(Clone, Copy)]
pub struct TopicsState {
    pub list: Signal<Vec<TopicResource>>,
    pub selected: Signal<Option<TopicDetailResource>>,
    pub seed_progress: Signal<HashMap<String, SeedProgress>>,
}

impl TopicsState {
    pub fn set_topics(&mut self, list: Vec<TopicResource>) {
        if let Ok(mut w) = self.list.try_write() {
            *w = list;
        }
    }

    pub fn upsert_topic(&mut self, topic: TopicResource) {
        if let Ok(mut w) = self.list.try_write() {
            if let Some(existing) = w.iter_mut().find(|t| t.name == topic.name) {
                *existing = topic.clone();
            } else {
                w.push(topic.clone());
            }
        }
        if self
            .selected
            .try_read()
            .ok()
            .and_then(|s| s.as_ref().map(|s| s.name == topic.name))
            .unwrap_or(false)
        {
            if let Ok(mut w) = self.selected.try_write() {
                *w = Some(topic.into());
            }
        }
    }

    pub fn set_seed_progress(&mut self, topic: String, records_generated: u32, total_records: u32) {
        if let Ok(mut w) = self.seed_progress.try_write() {
            w.insert(topic, SeedProgress { records_generated, total_records });
        }
    }

    pub fn remove_seed_progress(&mut self, topic: &str) {
        if let Ok(mut w) = self.seed_progress.try_write() {
            w.remove(topic);
        }
    }

    pub fn clear_seed_progress(&mut self) {
        if let Ok(mut w) = self.seed_progress.try_write() {
            w.clear();
        }
    }

    pub fn remove_topic(&mut self, name: &str) {
        if let Ok(mut w) = self.list.try_write() {
            w.retain(|t| t.name != name);
        }
        if self
            .selected
            .try_read()
            .ok()
            .and_then(|s| s.as_ref().map(|s| s.name == name))
            .unwrap_or(false)
        {
            if let Ok(mut w) = self.selected.try_write() {
                *w = None;
            }
        }
    }
}
