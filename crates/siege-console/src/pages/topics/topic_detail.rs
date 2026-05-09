use dioxus::prelude::*;
use siege_api_client::TopicDetailResource;

use super::topic_pills::TopicPills;
use crate::state::AppState;

#[component]
pub fn TopicDetailPanel(detail: TopicDetailResource) -> Element {
    let mut state = use_context::<AppState>();
    let name = detail.name.clone();

    rsx! {
        div {
            class: "fixed inset-0 bg-black/40 z-40",
            onclick: move |_| state.selected_topic.set(None),
        }
        div {
            class: "fixed top-0 right-0 w-[480px] h-screen bg-surface border-l border-border overflow-y-auto z-50",

            div { class: "flex items-center justify-between px-6 py-4 border-b border-border",
                h2 { class: "text-sm font-semibold truncate", "{detail.name}" }
                button {
                    class: "text-muted-foreground hover:text-foreground text-lg leading-none cursor-pointer",
                    onclick: move |_| state.selected_topic.set(None),
                    "\u{00d7}"
                }
            }

            div { class: "px-6 py-4 border-b border-border",
                TopicPills { partitions: detail.partitions, replication_factor: detail.replication_factor, config: detail.config.clone() }
            }

            if !detail.config.is_empty() {
                div { class: "px-6 py-4",
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
                                for (key, value) in detail.config.iter() {
                                    tr { class: "border-t border-border",
                                        td { class: "px-3 py-2 text-xs break-all", "{key}" }
                                        td { class: "px-3 py-2 text-xs text-muted-foreground break-all", "{value}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "px-6 py-4 border-t border-border",
                button {
                    class: "px-3 py-1.5 rounded-md text-sm font-medium bg-destructive text-destructive-foreground hover:bg-destructive-hover cursor-pointer transition-colors",
                    onclick: {
                        let name = name.clone();
                        move |_| {
                            let client = state.client();
                            let name = name.clone();
                            spawn(async move {
                                if client.delete_topic(&name).await.is_ok() {
                                    state.selected_topic.set(None);
                                }
                            });
                        }
                    },
                    "Delete topic"
                }
            }
        }
    }
}
