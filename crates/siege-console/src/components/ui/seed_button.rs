use dioxus::prelude::*;

use super::icon::{Icon, IconName};
use super::toast::Toaster;
use crate::state::AppState;

#[component]
pub fn SeedButton() -> Element {
    let state = use_context::<AppState>();
    let mut toaster = use_context::<Toaster>();

    rsx! {
        button {
            class: "self-start w-8 h-8 flex items-center justify-center rounded-full text-sidebar-foreground hover:bg-subtle hover:text-sidebar-active cursor-pointer transition-colors",
            title: "Seed topics",
            onclick: move |_| {
                let client = state.client();
                async move {
                    match client.seed().await {
                        Ok(()) => toaster.success("Topics seeded"),
                        Err(e) => toaster.error(format!("Seed failed: {e}")),
                    }
                }
            },
            Icon { name: IconName::Sprout }
        }
    }
}
