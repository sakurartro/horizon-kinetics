// Create's placement helpers. Shaft (`PoleHelper`): aimed at a shaft with a shaft in hand, the next free cell along its
// axis (past the shafts already in a row) is shown and gets the shaft on a right click. Cogwheel (`CogwheelBlockItem`):
// aimed at a cog with a cog in hand, the neighbouring or diagonal cell for meshing. Pure rules; the screen drawing is
// assist_hud.rs and the click assist_use.rs.

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

/// What the cog rules need to know of a cell: the cog it holds (`Some(large)`), its `axis` property if it has one, and
/// whether a block may be put there.
#[derive(Clone, Copy, Default)]
pub struct Cell {
    pub cog: Option<bool>,
    pub axis: Option<usize>,
    pub replaceable: bool,
}

/// `Some(true)` for a large cogwheel block (bare or cased), `Some(false)` for a small one.
pub fn cog_kind(block: &str) -> Option<bool> {
    let name = block.strip_prefix("kinetics:")?;
    match name {
        "cogwheel" | "andesite_encased_cogwheel" | "brass_encased_cogwheel" => Some(false),
        "large_cogwheel" | "andesite_encased_large_cogwheel" | "brass_encased_large_cogwheel" => Some(true),
        _ => None,
    }
}

/// A direction: axis and sign, in Minecraft's `Direction.values()` order (down, up, north, south, west, east).
type Dir = (usize, i32);
const DIRECTIONS: [Dir; 6] = [(1, -1), (1, 1), (2, -1), (2, 1), (0, -1), (0, 1)];

/// Catnip `IPlacementHelper.orderedByDistance`: the kept directions, the one the hit point lies toward first.
fn ordered(pos: [i32; 3], hit: [f64; 3], keep: impl Fn(Dir) -> bool) -> Vec<Dir> {
    let off: [f64; 3] = std::array::from_fn(|i| hit[i] - (f64::from(pos[i]) + 0.5));
    let dist = |d: Dir| (0..3).map(|i| (if i == d.0 { f64::from(d.1) } else { 0.0 } - off[i]).powi(2)).sum::<f64>();
    let mut v: Vec<Dir> = DIRECTIONS.into_iter().filter(|d| keep(*d)).collect();
    v.sort_by(|a, b| dist(*a).total_cmp(&dist(*b)));
    v
}

fn step(p: [i32; 3], d: Dir) -> [i32; 3] {
    let mut q = p;
    q[d.0] += d.1;
    q
}

/// `CogWheelBlock.isValidCogwheelPosition`: no large cog (or, for a large one, no cog) beside `cell` except along its axis.
fn cog_position_ok(large: bool, cell: [i32; 3], axis: usize, info: &impl Fn([i32; 3]) -> Cell) -> bool {
    DIRECTIONS.into_iter().filter(|d| d.0 != axis).all(|d| {
        let n = info(step(cell, d));
        n.axis == Some(d.0) || !(n.cog == Some(true) || (large && n.cog == Some(false)))
    })
}

/// `CogwheelBlockItem`'s `SmallCogHelper` / `LargeCogHelper` (with `DiagonalCogHelper`): aimed at the cog at `pos` with a cog
/// of size `hand_large` in hand, the cell for the new cog and its `axis`. Nothing when the hit is on the cog's shaft.
/// Not ported: the "integrated" helpers for non-dedicated cogs (cased cogwheels are aimed at as plain cogs).
pub fn cog_target(hand_large: bool, pos: [i32; 3], hit: [f64; 3], info: impl Fn([i32; 3]) -> Cell) -> Option<([i32; 3], usize)> {
    let here = info(pos);
    let (aimed_large, axis) = (here.cog?, here.axis?);
    // `hitOnShaft`: the 6 px pole along the axis, hit measured inside its block (Create aligns by flooring the hit).
    let on_shaft = (0..3).filter(|i| *i != axis).all(|i| {
        let l = hit[i] - hit[i].floor();
        (5.0 / 16.0 - 0.001..=11.0 / 16.0 + 0.001).contains(&l)
    });
    if on_shaft {
        return None;
    }
    let diagonal = || {
        let closest = *ordered(pos, hit, |d| d.0 != axis).first()?;
        ordered(pos, hit, |d| d.0 != axis && d.0 != closest.0).into_iter().find_map(|d| {
            let cell = step(step(pos, d), closest);
            (info(cell).replaceable && cog_position_ok(aimed_large, cell, axis, &info)).then_some((cell, axis))
        })
    };
    if hand_large != aimed_large {
        return diagonal();
    }
    if !hand_large {
        return ordered(pos, hit, |d| d.0 != axis).into_iter().find_map(|d| {
            let cell = step(pos, d);
            (cog_position_ok(false, cell, axis, &info) && info(cell).replaceable).then_some((cell, axis))
        });
    }
    let side = *ordered(pos, hit, |d| d.0 == axis).first()?;
    ordered(pos, hit, |d| d.0 != axis).into_iter().find_map(|d| {
        let cell = step(step(pos, d), side);
        (cog_position_ok(true, cell, d.0, &info) && info(cell).replaceable).then_some((cell, d.0))
    })
}

/// A cogwheel's box in `cell` on `axis`: the 4 px plate (2 px margin for a small one), from min to max corner.
pub fn cog_box(large: bool, axis: usize, cell: [i32; 3]) -> ([f64; 3], [f64; 3]) {
    let m = if large { 0.0 } else { 0.125 };
    let mut min = cell.map(|c| f64::from(c) + m);
    let mut max = cell.map(|c| f64::from(c) + 1.0 - m);
    min[axis] = f64::from(cell[axis]) + 0.375;
    max[axis] = f64::from(cell[axis]) + 0.625;
    (min, max)
}

/// The overlay for a ghost box `lo`..`hi` and an arrow from cell `pos` to cell `target`, as screen quads with ARGB colours:
/// translucent shaded faces turned to the camera, outline edges, and the arrow at the crosshair.
pub fn overlay(eye: [f64; 3], yaw: f32, pitch: f32, fov: f32, size: [f32; 2], lo: [f64; 3], hi: [f64; 3], pos: [i32; 3], target: [i32; 3]) -> Vec<([[f32; 2]; 4], u32)> {
    let project = |p: [f64; 3]| project(eye, yaw, pitch, fov, size, p);
    let mut out = Vec::new();
    let corner = |i: usize| [if i & 1 == 0 { lo[0] } else { hi[0] }, if i & 2 == 0 { lo[1] } else { hi[1] }, if i & 4 == 0 { lo[2] } else { hi[2] }];
    let screen: Vec<Option<[f32; 2]>> = (0..8).map(|i| project(corner(i))).collect();
    // The faces turned to the camera, each with its corner indexes in order round it and its shade.
    let faces: [([usize; 4], usize, bool, f32); 6] = [
        ([0, 2, 6, 4], 0, false, 0.6), ([1, 3, 7, 5], 0, true, 0.6),
        ([0, 1, 5, 4], 1, false, 0.5), ([2, 3, 7, 6], 1, true, 1.0),
        ([0, 1, 3, 2], 2, false, 0.8), ([4, 5, 7, 6], 2, true, 0.8),
    ];
    for (ids, face_axis, high, shade) in faces {
        let toward = if high { eye[face_axis] > hi[face_axis] } else { eye[face_axis] < lo[face_axis] };
        let corners: Option<Vec<[f32; 2]>> = ids.iter().map(|i| screen[*i]).collect();
        let Some(c) = corners.filter(|_| toward) else { continue };
        let grey = (0xB8 as f32 * shade) as u32;
        out.push(([c[0], c[1], c[2], c[3]], 0x90_000000 | grey << 16 | grey << 8 | grey));
    }
    for (from, to) in (0..8usize).flat_map(|i| [(i, i | 1), (i, i | 2), (i, i | 4)]).filter(|(i, j)| i != j && i & (i ^ j) == 0) {
        if let (Some(p), Some(q)) = (screen[from], screen[to]) {
            let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            let n = [-dy / len * 0.5, dx / len * 0.5];
            out.push(([[p[0] + n[0], p[1] + n[1]], [q[0] + n[0], q[1] + n[1]], [q[0] - n[0], q[1] - n[1]], [p[0] - n[0], p[1] - n[1]]], 0xC0_101010));
        }
    }
    // The arrow at the crosshair, pointing from the aimed block to the cell.
    let centre = |c: [i32; 3]| project([f64::from(c[0]) + 0.5, f64::from(c[1]) + 0.5, f64::from(c[2]) + 0.5]);
    let (Some(from), Some(to)) = (centre(pos), centre(target)) else { return out };
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return out;
    }
    let (d, n) = ([dx / len, dy / len], [-dy / len, dx / len]);
    let (cx, cy) = (size[0] / 2.0 + d[0] * 14.0, size[1] / 2.0 + d[1] * 14.0);
    let at = |along: f32, across: f32| [cx + d[0] * along + n[0] * across, cy + d[1] * along + n[1] * across];
    for (grow, color) in [(1.0, 0xFF_101010_u32), (0.0, 0xFF_FFFFFF)] {
        out.push(([at(-5.0 - grow, -1.5 - grow), at(1.0, -1.5 - grow), at(1.0, 1.5 + grow), at(-5.0 - grow, 1.5 + grow)], color));
        out.push(([at(0.0, -4.0 - grow), at(5.0 + grow * 1.5, 0.0), at(5.0 + grow * 1.5, 0.0), at(0.0, 4.0 + grow)], color));
    }
    out
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

    /// A world of cogs: (cell -> (large, axis)); everything else is free.
    fn world(cogs: Vec<([i32; 3], bool, usize)>) -> impl Fn([i32; 3]) -> Cell {
        move |p| match cogs.iter().find(|c| c.0 == p) {
            Some(c) => Cell { cog: Some(c.1), axis: Some(c.2), replaceable: false },
            None => Cell { cog: None, axis: None, replaceable: true },
        }
    }

    #[test]
    fn small_next_to_small_keeps_the_axis() {
        let w = world(vec![([0, 0, 0], false, 1)]);
        // y cog, aimed at the east edge of the plate: the cell to the east, same axis.
        assert_eq!(cog_target(false, [0, 0, 0], [0.95, 0.5, 0.5], &w), Some(([1, 0, 0], 1)));
        assert_eq!(cog_target(false, [0, 0, 0], [0.5, 0.5, 0.05], &w), Some(([0, 0, -1], 1)));
        // On the shaft: nothing.
        assert_eq!(cog_target(false, [0, 0, 0], [0.5, 0.5, 0.5], &w), None);
    }

    #[test]
    fn small_next_to_small_skips_blocked_cells() {
        // The east cell is occupied, so the next nearest one is offered.
        let w = world(vec![([0, 0, 0], false, 1), ([1, 0, 0], false, 0)]);
        assert_ne!(cog_target(false, [0, 0, 0], [0.95, 0.5, 0.6], &w).map(|t| t.0), Some([1, 0, 0]));
        // A large cog beside the cell makes it invalid: (1,0,0)'s neighbour (2,0,0) is large on a y axis.
        let w = world(vec![([0, 0, 0], false, 1), ([2, 0, 0], true, 1)]);
        assert_ne!(cog_target(false, [0, 0, 0], [0.95, 0.5, 0.5], &w).map(|t| t.0), Some([1, 0, 0]));
    }

    #[test]
    fn large_next_to_large_goes_diagonal_with_the_direction_axis() {
        let w = world(vec![([0, 0, 0], true, 1)]);
        // Aimed at the top face, east edge: step east and up, on the x axis.
        assert_eq!(cog_target(true, [0, 0, 0], [0.95, 0.9, 0.5], &w), Some(([1, 1, 0], 0)));
        assert_eq!(cog_target(true, [0, 0, 0], [0.5, 0.1, 0.95], &w), Some(([0, -1, 1], 2)));
    }

    #[test]
    fn mixed_sizes_go_diagonal_on_the_same_axis() {
        let w = world(vec![([0, 0, 0], false, 1)]);
        // Large in hand at a small y cog, near the east-south corner: the diagonal cell on the same axis.
        assert_eq!(cog_target(true, [0, 0, 0], [0.95, 0.5, 0.8], &w), Some(([1, 0, 1], 1)));
        let w = world(vec![([0, 0, 0], true, 1)]);
        assert_eq!(cog_target(false, [0, 0, 0], [0.95, 0.5, 0.8], &w), Some(([1, 0, 1], 1)));
    }

    #[test]
    fn kinds_and_boxes() {
        assert_eq!(cog_kind("kinetics:cogwheel"), Some(false));
        assert_eq!(cog_kind("kinetics:andesite_encased_large_cogwheel"), Some(true));
        assert_eq!(cog_kind("kinetics:shaft"), None);
        assert_eq!(cog_box(true, 1, [0, 0, 0]), ([0.0, 0.375, 0.0], [1.0, 0.625, 1.0]));
        assert_eq!(cog_box(false, 0, [0, 0, 0]), ([0.375, 0.125, 0.125], [0.625, 0.875, 0.875]));
    }
}
