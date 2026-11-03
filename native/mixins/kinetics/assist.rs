// Create's placement helper for a shaft (`PoleHelper`): aimed at a shaft with a shaft in hand, the next free cell along its
// axis (past the shafts already in a row) is shown and gets the shaft on a right click. Pure rules; the screen
// drawing is assist_hud.rs and the click assist_use.rs.

/// The shaft block and item.
pub const SHAFT: &str = "kinetics:shaft";
/// How far along a row the helper reaches (Create's `placementAssistRange`).
pub const RANGE: i32 = 12;

/// 0, 1, 2 for the axis `x`, `y`, `z`.
pub fn axis_index(axis: &str) -> Option<usize> {
    ["x", "y", "z"].iter().position(|a| *a == axis)
}

/// `PoleHelper.getOffset`: the free cell along `axis` next to the shaft at `pos`, on the side the hit point `hit` is nearer to
/// (the other side if that row is full or blocked). `is_pole` says whether a cell holds a shaft of this axis,
/// `replaceable` whether a shaft may be put there.
pub fn pole_target(
    axis: usize,
    pos: [i32; 3],
    hit: [f64; 3],
    is_pole: impl Fn([i32; 3]) -> bool,
    replaceable: impl Fn([i32; 3]) -> bool,
) -> Option<[i32; 3]> {
    let at = |dir: i32, n: i32| {
        let mut p = pos;
        p[axis] += dir * n;
        p
    };
    let distance = |dir: i32| {
        let c = at(dir, 1);
        (0..3).map(|i| (f64::from(c[i]) + 0.5 - hit[i]).powi(2)).sum::<f64>()
    };
    let mut dirs = [1, -1];
    dirs.sort_by(|a, b| distance(*a).total_cmp(&distance(*b)));
    for dir in dirs {
        let mut poles = 0;
        while poles < RANGE && is_pole(at(dir, poles + 1)) {
            poles += 1;
        }
        if poles < RANGE && replaceable(at(dir, poles + 1)) {
            return Some(at(dir, poles + 1));
        }
    }
    None
}

/// The shaft's box in `cell` (4 px thick, as the model), from min to max corner.
pub fn shaft_box(axis: usize, cell: [i32; 3]) -> ([f64; 3], [f64; 3]) {
    let mut min = cell.map(|c| f64::from(c) + 0.375);
    let mut max = cell.map(|c| f64::from(c) + 0.625);
    min[axis] = f64::from(cell[axis]);
    max[axis] = f64::from(cell[axis]) + 1.0;
    (min, max)
}

/// Where `p` is on a screen `w` by `h` GUI pixels for a camera at `eye` looking along `yaw`/`pitch` (degrees, Minecraft's)
/// with a vertical field of view `fov` (degrees); none at or behind the camera.
pub fn project(eye: [f64; 3], yaw: f32, pitch: f32, fov: f32, size: [f32; 2], p: [f64; 3]) -> Option<[f32; 2]> {
    let (y, q) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    let forward = [-y.sin() * q.cos(), -q.sin(), y.cos() * q.cos()];
    let right = [-y.cos(), 0.0, -y.sin()];
    let up = [
        right[1] * forward[2] - right[2] * forward[1],
        right[2] * forward[0] - right[0] * forward[2],
        right[0] * forward[1] - right[1] * forward[0],
    ];
    let d = [p[0] - eye[0], p[1] - eye[1], p[2] - eye[2]];
    let dot = |a: [f64; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
    let depth = dot(forward);
    if depth < 0.05 {
        return None;
    }
    let tan = (f64::from(fov).to_radians() / 2.0).tan();
    let aspect = f64::from(size[0] / size[1]);
    let x = dot(right) / (depth * tan * aspect);
    let v = dot(up) / (depth * tan);
    Some([((x * 0.5 + 0.5) as f32) * size[0], ((0.5 - v * 0.5) as f32) * size[1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearer_side_first() {
        // Aimed at the east half of an x shaft at the origin: the cell to the east.
        let free = |_| true;
        assert_eq!(pole_target(0, [0, 0, 0], [0.9, 0.5, 0.5], |_| false, free), Some([1, 0, 0]));
        assert_eq!(pole_target(0, [0, 0, 0], [0.1, 0.5, 0.5], |_| false, free), Some([-1, 0, 0]));
        assert_eq!(pole_target(1, [3, 4, 5], [3.5, 5.0, 5.5], |_| false, free), Some([3, 5, 5]));
    }

    #[test]
    fn walks_past_a_row_of_shafts() {
        let row = |p: [i32; 3]| p[2] >= 1 && p[2] <= 3;
        let free = |p: [i32; 3]| !row(p);
        assert_eq!(pole_target(2, [0, 0, 0], [0.5, 0.5, 0.9], row, free), Some([0, 0, 4]));
    }

    #[test]
    fn other_side_when_blocked_or_too_long() {
        let free = |p: [i32; 3]| p[0] < 0;
        assert_eq!(pole_target(0, [0, 0, 0], [0.9, 0.5, 0.5], |_| false, free), Some([-1, 0, 0]));
        let long = |p: [i32; 3]| p[0] >= 1;
        assert_eq!(pole_target(0, [0, 0, 0], [0.9, 0.5, 0.5], long, |_| true), Some([-1, 0, 0]));
        assert_eq!(pole_target(0, [0, 0, 0], [0.9, 0.5, 0.5], |_| true, |_| true), None);
        assert_eq!(pole_target(0, [0, 0, 0], [0.9, 0.5, 0.5], |_| false, |_| false), None);
    }

    #[test]
    fn box_and_axis() {
        assert_eq!(axis_index("z"), Some(2));
        assert_eq!(axis_index("w"), None);
        assert_eq!(shaft_box(0, [2, 0, 0]), ([2.0, 0.375, 0.375], [3.0, 0.625, 0.625]));
    }

    #[test]
    fn projection_centres_what_is_ahead() {
        // Yaw 0 looks south (+z); pitch 0 level. A point straight ahead is the screen's centre, east (+x) is to the left.
        let c = project([0.0; 3], 0.0, 0.0, 70.0, [320.0, 240.0], [0.0, 0.0, 5.0]).unwrap();
        assert!((c[0] - 160.0).abs() < 1e-3 && (c[1] - 120.0).abs() < 1e-3);
        let east = project([0.0; 3], 0.0, 0.0, 70.0, [320.0, 240.0], [1.0, 0.0, 5.0]).unwrap();
        assert!(east[0] < 160.0);
        let above = project([0.0; 3], 0.0, 0.0, 70.0, [320.0, 240.0], [0.0, 1.0, 5.0]).unwrap();
        assert!(above[1] < 120.0);
        assert!(project([0.0; 3], 0.0, 0.0, 70.0, [320.0, 240.0], [0.0, 0.0, -5.0]).is_none());
    }
}
