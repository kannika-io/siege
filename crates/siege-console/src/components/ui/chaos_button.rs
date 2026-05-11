use dioxus::prelude::*;

use super::icon::{Icon, IconName};

#[component]
pub fn ChaosButton(
    label: &'static str,
    icon: IconName,
    #[props(default = false)] destructive: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let btn_class = if destructive {
        "size-20 rounded-lg border-2 border-red-600/25 ring-2 ring-red-600/15 ring-offset-2 ring-offset-background text-red-600 hover:border-red-600/50 hover:ring-red-600/25 cursor-pointer transition-all flex flex-col items-center justify-center gap-1.5 p-2"
    } else {
        "size-20 rounded-lg border-2 border-amber-600/25 ring-2 ring-amber-600/15 ring-offset-2 ring-offset-background text-amber-600 hover:border-amber-600/50 hover:ring-amber-600/25 cursor-pointer transition-all flex flex-col items-center justify-center gap-1.5 p-2"
    };

    rsx! {
        button {
            class: btn_class,
            onclick: move |e| onclick.call(e),
            Icon { name: icon, size: 20 }
            span { class: "text-[11px] leading-tight font-medium", "{label}" }
        }
    }
}
