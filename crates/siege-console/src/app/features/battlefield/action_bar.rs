use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

use super::canvas;
use super::cooldown::CooldownOverlay;
use super::state::Weapon;

#[component]
pub fn ActionBar(
    selected: Option<Weapon>,
    cooldowns: [f64; 2],
    on_select: EventHandler<Weapon>,
) -> Element {
    rsx! {
        div { class: "flex justify-center gap-3 py-3 px-6 border-t border-border bg-background/80 z-10",
            WeaponSlot {
                weapon: Weapon::Crossbow,
                is_selected: selected == Some(Weapon::Crossbow),
                cooldown_expires: cooldowns[Weapon::Crossbow.slot_index()],
                on_click: move |_| on_select.call(Weapon::Crossbow),
            }
            WeaponSlot {
                weapon: Weapon::Trebuchet,
                is_selected: selected == Some(Weapon::Trebuchet),
                cooldown_expires: cooldowns[Weapon::Trebuchet.slot_index()],
                on_click: move |_| on_select.call(Weapon::Trebuchet),
            }
        }
    }
}

#[component]
fn WeaponSlot(
    weapon: Weapon,
    is_selected: bool,
    cooldown_expires: f64,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let mut tick = use_signal(|| 0u32);

    use_effect(move || {
        let current_now = canvas::now();
        if cooldown_expires <= current_now {
            return;
        }

        let tick_cb: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> =
            Rc::new(RefCell::new(None));
        let tick_cb_clone = tick_cb.clone();

        *tick_cb.borrow_mut() = Some(Closure::wrap(Box::new(move |_ts: f64| {
            let current = canvas::now();
            tick.set(tick() + 1);

            if current < cooldown_expires {
                if let Some(inner) = tick_cb_clone.borrow().as_ref() {
                    canvas::request_animation_frame(inner);
                }
            }
        }) as Box<dyn FnMut(f64)>));

        if let Some(inner) = tick_cb.borrow().as_ref() {
            canvas::request_animation_frame(inner);
        }

        std::mem::forget(tick_cb);
    });

    let _ = tick();
    let now = canvas::now();
    let on_cooldown = cooldown_expires > now;
    let cooldown_fraction = if on_cooldown {
        let total = weapon.cooldown_ms();
        let remaining = cooldown_expires - now;
        (remaining / total).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let border_class = if is_selected {
        "ring-2 ring-amber-500 border-amber-500"
    } else if on_cooldown {
        "border-border opacity-50"
    } else {
        "border-border hover:border-muted-foreground"
    };

    let icon = match weapon {
        Weapon::Crossbow => "\u{1F3F9}",
        Weapon::Trebuchet => "\u{1FA78}",
    };

    rsx! {
        button {
            class: "relative w-16 h-16 rounded-lg border-2 bg-surface flex flex-col items-center justify-center gap-0.5 cursor-pointer transition-all select-none {border_class}",
            disabled: on_cooldown,
            onclick: move |e| on_click.call(e),

            span {
                class: "absolute top-0.5 left-1 text-[9px] font-bold text-muted-foreground",
                "{weapon.keybind()}"
            }

            span { class: "text-lg leading-none", "{icon}" }

            span { class: "text-[8px] leading-tight text-muted-foreground font-medium text-center",
                "{weapon.action_label()}"
            }

            if on_cooldown {
                CooldownOverlay { fraction: cooldown_fraction }
            }
        }
    }
}
