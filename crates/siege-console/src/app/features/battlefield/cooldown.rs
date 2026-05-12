use dioxus::prelude::*;
use std::f64::consts::PI;

#[component]
pub fn CooldownOverlay(fraction: f64) -> Element {
    let angle = fraction * 2.0 * PI;
    let large_arc = if angle > PI { 1 } else { 0 };

    let cx = 32.0_f64;
    let cy = 32.0_f64;
    let r = 30.0_f64;

    let end_x = cx + r * (angle - PI / 2.0).cos();
    let end_y = cy + r * (angle - PI / 2.0).sin();

    let path = if fraction >= 0.999 {
        format!(
            "M {cx} {} A {r} {r} 0 1 1 {cx} {} A {r} {r} 0 1 1 {cx} {}",
            cy - r,
            cy + r,
            cy - r
        )
    } else if fraction < 0.001 {
        String::new()
    } else {
        format!(
            "M {cx} {cy} L {cx} {} A {r} {r} 0 {large_arc} 1 {end_x} {end_y} Z",
            cy - r
        )
    };

    if path.is_empty() {
        return rsx! {};
    }

    rsx! {
        svg {
            class: "absolute inset-0 w-full h-full pointer-events-none",
            "viewBox": "0 0 64 64",
            path {
                d: "{path}",
                fill: "rgba(0, 0, 0, 0.6)",
            }
        }
    }
}
