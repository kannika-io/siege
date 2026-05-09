use dioxus::prelude::*;

use crate::state::{AppState, Theme};

const ICON_SUN: &str = r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M19.78 4.22l-1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M19.78 19.78l-1.42-1.42"/></svg>"#;
const ICON_MOON: &str = r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>"#;
const ICON_MONITOR: &str = r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/></svg>"#;

#[component]
pub fn ThemeToggle() -> Element {
    let mut state = use_context::<AppState>();
    let theme = (state.theme)();

    let icon = match theme {
        Theme::Light => ICON_SUN,
        Theme::Dark => ICON_MOON,
        Theme::System => ICON_MONITOR,
    };

    rsx! {
        button {
            class: "self-start w-8 h-8 flex items-center justify-center rounded-full text-sidebar-foreground hover:bg-subtle hover:text-sidebar-active cursor-pointer transition-colors",
            title: "{theme.label()}",
            onclick: move |_| {
                state.theme.set(theme.next());
            },
            span { dangerous_inner_html: "{icon}" }
        }
    }
}
