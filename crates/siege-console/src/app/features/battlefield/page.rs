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
        let old = buildings();
        let mut new = build_targets(&names, 900.0, 550.0);
        for b in &mut new {
            if let Some(existing) = old.iter().find(|o| o.name == b.name) {
                b.visual = existing.visual.clone();
            }
        }
        buildings.set(new);
    });

    use_hook(move || {
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
    for weapon in [Weapon::Crossbow, Weapon::Trebuchet] {
        let origin = weapon.launch_origin(canvas_height);

        ctx.set_fill_style_str("#78716c");
        ctx.fill_rect(origin.x - 15.0, origin.y - 5.0, 30.0, 10.0);

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

        ctx.set_fill_style_str("#a8a29e");
        ctx.set_font("9px Outfit, sans-serif");
        ctx.set_text_align("center");
        let _ = ctx.fill_text(weapon.label(), origin.x, origin.y + 20.0);
    }
}

fn render_active_projectile(ctx: &CanvasRenderingContext2d, proj: &Projectile, t: f64) {
    let pos = canvas::projectile::bezier_point(t, proj.start, proj.control, proj.end);

    ctx.set_fill_style_str(proj.weapon.projectile_color());
    ctx.begin_path();
    let _ = ctx.arc(pos.x, pos.y, 5.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();

    ctx.set_global_alpha(0.3);
    for i in 1..=4 {
        let trail_t = (t - 0.03 * i as f64).max(0.0);
        let trail_pos =
            canvas::projectile::bezier_point(trail_t, proj.start, proj.control, proj.end);
        let radius = 4.0 - i as f64 * 0.7;
        ctx.begin_path();
        let _ = ctx.arc(
            trail_pos.x,
            trail_pos.y,
            radius,
            0.0,
            std::f64::consts::PI * 2.0,
        );
        ctx.fill();
    }
    ctx.set_global_alpha(1.0);
}
