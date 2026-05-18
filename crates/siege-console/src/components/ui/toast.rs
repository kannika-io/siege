use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Progress,
}

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub name: Option<String>,
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

    pub fn upsert(&mut self, name: impl Into<String>, message: impl Into<String>, kind: ToastKind) {
        let name = name.into();
        let message = message.into();
        let mut items = self.items.write();
        if let Some(toast) = items.iter_mut().find(|t| t.name.as_deref() == Some(&name)) {
            toast.message = message;
            toast.kind = kind;
        } else {
            let id = (self.next_id)();
            self.next_id.set(id + 1);
            items.push(Toast {
                id,
                name: Some(name),
                message,
                kind,
            });
        }
    }

    pub fn resolve(&mut self, name: &str, message: impl Into<String>, kind: ToastKind) {
        let message = message.into();
        let id = {
            let mut items = self.items.write();
            if let Some(toast) = items.iter_mut().find(|t| t.name.as_deref() == Some(name)) {
                toast.message = message;
                toast.kind = kind;
                toast.name = None;
                toast.id
            } else {
                let id = (self.next_id)();
                self.next_id.set(id + 1);
                items.push(Toast {
                    id,
                    name: None,
                    message,
                    kind,
                });
                id
            }
        };

        let mut items = self.items;
        let cb = Closure::once(move || {
            items.write().retain(|t| t.id != id);
        });
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                3_000,
            );
        }
        cb.forget();
    }

    fn push(&mut self, message: String, kind: ToastKind) {
        let id = (self.next_id)();
        self.next_id.set(id + 1);
        self.items.write().push(Toast { id, name: None, message, kind });

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
                        ToastKind::Progress => "bg-indigo-600 text-white",
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
