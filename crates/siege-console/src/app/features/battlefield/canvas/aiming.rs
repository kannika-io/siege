use web_sys::CanvasRenderingContext2d;

use super::super::state::Point;
use super::projectile::sample_trajectory;

pub fn render_aiming_preview(
    ctx: &CanvasRenderingContext2d,
    origin: Point,
    mouse: Point,
    target: Point,
    control: Point,
    color: &str,
    power: f64,
) {
    // --- Pull-back rubber band from weapon to mouse ---
    let band_alpha = 0.4 + power * 0.4;
    ctx.set_global_alpha(band_alpha);
    ctx.set_stroke_style_str("#e2e8f0");
    ctx.set_line_width(2.0 + power * 2.0);
    ctx.begin_path();
    ctx.move_to(origin.x, origin.y);
    ctx.line_to(mouse.x, mouse.y);
    ctx.stroke();

    // Pull handle dot at mouse position
    ctx.set_fill_style_str("#e2e8f0");
    ctx.begin_path();
    let _ = ctx.arc(mouse.x, mouse.y, 4.0 + power * 3.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();
    ctx.set_global_alpha(1.0);

    // --- Trajectory arc from weapon to computed target ---
    let points = sample_trajectory(origin, control, target, 30);

    ctx.set_fill_style_str(color);
    ctx.set_global_alpha(0.5);

    for (i, point) in points.iter().enumerate() {
        let radius = if i % 3 == 0 { 3.0 } else { 1.5 };
        ctx.begin_path();
        let _ = ctx.arc(point.x, point.y, radius, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    }

    // --- Landing crosshair at target ---
    ctx.set_global_alpha(0.7);
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(1.5);

    ctx.begin_path();
    ctx.move_to(target.x - 10.0, target.y);
    ctx.line_to(target.x + 10.0, target.y);
    ctx.stroke();

    ctx.begin_path();
    ctx.move_to(target.x, target.y - 10.0);
    ctx.line_to(target.x, target.y + 10.0);
    ctx.stroke();

    ctx.begin_path();
    let _ = ctx.arc(target.x, target.y, 8.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.stroke();

    ctx.set_global_alpha(1.0);
}
