use web_sys::CanvasRenderingContext2d;

use super::super::state::{BuildingTarget, BuildingVisual, BUILDING_HEIGHT, BUILDING_WIDTH};

pub fn render_terrain(ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
    ctx.set_fill_style_str("#0f172a");
    ctx.fill_rect(0.0, 0.0, width, height * 0.6);

    ctx.set_fill_style_str("#1a3a10");
    ctx.fill_rect(0.0, height * 0.6, width, height * 0.4);

    ctx.set_fill_style_str("#162e0d");
    ctx.begin_path();
    ctx.move_to(0.0, height * 0.6);
    ctx.quadratic_curve_to(width * 0.15, height * 0.48, width * 0.3, height * 0.58);
    ctx.quadratic_curve_to(width * 0.5, height * 0.50, width * 0.7, height * 0.56);
    ctx.quadratic_curve_to(width * 0.85, height * 0.52, width, height * 0.6);
    ctx.line_to(width, height * 0.6);
    ctx.close_path();
    ctx.fill();

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

    ctx.set_fill_style_str("#6b7280");
    ctx.fill_rect(x + 10.0, y + 25.0, BUILDING_WIDTH - 20.0, BUILDING_HEIGHT - 25.0);

    ctx.set_fill_style_str("#92400e");
    ctx.begin_path();
    ctx.move_to(x + 5.0, y + 25.0);
    ctx.line_to(x + BUILDING_WIDTH / 2.0, y + 5.0);
    ctx.line_to(x + BUILDING_WIDTH - 5.0, y + 25.0);
    ctx.close_path();
    ctx.fill();

    ctx.set_fill_style_str("#44403c");
    ctx.fill_rect(x + BUILDING_WIDTH / 2.0 - 5.0, y + BUILDING_HEIGHT - 18.0, 10.0, 18.0);

    ctx.set_fill_style_str("#fbbf24");
    ctx.set_global_alpha(0.6);
    ctx.fill_rect(x + 15.0, y + 35.0, 8.0, 8.0);
    ctx.fill_rect(x + BUILDING_WIDTH - 23.0, y + 35.0, 8.0, 8.0);
    ctx.set_global_alpha(1.0);

    ctx.set_fill_style_str("#e2e8f0");
    ctx.set_font("11px Outfit, sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(&b.name, b.position.x, b.position.y - BUILDING_HEIGHT / 2.0 - 6.0);
}

fn render_damaged_building(ctx: &CanvasRenderingContext2d, b: &BuildingTarget, chaos_label: &str) {
    let x = b.position.x - BUILDING_WIDTH / 2.0;
    let y = b.position.y - BUILDING_HEIGHT / 2.0;

    ctx.set_fill_style_str("#4b5563");
    ctx.fill_rect(x + 10.0, y + 25.0, BUILDING_WIDTH - 20.0, BUILDING_HEIGHT - 25.0);

    ctx.set_fill_style_str("#7c2d12");
    ctx.begin_path();
    ctx.move_to(x + 5.0, y + 25.0);
    ctx.line_to(x + BUILDING_WIDTH / 2.0, y + 8.0);
    ctx.line_to(x + BUILDING_WIDTH - 5.0, y + 25.0);
    ctx.close_path();
    ctx.fill();

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

    ctx.set_fill_style_str("#fbbf24");
    ctx.set_font("11px Outfit, sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(&b.name, b.position.x, b.position.y - BUILDING_HEIGHT / 2.0 - 6.0);

    ctx.set_fill_style_str("#22c55e");
    ctx.set_font("9px Outfit, sans-serif");
    let _ = ctx.fill_text(
        chaos_label,
        b.position.x,
        b.position.y + BUILDING_HEIGHT / 2.0 + 14.0,
    );
}

fn render_destroyed_building(ctx: &CanvasRenderingContext2d, b: &BuildingTarget) {
    let x = b.position.x - BUILDING_WIDTH / 2.0;
    let y = b.position.y + BUILDING_HEIGHT / 2.0 - 15.0;

    ctx.set_fill_style_str("#57534e");
    ctx.fill_rect(x + 8.0, y, 12.0, 8.0);
    ctx.fill_rect(x + 22.0, y - 3.0, 10.0, 11.0);
    ctx.fill_rect(x + 35.0, y + 2.0, 14.0, 6.0);
    ctx.set_fill_style_str("#44403c");
    ctx.fill_rect(x + 14.0, y - 5.0, 8.0, 5.0);
    ctx.fill_rect(x + 30.0, y - 2.0, 6.0, 4.0);

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

    ctx.set_fill_style_str("#ef4444");
    ctx.set_font("11px Outfit, sans-serif");
    ctx.set_text_align("center");
    let _ = ctx.fill_text(&b.name, b.position.x, b.position.y - BUILDING_HEIGHT / 2.0 - 6.0);
}
