use dioxus::prelude::*;
use siege_api_client::{ChaosExt, KafkaProperties, TopicDetailResource};

use super::topic_pills::TopicPills;
use crate::state::{AppState, TopicsState};

const DEFAULT_HIGHLIGHTED_KEYS: &[&str] = &[
    "cleanup.policy",
    "max.message.bytes",
    "min.insync.replicas",
    "retention.bytes",
    "retention.ms",
];

#[component]
pub fn TopicDetailPanel(detail: TopicDetailResource) -> Element {
    let mut topics_state = use_context::<TopicsState>();
    let name = detail.name.clone();
    let feedback = use_signal(|| None::<String>);
    let show_all_config = use_signal(|| false);

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "flex items-center justify-between px-6 py-4 border-b border-border",
                h2 { class: "text-sm font-semibold truncate", "{detail.name}" }
                button {
                    class: "text-muted-foreground hover:text-foreground text-lg leading-none cursor-pointer",
                    onclick: move |_| topics_state.selected.set(None),
                    "\u{00d7}"
                }
            }

            div { class: "flex-1 overflow-y-auto",
                div { class: "px-6 py-4 border-b border-border",
                    TopicPills { partitions: detail.partitions, replication_factor: detail.replication_factor, config: detail.config.clone() }
                }

                div { class: "px-6 py-4 border-b border-border",
                    h3 { class: "text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3",
                        "Chaos"
                    }

                    if let Some(ref msg) = feedback() {
                        div { class: "mb-3 px-3 py-2 rounded-md text-xs bg-muted text-foreground",
                            "{msg}"
                        }
                    }

                    div { class: "flex flex-col gap-2",
                        ChaosRow { action: ChaosAction::Delete, name: name.clone(), feedback }
                        ChaosRow { action: ChaosAction::ZeroRetention, name: name.clone(), feedback }
                        ChaosRow { action: ChaosAction::FlipCleanupPolicy, name: name.clone(), feedback }
                        ChaosRow { action: ChaosAction::IncreasePartitions(100), name: name.clone(), feedback }
                        ChaosRow { action: ChaosAction::PoisonPills(10), name: name.clone(), feedback }
                        ChaosRow { action: ChaosAction::SchemaBreak(10), name: name.clone(), feedback }
                    }
                }

                if !detail.config.is_empty() {
                    ConfigTable { config: detail.config.clone(), show_all: show_all_config }
                }
            }
        }
    }
}

#[component]
fn ConfigTable(config: KafkaProperties, mut show_all: Signal<bool>) -> Element {
    let mut all: Vec<(&String, &String)> = config.iter().collect();
    all.sort_by_key(|(k, _)| k.as_str());
    let highlighted: Vec<(&String, &String)> = all
        .iter()
        .filter(|(k, _)| DEFAULT_HIGHLIGHTED_KEYS.contains(&k.as_str()))
        .copied()
        .collect();
    let has_more = all.len() > highlighted.len();

    let entries = if show_all() { &all } else { &highlighted };

    rsx! {
        div { class: "px-6 py-4 border-b border-border",
            h3 { class: "text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3",
                "Configuration"
            }
            div { class: "border border-border rounded-lg overflow-hidden",
                table { class: "w-full text-sm table-fixed",
                    thead {
                        tr { class: "bg-muted",
                            th { class: "text-left px-3 py-2 text-xs font-medium text-muted-foreground w-1/2", "Key" }
                            th { class: "text-left px-3 py-2 text-xs font-medium text-muted-foreground w-1/2", "Value" }
                        }
                    }
                    tbody {
                        for (key, value) in entries {
                            tr { class: "border-t border-border",
                                td { class: "px-3 py-2 text-xs break-all", "{key}" }
                                td { class: "px-3 py-2 text-xs text-muted-foreground break-all", "{value}" }
                            }
                        }
                        if has_more {
                            tr { class: "border-t border-border",
                                td {
                                    class: "px-3 py-2 text-xs text-accent cursor-pointer hover:underline",
                                    colspan: 2,
                                    onclick: move |_| show_all.set(!show_all()),
                                    if show_all() { "Show less" } else { "Show all\u{2026}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum ChaosAction {
    Delete,
    ZeroRetention,
    FlipCleanupPolicy,
    IncreasePartitions(i64),
    PoisonPills(i64),
    SchemaBreak(i64),
}

impl ChaosAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Delete => "Delete topic",
            Self::ZeroRetention => "Zero retention",
            Self::FlipCleanupPolicy => "Flip cleanup policy",
            Self::IncreasePartitions(_) => "Increase partitions",
            Self::PoisonPills(_) => "Poison pills",
            Self::SchemaBreak(_) => "Schema break",
        }
    }

    fn is_destructive(&self) -> bool {
        matches!(self, Self::Delete)
    }

    fn has_input(&self) -> bool {
        matches!(self, Self::IncreasePartitions(_) | Self::PoisonPills(_) | Self::SchemaBreak(_))
    }

    fn default_value(&self) -> i64 {
        match self {
            Self::IncreasePartitions(v) | Self::PoisonPills(v) | Self::SchemaBreak(v) => *v,
            _ => 0,
        }
    }

    async fn execute(&self, topic: &siege_api_client::Topic<'_>, value: i64) -> Result<String, String> {
        match self {
            Self::Delete => topic.delete().await.map(|()| "Topic deleted".to_string()).map_err(|e| e.to_string()),
            Self::ZeroRetention => topic.zero_retention().await.map(|r| r.result).map_err(|e| e.to_string()),
            Self::FlipCleanupPolicy => topic.flip_cleanup_policy().await.map(|r| r.result).map_err(|e| e.to_string()),
            Self::IncreasePartitions(_) => topic.increase_partitions(value as i32).await.map(|r| r.result).map_err(|e| e.to_string()),
            Self::PoisonPills(_) => topic.poison_pills(value as u32).await.map(|r| r.result).map_err(|e| e.to_string()),
            Self::SchemaBreak(_) => topic.schema_break(value as u32).await.map(|r| r.result).map_err(|e| e.to_string()),
        }
    }
}

#[component]
fn ChaosRow(action: ChaosAction, name: String, feedback: Signal<Option<String>>) -> Element {
    let state = use_context::<AppState>();
    let mut topics_state = use_context::<TopicsState>();
    let default_value = action.default_value();
    let mut input_value = use_signal(move || default_value.to_string());

    let btn_class = if action.is_destructive() {
        "px-3 py-1.5 rounded-md text-xs font-medium bg-destructive text-destructive-foreground hover:bg-destructive-hover cursor-pointer transition-colors"
    } else {
        "px-3 py-1.5 rounded-md text-xs font-medium bg-amber-600 text-white hover:bg-amber-700 cursor-pointer transition-colors"
    };

    let has_input = action.has_input();
    let label = action.label();

    rsx! {
        div { class: "flex items-center gap-2",
            button {
                class: btn_class,
                onclick: {
                    let name = name.clone();
                    let action = action.clone();
                    move |_| {
                        let client = state.client();
                        let name = name.clone();
                        let action = action.clone();
                        let value = input_value().parse::<i64>().unwrap_or(action.default_value());
                        let mut feedback = feedback;
                        spawn(async move {
                            let topic = client.topic(&name);
                            match action.execute(&topic, value).await {
                                Ok(msg) => {
                                    if matches!(action, ChaosAction::Delete) {
                                        topics_state.selected.set(None);
                                    }
                                    feedback.set(Some(msg));
                                }
                                Err(e) => feedback.set(Some(format!("Error: {e}"))),
                            }
                        });
                    }
                },
                "{label}"
            }
            if has_input {
                input {
                    r#type: "number",
                    class: "w-20 px-2 py-1.5 rounded-md text-xs border border-border bg-background text-foreground",
                    value: "{input_value}",
                    oninput: move |e| input_value.set(e.value()),
                }
            }
        }
    }
}
