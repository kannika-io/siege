use dioxus::prelude::*;
use siege_api_client::TopicResource;
use web_sys::CanvasRenderingContext2d;

use crate::chaos_action::ChaosAction;
use crate::components::ui::toast::Toaster;
use crate::routes::Route;
use crate::state::AppState;

use super::canvas;
use super::state::*;

#[component]
pub fn BattlefieldPage() -> Element {
    let state = use_context::<AppState>();
    let mut toaster = use_context::<Toaster>();

    let mut topics: Signal<Vec<TopicResource>> = use_signal(Vec::new);
    let mut buildings: Signal<Vec<BuildingTarget>> = use_signal(Vec::new);
    let mut phase: Signal<Phase> = use_signal(|| Phase::Idle);
    let mut selected_weapon: Signal<Option<Weapon>> = use_signal(|| None);
    let mut aim_target: Signal<Option<Point>> = use_signal(|| None);
    let mut projectile: Signal<Option<Projectile>> = use_signal(|| None);
    let mut cooldowns: Signal<[f64; 2]> = use_signal(|| [0.0; 2]);
    let mut current_time: Signal<f64> = use_signal(|| 0.0);

    use_hook(move || {
        let client = state.client();
        spawn(async move {
            match client.list_topics().await {
                Ok(list) => topics.set(list),
                Err(_) => {}
            }
        });
    });

    use_effect(move || {
        let topic_list = topics();
        let names: Vec<String> = topic_list.iter().map(|t| t.name.clone()).collect();
        buildings.set(build_targets(&names, 900.0, 550.0));
    });

    use_effect(move || {
        canvas::start_render_loop("battlefield-canvas", move |ctx, w, h, timestamp| {
            ctx.clear_rect(0.0, 0.0, w, h);
            canvas::scene::render_terrain(ctx, w, h);
            canvas::scene::render_buildings(ctx, &buildings());

            render_weapon_positions(ctx, h);

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

            if let Some(proj) = projectile() {
                let elapsed = timestamp - proj.start_time;
                let duration = proj.weapon.flight_duration_ms();
                let t = (elapsed / duration).min(1.0);
                render_active_projectile(ctx, &proj, t);
            }
        });
    });

    use_hook(move || {
        spawn(async move {
            loop {
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    if let Some(w) = web_sys::window() {
                        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve, 50,
                        );
                    }
                });
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                let cd = cooldowns.peek();
                let now = canvas::now();
                if cd[0] > now || cd[1] > now {
                    current_time.set(now);
                }
            }
        });
    });

    let standing = buildings()
        .iter()
        .filter(|b| b.visual != BuildingVisual::Destroyed)
        .count();
    let standing_label = if standing != 1 {
        format!("{standing} targets standing")
    } else {
        format!("{standing} target standing")
    };

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
                handle_keydown(e, &mut selected_weapon, &mut phase, &cooldowns);
            },

            div { class: "flex items-center justify-between px-6 py-3 border-b border-border bg-background/50 z-10",
                div { class: "flex items-center gap-3",
                    Link {
                        to: Route::TopicsPage {},
                        class: "text-xs text-muted-foreground hover:text-foreground transition-colors",
                        "\u{2190} Topics"
                    }
                    h1 { class: "text-sm font-bold text-foreground tracking-tight", "The Battlefield" }
                }
                span { class: "text-xs text-muted-foreground",
                    "{standing_label}"
                }
            }

            div { class: "flex-1 relative",
                canvas {
                    id: "battlefield-canvas",
                    class: "w-full h-full block",
                    style: if selected_weapon().is_some() && phase() != Phase::Firing { "cursor: crosshair" } else { "" },
                    onmousedown: move |e: MouseEvent| {
                        let Some(weapon) = selected_weapon() else {
                            return;
                        };
                        if phase() == Phase::Firing {
                            return;
                        }
                        let coords = e.element_coordinates();
                        aim_target.set(Some(Point {
                            x: coords.x,
                            y: coords.y,
                        }));
                        phase.set(Phase::Aiming { weapon });
                    },
                    onmousemove: move |e: MouseEvent| {
                        if let Phase::Aiming { .. } = phase() {
                            let coords = e.element_coordinates();
                            aim_target.set(Some(Point {
                                x: coords.x,
                                y: coords.y,
                            }));
                        }
                    },
                    onmouseup: move |e: MouseEvent| {
                        let Phase::Aiming { weapon } = phase() else {
                            return;
                        };
                        let coords = e.element_coordinates();
                        let end = Point {
                            x: coords.x,
                            y: coords.y,
                        };
                        let start = weapon.launch_origin(550.0);
                        let control = canvas::projectile::arc_control_point(start, end);

                        let proj = Projectile {
                            weapon,
                            start,
                            control,
                            end,
                            start_time: canvas::now(),
                        };
                        projectile.set(Some(proj));
                        phase.set(Phase::Firing);
                        aim_target.set(None);

                        let mut cd = cooldowns();
                        cd[weapon.slot_index()] = canvas::now() + weapon.cooldown_ms();
                        cooldowns.set(cd);

                        let duration_ms = weapon.flight_duration_ms() as i32;
                        let buildings_snap = buildings();
                        let client = state.client();
                        spawn(async move {
                            let promise = js_sys::Promise::new(&mut |resolve, _| {
                                if let Some(w) = web_sys::window() {
                                    let _ = w
                                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                                            &resolve,
                                            duration_ms,
                                        );
                                }
                            });
                            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;

                            projectile.set(None);
                            phase.set(Phase::Idle);
                            selected_weapon.set(None);

                            let hit = buildings_snap.iter().find(|b| {
                                b.visual != BuildingVisual::Destroyed && b.hitbox.contains(end)
                            });

                            if let Some(target_building) = hit {
                                let name = target_building.name.clone();
                                let topic = client.topic(&name);
                                let result = match weapon {
                                    Weapon::Crossbow => {
                                        ChaosAction::PoisonPills.execute(&topic).await
                                    }
                                    Weapon::Trebuchet => ChaosAction::Delete.execute(&topic).await,
                                };
                                match result {
                                    Ok(()) => {
                                        let mut current = buildings();
                                        if let Some(b) =
                                            current.iter_mut().find(|b| b.name == name)
                                        {
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
                                            Weapon::Crossbow => format!(
                                                "Poison pills launched at '{name}' \u{2014} 10 messages sent"
                                            ),
                                            Weapon::Trebuchet => {
                                                format!(
                                                    "'{name}' has been razed to the ground"
                                                )
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

            super::action_bar::ActionBar {
                selected: selected_weapon(),
                cooldowns: cooldowns(),
                current_time: current_time(),
                on_select: move |weapon: Weapon| {
                    if phase() == Phase::Firing {
                        return;
                    }
                    if cooldowns()[weapon.slot_index()] > canvas::now() {
                        return;
                    }
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
    selected_weapon: &mut Signal<Option<Weapon>>,
    phase: &mut Signal<Phase>,
    cooldowns: &Signal<[f64; 2]>,
) {
    match e.key() {
        Key::Character(ref c) if c == "1" => {
            if phase() == Phase::Firing {
                return;
            }
            if cooldowns()[Weapon::Crossbow.slot_index()] > canvas::now() {
                return;
            }
            selected_weapon.set(Some(Weapon::Crossbow));
            phase.set(Phase::Idle);
        }
        Key::Character(ref c) if c == "2" => {
            if phase() == Phase::Firing {
                return;
            }
            if cooldowns()[Weapon::Trebuchet.slot_index()] > canvas::now() {
                return;
            }
            selected_weapon.set(Some(Weapon::Trebuchet));
            phase.set(Phase::Idle);
        }
        Key::Escape => {
            if let Phase::Aiming { .. } = phase() {
                phase.set(Phase::Idle);
            }
            selected_weapon.set(None);
        }
        _ => {}
    }
}

fn render_weapon_positions(ctx: &CanvasRenderingContext2d, canvas_height: f64) {
    let p = 3.0_f64; // pixel size matching scene.rs PX

    for weapon in [Weapon::Crossbow, Weapon::Trebuchet] {
        let origin = weapon.launch_origin(canvas_height);
        let ox = (origin.x / p).floor() * p;
        let oy = (origin.y / p).floor() * p;

        match weapon {
            Weapon::Crossbow => {
                // Pixel art crossbow (~12x12 grid scaled by p)
                let wood_dark = "#6b4f12";
                let wood_light = "#8b6914";
                let limb = "#4a3a0a";
                let string_color = "#a8a29e";

                // Stock (horizontal bar)
                for i in 0..6 {
                    let c = if i % 2 == 0 { wood_light } else { wood_dark };
                    ctx.set_fill_style_str(c);
                    ctx.fill_rect(ox - p * 3.0 + (i as f64) * p, oy, p, p);
                }
                // Stock thickness row below
                for i in 0..6 {
                    let c = if i % 2 == 0 { wood_dark } else { wood_light };
                    ctx.set_fill_style_str(c);
                    ctx.fill_rect(ox - p * 3.0 + (i as f64) * p, oy + p, p, p);
                }

                // Bow limbs (vertical, at the front of the stock)
                ctx.set_fill_style_str(limb);
                // Upper limb
                for i in 1..=4 {
                    ctx.fill_rect(ox + p * 2.0, oy - (i as f64) * p, p, p);
                }
                // Limb tip
                ctx.fill_rect(ox + p * 3.0, oy - p * 4.0, p, p);
                // Lower limb
                for i in 1..=4 {
                    ctx.fill_rect(ox + p * 2.0, oy + p + (i as f64) * p, p, p);
                }
                // Limb tip
                ctx.fill_rect(ox + p * 3.0, oy + p * 5.0, p, p);

                // Bowstring
                ctx.set_fill_style_str(string_color);
                ctx.fill_rect(ox + p * 3.0, oy - p * 3.0, p, p);
                ctx.fill_rect(ox + p * 3.0, oy + p * 4.0, p, p);
                // String center (pulled back)
                ctx.fill_rect(ox + p, oy, p, p);
                ctx.fill_rect(ox + p, oy + p, p, p);

                // Trigger/grip
                ctx.set_fill_style_str(wood_dark);
                ctx.fill_rect(ox - p * 2.0, oy + p * 2.0, p, p * 2.0);
            }
            Weapon::Trebuchet => {
                // Pixel art trebuchet (~16x16 grid scaled by p)
                let wood_dark = "#6b4f12";
                let wood_light = "#8b6914";
                let frame = "#5a4510";
                let weight = "#4a4a4a";

                // Base platform
                ctx.set_fill_style_str(wood_dark);
                ctx.fill_rect(ox - p * 6.0, oy + p * 2.0, p * 12.0, p);

                // A-frame supports (two angled legs)
                ctx.set_fill_style_str(frame);
                // Left leg
                for i in 0..5 {
                    ctx.fill_rect(
                        ox - p * 2.0 - (i as f64) * p * 0.5,
                        oy - (i as f64) * p,
                        p,
                        p,
                    );
                }
                // Right leg
                for i in 0..5 {
                    ctx.fill_rect(
                        ox + p + (i as f64) * p * 0.5,
                        oy - (i as f64) * p,
                        p,
                        p,
                    );
                }

                // Pivot point
                ctx.set_fill_style_str(weight);
                ctx.fill_rect(ox - p * 0.5, oy - p * 4.0, p, p);

                // Throwing arm (long beam through pivot)
                ctx.set_fill_style_str(wood_light);
                for i in 0..10 {
                    ctx.fill_rect(ox - p * 5.0 + (i as f64) * p, oy - p * 4.0, p, p);
                }

                // Counterweight (left end, hanging)
                ctx.set_fill_style_str(weight);
                ctx.fill_rect(ox - p * 5.0, oy - p * 3.0, p * 2.0, p * 2.0);
                ctx.fill_rect(ox - p * 5.0, oy - p * 1.0, p * 2.0, p);

                // Sling (right end)
                ctx.set_fill_style_str("#a8a29e");
                ctx.fill_rect(ox + p * 4.0, oy - p * 3.0, p, p);
                ctx.fill_rect(ox + p * 4.5, oy - p * 2.0, p, p);

                // Wheels
                ctx.set_fill_style_str(wood_dark);
                ctx.fill_rect(ox - p * 5.0, oy + p * 3.0, p * 2.0, p * 2.0);
                ctx.fill_rect(ox + p * 3.0, oy + p * 3.0, p * 2.0, p * 2.0);
            }
        }

        // Label
        ctx.set_fill_style_str("#a8a29e");
        ctx.set_font("9px Outfit, sans-serif");
        ctx.set_text_align("center");
        let _ = ctx.fill_text(weapon.label(), origin.x, origin.y + p * 8.0);
    }
}

fn render_active_projectile(ctx: &CanvasRenderingContext2d, proj: &Projectile, t: f64) {
    let p = 3.0_f64; // pixel size
    let pos = canvas::projectile::bezier_point(t, proj.start, proj.control, proj.end);
    let px = (pos.x / p).floor() * p;
    let py = (pos.y / p).floor() * p;

    /// Simple hash for trail scatter.
    fn trail_hash(a: i32, b: i32) -> u32 {
        let mut h = (a as u32).wrapping_mul(2654435761);
        h ^= (b as u32).wrapping_mul(2246822519);
        h ^= h >> 13;
        h
    }

    match proj.weapon {
        Weapon::Crossbow => {
            // Crossbow bolt: elongated pixel shape
            let bolt_color = "#22c55e";
            let tip_color = "#39ff14";
            // Bolt body (3 pixels long, 1 pixel tall)
            ctx.set_fill_style_str(bolt_color);
            ctx.fill_rect(px - p * 2.0, py, p, p);
            ctx.fill_rect(px - p, py, p, p);
            ctx.fill_rect(px, py, p, p);
            // Tip
            ctx.set_fill_style_str(tip_color);
            ctx.fill_rect(px + p, py, p, p);

            // Green trail: scattered individual pixels
            ctx.set_global_alpha(0.5);
            let trail_colors = ["#39ff14", "#22cc00", "#22c55e"];
            for i in 1..=8 {
                let trail_t = (t - 0.015 * i as f64).max(0.0);
                let tp =
                    canvas::projectile::bezier_point(trail_t, proj.start, proj.control, proj.end);
                let h = trail_hash(i, (t * 100.0) as i32);
                let offset_x = ((h % 5) as f64 - 2.0) * p;
                let offset_y = ((h >> 4) % 5) as f64 * p - p * 2.0;
                let color_idx = (h as usize) % trail_colors.len();
                if let Some(color) = trail_colors.get(color_idx) {
                    let tx = (tp.x / p).floor() * p + offset_x;
                    let ty = (tp.y / p).floor() * p + offset_y;
                    ctx.set_fill_style_str(color);
                    ctx.fill_rect(tx, ty, p, p);
                }
            }
            ctx.set_global_alpha(1.0);
        }
        Weapon::Trebuchet => {
            // Trebuchet boulder: 2x2 pixel block
            let rock1 = "#8b8b8b";
            let rock2 = "#696969";
            ctx.set_fill_style_str(rock1);
            ctx.fill_rect(px, py, p, p);
            ctx.fill_rect(px + p, py + p, p, p);
            ctx.set_fill_style_str(rock2);
            ctx.fill_rect(px + p, py, p, p);
            ctx.fill_rect(px, py + p, p, p);

            // Orange/red trail: scattered fire pixels
            ctx.set_global_alpha(0.5);
            let trail_colors = ["#ff6600", "#cc4400", "#ef4444", "#ff8800"];
            for i in 1..=10 {
                let trail_t = (t - 0.012 * i as f64).max(0.0);
                let tp =
                    canvas::projectile::bezier_point(trail_t, proj.start, proj.control, proj.end);
                let h = trail_hash(i, (t * 80.0) as i32);
                let offset_x = ((h % 7) as f64 - 3.0) * p;
                let offset_y = ((h >> 4) % 7) as f64 * p - p * 3.0;
                let color_idx = (h as usize) % trail_colors.len();
                if let Some(color) = trail_colors.get(color_idx) {
                    let tx = (tp.x / p).floor() * p + offset_x;
                    let ty = (tp.y / p).floor() * p + offset_y;
                    ctx.set_fill_style_str(color);
                    ctx.fill_rect(tx, ty, p, p);
                }
            }
            ctx.set_global_alpha(1.0);
        }
    }
}
