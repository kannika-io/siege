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
