use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, PartialEq)]
pub struct WheelSlice<T: Clone + PartialEq + 'static> {
    pub label: String,
    pub color: String,
    pub payload: T,
}

#[derive(Clone, PartialEq)]
pub struct WheelTickEvent {
    pub velocity: f64,
    pub angle: f64,
}

const FRICTION: f64 = 0.985;
const STOP_THRESHOLD: f64 = 0.003;
const CENTER: f64 = 250.0;
const RADIUS: f64 = 230.0;

fn window() -> Option<web_sys::Window> {
    web_sys::window()
}

fn now() -> f64 {
    window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

fn request_animation_frame(cb: &Closure<dyn FnMut(f64)>) {
    if let Some(w) = window() {
        let _ = w.request_animation_frame(cb.as_ref().unchecked_ref());
    }
}

fn slice_path(cx: f64, cy: f64, r: f64, start_angle: f64, end_angle: f64) -> String {
    let x1 = cx + r * start_angle.cos();
    let y1 = cy + r * start_angle.sin();
    let x2 = cx + r * end_angle.cos();
    let y2 = cy + r * end_angle.sin();
    let large_arc = if (end_angle - start_angle).abs() > PI { 1 } else { 0 };
    format!("M {cx} {cy} L {x1} {y1} A {r} {r} 0 {large_arc} 1 {x2} {y2} Z")
}

#[component]
pub fn Wheel<T: Clone + PartialEq + 'static>(
    slices: Vec<WheelSlice<T>>,
    angle: Signal<f64>,
    angular_velocity: Signal<f64>,
    spinning: Signal<bool>,
    #[props(default)] on_spin_start: Option<EventHandler>,
    #[props(default)] on_tick: Option<EventHandler<WheelTickEvent>>,
    on_result: EventHandler<T>,
) -> Element {
    let mut drag_active = use_signal(|| false);
    let mut last_pointer_angle = use_signal(|| 0.0_f64);
    let mut last_pointer_time = use_signal(|| 0.0_f64);
    let mut drag_velocity = use_signal(|| 0.0_f64);

    let slices_for_effect = slices.clone();
    use_effect(move || {
        if !spinning() {
            return;
        }

        let mut angle = angle;
        let mut angular_velocity = angular_velocity;
        let mut spinning = spinning;
        let on_tick = on_tick.clone();
        let on_result = on_result.clone();
        let slices = slices_for_effect.clone();

        let cb: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        let cb_clone = cb.clone();

        *cb.borrow_mut() = Some(Closure::wrap(Box::new(move |_timestamp: f64| {
            let vel = angular_velocity() * FRICTION;
            let ang = angle() + vel;

            angular_velocity.set(vel);
            angle.set(ang);

            if let Some(ref handler) = on_tick {
                handler.call(WheelTickEvent { velocity: vel, angle: ang });
            }

            if vel.abs() < STOP_THRESHOLD {
                spinning.set(false);
                angular_velocity.set(0.0);

                if !slices.is_empty() {
                    let n = slices.len();
                    let slice_angle = 2.0 * PI / n as f64;
                    let normalized = (-ang - PI / 2.0).rem_euclid(2.0 * PI);
                    let idx = (normalized / slice_angle) as usize % n;
                    on_result.call(slices[idx].payload.clone());
                }
                return;
            }

            if let Some(inner) = cb_clone.borrow().as_ref() {
                request_animation_frame(inner);
            }
        }) as Box<dyn FnMut(f64)>));

        if let Some(inner) = cb.borrow().as_ref() {
            request_animation_frame(inner);
        }

        std::mem::forget(cb);
    });

    let n = slices.len();
    let slice_angle = if n > 0 { 2.0 * PI / n as f64 } else { 2.0 * PI };
    let start_offset = -PI / 2.0 - slice_angle / 2.0;

    struct SliceData {
        path: String,
        color: String,
        label: String,
        label_x: f64,
        label_y: f64,
        label_rotate: f64,
    }

    let slice_data: Vec<SliceData> = slices
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let a0 = start_offset + i as f64 * slice_angle;
            let a1 = a0 + slice_angle;
            let mid = (a0 + a1) / 2.0;
            let label_r = RADIUS * 0.65;
            SliceData {
                path: slice_path(CENTER, CENTER, RADIUS, a0, a1),
                color: s.color.clone(),
                label: s.label.clone(),
                label_x: CENTER + label_r * mid.cos(),
                label_y: CENTER + label_r * mid.sin(),
                label_rotate: mid.to_degrees(),
            }
        })
        .collect();

    let current_angle = angle();
    let transform = format!("rotate({} {CENTER} {CENTER})", current_angle.to_degrees());

    let pointer_angle_from_element = |ex: f64, ey: f64| -> f64 {
        (ey - CENTER).atan2(ex - CENTER)
    };

    rsx! {
        div { class: "flex flex-col items-center select-none",
            svg {
                width: "100%",
                "viewBox": "0 0 500 500",
                style: "touch-action: none; max-width: min(80vh, 80vw);",

                onpointerdown: {
                    move |e: PointerEvent| {
                        if spinning() {
                            return;
                        }
                        let coords = e.element_coordinates();
                        let a = pointer_angle_from_element(coords.x, coords.y);
                        drag_active.set(true);
                        last_pointer_angle.set(a);
                        last_pointer_time.set(now());
                        drag_velocity.set(0.0);
                    }
                },
                onpointermove: {
                    move |e: PointerEvent| {
                        if !drag_active() || spinning() {
                            return;
                        }
                        let coords = e.element_coordinates();
                        let a = pointer_angle_from_element(coords.x, coords.y);
                        let prev = last_pointer_angle();
                        let mut delta = a - prev;
                        if delta > PI { delta -= 2.0 * PI; }
                        if delta < -PI { delta += 2.0 * PI; }

                        let t = now();
                        let dt = t - last_pointer_time();
                        if dt > 0.0 {
                            drag_velocity.set((delta / dt) * 16.0);
                        }

                        last_pointer_angle.set(a);
                        last_pointer_time.set(t);
                        angle.set(angle() + delta);
                    }
                },
                onpointerup: {
                    let on_spin_start = on_spin_start.clone();
                    move |_e: PointerEvent| {
                        if !drag_active() {
                            return;
                        }
                        drag_active.set(false);
                        let vel = drag_velocity();
                        if vel.abs() > STOP_THRESHOLD && !spinning() {
                            angular_velocity.set(vel);
                            spinning.set(true);
                            if let Some(ref h) = on_spin_start {
                                h.call(());
                            }
                        } else {
                            drag_velocity.set(0.0);
                        }
                    }
                },

                g { transform: "{transform}",
                    for sd in slice_data.iter() {
                        path {
                            key: "{sd.label}",
                            d: "{sd.path}",
                            fill: "{sd.color}",
                            stroke: "white",
                            "stroke-width": "1.5",
                        }
                        text {
                            x: "{sd.label_x}",
                            y: "{sd.label_y}",
                            "text-anchor": "middle",
                            "dominant-baseline": "middle",
                            transform: "rotate({sd.label_rotate} {sd.label_x} {sd.label_y})",
                            fill: "white",
                            "font-size": "13",
                            "font-weight": "600",
                            "font-family": "Outfit, sans-serif",
                            "pointer-events": "none",
                            "{sd.label}"
                        }
                    }
                }

                circle {
                    cx: "{CENTER}",
                    cy: "{CENTER}",
                    r: "18",
                    fill: "#1a1a2e",
                    stroke: "white",
                    "stroke-width": "2",
                }

                polygon {
                    points: "250,32 238,8 262,8",
                    fill: "#f59e0b",
                    stroke: "#92400e",
                    "stroke-width": "1.5",
                    "stroke-linejoin": "round",
                }
            }
        }
    }
}

const MIN_VELOCITY: f64 = 0.15;
const MAX_VELOCITY: f64 = 0.5;
const OSCILLATION_PERIOD_MS: f64 = 1500.0;

#[component]
pub fn SpinButton(
    angular_velocity: Signal<f64>,
    spinning: Signal<bool>,
    #[props(default)] on_spin_start: Option<EventHandler>,
    #[props(default)] on_charge_start: Option<EventHandler>,
    #[props(default)] on_charge_release: Option<EventHandler>,
) -> Element {
    let mut charging = use_signal(|| false);
    let mut power = use_signal(|| 0.0_f64);
    let mut charge_start_time = use_signal(|| 0.0_f64);

    use_effect(move || {
        if !charging() {
            return;
        }

        let mut power = power;
        let start = charge_start_time();
        let charging_check = charging;

        let cb: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        let cb_clone = cb.clone();

        *cb.borrow_mut() = Some(Closure::wrap(Box::new(move |_ts: f64| {
            if !charging_check() {
                return;
            }
            let elapsed = now() - start;
            let p = 50.0 + 50.0 * ((elapsed / OSCILLATION_PERIOD_MS * 2.0 * PI).sin());
            power.set(p);

            if let Some(inner) = cb_clone.borrow().as_ref() {
                request_animation_frame(inner);
            }
        }) as Box<dyn FnMut(f64)>));

        if let Some(inner) = cb.borrow().as_ref() {
            request_animation_frame(inner);
        }

        std::mem::forget(cb);
    });

    let release = {
        let on_spin_start = on_spin_start.clone();
        let on_charge_release = on_charge_release.clone();
        move || {
            if !charging() {
                return;
            }
            charging.set(false);
            let p = power();
            power.set(0.0);
            if let Some(ref h) = on_charge_release {
                h.call(());
            }
            let vel = MIN_VELOCITY + (MAX_VELOCITY - MIN_VELOCITY) * (p / 100.0);
            angular_velocity.set(vel);
            spinning.set(true);
            if let Some(ref h) = on_spin_start {
                h.call(());
            }
        }
    };

    let label = if spinning() {
        "Spinning..."
    } else if charging() {
        "Release!"
    } else {
        "Hold to Spin"
    };

    let power_pct = power();

    rsx! {
        div { class: "flex flex-col items-center gap-3 w-48",
            if charging() {
                div { class: "w-full h-3 rounded-full bg-muted overflow-hidden border border-border",
                    div {
                        class: "h-full rounded-full transition-none",
                        style: "width: {power_pct:.1}%; background: linear-gradient(90deg, #f59e0b, #ef4444);",
                    }
                }
            }

            button {
                class: "w-full px-5 py-2.5 rounded-lg text-sm font-semibold transition-all select-none cursor-pointer border-2 border-amber-600/40 bg-amber-600/10 text-amber-600 hover:bg-amber-600/20 disabled:opacity-50 disabled:cursor-not-allowed",
                disabled: spinning(),
                onmousedown: {
                    let on_charge_start = on_charge_start.clone();
                    move |_| {
                        if spinning() {
                            return;
                        }
                        charge_start_time.set(now());
                        charging.set(true);
                        if let Some(ref h) = on_charge_start {
                            h.call(());
                        }
                    }
                },
                onmouseup: {
                    let mut release = release.clone();
                    move |_| release()
                },
                onmouseleave: {
                    let mut release = release.clone();
                    move |_| release()
                },
                "{label}"
            }
        }
    }
}
