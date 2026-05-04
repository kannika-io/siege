use dioxus::prelude::*;

#[component]
pub fn Layout(children: Element) -> Element {
    rsx! {
        head {
            link {
                rel: "stylesheet",
                href: "https://fonts.googleapis.com/css2?family=MedievalSharp&family=Crimson+Text:ital,wght@0,400;0,600;1,400&display=swap",
            }
            link {
                rel: "stylesheet",
                href: asset!("/assets/tailwind.css"),
            }
        }
        div { class: "bg-stone-900 text-parchment font-serif min-h-screen",
            div { class: "bg-stone-800 border-b-2 border-gold py-4 px-8 text-center",
                h1 { class: "font-medieval text-gold text-4xl tracking-wide drop-shadow-lg",
                    "Siege"
                }
                p { class: "text-parchment-dim italic mt-1",
                    "Lay waste to your Kafka kingdom"
                }
            }
            div { class: "max-w-7xl mx-auto p-8",
                {children}
            }
        }
    }
}
