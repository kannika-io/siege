use dioxus::prelude::*;

use crate::components::ui::theme_toggle::ThemeToggle;
use crate::routes::Route;
use crate::state::AppState;

#[component]
pub fn Layout() -> Element {
    let state = use_context::<AppState>();
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

                    div { class: "flex-1 flex flex-col gap-0.5",
                        NavItem { to: Route::TopicsPage {}, label: "Topics" }
                        NavItem { to: Route::WheelOfChaosPage {}, label: "Wheel of Chaos" }
                    }

                    ThemeToggle {}
                }

                div { class: "flex-1 bg-surface rounded-xl overflow-hidden flex flex-col border border-border",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

#[component]
fn NavItem(to: Route, label: &'static str) -> Element {
    let current_route: Route = use_route();
    let is_active = current_route == to;

    let class = if is_active {
        "flex items-center px-3 py-1.5 rounded-md text-sm font-medium bg-subtle text-sidebar-active"
    } else {
        "flex items-center px-3 py-1.5 rounded-md text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-subtle/50 transition-colors"
    };

    rsx! {
        Link { to: to, class: class, "{label}" }
    }
}
