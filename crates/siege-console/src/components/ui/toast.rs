use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
}

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub kind: ToastKind,
}

#[derive(Clone, Copy)]
pub struct Toaster {
    items: Signal<Vec<Toast>>,
    next_id: Signal<u64>,
}

impl Toaster {
    pub fn new() -> Self {
        Self {
            items: Signal::new(Vec::new()),
            next_id: Signal::new(0),
        }
    }

    pub fn success(&mut self, message: impl Into<String>) {
        self.push(message.into(), ToastKind::Success);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.push(message.into(), ToastKind::Error);
    }

    fn push(&mut self, message: String, kind: ToastKind) {
        let id = (self.next_id)();
        self.next_id.set(id + 1);
        self.items.write().push(Toast { id, message, kind });

        let mut items = self.items;
        let cb = Closure::once(move || {
            items.write().retain(|t| t.id != id);
        });
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                3_000,
            )
            .unwrap();
        cb.forget();
    }
}

#[component]
pub fn ToastContainer() -> Element {
    let toaster = use_context::<Toaster>();
    let items = (toaster.items)();

    if items.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "fixed bottom-4 right-4 z-50 flex flex-col gap-2",
            for toast in items.iter() {
                {
                    let bg = match toast.kind {
                        ToastKind::Success => "bg-emerald-600 text-white",
                        ToastKind::Error => "bg-destructive text-destructive-foreground",
                    };
                    rsx! {
                        div {
                            key: "{toast.id}",
                            class: "px-4 py-2.5 rounded-lg shadow-lg text-xs font-medium {bg}",
                            "{toast.message}"
                        }
                    }
                }
            }
        }
    }
}
