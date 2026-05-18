pub mod topic_detail;
pub mod topic_list;
pub mod topic_pills;

use std::collections::HashMap;

use dioxus::prelude::*;
use siege_api_client::TopicDetailResource;

use topic_detail::TopicDetailPanel;
use topic_list::TopicList;
use crate::sse;
use crate::state::TopicsState;

#[component]
pub fn TopicsPage() -> Element {
    use_context_provider(|| TopicsState {
        list: Signal::new(Vec::new()),
        selected: Signal::new(None::<TopicDetailResource>),
        seed_progress: Signal::new(HashMap::new()),
    });

    sse::use_sse_subscription();

    let selected = (use_context::<TopicsState>().selected)();

    rsx! {
        div { class: "flex flex-1 overflow-hidden",
            div { class: if selected.is_some() { "w-1/2 flex flex-col overflow-hidden" } else { "flex-1 flex flex-col overflow-hidden" },
                TopicList {}
            }
            if let Some(ref detail) = selected {
                div { class: "w-1/2 border-l border-border overflow-y-auto",
                    TopicDetailPanel { detail: detail.clone() }
                }
            }
        }
    }
}
