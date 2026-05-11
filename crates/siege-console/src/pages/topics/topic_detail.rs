use dioxus::prelude::*;
use siege_api_client::{ChaosExt, KafkaProperties, TopicDetailResource};

use super::topic_pills::TopicPills;
use crate::state::AppState;

const DEFAULT_HIGHLIGHTED_KEYS: &[&str] = &[
    "cleanup.policy",
    "max.message.bytes",
    "min.insync.replicas",
    "retention.bytes",
    "retention.ms",
];

#[component]
pub fn TopicDetailPanel(detail: TopicDetailResource) -> Element {
    let mut state = use_context::<AppState>();
    let name = detail.name.clone();
    let feedback = use_signal(|| None::<String>);
    let show_all_config = use_signal(|| false);

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "flex items-center justify-between px-6 py-4 border-b border-border",
                h2 { class: "text-sm font-semibold truncate", "{detail.name}" }
                button {
                    class: "text-muted-foreground hover:text-foreground text-lg leading-none cursor-pointer",
                    onclick: move |_| state.selected_topic.set(None),
                    "\u{00d7}"
                }
            }

            div { class: "flex-1 overflow-y-auto",
                div { class: "px-6 py-4 border-b border-border",
                    TopicPills { partitions: detail.partitions, replication_factor: detail.replication_factor, config: detail.config.clone() }
                }

                if !detail.config.is_empty() {
                    ConfigTable { config: detail.config.clone(), show_all: show_all_config }
                }

                div { class: "px-6 py-4",
                    h3 { class: "text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3",
                        "Chaos"
                    }

                    if let Some(ref msg) = feedback() {
                        div { class: "mb-3 px-3 py-2 rounded-md text-xs bg-muted text-foreground",
                            "{msg}"
                        }
                    }

                    div { class: "flex flex-col gap-2",
                        ChaosButton {
                            label: "Delete topic",
                            destructive: true,
                            name: name.clone(),
                            feedback: feedback,
                        }
                        ChaosButton {
                            label: "Zero retention",
                            destructive: false,
                            name: name.clone(),
                            feedback: feedback,
                        }
                        ChaosButton {
                            label: "Flip cleanup policy",
                            destructive: false,
                            name: name.clone(),
                            feedback: feedback,
                        }
                        ChaosNumberInput {
                            label: "Increase partitions",
                            name: name.clone(),
                            feedback: feedback,
                            default_value: 100,
                        }
                        ChaosNumberInput {
                            label: "Poison pills",
                            name: name.clone(),
                            feedback: feedback,
                            default_value: 10,
                        }
                        ChaosNumberInput {
                            label: "Schema break",
                            name: name.clone(),
                            feedback: feedback,
                            default_value: 10,
                        }
                    }
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
                table { class: "w-full text-sm",
                    thead {
                        tr { class: "bg-muted",
                            th { class: "text-left px-3 py-2 text-xs font-medium text-muted-foreground", "Key" }
                            th { class: "text-left px-3 py-2 text-xs font-medium text-muted-foreground", "Value" }
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

#[component]
fn ChaosButton(
    label: &'static str,
    destructive: bool,
    name: String,
    feedback: Signal<Option<String>>,
) -> Element {
    let mut state = use_context::<AppState>();
    let btn_class = if destructive {
        "px-3 py-1.5 rounded-md text-xs font-medium bg-destructive text-destructive-foreground hover:bg-destructive-hover cursor-pointer transition-colors"
    } else {
        "px-3 py-1.5 rounded-md text-xs font-medium bg-amber-600 text-white hover:bg-amber-700 cursor-pointer transition-colors"
    };

    rsx! {
        button {
            class: btn_class,
            onclick: {
                let name = name.clone();
                let label = label;
                move |_| {
                    let client = state.client();
                    let name = name.clone();
                    let mut feedback = feedback;
                    spawn(async move {
                        let topic = client.topic(&name);
                        let result = match label {
                            "Delete topic" => {
                                match topic.delete().await {
                                    Ok(()) => {
                                        state.selected_topic.set(None);
                                        Ok("Topic deleted".to_string())
                                    }
                                    Err(e) => Err(e.to_string()),
                                }
                            }
                            "Zero retention" => {
                                match topic.zero_retention().await {
                                    Ok(r) => Ok(format!("{}: {}", r.topic, r.result)),
                                    Err(e) => Err(e.to_string()),
                                }
                            }
                            "Flip cleanup policy" => {
                                match topic.flip_cleanup_policy().await {
                                    Ok(r) => Ok(format!("{}: {}", r.topic, r.result)),
                                    Err(e) => Err(e.to_string()),
                                }
                            }
                            _ => Ok("Unknown action".to_string()),
                        };
                        match result {
                            Ok(msg) => feedback.set(Some(msg)),
                            Err(e) => feedback.set(Some(format!("Error: {e}"))),
                        }
                    });
                }
            },
            "{label}"
        }
    }
}

#[component]
fn ChaosNumberInput(
    label: &'static str,
    name: String,
    feedback: Signal<Option<String>>,
    default_value: i64,
) -> Element {
    let state = use_context::<AppState>();
    let mut input_value = use_signal(move || default_value.to_string());

    rsx! {
        div { class: "flex items-center gap-2",
            button {
                class: "px-3 py-1.5 rounded-md text-xs font-medium bg-amber-600 text-white hover:bg-amber-700 cursor-pointer transition-colors",
                onclick: {
                    let name = name.clone();
                    let label = label;
                    move |_| {
                        let client = state.client();
                        let name = name.clone();
                        let value = input_value().parse::<i64>().unwrap_or(default_value);
                        let mut feedback = feedback;
                        spawn(async move {
                            let topic = client.topic(&name);
                            let result = match label {
                                "Increase partitions" => {
                                    match topic.increase_partitions(value as i32).await {
                                        Ok(r) => Ok(format!("{}: {}", r.topic, r.result)),
                                        Err(e) => Err(e.to_string()),
                                    }
                                }
                                "Poison pills" => {
                                    match topic.poison_pills(value as u32).await {
                                        Ok(r) => Ok(format!("{}: {}", r.topic, r.result)),
                                        Err(e) => Err(e.to_string()),
                                    }
                                }
                                "Schema break" => {
                                    match topic.schema_break(value as u32).await {
                                        Ok(r) => Ok(format!("{}: {}", r.topic, r.result)),
                                        Err(e) => Err(e.to_string()),
                                    }
                                }
                                _ => Ok("Unknown action".to_string()),
                            };
                            match result {
                                Ok(msg) => feedback.set(Some(msg)),
                                Err(e) => feedback.set(Some(format!("Error: {e}"))),
                            }
                        });
                    }
                },
                "{label}"
            }
            input {
                r#type: "number",
                class: "w-20 px-2 py-1.5 rounded-md text-xs border border-border bg-background text-foreground",
                value: "{input_value}",
                oninput: move |e| input_value.set(e.value()),
            }
        }
    }
}
