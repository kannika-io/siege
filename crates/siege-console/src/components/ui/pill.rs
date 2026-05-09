use dioxus::prelude::*;

use super::icon::{Icon, IconName};

#[derive(Clone, Copy, PartialEq, Default)]
pub enum PillVariant {
    #[default]
    Default,
    Accent,
    Destructive,
}

impl PillVariant {
    fn dot_class(self) -> &'static str {
        match self {
            PillVariant::Default => "bg-muted-foreground",
            PillVariant::Accent => "bg-accent",
            PillVariant::Destructive => "bg-destructive",
        }
    }

    fn icon_class(self) -> &'static str {
        match self {
            PillVariant::Default => "text-muted-foreground",
            PillVariant::Accent => "text-accent",
            PillVariant::Destructive => "text-destructive",
        }
    }
}

#[component]
pub fn Pill(
    #[props(default)] variant: PillVariant,
    #[props(default)] icon: Option<IconName>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border border-border text-xs text-muted-foreground",
            if let Some(name) = icon {
                span { class: "{variant.icon_class()}",
                    Icon { name, size: 12 }
                }
            } else {
                span { class: "w-2 h-2 rounded-full {variant.dot_class()}" }
            }
            {children}
        }
    }
}
