use dioxus::prelude::*;
use siege_api_client::{KafkaProperties, TopicDetailResource};

use super::topic_pills::TopicPills;
use crate::chaos_action::ChaosAction;
use crate::components::ui::chaos_button::ChaosButton;
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
    let state = use_context::<AppState>();
    let mut topics_state = use_context::<TopicsState>();
    let mut toaster = use_context::<Toaster>();
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

                div { class: "px-6 py-4 border-b border-border",
                    h3 { class: "text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3",
                        "Chaos"
                    }

                    div { class: "flex flex-wrap gap-3",
                        for &action in ChaosAction::ALL {
                            ChaosButton {
                                key: "{action.label()}",
                                label: action.label(),
                                icon: action.icon(),
                                destructive: action.is_destructive(),
                                onclick: {
                                    let name = name.clone();
                                    move |_| {
                                        let client = state.client();
                                        let name = name.clone();
                                        let msg = action.success_message(&name);
                                        spawn(async move {
                                            let topic = client.topic(&name);
                                            match action.execute(&topic).await {
                                                Ok(()) => {
                                                    if action.is_destructive() {
                                                        topics_state.selected.set(None);
                                                    }
                                                    toaster.success(msg);
                                                }
                                                Err(e) => toaster.error(e),
                                            }
                                        });
                                    }
                                },
                            }
                        }
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

