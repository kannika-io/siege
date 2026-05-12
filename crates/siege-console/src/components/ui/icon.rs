use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum IconName {
    Sun,
    Moon,
    Monitor,
    Layers,
    Copy,
    Compress,
    Skull,
    Hourglass,
    Swords,
    Shield,
    Flask,
    Zap,
    Sprout,
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
            IconName::Compress => rsx! {
                path { d: "M4 14l5-5M4 14h5v-5" }
                path { d: "M20 10l-5 5M20 10h-5v5" }
            },
            IconName::Skull => rsx! {
                path { d: "M12 2a8 8 0 0 0-8 8c0 3 2 5.5 4 7v5h8v-5c2-1.5 4-4 4-7a8 8 0 0 0-8-8z" }
                path { d: "M9 10h.01M15 10h.01" }
                path { d: "M8 17h8" }
            },
            IconName::Hourglass => rsx! {
                path { d: "M5 2h14M5 22h14" }
                path { d: "M7 2v5l5 5-5 5v5" }
                path { d: "M17 2v5l-5 5 5 5v5" }
            },
            IconName::Swords => rsx! {
                path { d: "M14.5 17.5 3 6V3h3l11.5 11.5" }
                path { d: "M13 19l6-6M16 16l4 4M19 21l2-2" }
                path { d: "M14.5 6.5 18 3h3v3l-3.5 3.5" }
                path { d: "M5 14l4 4M7 17l-3 3M3 19l2 2" }
            },
            IconName::Shield => rsx! {
                path { d: "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" }
                path { d: "M12 8v8M8 12h8" }
            },
            IconName::Flask => rsx! {
                path { d: "M9 3h6" }
                path { d: "M10 3v5.2L5 16a2 2 0 0 0 1.7 3h10.6a2 2 0 0 0 1.7-3l-5-7.8V3" }
                path { d: "M6 15h12" }
            },
            IconName::Zap => rsx! {
                path { d: "M13 2 3 14h9l-1 8 10-12h-9z" }
            },
            IconName::Sprout => rsx! {
                path { d: "M7 20h10" }
                path { d: "M10 20c5.5-2.5.8-6.4 3-10" }
                path { d: "M9.5 9.4c1.1.8 1.8 2.2 2.3 3.7-2 .4-3.5.4-4.8-.3-1.2-.6-2.3-1.9-3-4.2 2.8-.5 4.4 0 5.5.8z" }
                path { d: "M14.1 6a7 7 0 0 0-1.1 4c1.9-.1 3.3-.6 4.3-1.4 1-1 1.6-2.3 1.7-4.6-2.7.1-4 1-4.9 2z" }
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
