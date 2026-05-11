use dioxus::prelude::*;

use super::icon::{Icon, IconName};
use crate::state::{AppState, Theme};

#[component]
pub fn ThemeToggle() -> Element {
    let mut state = use_context::<AppState>();
    let theme = (state.theme)();

    let icon_name = match theme {
        Theme::Light => IconName::Sun,
        Theme::Dark => IconName::Moon,
        Theme::System => IconName::Monitor,
    };

    rsx! {
        button {
            class: "self-start w-8 h-8 flex items-center justify-center rounded-full text-sidebar-foreground hover:bg-subtle hover:text-sidebar-active cursor-pointer transition-colors",
            title: "{theme.label()}",
            onclick: move |_| {
                let next = theme.next();
                next.save();
                state.theme.set(next);
            },
            Icon { name: icon_name }
        }
    }
}
