use super::super::state::Point;

pub fn bezier_point(t: f64, p0: Point, p1: Point, p2: Point) -> Point {
    let inv = 1.0 - t;
    Point {
        x: inv * inv * p0.x + 2.0 * inv * t * p1.x + t * t * p2.x,
        y: inv * inv * p0.y + 2.0 * inv * t * p1.y + t * t * p2.y,
    }
}
