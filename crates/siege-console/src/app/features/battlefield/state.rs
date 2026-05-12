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
