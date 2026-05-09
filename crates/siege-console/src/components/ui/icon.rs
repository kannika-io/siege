use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum IconName {
    Sun,
    Moon,
    Monitor,
    Layers,
    Copy,
}

impl IconName {
    fn paths(self) -> Element {
        match self {
            IconName::Sun => rsx! {
                path { d: "M17 12a5 5 0 1 1-10 0 5 5 0 0 1 10 0z" }
                path { d: "M12 1v2M12 21v2M4.22 4.22l1.42 1.42M19.78 4.22l-1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M19.78 19.78l-1.42-1.42" }
            },
            IconName::Moon => rsx! {
                path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }
            },
            IconName::Monitor => rsx! {
                path { d: "M4 3h16a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" }
                path { d: "M8 21h8M12 17v4" }
            },
            IconName::Layers => rsx! {
                path { d: "M12 2 2 7l10 5 10-5-10-5z" }
                path { d: "M2 17l10 5 10-5" }
                path { d: "M2 12l10 5 10-5" }
            },
            IconName::Copy => rsx! {
                path { d: "M11 9h9a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-9a2 2 0 0 1-2-2v-9a2 2 0 0 1 2-2z" }
                path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
            },
        }
    }
}

#[component]
pub fn Icon(name: IconName, #[props(default = 16)] size: u32) -> Element {
    rsx! {
        svg {
            width: "{size}",
            height: "{size}",
            "viewBox": "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            "stroke-width": "2",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            {name.paths()}
        }
    }
}
