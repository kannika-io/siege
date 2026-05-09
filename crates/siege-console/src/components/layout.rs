use dioxus::prelude::*;

use crate::state::AppState;

#[component]
pub fn Layout(children: Element) -> Element {
    let mut state = use_context::<AppState>();
    let theme = (state.theme)();
    let theme_class = theme.css_class();

    rsx! {
        head {
            link {
                rel: "stylesheet",
                href: "https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700&display=swap",
            }
            link {
                rel: "stylesheet",
                href: asset!("/assets/tailwind.css"),
            }
        }
        div { class: "{theme_class}",
            div { class: "bg-background text-foreground h-screen flex font-sans antialiased p-2",
                nav { class: "w-56 shrink-0 flex flex-col px-3 py-4",
                    div { class: "flex items-center gap-2.5 px-3 py-2 mb-6",
                        div { class: "w-5 h-5 rounded bg-accent flex items-center justify-center",
                            span { class: "text-accent-foreground text-[10px] font-bold leading-none", "S" }
                        }
                        span { class: "text-foreground font-semibold text-sm tracking-tight", "Siege" }
                    }

                    div { class: "flex-1",
                        div { class: "flex items-center px-3 py-1.5 rounded-md text-sm font-medium bg-subtle text-sidebar-active",
                            "Topics"
                        }
                    }

                    button {
                        class: "flex items-center px-3 py-1.5 rounded-md text-xs text-sidebar-foreground hover:bg-subtle hover:text-sidebar-active cursor-pointer transition-colors",
                        onclick: move |_| {
                            state.theme.set(theme.next());
                        },
                        "Theme: {theme.label()}"
                    }
                }

                div { class: "flex-1 bg-surface rounded-xl overflow-hidden flex flex-col border border-border",
                    {children}
                }
            }
        }
    }
}
