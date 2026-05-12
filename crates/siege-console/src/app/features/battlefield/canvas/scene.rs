use web_sys::CanvasRenderingContext2d;

use super::super::state::{BuildingTarget, BuildingVisual, BUILDING_HEIGHT, BUILDING_WIDTH};

/// Pixel size for the chunky retro look (3x3 canvas pixels per "pixel").
const PX: f64 = 3.0;

/// Draw a single "pixel" as a filled square.
fn px(ctx: &CanvasRenderingContext2d, x: f64, y: f64, size: f64, color: &str) {
    ctx.set_fill_style_str(color);
    ctx.fill_rect(x, y, size, size);
}

/// Snap a coordinate to the pixel grid.
fn snap(v: f64) -> f64 {
    (v / PX).floor() * PX
}

/// Simple deterministic hash from two coordinates to produce pseudo-random scatter.
fn hash_xy(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393);
    h = h.wrapping_add((y as u32).wrapping_mul(668265263));
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    h
}

pub fn render_terrain(ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
    // --- Dark pixel art sky ---
    let sky_color = "#0f0f23";
    ctx.set_fill_style_str(sky_color);
    ctx.fill_rect(0.0, 0.0, width, height * 0.6);

    // Stars: scattered 2x2 white/gray dots
    let star_size = 2.0;
    let sky_h = height * 0.6;
    let cols = (width / 18.0) as i32;
    let rows = (sky_h / 18.0) as i32;
    for row in 0..rows {
        for col in 0..cols {
            let h = hash_xy(col, row);
            if h % 7 == 0 {
                let sx = snap((col as f64) * 18.0 + (h % 13) as f64);
                let sy = snap((row as f64) * 18.0 + ((h >> 4) % 11) as f64);
                let star_color = if h % 3 == 0 { "#ffffff" } else { "#cccccc" };
                px(ctx, sx, sy, star_size, star_color);
            }
        }
    }

    // --- Pixelated rolling hills (stepped rectangular blocks) ---
    let hill_step = PX * 2.0; // width of each step column
    let ground_line = height * 0.6;
    let mut hx = 0.0;
    while hx < width {
        // Two overlapping sine-ish waves for hill shape
        let ratio = hx / width;
        let wave1 = (ratio * std::f64::consts::PI * 2.5).sin() * height * 0.06;
        let wave2 = (ratio * std::f64::consts::PI * 1.2 + 1.0).sin() * height * 0.04;
        let hill_top = snap(ground_line - 12.0 - wave1 - wave2);
        let hill_h = ground_line - hill_top;
        if hill_h > 0.0 {
            // Darker green for distant hills
            px(ctx, snap(hx), hill_top, hill_step, "#1a3a10");
            ctx.fill_rect(snap(hx), hill_top + PX, hill_step, hill_h - PX);
        }
        hx += hill_step;
    }

    // --- Ground fill ---
    ctx.set_fill_style_str("#2d5a1e");
    ctx.fill_rect(0.0, ground_line, width, height - ground_line);

    // --- Dithered grass texture (checkerboard dark/light green) ---
    let ground_top = ground_line;
    let dark_green = "#2d5a1e";
    let light_green = "#3a7a28";
    let mut gy = snap(ground_top);
    while gy < height {
        let mut gx = 0.0;
        while gx < width {
            let grid_x = (gx / PX) as i32;
            let grid_y = (gy / PX) as i32;
            let color = if (grid_x + grid_y) % 2 == 0 {
                dark_green
            } else {
                light_green
            };
            px(ctx, gx, gy, PX, color);
            gx += PX;
        }
        gy += PX;
    }

    // --- Dirt path in the middle ---
    let path_x = snap(width * 0.28);
    let path_w = PX * 4.0;
    let dirt_dark = "#6b4f12";
    let dirt_light = "#8b6914";
    let mut py = snap(ground_top);
    while py < height {
        let grid_y = (py / PX) as i32;
        // Slight wobble
        let wobble = if grid_y % 3 == 0 { PX } else { 0.0 };
        let mut px_x = path_x + wobble;
        let end_x = path_x + path_w + wobble;
        while px_x < end_x {
            let grid_x = (px_x / PX) as i32;
            let c = if (grid_x + grid_y) % 2 == 0 {
                dirt_dark
            } else {
                dirt_light
            };
            px(ctx, px_x, py, PX, c);
            px_x += PX;
        }
        py += PX;
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
    let base_x = snap(b.position.x - BUILDING_WIDTH / 2.0);
    let base_y = snap(b.position.y - BUILDING_HEIGHT / 2.0);

    let wall_x = base_x + PX * 3.0;
    let wall_y = base_y + PX * 6.0;
    let wall_w = BUILDING_WIDTH - PX * 6.0;
    let wall_h = BUILDING_HEIGHT - PX * 6.0;

    // --- Stone wall with brick-like texture ---
    let stone1 = "#8b8b8b";
    let stone2 = "#7a7a7a";
    let stone3 = "#696969";

    let mut wy = snap(wall_y);
    let row_height = PX;
    let mut brick_row = 0;
    while wy < wall_y + wall_h {
        let offset = if brick_row % 2 == 0 { 0.0 } else { PX * 2.0 };
        let mut wx = snap(wall_x);
        let mut brick_col = 0;
        while wx < wall_x + wall_w {
            let shifted = wx + offset;
            if shifted >= wall_x && shifted < wall_x + wall_w {
                let color = match (brick_row + brick_col) % 3 {
                    0 => stone1,
                    1 => stone2,
                    _ => stone3,
                };
                px(ctx, shifted, wy, PX, color);
            }
            wx += PX;
            brick_col += 1;
        }
        wy += row_height;
        brick_row += 1;
    }

    // --- Crenellations (battlements) at the top ---
    let merlon_w = PX * 2.0;
    let merlon_h = PX * 2.0;
    let gap = PX * 2.0;
    let mut mx = wall_x;
    let mut merlon_idx = 0;
    while mx + merlon_w <= wall_x + wall_w {
        // Alternating merlon / gap
        if merlon_idx % 2 == 0 {
            let color = if merlon_idx % 4 == 0 { stone1 } else { stone2 };
            ctx.set_fill_style_str(color);
            ctx.fill_rect(snap(mx), snap(wall_y - merlon_h), merlon_w, merlon_h);
        }
        mx += if merlon_idx % 2 == 0 { merlon_w } else { gap };
        merlon_idx += 1;
    }

    // --- Arrow slit windows (thin dark rectangles) ---
    let slit_color = "#1a1a2e";
    let slit_w = PX;
    let slit_h = PX * 3.0;
    // Left slit
    px(ctx, snap(wall_x + PX * 3.0), snap(wall_y + PX * 4.0), PX, slit_color);
    ctx.set_fill_style_str(slit_color);
    ctx.fill_rect(
        snap(wall_x + PX * 3.0),
        snap(wall_y + PX * 4.0),
        slit_w,
        slit_h,
    );
    // Right slit
    ctx.fill_rect(
        snap(wall_x + wall_w - PX * 4.0),
        snap(wall_y + PX * 4.0),
        slit_w,
        slit_h,
    );

    // --- Wooden door at the bottom ---
    let door_w = PX * 4.0;
    let door_h = PX * 5.0;
    let door_x = snap(wall_x + wall_w / 2.0 - door_w / 2.0);
    let door_y = snap(wall_y + wall_h - door_h);

    // Door frame (darker)
    ctx.set_fill_style_str("#6b4f12");
    ctx.fill_rect(door_x - PX, door_y - PX, door_w + PX * 2.0, door_h + PX);
    // Door panel
    ctx.set_fill_style_str("#8b6914");
    ctx.fill_rect(door_x, door_y, door_w, door_h);
    // Door details: horizontal plank lines
    px(ctx, door_x, snap(door_y + PX * 2.0), door_w, "#6b4f12");
    ctx.fill_rect(door_x, snap(door_y + PX * 2.0), door_w, 1.0);

    // --- Flag on top ---
    let flag_pole_x = snap(wall_x + wall_w / 2.0);
    let flag_pole_top = snap(wall_y - merlon_h - PX * 4.0);
    // Pole (1px wide line)
    ctx.set_fill_style_str("#4a4a4a");
    ctx.fill_rect(flag_pole_x, flag_pole_top, PX, PX * 4.0);
    // Flag banner
    ctx.set_fill_style_str("#c41e3a");
    ctx.fill_rect(flag_pole_x + PX, flag_pole_top, PX * 3.0, PX * 2.0);

    // --- Name label ---
    ctx.set_fill_style_str("#e2e8f0");
    ctx.set_font("11px Outfit, sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(&b.name, b.position.x, base_y - PX * 2.0);
}

fn render_damaged_building(ctx: &CanvasRenderingContext2d, b: &BuildingTarget, chaos_label: &str) {
    let base_x = snap(b.position.x - BUILDING_WIDTH / 2.0);
    let base_y = snap(b.position.y - BUILDING_HEIGHT / 2.0);

    let wall_x = base_x + PX * 3.0;
    let wall_y = base_y + PX * 6.0;
    let wall_w = BUILDING_WIDTH - PX * 6.0;
    let wall_h = BUILDING_HEIGHT - PX * 6.0;

    // --- Damaged stone wall (darker tones) ---
    let stone1 = "#7a7a7a";
    let stone2 = "#696969";
    let stone3 = "#5a5a5a";

    let mut wy = snap(wall_y);
    let mut brick_row = 0;
    while wy < wall_y + wall_h {
        let offset = if brick_row % 2 == 0 { 0.0 } else { PX * 2.0 };
        let mut wx = snap(wall_x);
        let mut brick_col = 0;
        while wx < wall_x + wall_w {
            let shifted = wx + offset;
            if shifted >= wall_x && shifted < wall_x + wall_w {
                let color = match (brick_row + brick_col) % 3 {
                    0 => stone1,
                    1 => stone2,
                    _ => stone3,
                };
                px(ctx, shifted, wy, PX, color);
            }
            wx += PX;
            brick_col += 1;
        }
        wy += PX;
        brick_row += 1;
    }

    // --- Damaged crenellations (gaps = missing pieces) ---
    let merlon_w = PX * 2.0;
    let merlon_h = PX * 2.0;
    let gap = PX * 2.0;
    let mut mx = wall_x;
    let mut merlon_idx = 0;
    while mx + merlon_w <= wall_x + wall_w {
        if merlon_idx % 2 == 0 {
            // Skip some merlons to show damage
            let h = hash_xy(merlon_idx, (base_x as i32) ^ 0x55);
            if h % 3 != 0 {
                let color = if merlon_idx % 4 == 0 { stone1 } else { stone2 };
                ctx.set_fill_style_str(color);
                ctx.fill_rect(snap(mx), snap(wall_y - merlon_h), merlon_w, merlon_h);
            }
        }
        mx += if merlon_idx % 2 == 0 { merlon_w } else { gap };
        merlon_idx += 1;
    }

    // --- Cracks in the wall (dark pixels) ---
    let crack_color = "#1a1a2e";
    // Diagonal crack from top-left
    for i in 0..5 {
        let cx = snap(wall_x + PX * (2.0 + i as f64));
        let cy = snap(wall_y + PX * (3.0 + i as f64));
        px(ctx, cx, cy, PX, crack_color);
    }
    // Short horizontal crack on right side
    for i in 0..3 {
        let cx = snap(wall_x + wall_w - PX * (3.0 + i as f64));
        let cy = snap(wall_y + PX * 8.0);
        px(ctx, cx, cy, PX, crack_color);
    }

    // --- Fallen bricks at the base ---
    let rubble_y = snap(wall_y + wall_h);
    px(ctx, snap(wall_x - PX), rubble_y, PX, stone2);
    px(ctx, snap(wall_x + PX * 2.0), snap(rubble_y + PX), PX, stone3);
    px(ctx, snap(wall_x + wall_w), rubble_y, PX, stone1);
    px(
        ctx,
        snap(wall_x + wall_w - PX * 3.0),
        snap(rubble_y + PX),
        PX,
        stone2,
    );

    // --- Door (slightly damaged) ---
    let door_w = PX * 4.0;
    let door_h = PX * 5.0;
    let door_x = snap(wall_x + wall_w / 2.0 - door_w / 2.0);
    let door_y = snap(wall_y + wall_h - door_h);
    ctx.set_fill_style_str("#6b4f12");
    ctx.fill_rect(door_x - PX, door_y - PX, door_w + PX * 2.0, door_h + PX);
    ctx.set_fill_style_str("#8b6914");
    ctx.fill_rect(door_x, door_y, door_w, door_h);

    // --- Poison cloud: scattered green pixels ---
    let cloud_cx = b.position.x;
    let cloud_cy = b.position.y - PX * 2.0;
    let cloud_radius = BUILDING_WIDTH * 0.6;
    let green1 = "#39ff14";
    let green2 = "#22cc00";

    ctx.set_global_alpha(0.6);
    for row in -8..=8 {
        for col in -10..=10 {
            let h = hash_xy(col + (base_x as i32), row + (base_y as i32));
            if h % 5 == 0 {
                let px_x = snap(cloud_cx + col as f64 * PX * 1.5);
                let px_y = snap(cloud_cy + row as f64 * PX * 1.5);
                let dx = px_x - cloud_cx;
                let dy = px_y - cloud_cy;
                if dx * dx + dy * dy < cloud_radius * cloud_radius {
                    let color = if h % 2 == 0 { green1 } else { green2 };
                    px(ctx, px_x, px_y, PX, color);
                }
            }
        }
    }
    ctx.set_global_alpha(1.0);

    // --- Name label ---
    ctx.set_fill_style_str("#fbbf24");
    ctx.set_font("11px Outfit, sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(&b.name, b.position.x, base_y - PX * 2.0);

    // --- Chaos label ---
    ctx.set_fill_style_str("#22c55e");
    ctx.set_font("9px Outfit, sans-serif");
    let _ = ctx.fill_text(
        chaos_label,
        b.position.x,
        b.position.y + BUILDING_HEIGHT / 2.0 + 14.0,
    );
}

fn render_destroyed_building(ctx: &CanvasRenderingContext2d, b: &BuildingTarget) {
    let base_x = snap(b.position.x - BUILDING_WIDTH / 2.0);
    let base_y = snap(b.position.y + BUILDING_HEIGHT / 2.0 - PX * 5.0);

    // --- Rubble pile: scattered gray/brown rectangles ---
    let rubble_colors = ["#5a5a5a", "#4a4a4a", "#3a3a3a", "#696969", "#6b4f12"];
    for row in 0..4 {
        for col in 0..8 {
            let h = hash_xy(col + (base_x as i32), row + (base_y as i32));
            if h % 3 != 2 {
                let rx = snap(base_x + col as f64 * PX * 2.5 + (h % 3) as f64);
                let ry = snap(base_y + row as f64 * PX * 1.5 - (h % 5) as f64);
                let size = PX + (h % 2) as f64 * PX;
                let color_idx = (h as usize) % rubble_colors.len();
                if let Some(color) = rubble_colors.get(color_idx) {
                    px(ctx, rx, ry, size, color);
                }
            }
        }
    }

    // --- Scorch marks: dark spots on the ground ---
    let scorch_color = "#1a1a1a";
    ctx.set_global_alpha(0.6);
    for i in 0..5 {
        let h = hash_xy(i, (base_x as i32) ^ 0xAB);
        let sx = snap(base_x + (h % 50) as f64);
        let sy = snap(base_y + PX * 4.0 + (h % 8) as f64);
        px(ctx, sx, sy, PX * 2.0, scorch_color);
    }
    ctx.set_global_alpha(1.0);

    // --- Smoke wisps: small clusters of dark gray pixels rising ---
    let smoke_colors = ["#4a4a4a", "#3a3a3a", "#2a2a2a"];
    ctx.set_global_alpha(0.5);
    for wisp in 0..3 {
        let h = hash_xy(wisp, (b.position.x as i32) ^ 0x77);
        let wisp_x = snap(base_x + PX * 3.0 + (wisp as f64) * PX * 6.0);
        for rise in 0..4 {
            let h2 = hash_xy(wisp, rise);
            let rx = wisp_x + if h2 % 2 == 0 { PX } else { -PX };
            let ry = snap(base_y - PX * (rise as f64) * 2.0);
            let color_idx = ((h as usize) + rise as usize) % smoke_colors.len();
            if let Some(color) = smoke_colors.get(color_idx) {
                px(ctx, rx, ry, PX, color);
            }
        }
    }
    ctx.set_global_alpha(1.0);

    // --- Name label ---
    ctx.set_fill_style_str("#ef4444");
    ctx.set_font("11px Outfit, sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(
        &b.name,
        b.position.x,
        b.position.y - BUILDING_HEIGHT / 2.0 - 6.0,
    );
}
