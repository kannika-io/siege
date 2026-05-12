use dioxus::prelude::*;

#[component]
pub fn BattlefieldPage() -> Element {
    rsx! {
        div { class: "flex-1 flex flex-col items-center justify-center",
            h1 { class: "text-xl font-bold text-foreground", "The Battlefield" }
            p { class: "text-muted-foreground text-sm mt-2", "Coming soon..." }
        }
    }
}
