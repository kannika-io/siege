use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    rsx! { h1 { "Siege" } }
}
