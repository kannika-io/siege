use dioxus::prelude::*;

#[component]
pub fn Modal(
    open: bool,
    on_close: EventHandler,
    title: String,
    children: Element,
) -> Element {
    if !open {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center",
            onclick: move |_| on_close.call(()),

            div {
                class: "absolute inset-0",
                style: "background: rgba(0, 0, 0, 0.5);",
            }

            div {
                class: "relative rounded-xl shadow-2xl max-w-md w-full mx-4 p-6 border border-border bg-surface",
                onclick: move |e| e.stop_propagation(),

                h2 { class: "text-lg font-bold text-foreground mb-3", "{title}" }

                div { class: "text-sm text-muted-foreground leading-relaxed", {children} }

                button {
                    class: "mt-5 w-full px-4 py-2 rounded-lg text-sm font-semibold bg-accent text-accent-foreground hover:bg-accent-hover cursor-pointer",
                    onclick: move |_| on_close.call(()),
                    "Okay"
                }
            }
        }
    }
}
