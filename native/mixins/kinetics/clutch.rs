// Kinetics: the clutch and the gearshift (redstone-switched shaft halves). Their rules without game
// types: when a redstone change flips them, and how their two halves are drawn. It is a module of the
// bundle like runtime.rs (`game::kinetics_clutch`), with its tests in a target of its own; the network
// side (`Kind::Clutch`, `Kind::Gearshift`, `solve_halves`) is in runtime.rs.
//
// Ported from Create's `content/kinetics/transmission/ClutchBlock.java`, `GearshiftBlock.java`
// (neighborChanged, detachKinetics: the network is cut at once and joined again a tick later),
// `ClutchBlockEntity` / `GearshiftBlockEntity` (getRotationSpeedModifier) and `SplitShaftRenderer`
// (a half of the shaft drawn per direction, https://github.com/Creators-of-Create/Create,
// branch mc1.21.1/dev, commit 4148fd8).
//
// Copyright (c) The Create Team / The Creators of Create
// Licensed under the MIT License:
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

pub type Pos = [i32; 3];

/// The block entity's saved key for the speed of the +axis half (`speed` holds the -axis half's).
pub const SPEED_POS_KEY: &str = "speed_pos";

/// The standalone half-shaft model of ours (built along y, the +y half) and Create's (built along z, the
/// +z half; it exists only while the pack from `tools/create_look.py` is on). Ours is the fallback.
pub const HALF_MODEL: &str = "kinetics:block/clutch_half_part";
pub const LOOK_HALF_MODEL: &str = "create:block/shaft_half";

/// Whether `name` is a clutch or a gearshift.
/// Create's static models of the split blocks, built along y, drawn over our placeholders (the blockstate models)
/// while the pack from `tools/create_look.py` is on: clutch, clutch powered, gearshift, gearshift powered.
pub const STATIC_LOOK_MODELS: [&str; 4] = [
    "create:block/clutch/block",
    "create:block/clutch/block_powered",
    "create:block/gearshift/block",
    "create:block/gearshift/block_powered",
];

pub fn static_look(name: &str, powered: bool) -> Option<&'static str> {
    let first = match name {
        "kinetics:clutch" => 0,
        "kinetics:gearshift" => 2,
        _ => return None,
    };
    Some(STATIC_LOOK_MODELS[first + usize::from(powered)])
}

pub fn is_split(name: &str) -> bool {
    matches!(name, "kinetics:clutch" | "kinetics:gearshift")
}

/// What a clutch or gearshift does when the signal at it differs from its `powered` property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flip {
    /// The new `powered`.
    pub powered: bool,
    /// Whether the network is cut for a tick before it joins again (Create's `detachKinetics`
    /// with `reAttachNextTick`): a gearshift always, a clutch only when it lets go. A clutch that
    /// gets power cuts through its new state at once.
    pub detach: bool,
}

/// The flip a block needs to follow `signal` (the strongest redstone signal at it, 0 to 15), none when
/// it already agrees (`neighborChanged`).
pub fn flip(name: &str, powered: bool, signal: i32) -> Option<Flip> {
    let want = signal > 0;
    if !is_split(name) || want == powered {
        return None;
    }
    Some(Flip {
        powered: want,
        detach: name == "kinetics:gearshift" || powered,
    })
}

// Positions whose neighbour changed (`native_neighbor_changed`), checked at the end of their level's tick
// (`native_level_tick_post`), where the signal can be read and the block written. Both run on the server thread.
static CHECKS: std::sync::Mutex<Vec<(&'static str, Pos)>> = std::sync::Mutex::new(Vec::new());

/// Marks the clutch or gearshift at `pos` in `dimension` (`minecraft:overworld`) to be checked.
pub fn mark(dimension: &'static str, pos: Pos) {
    let mut checks = CHECKS.lock().unwrap_or_else(|e| e.into_inner());
    if !checks.contains(&(dimension, pos)) {
        checks.push((dimension, pos));
    }
}

/// The positions marked in `dimension` since the last call.
pub fn take(dimension: &str) -> Vec<Pos> {
    let mut checks = CHECKS.lock().unwrap_or_else(|e| e.into_inner());
    let mut out = Vec::new();
    checks.retain(|&(d, p)| {
        if d == dimension {
            out.push(p);
            false
        } else {
            true
        }
    });
    out
}

/// The turns, outermost first (the first is applied last to the model), that draw one half of a
/// split block turning `angle` degrees: `positive` is the half toward +axis, `look` says the model is
/// Create's (built along z) rather than ours (built along y). Both are built as a +half.
pub fn half_turns(axis: usize, positive: bool, look: bool, angle: f32) -> Vec<([f32; 3], f32)> {
    let mut v = vec![(axis_vector(axis), angle)];
    // Lay the model along the axis: y to x is -90 about z, y to z is 90 about x; z to x is 90
    // about y, z to y is -90 about x.
    let lay = match (look, axis) {
        (false, 0) => Some(([0.0, 0.0, 1.0], -90.0)),
        (false, 2) => Some(([1.0, 0.0, 0.0], 90.0)),
        (true, 0) => Some(([0.0, 1.0, 0.0], 90.0)),
        (true, 1) => Some(([1.0, 0.0, 0.0], -90.0)),
        _ => None,
    };
    v.extend(lay);
    // The -axis half is the +half turned over.
    if !positive {
        v.push(([1.0, 0.0, 0.0], 180.0));
    }
    v
}

fn axis_vector(axis: usize) -> [f32; 3] {
    let mut v = [0.0; 3];
    v[axis] = 1.0;
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_static_look_follows_the_block_and_its_power() {
        assert_eq!(static_look("kinetics:clutch", false), Some("create:block/clutch/block"));
        assert_eq!(static_look("kinetics:clutch", true), Some("create:block/clutch/block_powered"));
        assert_eq!(static_look("kinetics:gearshift", true), Some("create:block/gearshift/block_powered"));
        assert_eq!(static_look("kinetics:gearbox", false), None);
    }

    #[test]
    fn a_clutch_follows_the_signal_and_cuts_a_tick_only_when_it_lets_go() {
        let c = "kinetics:clutch";
        assert_eq!(flip(c, false, 0), None);
        assert_eq!(flip(c, true, 7), None);
        assert_eq!(flip(c, false, 15), Some(Flip { powered: true, detach: false }));
        assert_eq!(flip(c, true, 0), Some(Flip { powered: false, detach: true }));
    }

    #[test]
    fn a_gearshift_always_detaches_when_it_flips() {
        let g = "kinetics:gearshift";
        assert_eq!(flip(g, false, 1), Some(Flip { powered: true, detach: true }));
        assert_eq!(flip(g, true, 0), Some(Flip { powered: false, detach: true }));
        assert_eq!(flip("kinetics:shaft", false, 15), None);
    }

    #[test]
    fn marks_are_taken_once_per_dimension() {
        mark("test:a", [1, 2, 3]);
        mark("test:a", [1, 2, 3]);
        mark("test:b", [4, 5, 6]);
        assert_eq!(take("test:a"), vec![[1, 2, 3]]);
        assert!(take("test:a").is_empty());
        assert_eq!(take("test:b"), vec![[4, 5, 6]]);
    }

    /// Rotation by `deg` about the unit vector `a` (Rodrigues).
    fn rotate(a: [f32; 3], deg: f32, v: [f32; 3]) -> [f32; 3] {
        let t = deg.to_radians();
        let (s, c) = t.sin_cos();
        let dot = a[0] * v[0] + a[1] * v[1] + a[2] * v[2];
        let cross = [
            a[1] * v[2] - a[2] * v[1],
            a[2] * v[0] - a[0] * v[2],
            a[0] * v[1] - a[1] * v[0],
        ];
        [0, 1, 2].map(|i| v[i] * c + cross[i] * s + a[i] * dot * (1.0 - c))
    }

    /// Where the half's end ends up, the first turn being applied last.
    fn end_of(turns: &[([f32; 3], f32)], built_along: usize) -> [f32; 3] {
        let mut v = axis_vector(built_along);
        for &(a, d) in turns.iter().rev() {
            v = rotate(a, d, v);
        }
        v.map(|x| (x * 1000.0).round() / 1000.0)
    }

    #[test]
    fn each_half_points_along_its_side_of_the_axis() {
        for look in [false, true] {
            let built = if look { 2 } else { 1 };
            for axis in 0..3 {
                for positive in [true, false] {
                    let mut want = [0.0; 3];
                    want[axis] = if positive { 1.0 } else { -1.0 };
                    let got = end_of(&half_turns(axis, positive, look, 0.0), built);
                    assert_eq!(got, want, "look {look} axis {axis} positive {positive}");
                }
            }
        }
    }

    #[test]
    fn the_spin_is_about_the_blocks_axis() {
        let t = half_turns(2, true, false, 33.0);
        assert_eq!(t[0], ([0.0, 0.0, 1.0], 33.0));
    }
}
