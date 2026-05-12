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
