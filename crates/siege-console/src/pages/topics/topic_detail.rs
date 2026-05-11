use dioxus::prelude::*;
use siege_api_client::{ChaosExt, KafkaProperties, TopicDetailResource};

use super::topic_pills::TopicPills;
use crate::components::ui::toast::Toaster;
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

                div { class: "@container px-6 py-4 border-b border-border",
                    h3 { class: "text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3",
                        "Chaos"
                    }

                    div { class: "flex flex-wrap gap-2",
                        ChaosButton { action: ChaosAction::Delete, name: name.clone() }
                        ChaosButton { action: ChaosAction::ZeroRetention, name: name.clone() }
                        ChaosButton { action: ChaosAction::FlipCleanupPolicy, name: name.clone() }
                        ChaosButton { action: ChaosAction::IncreasePartitions, name: name.clone() }
                        ChaosButton { action: ChaosAction::PoisonPills, name: name.clone() }
                        ChaosButton { action: ChaosAction::SchemaBreak, name: name.clone() }
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
    IncreasePartitions,
    PoisonPills,
    SchemaBreak,
}

impl ChaosAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Delete => "Delete topic",
            Self::ZeroRetention => "Zero retention",
            Self::FlipCleanupPolicy => "Flip cleanup policy",
            Self::IncreasePartitions => "Add partition",
            Self::PoisonPills => "Poison pills",
            Self::SchemaBreak => "Schema break",
        }
    }

    fn is_destructive(&self) -> bool {
        matches!(self, Self::Delete)
    }

    fn success_message(&self, name: &str) -> String {
        match self {
            Self::Delete => format!("Deleted topic '{name}'"),
            Self::ZeroRetention => format!("Set retention to zero for '{name}'"),
            Self::FlipCleanupPolicy => format!("Flipped cleanup policy for '{name}'"),
            Self::IncreasePartitions => format!("Increased partitions for '{name}'"),
            Self::PoisonPills => format!("Sent 10 poison pills to '{name}'"),
            Self::SchemaBreak => format!("Sent 10 schema-breaking messages to '{name}'"),
        }
    }

    async fn execute(&self, topic: &siege_api_client::Topic<'_>) -> Result<(), String> {
        match self {
            Self::Delete => topic.delete().await.map_err(|e| e.to_string()),
            Self::ZeroRetention => topic.zero_retention().await.map(|_| ()).map_err(|e| e.to_string()),
            Self::FlipCleanupPolicy => topic.flip_cleanup_policy().await.map(|_| ()).map_err(|e| e.to_string()),
            Self::IncreasePartitions => topic.increase_partitions(1).await.map(|_| ()).map_err(|e| e.to_string()),
            Self::PoisonPills => topic.poison_pills(10).await.map(|_| ()).map_err(|e| e.to_string()),
            Self::SchemaBreak => topic.schema_break(10).await.map(|_| ()).map_err(|e| e.to_string()),
        }
    }
}

#[component]
fn ChaosButton(action: ChaosAction, name: String) -> Element {
    let state = use_context::<AppState>();
    let mut topics_state = use_context::<TopicsState>();
    let mut toaster = use_context::<Toaster>();

    let btn_class = if action.is_destructive() {
        "size-20 rounded-lg text-xs font-medium bg-destructive text-destructive-foreground hover:bg-destructive-hover cursor-pointer transition-colors flex items-center justify-center text-center p-2"
    } else {
        "size-20 rounded-lg text-xs font-medium bg-amber-600 text-white hover:bg-amber-700 cursor-pointer transition-colors flex items-center justify-center text-center p-2"
    };

    let label = action.label();

    rsx! {
        button {
            class: btn_class,
            onclick: {
                let name = name.clone();
                let action = action.clone();
                move |_| {
                    let client = state.client();
                    let name = name.clone();
                    let action = action.clone();
                    let msg = action.success_message(&name);
                    spawn(async move {
                        let topic = client.topic(&name);
                        match action.execute(&topic).await {
                            Ok(()) => {
                                if matches!(action, ChaosAction::Delete) {
                                    topics_state.selected.set(None);
                                }
                                toaster.success(msg);
                            }
                            Err(e) => toaster.error(e),
                        }
                    });
                }
            },
            "{label}"
        }
    }
}
