# Battlefield Mini-Game Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a medieval siege mini-game at `/battlefield` where you aim and fire chaos actions (crossbow = poison pills, trebuchet = delete) at Kafka topics rendered as buildings on an HTML canvas.

**Architecture:** Canvas for the game scene (terrain, buildings, projectiles, aiming arc) with a DOM overlay for the WoW-style action bar, cooldowns, and HUD. All game state lives in Dioxus signals. On hit, existing `SiegeClient` chaos endpoints are called. Topics are fetched on mount; buildings update based on chaos results.

**Tech Stack:** Dioxus 0.7.5 (WASM), `web-sys` canvas API (`CanvasRenderingContext2d`), `wasm-bindgen` closures for `requestAnimationFrame`, existing `siege-api-client` for chaos calls.

---

### Task 1: Add web-sys canvas features and module scaffolding

**Files:**
- Modify: `Cargo.toml:63-73` (add canvas web-sys features)
- Create: `crates/siege-console/src/app/features/battlefield/mod.rs`
- Create: `crates/siege-console/src/app/features/battlefield/page.rs`
- Create: `crates/siege-console/src/app/features/battlefield/state.rs`
- Create: `crates/siege-console/src/app/features/battlefield/action_bar.rs`
- Create: `crates/siege-console/src/app/features/battlefield/cooldown.rs`
- Create: `crates/siege-console/src/app/features/battlefield/canvas/mod.rs`
- Create: `crates/siege-console/src/app/features/battlefield/canvas/scene.rs`
- Create: `crates/siege-console/src/app/features/battlefield/canvas/projectile.rs`
- Create: `crates/siege-console/src/app/features/battlefield/canvas/aiming.rs`
- Modify: `crates/siege-console/src/app/features/mod.rs`
- Modify: `crates/siege-console/src/routes.rs`
- Modify: `crates/siege-console/src/layouts/default.rs`

- [ ] **Step 1: Add web-sys canvas features to workspace Cargo.toml**

In `Cargo.toml`, add `"CanvasRenderingContext2d"`, `"Document"`, and `"HtmlCanvasElement"` to the `web-sys` features list:

```toml
web-sys = { version = "0.3", features = [
    "CanvasRenderingContext2d",
    "Document",
    "DomRect",
    "EventSource",
    "HtmlCanvasElement",
    "HtmlElement",
    "MessageEvent",
    "MouseEvent",
    "Performance",
    "PointerEvent",
    "Storage",
    "Window",
] }
```

- [ ] **Step 2: Create empty battlefield module files**

Create the directory structure with minimal contents.

`crates/siege-console/src/app/features/battlefield/state.rs`:
```rust
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

`crates/siege-console/src/app/features/battlefield/canvas/scene.rs`:
```rust
use web_sys::CanvasRenderingContext2d;

pub fn render_terrain(_ctx: &CanvasRenderingContext2d, _width: f64, _height: f64) {}
```

`crates/siege-console/src/app/features/battlefield/canvas/projectile.rs`:
```rust
use super::super::state::Point;

pub fn bezier_point(t: f64, p0: Point, p1: Point, p2: Point) -> Point {
    let inv = 1.0 - t;
    Point {
        x: inv * inv * p0.x + 2.0 * inv * t * p1.x + t * t * p2.x,
        y: inv * inv * p0.y + 2.0 * inv * t * p1.y + t * t * p2.y,
    }
}
```

`crates/siege-console/src/app/features/battlefield/canvas/aiming.rs`:
```rust
use web_sys::CanvasRenderingContext2d;

pub fn render_trajectory_preview(
    _ctx: &CanvasRenderingContext2d,
    _start: super::super::state::Point,
    _control: super::super::state::Point,
    _end: super::super::state::Point,
    _color: &str,
) {
}
```

`crates/siege-console/src/app/features/battlefield/canvas/mod.rs`:
```rust
pub mod aiming;
pub mod projectile;
pub mod scene;
```

`crates/siege-console/src/app/features/battlefield/cooldown.rs`:
```rust
use dioxus::prelude::*;

#[component]
pub fn CooldownOverlay(fraction: f64) -> Element {
    rsx! {}
}
```

`crates/siege-console/src/app/features/battlefield/action_bar.rs`:
```rust
use dioxus::prelude::*;

#[component]
pub fn ActionBar() -> Element {
    rsx! {}
}
```

`crates/siege-console/src/app/features/battlefield/page.rs`:
```rust
use dioxus::prelude::*;

#[component]
pub fn BattlefieldPage() -> Element {
    rsx! {
        div { class: "flex-1 flex flex-col items-center justify-center",
            h1 { class: "text-xl font-bold text-foreground", "The Battlefield" }
            p { class: "text-muted-foreground text-sm mt-2", "Coming soon..." }
        }
    }
}
```

`crates/siege-console/src/app/features/battlefield/mod.rs`:
```rust
mod action_bar;
mod canvas;
mod cooldown;
mod page;
pub mod state;

pub use page::BattlefieldPage;
```

- [ ] **Step 3: Register the feature module**

In `crates/siege-console/src/app/features/mod.rs`, add:
```rust
pub mod battlefield;
pub mod wheel;
```

- [ ] **Step 4: Add route**

In `crates/siege-console/src/routes.rs`, add the import and route variant:

```rust
use crate::app::features::battlefield::BattlefieldPage;
use crate::app::features::wheel::WheelOfChaosPage;
use crate::layouts::default::Layout;
use crate::pages::topics::TopicsPage;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    TopicsPage {},
    #[route("/wheel")]
    WheelOfChaosPage {},
    #[route("/battlefield")]
    BattlefieldPage {},
}
```

- [ ] **Step 5: Add navigation link**

In `crates/siege-console/src/layouts/default.rs`, add a nav item inside the `div { class: "flex-1 flex flex-col gap-0.5"` block, after the WheelOfChaosPage nav item:

```rust
NavItem { to: Route::BattlefieldPage {}, label: "Battlefield" }
```

- [ ] **Step 6: Verify it compiles**

Run: `dx build --platform web` from `crates/siege-console/`
Expected: Builds successfully

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(battlefield): scaffold module, route, and empty page"
```

---

### Task 2: Battlefield state types

**Files:**
- Modify: `crates/siege-console/src/app/features/battlefield/state.rs`

- [ ] **Step 1: Define all state types**

Replace `crates/siege-console/src/app/features/battlefield/state.rs` with:

```rust
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x
            && p.x <= self.x + self.width
            && p.y >= self.y
            && p.y <= self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Weapon {
    Crossbow,
    Trebuchet,
}

impl Weapon {
    pub fn label(self) -> &'static str {
        match self {
            Self::Crossbow => "Crossbow",
            Self::Trebuchet => "Trebuchet",
        }
    }

    pub fn action_label(self) -> &'static str {
        match self {
            Self::Crossbow => "Poison Pills",
            Self::Trebuchet => "Delete Topic",
        }
    }

    pub fn cooldown_ms(self) -> f64 {
        match self {
            Self::Crossbow => 3000.0,
            Self::Trebuchet => 6000.0,
        }
    }

    pub fn keybind(self) -> &'static str {
        match self {
            Self::Crossbow => "1",
            Self::Trebuchet => "2",
        }
    }

    pub fn slot_index(self) -> usize {
        match self {
            Self::Crossbow => 0,
            Self::Trebuchet => 1,
        }
    }

    pub fn launch_origin(self, canvas_height: f64) -> Point {
        match self {
            Self::Crossbow => Point {
                x: 90.0,
                y: canvas_height * 0.65,
            },
            Self::Trebuchet => Point {
                x: 90.0,
                y: canvas_height * 0.45,
            },
        }
    }

    pub fn projectile_color(self) -> &'static str {
        match self {
            Self::Crossbow => "#22c55e",
            Self::Trebuchet => "#ef4444",
        }
    }

    pub fn flight_duration_ms(self) -> f64 {
        match self {
            Self::Crossbow => 600.0,
            Self::Trebuchet => 900.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BuildingVisual {
    Healthy,
    Damaged { chaos_label: String },
    Destroyed,
}

#[derive(Clone, Debug)]
pub struct BuildingTarget {
    pub name: String,
    pub position: Point,
    pub hitbox: Rect,
    pub visual: BuildingVisual,
}

#[derive(Clone, Debug)]
pub struct Projectile {
    pub weapon: Weapon,
    pub start: Point,
    pub control: Point,
    pub end: Point,
    pub start_time: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Idle,
    Aiming { weapon: Weapon },
    Firing,
}

pub const BUILDING_WIDTH: f64 = 60.0;
pub const BUILDING_HEIGHT: f64 = 80.0;
pub const HITBOX_PADDING: f64 = 10.0;

pub fn build_targets(topic_names: &[String], canvas_width: f64, canvas_height: f64) -> Vec<BuildingTarget> {
    let count = topic_names.len();
    if count == 0 {
        return Vec::new();
    }

    let start_x = canvas_width * 0.35;
    let end_x = canvas_width * 0.90;
    let start_y = canvas_height * 0.15;
    let end_y = canvas_height * 0.75;

    let cols = ((count as f64).sqrt().ceil() as usize).max(1);
    let rows = (count + cols - 1) / cols;

    let col_gap = if cols > 1 {
        (end_x - start_x) / (cols - 1) as f64
    } else {
        0.0
    };
    let row_gap = if rows > 1 {
        (end_y - start_y) / (rows - 1) as f64
    } else {
        0.0
    };

    topic_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let col = i % cols;
            let row = i / cols;
            let cx = if cols > 1 {
                start_x + col as f64 * col_gap
            } else {
                (start_x + end_x) / 2.0
            };
            let cy = if rows > 1 {
                start_y + row as f64 * row_gap
            } else {
                (start_y + end_y) / 2.0
            };

            let position = Point { x: cx, y: cy };
            let hitbox = Rect {
                x: cx - (BUILDING_WIDTH + HITBOX_PADDING) / 2.0,
                y: cy - (BUILDING_HEIGHT + HITBOX_PADDING) / 2.0,
                width: BUILDING_WIDTH + HITBOX_PADDING,
                height: BUILDING_HEIGHT + HITBOX_PADDING,
            };

            BuildingTarget {
                name: name.clone(),
                position,
                hitbox,
                visual: BuildingVisual::Healthy,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_point_inside() {
        let r = Rect { x: 10.0, y: 10.0, width: 50.0, height: 50.0 };
        assert!(r.contains(Point { x: 30.0, y: 30.0 }));
    }

    #[test]
    fn rect_does_not_contain_point_outside() {
        let r = Rect { x: 10.0, y: 10.0, width: 50.0, height: 50.0 };
        assert!(!r.contains(Point { x: 5.0, y: 30.0 }));
        assert!(!r.contains(Point { x: 70.0, y: 30.0 }));
        assert!(!r.contains(Point { x: 30.0, y: 5.0 }));
        assert!(!r.contains(Point { x: 30.0, y: 70.0 }));
    }

    #[test]
    fn rect_contains_point_on_edge() {
        let r = Rect { x: 10.0, y: 10.0, width: 50.0, height: 50.0 };
        assert!(r.contains(Point { x: 10.0, y: 10.0 }));
        assert!(r.contains(Point { x: 60.0, y: 60.0 }));
    }

    #[test]
    fn build_targets_empty_list() {
        let targets = build_targets(&[], 800.0, 600.0);
        assert!(targets.is_empty());
    }

    #[test]
    fn build_targets_single_topic() {
        let names = vec!["orders".to_string()];
        let targets = build_targets(&names, 800.0, 600.0);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].name, "orders");
    }

    #[test]
    fn build_targets_multiple_topics_have_distinct_positions() {
        let names: Vec<String> = (0..6).map(|i| format!("topic-{i}")).collect();
        let targets = build_targets(&names, 800.0, 600.0);
        assert_eq!(targets.len(), 6);
        for i in 0..targets.len() {
            for j in (i + 1)..targets.len() {
                let dist = ((targets[i].position.x - targets[j].position.x).powi(2)
                    + (targets[i].position.y - targets[j].position.y).powi(2))
                .sqrt();
                assert!(dist > 1.0, "targets {i} and {j} overlap");
            }
        }
    }

    #[test]
    fn weapon_slot_indices_are_unique() {
        assert_ne!(Weapon::Crossbow.slot_index(), Weapon::Trebuchet.slot_index());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p siege-console --lib -- battlefield::state`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(battlefield): add state types with tests"
```

---

### Task 3: Trajectory math with tests

**Files:**
- Modify: `crates/siege-console/src/app/features/battlefield/canvas/projectile.rs`

- [ ] **Step 1: Write trajectory tests first**

Replace `crates/siege-console/src/app/features/battlefield/canvas/projectile.rs` with tests and stubs:

```rust
use super::super::state::Point;

pub fn bezier_point(t: f64, p0: Point, p1: Point, p2: Point) -> Point {
    let inv = 1.0 - t;
    Point {
        x: inv * inv * p0.x + 2.0 * inv * t * p1.x + t * t * p2.x,
        y: inv * inv * p0.y + 2.0 * inv * t * p1.y + t * t * p2.y,
    }
}

pub fn arc_control_point(start: Point, end: Point) -> Point {
    let mid_x = (start.x + end.x) / 2.0;
    let dist = (end.x - start.x).abs();
    let peak_y = start.y.min(end.y) - dist * 0.3;
    Point {
        x: mid_x,
        y: peak_y,
    }
}

pub fn sample_trajectory(
    start: Point,
    control: Point,
    end: Point,
    samples: usize,
) -> Vec<Point> {
    (0..=samples)
        .map(|i| {
            let t = i as f64 / samples as f64;
            bezier_point(t, start, control, end)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezier_at_t0_returns_start() {
        let p0 = Point { x: 0.0, y: 0.0 };
        let p1 = Point { x: 50.0, y: -100.0 };
        let p2 = Point { x: 100.0, y: 0.0 };
        let result = bezier_point(0.0, p0, p1, p2);
        assert!((result.x - p0.x).abs() < f64::EPSILON);
        assert!((result.y - p0.y).abs() < f64::EPSILON);
    }

    #[test]
    fn bezier_at_t1_returns_end() {
        let p0 = Point { x: 0.0, y: 0.0 };
        let p1 = Point { x: 50.0, y: -100.0 };
        let p2 = Point { x: 100.0, y: 0.0 };
        let result = bezier_point(1.0, p0, p1, p2);
        assert!((result.x - p2.x).abs() < f64::EPSILON);
        assert!((result.y - p2.y).abs() < f64::EPSILON);
    }

    #[test]
    fn bezier_midpoint_pulled_toward_control() {
        let p0 = Point { x: 0.0, y: 0.0 };
        let p1 = Point { x: 50.0, y: -100.0 };
        let p2 = Point { x: 100.0, y: 0.0 };
        let mid = bezier_point(0.5, p0, p1, p2);
        assert!(mid.y < 0.0, "midpoint should be above (negative y) due to control");
        assert!((mid.x - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sample_trajectory_count() {
        let start = Point { x: 0.0, y: 0.0 };
        let control = Point { x: 50.0, y: -50.0 };
        let end = Point { x: 100.0, y: 0.0 };
        let points = sample_trajectory(start, control, end, 20);
        assert_eq!(points.len(), 21);
    }

    #[test]
    fn sample_trajectory_starts_and_ends_correctly() {
        let start = Point { x: 10.0, y: 400.0 };
        let control = Point { x: 300.0, y: 100.0 };
        let end = Point { x: 600.0, y: 350.0 };
        let points = sample_trajectory(start, control, end, 30);
        assert!((points[0].x - start.x).abs() < f64::EPSILON);
        assert!((points[0].y - start.y).abs() < f64::EPSILON);
        let last = &points[points.len() - 1];
        assert!((last.x - end.x).abs() < f64::EPSILON);
        assert!((last.y - end.y).abs() < f64::EPSILON);
    }

    #[test]
    fn arc_control_point_above_endpoints() {
        let start = Point { x: 100.0, y: 400.0 };
        let end = Point { x: 500.0, y: 350.0 };
        let control = arc_control_point(start, end);
        assert!(control.y < start.y.min(end.y));
        assert!((control.x - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn arc_control_height_scales_with_distance() {
        let start = Point { x: 100.0, y: 400.0 };
        let near = Point { x: 200.0, y: 400.0 };
        let far = Point { x: 600.0, y: 400.0 };
        let c_near = arc_control_point(start, near);
        let c_far = arc_control_point(start, far);
        assert!(c_far.y < c_near.y, "farther target should produce higher arc");
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p siege-console --lib -- battlefield::canvas::projectile`
Expected: All 6 tests pass

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(battlefield): add trajectory math with tests"
```

---

### Task 4: Canvas setup and game loop with terrain

**Files:**
- Modify: `crates/siege-console/src/app/features/battlefield/canvas/mod.rs`
- Modify: `crates/siege-console/src/app/features/battlefield/canvas/scene.rs`
- Modify: `crates/siege-console/src/app/features/battlefield/page.rs`

- [ ] **Step 1: Implement canvas utilities**

Replace `crates/siege-console/src/app/features/battlefield/canvas/mod.rs`:

```rust
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
```

- [ ] **Step 2: Implement terrain rendering**

Replace `crates/siege-console/src/app/features/battlefield/canvas/scene.rs`:

```rust
use web_sys::CanvasRenderingContext2d;

use super::super::state::{
    BuildingTarget, BuildingVisual, BUILDING_HEIGHT, BUILDING_WIDTH,
};

pub fn render_terrain(ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
    // Sky gradient
    if let Ok(gradient) = ctx.create_linear_gradient(0.0, 0.0, 0.0, height * 0.6) {
        let _ = gradient.add_color_stop(0.0, "#0f172a");
        let _ = gradient.add_color_stop(1.0, "#1e293b");
        ctx.set_fill_style_str("#0f172a");
        ctx.fill_rect(0.0, 0.0, width, height * 0.6);
    }

    // Ground
    ctx.set_fill_style_str("#1a3a10");
    ctx.fill_rect(0.0, height * 0.6, width, height * 0.4);

    // Distant hills
    ctx.set_fill_style_str("#162e0d");
    ctx.begin_path();
    ctx.move_to(0.0, height * 0.6);
    ctx.quadratic_curve_to(width * 0.15, height * 0.48, width * 0.3, height * 0.58);
    ctx.quadratic_curve_to(width * 0.5, height * 0.50, width * 0.7, height * 0.56);
    ctx.quadratic_curve_to(width * 0.85, height * 0.52, width, height * 0.6);
    ctx.line_to(width, height * 0.6);
    ctx.close_path();
    ctx.fill();

    // Ground texture lines
    ctx.set_stroke_style_str("#1f4412");
    ctx.set_line_width(0.5);
    let mut y = height * 0.65;
    while y < height {
        ctx.begin_path();
        ctx.move_to(0.0, y);
        ctx.line_to(width, y + 5.0);
        ctx.stroke();
        y += 30.0;
    }
}

pub fn render_buildings(ctx: &CanvasRenderingContext2d, buildings: &[BuildingTarget]) {
    for building in buildings {
        match &building.visual {
            BuildingVisual::Healthy => render_healthy_building(ctx, building),
            BuildingVisual::Damaged { chaos_label } => {
                render_damaged_building(ctx, building, chaos_label)
            }
            BuildingVisual::Destroyed => render_destroyed_building(ctx, building),
        }
    }
}

fn render_healthy_building(ctx: &CanvasRenderingContext2d, b: &BuildingTarget) {
    let x = b.position.x - BUILDING_WIDTH / 2.0;
    let y = b.position.y - BUILDING_HEIGHT / 2.0;

    // Wall
    ctx.set_fill_style_str("#6b7280");
    ctx.fill_rect(x + 10.0, y + 25.0, BUILDING_WIDTH - 20.0, BUILDING_HEIGHT - 25.0);

    // Roof (triangle)
    ctx.set_fill_style_str("#92400e");
    ctx.begin_path();
    ctx.move_to(x + 5.0, y + 25.0);
    ctx.line_to(x + BUILDING_WIDTH / 2.0, y + 5.0);
    ctx.line_to(x + BUILDING_WIDTH - 5.0, y + 25.0);
    ctx.close_path();
    ctx.fill();

    // Door
    ctx.set_fill_style_str("#44403c");
    ctx.fill_rect(
        x + BUILDING_WIDTH / 2.0 - 5.0,
        y + BUILDING_HEIGHT - 18.0,
        10.0,
        18.0,
    );

    // Window
    ctx.set_fill_style_str("#fbbf24");
    ctx.set_global_alpha(0.6);
    ctx.fill_rect(x + 15.0, y + 35.0, 8.0, 8.0);
    ctx.fill_rect(x + BUILDING_WIDTH - 23.0, y + 35.0, 8.0, 8.0);
    ctx.set_global_alpha(1.0);

    // Name label
    ctx.set_fill_style_str("#e2e8f0");
    ctx.set_font("11px Outfit, sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(&b.name, b.position.x, b.position.y - BUILDING_HEIGHT / 2.0 - 6.0);
}

fn render_damaged_building(
    ctx: &CanvasRenderingContext2d,
    b: &BuildingTarget,
    chaos_label: &str,
) {
    let x = b.position.x - BUILDING_WIDTH / 2.0;
    let y = b.position.y - BUILDING_HEIGHT / 2.0;

    // Damaged wall (darker)
    ctx.set_fill_style_str("#4b5563");
    ctx.fill_rect(x + 10.0, y + 25.0, BUILDING_WIDTH - 20.0, BUILDING_HEIGHT - 25.0);

    // Damaged roof
    ctx.set_fill_style_str("#7c2d12");
    ctx.begin_path();
    ctx.move_to(x + 5.0, y + 25.0);
    ctx.line_to(x + BUILDING_WIDTH / 2.0, y + 8.0);
    ctx.line_to(x + BUILDING_WIDTH - 5.0, y + 25.0);
    ctx.close_path();
    ctx.fill();

    // Poison cloud
    ctx.set_fill_style_str("#22c55e");
    ctx.set_global_alpha(0.25);
    ctx.begin_path();
    let _ = ctx.arc(
        b.position.x,
        b.position.y - 5.0,
        BUILDING_WIDTH * 0.6,
        0.0,
        std::f64::consts::PI * 2.0,
    );
    ctx.fill();
    ctx.set_global_alpha(1.0);

    // Name label
    ctx.set_fill_style_str("#fbbf24");
    ctx.set_font("11px Outfit, sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(&b.name, b.position.x, b.position.y - BUILDING_HEIGHT / 2.0 - 6.0);

    // Chaos label
    ctx.set_fill_style_str("#22c55e");
    ctx.set_font("9px Outfit, sans-serif");
    let _ = ctx.fill_text(chaos_label, b.position.x, b.position.y + BUILDING_HEIGHT / 2.0 + 14.0);
}

fn render_destroyed_building(ctx: &CanvasRenderingContext2d, b: &BuildingTarget) {
    let x = b.position.x - BUILDING_WIDTH / 2.0;
    let y = b.position.y + BUILDING_HEIGHT / 2.0 - 15.0;

    // Rubble
    ctx.set_fill_style_str("#57534e");
    ctx.fill_rect(x + 8.0, y, 12.0, 8.0);
    ctx.fill_rect(x + 22.0, y - 3.0, 10.0, 11.0);
    ctx.fill_rect(x + 35.0, y + 2.0, 14.0, 6.0);
    ctx.set_fill_style_str("#44403c");
    ctx.fill_rect(x + 14.0, y - 5.0, 8.0, 5.0);
    ctx.fill_rect(x + 30.0, y - 2.0, 6.0, 4.0);

    // Scorch marks
    ctx.set_fill_style_str("#292524");
    ctx.set_global_alpha(0.4);
    ctx.begin_path();
    let _ = ctx.ellipse(
        b.position.x,
        y + 10.0,
        BUILDING_WIDTH * 0.5,
        8.0,
        0.0,
        0.0,
        std::f64::consts::PI * 2.0,
    );
    ctx.fill();
    ctx.set_global_alpha(1.0);

    // Name label (struck through)
    ctx.set_fill_style_str("#ef4444");
    ctx.set_font("11px Outfit, sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(&b.name, b.position.x, b.position.y - BUILDING_HEIGHT / 2.0 - 6.0);
}
```

- [ ] **Step 3: Implement the page with canvas and game loop**

Replace `crates/siege-console/src/app/features/battlefield/page.rs`:

```rust
use dioxus::prelude::*;
use siege_api_client::TopicResource;

use crate::components::ui::toast::Toaster;
use crate::state::AppState;

use super::canvas;
use super::state::*;

#[component]
pub fn BattlefieldPage() -> Element {
    let state = use_context::<AppState>();
    let toaster = use_context::<Toaster>();

    let mut topics: Signal<Vec<TopicResource>> = use_signal(Vec::new);
    let mut buildings: Signal<Vec<BuildingTarget>> = use_signal(Vec::new);
    let phase: Signal<Phase> = use_signal(|| Phase::Idle);
    let selected_weapon: Signal<Option<Weapon>> = use_signal(|| None);
    let aim_target: Signal<Option<Point>> = use_signal(|| None);
    let projectile: Signal<Option<Projectile>> = use_signal(|| None);
    let cooldowns: Signal<[f64; 2]> = use_signal(|| [0.0; 2]);

    // Fetch topics on mount
    use_hook(move || {
        let client = state.client();
        spawn(async move {
            match client.list_topics().await {
                Ok(list) => topics.set(list),
                Err(_) => {}
            }
        });
    });

    // Rebuild buildings when topics change
    use_effect(move || {
        let topic_list = topics();
        let names: Vec<String> = topic_list.iter().map(|t| t.name.clone()).collect();
        // Preserve visual state for existing buildings
        let old = buildings();
        let mut new = build_targets(&names, 900.0, 550.0);
        for b in &mut new {
            if let Some(existing) = old.iter().find(|o| o.name == b.name) {
                b.visual = existing.visual.clone();
            }
        }
        buildings.set(new);
    });

    // Start render loop
    use_hook(move || {
        canvas::start_render_loop("battlefield-canvas", move |ctx, w, h, timestamp| {
            ctx.clear_rect(0.0, 0.0, w, h);
            canvas::scene::render_terrain(ctx, w, h);
            canvas::scene::render_buildings(ctx, &buildings());

            // Render weapon icons on left side
            render_weapon_positions(ctx, h);

            // Render aim trajectory
            if let Phase::Aiming { weapon } = phase() {
                if let Some(target) = aim_target() {
                    let start = weapon.launch_origin(h);
                    let control = canvas::projectile::arc_control_point(start, target);
                    canvas::aiming::render_trajectory_preview(
                        ctx,
                        start,
                        control,
                        target,
                        weapon.projectile_color(),
                    );
                }
            }

            // Render active projectile
            if let Some(proj) = projectile() {
                let elapsed = timestamp - proj.start_time;
                let duration = proj.weapon.flight_duration_ms();
                let t = (elapsed / duration).min(1.0);
                render_active_projectile(ctx, &proj, t);
            }
        });
    });

    let standing = buildings().iter().filter(|b| b.visual != BuildingVisual::Destroyed).count();

    rsx! {
        div {
            class: "flex-1 flex flex-col relative",
            tabindex: "0",
            onmounted: move |e: MountedEvent| {
                spawn(async move {
                    let _ = e.set_focus(true).await;
                });
            },
            onkeydown: move |e: KeyboardEvent| {
                handle_keydown(e, selected_weapon, phase, cooldowns);
            },

            // HUD top bar
            div { class: "flex items-center justify-between px-6 py-3 border-b border-border bg-background/50 z-10",
                div { class: "flex items-center gap-3",
                    Link {
                        to: crate::routes::Route::TopicsPage {},
                        class: "text-xs text-muted-foreground hover:text-foreground transition-colors",
                        "\u{2190} Topics"
                    }
                    h1 { class: "text-sm font-bold text-foreground tracking-tight", "The Battlefield" }
                }
                span { class: "text-xs text-muted-foreground",
                    "{standing} target{if standing != 1 {\"s\"} else {\"\"}} standing"
                }
            }

            // Canvas
            div { class: "flex-1 relative",
                canvas {
                    id: "battlefield-canvas",
                    class: "w-full h-full block",
                    style: if selected_weapon().is_some() && phase() != Phase::Firing { "cursor: crosshair" } else { "" },
                    onmousedown: move |e: MouseEvent| {
                        let Some(weapon) = selected_weapon() else { return };
                        if phase() == Phase::Firing { return };
                        let coords = e.element_coordinates();
                        let target = Point { x: coords.x, y: coords.y };
                        aim_target.set(Some(target));
                        phase.set(Phase::Aiming { weapon });
                    },
                    onmousemove: move |e: MouseEvent| {
                        if let Phase::Aiming { .. } = phase() {
                            let coords = e.element_coordinates();
                            aim_target.set(Some(Point { x: coords.x, y: coords.y }));
                        }
                    },
                    onmouseup: move |e: MouseEvent| {
                        let Phase::Aiming { weapon } = phase() else { return };
                        let coords = e.element_coordinates();
                        let end = Point { x: coords.x, y: coords.y };
                        let start = weapon.launch_origin(550.0);
                        let control = canvas::projectile::arc_control_point(start, end);

                        let proj = Projectile {
                            weapon,
                            start,
                            control,
                            end,
                            start_time: canvas::now(),
                        };
                        projectile.set(Some(proj.clone()));
                        phase.set(Phase::Firing);
                        aim_target.set(None);

                        // Start cooldown
                        let mut cd = cooldowns();
                        cd[weapon.slot_index()] = canvas::now() + weapon.cooldown_ms();
                        cooldowns.set(cd);

                        // After flight, check hit
                        let duration_ms = weapon.flight_duration_ms() as i32;
                        let buildings_snap = buildings();
                        let client = state.client();
                        spawn(async move {
                            gloo_timers::future::TimeoutFuture::new(duration_ms as u32).await;
                            projectile.set(None);
                            phase.set(Phase::Idle);
                            selected_weapon.set(None);

                            // Check collision
                            let hit = buildings_snap.iter().find(|b| {
                                b.visual != BuildingVisual::Destroyed && b.hitbox.contains(end)
                            });

                            if let Some(target_building) = hit {
                                let name = target_building.name.clone();
                                let topic = client.topic(&name);
                                let result = match weapon {
                                    Weapon::Crossbow => {
                                        crate::chaos_action::ChaosAction::PoisonPills
                                            .execute(&topic)
                                            .await
                                    }
                                    Weapon::Trebuchet => {
                                        crate::chaos_action::ChaosAction::Delete
                                            .execute(&topic)
                                            .await
                                    }
                                };
                                match result {
                                    Ok(()) => {
                                        // Update building visual
                                        let mut current = buildings();
                                        if let Some(b) = current.iter_mut().find(|b| b.name == name) {
                                            match weapon {
                                                Weapon::Crossbow => {
                                                    b.visual = BuildingVisual::Damaged {
                                                        chaos_label: "poison pills".into(),
                                                    };
                                                }
                                                Weapon::Trebuchet => {
                                                    b.visual = BuildingVisual::Destroyed;
                                                }
                                            }
                                        }
                                        buildings.set(current);
                                        let msg = match weapon {
                                            Weapon::Crossbow => {
                                                format!("Poison pills launched at '{name}' \u{2014} 10 messages sent")
                                            }
                                            Weapon::Trebuchet => {
                                                format!("'{name}' has been razed to the ground")
                                            }
                                        };
                                        toaster.success(msg);
                                    }
                                    Err(e) => {
                                        toaster.error(format!("Attack failed: {e}"));
                                    }
                                }
                            } else {
                                toaster.error("The shot lands in the dirt. Wasted.");
                            }
                        });
                    },
                    oncontextmenu: move |e: MouseEvent| {
                        e.prevent_default();
                        if let Phase::Aiming { .. } = phase() {
                            phase.set(Phase::Idle);
                            aim_target.set(None);
                        }
                    },
                }
            }

            // Action bar
            super::action_bar::ActionBar {
                selected: selected_weapon(),
                cooldowns: cooldowns(),
                on_select: move |weapon: Weapon| {
                    if phase() == Phase::Firing { return }
                    if cooldowns()[weapon.slot_index()] > canvas::now() { return }
                    if selected_weapon() == Some(weapon) {
                        selected_weapon.set(None);
                        phase.set(Phase::Idle);
                    } else {
                        selected_weapon.set(Some(weapon));
                        phase.set(Phase::Idle);
                    }
                },
            }
        }
    }
}

fn handle_keydown(
    e: KeyboardEvent,
    mut selected_weapon: Signal<Option<Weapon>>,
    mut phase: Signal<Phase>,
    cooldowns: Signal<[f64; 2]>,
) {
    match e.key().as_str() {
        "1" => {
            if phase() == Phase::Firing {
                return;
            }
            if cooldowns()[Weapon::Crossbow.slot_index()] > canvas::now() {
                return;
            }
            selected_weapon.set(Some(Weapon::Crossbow));
            phase.set(Phase::Idle);
        }
        "2" => {
            if phase() == Phase::Firing {
                return;
            }
            if cooldowns()[Weapon::Trebuchet.slot_index()] > canvas::now() {
                return;
            }
            selected_weapon.set(Some(Weapon::Trebuchet));
            phase.set(Phase::Idle);
        }
        "Escape" => {
            if let Phase::Aiming { .. } = phase() {
                phase.set(Phase::Idle);
            }
            selected_weapon.set(None);
        }
        _ => {}
    }
}

fn render_weapon_positions(ctx: &CanvasRenderingContext2d, canvas_height: f64) {
    for weapon in [Weapon::Crossbow, Weapon::Trebuchet] {
        let origin = weapon.launch_origin(canvas_height);

        // Weapon base
        ctx.set_fill_style_str("#78716c");
        ctx.fill_rect(origin.x - 15.0, origin.y - 5.0, 30.0, 10.0);

        // Weapon body
        ctx.set_fill_style_str("#a8a29e");
        match weapon {
            Weapon::Crossbow => {
                ctx.fill_rect(origin.x - 3.0, origin.y - 18.0, 6.0, 18.0);
                ctx.fill_rect(origin.x - 12.0, origin.y - 15.0, 24.0, 4.0);
            }
            Weapon::Trebuchet => {
                ctx.fill_rect(origin.x - 4.0, origin.y - 25.0, 8.0, 25.0);
                ctx.fill_rect(origin.x - 20.0, origin.y - 25.0, 40.0, 5.0);
                ctx.set_fill_style_str("#78716c");
                ctx.fill_rect(origin.x + 14.0, origin.y - 28.0, 6.0, 8.0);
            }
        }

        // Label
        ctx.set_fill_style_str("#a8a29e");
        ctx.set_font("9px Outfit, sans-serif");
        ctx.set_text_align("center");
        let _ = ctx.fill_text(weapon.label(), origin.x, origin.y + 20.0);
    }
}

fn render_active_projectile(
    ctx: &CanvasRenderingContext2d,
    proj: &Projectile,
    t: f64,
) {
    let pos = canvas::projectile::bezier_point(t, proj.start, proj.control, proj.end);

    ctx.set_fill_style_str(proj.weapon.projectile_color());
    ctx.begin_path();
    let _ = ctx.arc(pos.x, pos.y, 5.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();

    // Trail
    ctx.set_global_alpha(0.3);
    for i in 1..=4 {
        let trail_t = (t - 0.03 * i as f64).max(0.0);
        let trail_pos = canvas::projectile::bezier_point(trail_t, proj.start, proj.control, proj.end);
        let radius = 4.0 - i as f64 * 0.7;
        ctx.begin_path();
        let _ = ctx.arc(trail_pos.x, trail_pos.y, radius, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    }
    ctx.set_global_alpha(1.0);
}
```

- [ ] **Step 4: Verify it compiles**

Run: `dx build --platform web` from `crates/siege-console/`
Expected: Builds successfully. Note: `gloo_timers` may not be available — see next step.

- [ ] **Step 5: Add gloo-timers dependency**

If `gloo_timers` is not available, add it to `crates/siege-console/Cargo.toml`:

```toml
gloo-timers = { version = "0.3", features = ["futures"] }
```

Alternatively, replace the `gloo_timers::future::TimeoutFuture` usage in `page.rs` with a raw `wasm-bindgen` timeout:

```rust
// Replace:
// gloo_timers::future::TimeoutFuture::new(duration_ms as u32).await;

// With a promise-based timeout:
let promise = js_sys::Promise::new(&mut |resolve, _| {
    if let Some(w) = web_sys::window() {
        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
            &resolve,
            duration_ms,
        );
    }
});
let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
```

And add `wasm-bindgen-futures` to dependencies if not already present, and `js-sys` (already present).

- [ ] **Step 6: Verify build again**

Run: `dx build --platform web` from `crates/siege-console/`
Expected: Builds successfully

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(battlefield): canvas game loop, terrain, and page skeleton"
```

---

### Task 5: Aiming trajectory preview

**Files:**
- Modify: `crates/siege-console/src/app/features/battlefield/canvas/aiming.rs`

- [ ] **Step 1: Implement trajectory preview rendering**

Replace `crates/siege-console/src/app/features/battlefield/canvas/aiming.rs`:

```rust
use web_sys::CanvasRenderingContext2d;

use super::super::state::Point;
use super::projectile::sample_trajectory;

pub fn render_trajectory_preview(
    ctx: &CanvasRenderingContext2d,
    start: Point,
    control: Point,
    end: Point,
    color: &str,
) {
    let points = sample_trajectory(start, control, end, 30);

    ctx.set_fill_style_str(color);
    ctx.set_global_alpha(0.5);

    for (i, point) in points.iter().enumerate() {
        let radius = if i % 3 == 0 { 3.0 } else { 1.5 };
        ctx.begin_path();
        let _ = ctx.arc(point.x, point.y, radius, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    }

    // Landing crosshair at endpoint
    ctx.set_global_alpha(0.7);
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(1.5);

    ctx.begin_path();
    ctx.move_to(end.x - 10.0, end.y);
    ctx.line_to(end.x + 10.0, end.y);
    ctx.stroke();

    ctx.begin_path();
    ctx.move_to(end.x, end.y - 10.0);
    ctx.line_to(end.x, end.y + 10.0);
    ctx.stroke();

    ctx.begin_path();
    let _ = ctx.arc(end.x, end.y, 8.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.stroke();

    ctx.set_global_alpha(1.0);
}
```

- [ ] **Step 2: Verify build**

Run: `dx build --platform web` from `crates/siege-console/`
Expected: Builds successfully

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(battlefield): aiming trajectory preview"
```

---

### Task 6: Action bar component

**Files:**
- Modify: `crates/siege-console/src/app/features/battlefield/action_bar.rs`

- [ ] **Step 1: Implement the WoW-style action bar**

Replace `crates/siege-console/src/app/features/battlefield/action_bar.rs`:

```rust
use dioxus::prelude::*;

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

            // Keybind badge
            span {
                class: "absolute top-0.5 left-1 text-[9px] font-bold text-muted-foreground",
                "{weapon.keybind()}"
            }

            // Weapon icon
            span { class: "text-lg leading-none", "{icon}" }

            // Action label
            span { class: "text-[8px] leading-tight text-muted-foreground font-medium text-center",
                "{weapon.action_label()}"
            }

            // Cooldown overlay
            if on_cooldown {
                CooldownOverlay { fraction: cooldown_fraction }
            }
        }
    }
}
```

- [ ] **Step 2: Verify build**

Run: `dx build --platform web` from `crates/siege-console/`
Expected: Builds successfully

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(battlefield): WoW-style action bar with weapon slots"
```

---

### Task 7: Cooldown overlay component

**Files:**
- Modify: `crates/siege-console/src/app/features/battlefield/cooldown.rs`

- [ ] **Step 1: Implement radial cooldown sweep**

Replace `crates/siege-console/src/app/features/battlefield/cooldown.rs`:

```rust
use dioxus::prelude::*;
use std::f64::consts::PI;

#[component]
pub fn CooldownOverlay(fraction: f64) -> Element {
    // SVG pie sweep from 12 o'clock, clockwise
    // fraction: 1.0 = fully covered (just fired), 0.0 = ready
    let angle = fraction * 2.0 * PI;
    let large_arc = if angle > PI { 1 } else { 0 };

    // Center at 32,32 in a 64x64 viewbox, radius 30
    let cx = 32.0_f64;
    let cy = 32.0_f64;
    let r = 30.0_f64;

    let end_x = cx + r * (angle - PI / 2.0).cos();
    let end_y = cy + r * (angle - PI / 2.0).sin();

    // Starting from 12 o'clock (cx, cy - r)
    let path = if fraction >= 0.999 {
        // Full circle — use two arcs to avoid degenerate path
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
```

- [ ] **Step 2: The cooldown needs to animate (re-render as time passes)**

The `CooldownOverlay` receives `fraction` which is computed from `cooldown_expires - now()`. Since `now()` is called once when the component renders, the fraction won't update by itself. To animate the cooldown, we need the action bar to re-render periodically while a cooldown is active.

Add a timer-based re-render in `action_bar.rs` `WeaponSlot`. Add a signal that triggers re-renders:

In `WeaponSlot`, add a tick signal that updates via RAF while on cooldown:

```rust
#[component]
fn WeaponSlot(
    weapon: Weapon,
    is_selected: bool,
    cooldown_expires: f64,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let mut tick = use_signal(|| 0u32);

    // Animate cooldown: force re-render every frame while on cooldown
    use_effect(move || {
        let now = canvas::now();
        if cooldown_expires <= now {
            return;
        }

        let tick_cb: std::rc::Rc<std::cell::RefCell<Option<wasm_bindgen::prelude::Closure<dyn FnMut(f64)>>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));
        let tick_cb_clone = tick_cb.clone();

        *tick_cb.borrow_mut() = Some(wasm_bindgen::prelude::Closure::wrap(Box::new(move |_ts: f64| {
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

    let _ = tick(); // subscribe to ticks for reactivity
    let now = canvas::now();
    let on_cooldown = cooldown_expires > now;
    // ... rest unchanged
```

- [ ] **Step 3: Verify build**

Run: `dx build --platform web` from `crates/siege-console/`
Expected: Builds successfully

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(battlefield): cooldown overlay with animated radial sweep"
```

---

### Task 8: Visual verification and polish

**Files:**
- Possibly adjust: any of the battlefield files for visual tweaks

- [ ] **Step 1: Start the dev server**

Run from `crates/siege-console/`: `dx serve --platform web`

- [ ] **Step 2: Navigate to /battlefield**

Open the browser, click "Battlefield" in the sidebar. Verify:
- The page renders with terrain (sky gradient, green ground, hills)
- Buildings appear for each topic (if topics exist — seed first if empty)
- Action bar at the bottom with two weapon buttons (Crossbow, Trebuchet)
- HUD shows "N targets standing"

- [ ] **Step 3: Test weapon selection**

- Press `1` — crossbow button should highlight with amber border
- Press `2` — trebuchet button should highlight
- Press `Escape` — selection should clear
- Click the buttons directly — same behavior

- [ ] **Step 4: Test aiming**

- Select a weapon (press `1`)
- Cursor should become crosshair over canvas
- Click and drag on canvas — dotted trajectory arc should appear from weapon to cursor
- Move mouse — arc follows cursor
- Right-click to cancel — arc disappears

- [ ] **Step 5: Test firing**

- Select weapon, aim at a building, release mouse
- Projectile should animate along the arc (~0.6s for crossbow)
- On hit: toast message, building visual changes (green cloud for poison, rubble for delete)
- Cooldown sweep appears on the used weapon button

- [ ] **Step 6: Test miss**

- Select weapon, aim at empty space between buildings, release
- Projectile flies and lands in dirt
- Toast: "The shot lands in the dirt. Wasted."
- Cooldown still consumed

- [ ] **Step 7: Test cooldown**

- Fire a weapon, immediately try pressing its keybind again
- Should be ignored while on cooldown
- Sweep animation fills clockwise and clears when ready

- [ ] **Step 8: Fix any visual issues found**

Adjust colors, positions, sizes as needed. Common issues:
- Canvas not filling container: check CSS `w-full h-full block` on canvas
- Buildings too small/large: adjust `BUILDING_WIDTH`/`BUILDING_HEIGHT` in `state.rs`
- Trajectory looks wrong: adjust `arc_control_point` multiplier (0.3)
- Hitboxes too strict: increase `HITBOX_PADDING` in `state.rs`

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat(battlefield): visual polish after manual testing"
```

---

### Task 9: Final integration test

**Files:**
- No new files

- [ ] **Step 1: Run all unit tests**

Run: `cargo test -p siege-console --lib`
Expected: All tests pass (state tests + trajectory tests)

- [ ] **Step 2: Verify full build**

Run: `dx build --platform web` from `crates/siege-console/`
Expected: Clean build with no warnings relevant to battlefield

- [ ] **Step 3: Test navigation doesn't break other pages**

- Navigate to Topics page (`/`) — should work as before
- Navigate to Wheel of Chaos (`/wheel`) — should work as before
- Navigate to Battlefield (`/battlefield`) — should render correctly
- Navigate back and forth — no errors in console

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat(battlefield): complete Kafka chaos mini-game"
```
