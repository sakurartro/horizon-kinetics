// Kinetics: shafts and cogwheels for Horizon. The rotation network's rules and
// its solver, with no game types: this file is both the crate's library (its
// tests) and the module the bundle adds to the game as `game::kinetics`.
//
// The connection rules (`modifier`) are ported from Create's
// `content/kinetics/RotationPropagator.java` (getRotationSpeedModifier,
// isLargeToLargeGear, isLargeToSmallCog) and the neighbour offsets from
// `KineticBlockEntity.addPropagationLocations` /
// `SimpleKineticBlockEntity.addPropagationLocations`,
// https://github.com/Creators-of-Create/Create, branch mc1.21.1/dev, commit 4148fd8.
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
//
// Unlike Create, which updates the network incrementally and breaks a block on
// a conflict, the whole connected network is solved again from its sources on
// every change, and a conflicting network stands still (speed 0 everywhere).

use std::collections::{HashMap, HashSet, VecDeque};

/// The test source's fixed speed, in RPM (positive: counter-clockwise seen
/// from the positive end of its axis).
pub const SOURCE_SPEED: f32 = 16.0;
/// The block entity's saved key for its speed.
pub const SPEED_KEY: &str = "speed";
// shortcut: a network larger than this is solved only up to the cap, raise it if real builds hit it.
pub const MAX_NODES: usize = 4096;

pub type Pos = [i32; 3];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Shaft,
    Cog,
    LargeCog,
    Source,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    pub kind: Kind,
    /// 0 = x, 1 = y, 2 = z.
    pub axis: usize,
}

/// The kind of block `name` (`kinetics:shaft`), none for any other block.
pub fn kind_of(name: &str) -> Option<Kind> {
    match name.strip_prefix("kinetics:")? {
        "shaft" => Some(Kind::Shaft),
        "cogwheel" => Some(Kind::Cog),
        "large_cogwheel" => Some(Kind::LargeCog),
        "source" => Some(Kind::Source),
        _ => None,
    }
}

/// The block state property `axis` as an index.
pub fn axis_of(value: &str) -> Option<usize> {
    match value {
        "x" => Some(0),
        "y" => Some(1),
        "z" => Some(2),
        _ => None,
    }
}

/// The node for a block name and its `axis` property.
pub fn node(name: &str, axis: Option<&str>) -> Option<Node> {
    Some(Node {
        kind: kind_of(name)?,
        axis: axis_of(axis?)?,
    })
}

fn small(n: Node) -> bool {
    n.kind == Kind::Cog
}

fn large(n: Node) -> bool {
    n.kind == Kind::LargeCog
}

/// Every position a node may connect to: the six faces and the twelve edge
/// diagonals (Create checks the diagonals for cogwheels only; `modifier`
/// answers 0 for everything else there).
pub const OFFSETS: [Pos; 18] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
    [1, 1, 0],
    [1, -1, 0],
    [-1, 1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [1, 0, -1],
    [-1, 0, 1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, 1, -1],
    [0, -1, 1],
    [0, -1, -1],
];

/// How `to`'s speed follows `from`'s when `to` sits at `from + diff`:
/// 1 along a shaft, -1 between two cogwheels, -2 / -0.5 between a large and a
/// small one, 0 when they don't connect. `getRotationSpeedModifier`.
pub fn modifier(from: Node, to: Node, diff: Pos) -> f32 {
    let nonzero: Vec<usize> = (0..3).filter(|&a| diff[a] != 0).collect();
    // Axis <-> axis: face neighbours along both shafts. Every kind here has a
    // shaft through its own axis (`hasShaftTowards`).
    if nonzero.len() == 1 {
        let a = nonzero[0];
        if diff[a].abs() == 1 && from.axis == a && to.axis == a {
            return 1.0;
        }
    }
    // Large <-> large, axes crossed (`isLargeToLargeGear`).
    if large(from) && large(to) && from.axis != to.axis {
        let third = 3 - from.axis - to.axis;
        if diff[from.axis] != 0 && diff[to.axis] != 0 && diff[third] == 0 {
            return if (diff[from.axis] > 0) ^ (diff[to.axis] > 0) {
                -1.0
            } else {
                1.0
            };
        }
        return 0.0;
    }
    // Large <-> small: same axis, one diagonal step around it (`isLargeToSmallCog`).
    let large_to_small = |l: Node, s: Node| {
        l.axis == s.axis && diff[l.axis] == 0 && (0..3).all(|a| a == l.axis || diff[a].abs() == 1)
    };
    if large(from) && small(to) && large_to_small(from, to) {
        return -2.0;
    }
    if small(from) && large(to) && large_to_small(to, from) {
        return -0.5;
    }
    // Small <-> small: face neighbours beside each other, same axis.
    if small(from)
        && small(to)
        && nonzero.len() == 1
        && diff[nonzero[0]].abs() == 1
        && nonzero[0] != from.axis
        && from.axis == to.axis
    {
        return -1.0;
    }
    0.0
}

fn add(p: Pos, d: Pos) -> Pos {
    [p[0] + d[0], p[1] + d[1], p[2] + d[2]]
}

/// The connected neighbours of the node at `p`, each with the factor from
/// `p`'s speed to its own.
fn links(p: Pos, n: Node, get: &impl Fn(Pos) -> Option<Node>) -> Vec<(Pos, f32)> {
    let mut out = Vec::new();
    for d in OFFSETS {
        let q = add(p, d);
        let Some(m) = get(q) else { continue };
        let f = modifier(n, m, d);
        if f != 0.0 {
            out.push((q, f));
        } else {
            // A one-sided rule (Create checks both ways): use the inverse.
            let back = modifier(m, n, [-d[0], -d[1], -d[2]]);
            if back != 0.0 {
                out.push((q, 1.0 / back));
            }
        }
    }
    out
}

/// Solves the networks touching `changed` (positions whose block changed,
/// kinetic or not any more) and gives every node in them its speed. A network
/// with no source, or whose speeds disagree somewhere (two sources fighting,
/// a loop of gears that can't turn), stands still: speed 0 for all its nodes.
pub fn solve(changed: &[Pos], get: impl Fn(Pos) -> Option<Node>) -> HashMap<Pos, f32> {
    let mut out = HashMap::new();
    let mut starts: Vec<Pos> = Vec::new();
    for &c in changed {
        starts.push(c);
        starts.extend(OFFSETS.iter().map(|&d| add(c, d)));
    }
    for s in starts {
        if out.contains_key(&s) {
            continue;
        }
        let Some(n) = get(s) else { continue };
        // Gather the network.
        let mut nodes: Vec<(Pos, Node)> = vec![(s, n)];
        let mut seen: HashSet<Pos> = HashSet::from([s]);
        let mut i = 0;
        while i < nodes.len() && nodes.len() < MAX_NODES {
            let (p, n) = nodes[i];
            i += 1;
            for (q, _) in links(p, n, &get) {
                if seen.insert(q) {
                    nodes.push((q, get(q).expect("linked node exists")));
                }
            }
        }
        // Spread the sources' speed.
        let mut speed: HashMap<Pos, f32> = HashMap::new();
        let mut queue: VecDeque<Pos> = VecDeque::new();
        for &(p, n) in &nodes {
            if n.kind == Kind::Source {
                speed.insert(p, SOURCE_SPEED);
                queue.push_back(p);
            }
        }
        let mut locked = false;
        while let Some(p) = queue.pop_front() {
            let n = get(p).expect("queued node exists");
            let v = speed[&p];
            for (q, f) in links(p, n, &get) {
                if !seen.contains(&q) {
                    continue; // past the cap
                }
                let want = v * f;
                match speed.get(&q) {
                    Some(&have) if (have - want).abs() > 1e-4 * want.abs().max(1.0) => {
                        locked = true;
                    }
                    Some(_) => {}
                    None => {
                        speed.insert(q, want);
                        queue.push_back(q);
                    }
                }
            }
            if locked {
                break;
            }
        }
        for (p, _) in nodes {
            let v = if locked {
                0.0
            } else {
                speed.get(&p).copied().unwrap_or(0.0)
            };
            out.insert(p, v + 0.0); // no -0.0: its bits would read as a change
        }
    }
    out
}

/// The angle in degrees a part turning at `rpm` has reached at `ticks` (game
/// time plus the partial tick): a full turn per minute per RPM, 20 ticks a second.
pub fn angle(rpm: f32, ticks: f64) -> f32 {
    ((ticks * rpm as f64 * 360.0 / 1200.0) % 360.0) as f32
}

// Positions whose block changed (`native_block_changed`), solved at the end of
// their level's tick (`native_level_tick_post`). Both run on the server thread.
static DIRTY: std::sync::Mutex<Vec<(&'static str, Pos)>> = std::sync::Mutex::new(Vec::new());

/// Marks `pos` in `dimension` (`minecraft:overworld`) to be solved again.
pub fn mark(dimension: &'static str, pos: Pos) {
    DIRTY
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((dimension, pos));
}

/// The positions marked in `dimension` since the last call.
pub fn take(dimension: &str) -> Vec<Pos> {
    let mut dirty = DIRTY.lock().unwrap_or_else(|e| e.into_inner());
    let mut out = Vec::new();
    dirty.retain(|&(d, p)| {
        if d == dimension {
            out.push(p);
            false
        } else {
            true
        }
    });
    out
}

/// Whether block `name` gets a block entity: every part that turns. The source
/// is solved from its block alone.
pub fn has_block_entity(name: &str) -> bool {
    matches!(
        kind_of(name),
        Some(Kind::Shaft | Kind::Cog | Kind::LargeCog)
    )
}

/// The standalone models the block entities draw, built along the y axis.
pub const PART_MODELS: [&str; 3] = [
    "kinetics:block/shaft_part",
    "kinetics:block/cogwheel_part",
    "kinetics:block/large_cogwheel_part",
];

pub fn part_model(kind: Kind) -> Option<&'static str> {
    match kind {
        Kind::Shaft => Some(PART_MODELS[0]),
        Kind::Cog => Some(PART_MODELS[1]),
        Kind::LargeCog => Some(PART_MODELS[2]),
        Kind::Source => None,
    }
}

pub fn axis_vector(axis: usize) -> [f32; 3] {
    let mut v = [0.0; 3];
    v[axis] = 1.0;
    v
}

/// The turn that lays a model built along y along `axis`: y to x is -90
/// degrees about z, y to z is 90 degrees about x.
pub fn lay_along(axis: usize) -> Option<([f32; 3], f32)> {
    match axis {
        0 => Some(([0.0, 0.0, 1.0], -90.0)),
        2 => Some(([1.0, 0.0, 0.0], 90.0)),
        _ => None,
    }
}

/// Half a tooth (22.5 degrees, 8 teeth) on every other small cogwheel, so
/// neighbours' teeth fall between each other (Create offsets them too).
pub fn tooth_offset(kind: Kind, axis: usize, pos: Pos) -> f32 {
    let sum: i32 = (0..3).filter(|&a| a != axis).map(|a| pos[a]).sum();
    if kind == Kind::Cog && sum.rem_euclid(2) == 0 {
        22.5
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(blocks: &[(Pos, Kind, usize)]) -> impl Fn(Pos) -> Option<Node> + '_ {
        move |p| {
            blocks.iter().find(|b| b.0 == p).map(|b| Node {
                kind: b.1,
                axis: b.2,
            })
        }
    }

    const X: usize = 0;
    const Y: usize = 1;
    const Z: usize = 2;
    use Kind::*;

    #[test]
    fn shafts_carry_the_source_speed_along_their_axis() {
        let w = [
            ([0, 0, 0], Source, X),
            ([1, 0, 0], Shaft, X),
            ([2, 0, 0], Shaft, X),
            ([3, 0, 0], Shaft, X),
            // Beside the line but on another axis: not connected.
            ([1, 1, 0], Shaft, Y),
        ];
        let s = solve(&[[0, 0, 0]], world(&w));
        assert_eq!(s[&[3, 0, 0]], 16.0);
        assert_eq!(s[&[1, 0, 0]], 16.0);
        assert_eq!(s.get(&[1, 1, 0]).copied().unwrap_or(0.0), 0.0);
    }

    #[test]
    fn meshing_cogwheels_turn_opposite_ways() {
        // Source, shaft, cog, cog beside it, cog beside that one.
        let w = [
            ([0, 0, 0], Source, Y),
            ([0, 1, 0], Shaft, Y),
            ([0, 2, 0], Cog, Y),
            ([1, 2, 0], Cog, Y),
            ([1, 2, 1], Cog, Y),
        ];
        let s = solve(&[[0, 2, 0]], world(&w));
        assert_eq!(s[&[0, 2, 0]], 16.0);
        assert_eq!(s[&[1, 2, 0]], -16.0);
        assert_eq!(s[&[1, 2, 1]], 16.0);
    }

    #[test]
    fn cogwheels_stacked_on_their_axis_turn_together() {
        let w = [
            ([0, 0, 0], Source, Z),
            ([0, 0, 1], Cog, Z),
            ([0, 0, 2], Cog, Z),
        ];
        let s = solve(&[[0, 0, 0]], world(&w));
        assert_eq!(s[&[0, 0, 2]], 16.0);
    }

    #[test]
    fn a_large_cogwheel_doubles_and_halves() {
        // Large at the origin (axis y), small one diagonal step around it.
        let w = [
            ([0, -1, 0], Source, Y),
            ([0, 0, 0], LargeCog, Y),
            ([1, 0, 1], Cog, Y),
        ];
        let s = solve(&[[0, 0, 0]], world(&w));
        assert_eq!(s[&[1, 0, 1]], -32.0);
        // Driven from the small side instead: half, reversed.
        let w = [
            ([1, -1, 1], Source, Y),
            ([1, 0, 1], Cog, Y),
            ([0, 0, 0], LargeCog, Y),
        ];
        let s = solve(&[[1, 0, 1]], world(&w));
        assert_eq!(s[&[0, 0, 0]], -8.0);
    }

    #[test]
    fn no_source_means_no_speed() {
        let w = [([0, 0, 0], Shaft, X), ([1, 0, 0], Shaft, X)];
        let s = solve(&[[0, 0, 0]], world(&w));
        assert_eq!(s[&[0, 0, 0]], 0.0);
        assert_eq!(s[&[1, 0, 0]], 0.0);
    }

    #[test]
    fn a_consistent_loop_of_four_cogwheels_turns() {
        let w = [
            ([0, -1, 0], Source, Y),
            ([0, 0, 0], Cog, Y),
            ([1, 0, 0], Cog, Y),
            ([1, 0, 1], Cog, Y),
            ([0, 0, 1], Cog, Y),
        ];
        let s = solve(&[[0, 0, 0]], world(&w));
        assert_eq!(s[&[1, 0, 1]], 16.0);
        assert_eq!(s[&[0, 0, 1]], -16.0);
    }

    #[test]
    fn a_loop_whose_directions_conflict_locks_the_whole_network() {
        // Source 1 drives cog A; A drives cog B the other way; B's shaft
        // reaches source 2, which turns the same way as source 1: B can't
        // turn both ways, so nothing turns.
        let w = [
            ([0, 0, 0], Source, Y),
            ([0, 1, 0], Cog, Y),
            ([1, 1, 0], Cog, Y),
            ([1, 0, 0], Source, Y),
            ([0, 2, 0], Shaft, Y),
        ];
        let s = solve(&[[0, 1, 0]], world(&w));
        for p in [[0, 0, 0], [0, 1, 0], [1, 1, 0], [1, 0, 0], [0, 2, 0]] {
            assert_eq!(s[&p], 0.0, "{p:?}");
        }
    }

    #[test]
    fn breaking_a_shaft_stops_what_was_past_it() {
        // [1,0,0] was broken: the change is solved from its neighbours.
        let w = [
            ([0, 0, 0], Source, X),
            ([2, 0, 0], Shaft, X),
            ([3, 0, 0], Shaft, X),
        ];
        let s = solve(&[[1, 0, 0]], world(&w));
        assert_eq!(s[&[0, 0, 0]], 16.0);
        assert_eq!(s[&[2, 0, 0]], 0.0);
        assert_eq!(s[&[3, 0, 0]], 0.0);
    }

    #[test]
    fn large_cogwheels_with_crossed_axes_mesh() {
        let w = [
            ([0, -1, 0], Source, Y),
            ([0, 0, 0], LargeCog, Y),
            ([1, 1, 0], LargeCog, X),
        ];
        let s = solve(&[[0, 0, 0]], world(&w));
        assert_eq!(s[&[1, 1, 0]].abs(), 16.0);
    }

    #[test]
    fn angle_turns_once_a_minute_per_rpm() {
        assert_eq!(angle(1.0, 1200.0), 0.0);
        assert_eq!(angle(1.0, 300.0), 90.0);
        assert_eq!(angle(-16.0, 0.0), 0.0);
    }

    #[test]
    fn marks_are_taken_once_per_dimension() {
        mark("test:a", [1, 2, 3]);
        mark("test:b", [4, 5, 6]);
        assert_eq!(take("test:a"), vec![[1, 2, 3]]);
        assert!(take("test:a").is_empty());
        assert_eq!(take("test:b"), vec![[4, 5, 6]]);
    }

    #[test]
    fn block_names_and_axes() {
        assert_eq!(
            node("kinetics:cogwheel", Some("z")),
            Some(Node {
                kind: Kind::Cog,
                axis: 2
            })
        );
        assert_eq!(node("minecraft:stone", Some("y")), None);
        assert_eq!(node("kinetics:shaft", None), None);
    }
}
