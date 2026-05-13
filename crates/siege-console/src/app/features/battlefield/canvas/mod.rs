pub mod aiming;
pub mod projectile;
pub mod scene;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub fn window() -> Option<web_sys::Window> {
    web_sys::window()
}

pub fn now() -> f64 {
    window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

pub fn request_animation_frame(cb: &Closure<dyn FnMut(f64)>) {
    if let Some(w) = window() {
        let _ = w.request_animation_frame(cb.as_ref().unchecked_ref());
    }
}

pub const LOGICAL_W: f64 = 900.0;
pub const LOGICAL_H: f64 = 550.0;

pub fn canvas_client_size(id: &str) -> (f64, f64) {
    let Some(doc) = window().and_then(|w| w.document()) else {
        return (LOGICAL_W, LOGICAL_H);
    };
    let Some(el) = doc.get_element_by_id(id) else {
        return (LOGICAL_W, LOGICAL_H);
    };
    let Ok(canvas) = el.dyn_into::<HtmlCanvasElement>() else {
        return (LOGICAL_W, LOGICAL_H);
    };
    let w = canvas.client_width() as f64;
    let h = canvas.client_height() as f64;
    if w < 1.0 || h < 1.0 {
        (LOGICAL_W, LOGICAL_H)
    } else {
        (w, h)
    }
}

pub fn to_logical(css_x: f64, css_y: f64, client_w: f64, client_h: f64) -> (f64, f64) {
    (css_x * LOGICAL_W / client_w, css_y * LOGICAL_H / client_h)
}

pub fn get_canvas(id: &str) -> Option<(HtmlCanvasElement, CanvasRenderingContext2d)> {
    let document = window()?.document()?;
    let el = document.get_element_by_id(id)?;
    let canvas: HtmlCanvasElement = el.dyn_into().ok()?;
    let obj = canvas.get_context("2d").ok()??;
    let ctx: CanvasRenderingContext2d = obj.dyn_into().ok()?;
    Some((canvas, ctx))
}

pub fn start_render_loop<F>(canvas_id: &'static str, mut render: F)
where
    F: FnMut(&CanvasRenderingContext2d, f64, f64, f64) + 'static,
{
    let Some((_canvas, ctx)) = get_canvas(canvas_id) else {
        return;
    };

    let ctx = Rc::new(ctx);
    let cb: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let cb_clone = cb.clone();
    let ctx_clone = ctx.clone();

    *cb.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        let Some((canvas, _)) = get_canvas(canvas_id) else {
            return;
        };
        let w = canvas.client_width() as f64;
        let h = canvas.client_height() as f64;

        if canvas.width() != w as u32 {
            canvas.set_width(w as u32);
        }
        if canvas.height() != h as u32 {
            canvas.set_height(h as u32);
        }

        render(&ctx_clone, w, h, timestamp);

        if let Some(inner) = cb_clone.borrow().as_ref() {
            request_animation_frame(inner);
        }
    }) as Box<dyn FnMut(f64)>));

    if let Some(inner) = cb.borrow().as_ref() {
        request_animation_frame(inner);
    }

    std::mem::forget(cb);
}
