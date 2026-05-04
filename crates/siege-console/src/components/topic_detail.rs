use dioxus::prelude::*;
use siege_core::TopicDetail;

use crate::state::AppState;

#[component]
pub fn TopicDetailPanel(detail: TopicDetail) -> Element {
    let mut state = use_context::<AppState>();
    let name = detail.name.clone();

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 z-40",
            onclick: move |_| state.selected_topic.set(None),
        }
        div {
            class: "fixed top-0 right-0 w-[480px] h-screen bg-stone-800 border-l-2 border-gold p-8 overflow-y-auto z-50 shadow-[-4px_0_12px_rgba(0,0,0,0.3)]",
            button {
                class: "absolute top-4 right-4 bg-transparent border-none text-parchment-dim text-2xl cursor-pointer hover:text-parchment",
                onclick: move |_| state.selected_topic.set(None),
                "x"
            }
            h2 { class: "font-medieval text-gold text-2xl mb-6", "{detail.name}" }
            div { class: "text-parchment-dim text-sm",
                span { "Partitions: {detail.partitions}" }
                span { " | RF: {detail.replication_factor}" }
            }

            if !detail.config.is_empty() {
                p { class: "font-medieval text-gold text-lg mt-6 mb-2", "Configuration" }
                table { class: "w-full border-collapse my-4",
                    thead {
                        tr {
                            th { class: "text-left p-2 border-b border-stone-600 text-gold font-medieval text-sm", "Key" }
                            th { class: "text-left p-2 border-b border-stone-600 text-gold font-medieval text-sm", "Value" }
                        }
                    }
                    tbody {
                        for (key, value) in &detail.config {
                            tr {
                                td { class: "text-left p-2 border-b border-stone-600 text-sm break-all", "{key}" }
                                td { class: "text-left p-2 border-b border-stone-600 text-sm break-all", "{value}" }
                            }
                        }
                    }
                }
            }

            div { class: "mt-6",
                p { class: "font-medieval text-gold text-lg mb-2", "Actions" }
                button {
                    class: "font-medieval px-6 py-2 rounded border border-blood bg-blood text-parchment cursor-pointer transition-all duration-200 hover:bg-blood-bright",
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
                    "Delete Topic"
                }
            }
        }
    }
}
