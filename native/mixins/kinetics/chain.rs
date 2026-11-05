// Kinetics: the encased chain drive and the adjustable chain gearshift. Their rules without game types: how
// neighbouring drives chain together (`part`, `axis_along_first`), the gearshift's redstone, and which model
// Create draws for each state. It is a module of the bundle like clutch.rs (`game::kinetics_chain`), with its
// tests in a target of its own; the network side (`Kind::Chain`, `chain_node`, `modifier`) is in runtime.rs.
//
// Ported from Create's `content/kinetics/chainDrive/ChainDriveBlock.java` (updateShape, getConnectionAxis),
// `ChainGearshiftBlock.java`, `ChainGearshiftBlockEntity.java` and `ChainDriveGenerator.java`
// (https://github.com/Creators-of-Create/Create, branch mc1.21.1/dev, commit 4148fd8).
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

use std::collections::VecDeque;

pub type Pos = [i32; 3];

pub const DRIVE: &str = "kinetics:encased_chain_drive";
pub const GEARSHIFT: &str = "kinetics:adjustable_chain_gearshift";
/// The block state property naming the axis the drive is chained along, relative to its own (`CONNECTED_ALONG_FIRST_COORDINATE`).
pub const FIRST: &str = "axis_along_first";
/// The gearshift's redstone signal, saved in its block entity (the `powered` property only says it is above 0).
pub const SIGNAL_KEY: &str = "signal";

pub fn is_chain(name: &str) -> bool {
    name == DRIVE || name == GEARSHIFT
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Start,
    Middle,
    End,
    None,
}

impl Part {
    pub fn parse(s: &str) -> Option<Part> {
        Some(match s {
            "start" => Part::Start,
            "middle" => Part::Middle,
            "end" => Part::End,
            "none" => Part::None,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Part::Start => "start",
            Part::Middle => "middle",
            Part::End => "end",
            Part::None => "none",
        }
    }
}

/// The part of a chain drive's block state that decides how it chains.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    /// 0 = x, 1 = y, 2 = z: the axis of the shaft inside.
    pub axis: usize,
    pub first: bool,
    pub part: Part,
}

/// The axis the drive is chained along (`getConnectionAxis`).
pub fn connection_axis(axis: usize, first: bool) -> usize {
    match (first, axis) {
        (true, 0) => 1,
        (true, _) => 0,
        (false, 2) => 1,
        (false, _) => 2,
    }
}

/// `ChainDriveBlock.updateShape` for the neighbour on the face `face_axis` (positive side if `positive`), none if
/// that is not a chain drive.
pub fn update_shape(s: Shape, face_axis: usize, positive: bool, neighbour: Option<Shape>) -> Shape {
    if s.axis == face_axis {
        return s;
    }
    let conn = connection_axis(s.axis, s.first);
    let facing_first = if s.axis == 0 { face_axis == 1 } else { face_axis == 0 };
    let Some(o) = neighbour else {
        if facing_first != s.first || s.part == Part::None {
            return s;
        }
        let part = match s.part {
            Part::Middle if positive => Part::End,
            Part::Middle => Part::Start,
            p if (p == Part::Start) == positive => Part::None,
            p => p,
        };
        return Shape { part, ..s };
    };
    if o.axis == face_axis || (o.part != Part::None && connection_axis(o.axis, o.first) != face_axis) {
        return s;
    }
    let (mut part, mut first) = (s.part, s.first);
    if part == Part::None {
        part = if positive { Part::Start } else { Part::End };
        first = facing_first;
    } else if conn != face_axis {
        return s;
    }
    if (part == Part::Start) != positive {
        part = Part::Middle;
    }
    Shape { axis: s.axis, first, part }
}

/// Create's `Iterate.directions` order: down, up, north, south, west, east, as (axis, positive).
const FACES: [(usize, bool); 6] = [(1, false), (1, true), (2, false), (2, true), (0, false), (0, true)];

fn step(p: Pos, axis: usize, positive: bool) -> Pos {
    let mut q = p;
    q[axis] += if positive { 1 } else { -1 };
    q
}

/// A chain gearshift's `powered` and what it needs of its block entity.
pub trait World {
    fn block(&self, p: Pos) -> String;
    fn prop(&self, p: Pos, key: &str) -> Option<String>;
    fn set_prop(&mut self, p: Pos, key: &str, value: &str) -> bool;
    /// The strongest redstone signal at `p`, 0 to 15.
    fn signal(&mut self, p: Pos) -> i32;
    fn stored_signal(&mut self, p: Pos) -> i32;
    fn store_signal(&mut self, p: Pos, signal: i32);
}

fn shape_at(w: &impl World, p: Pos) -> Option<Shape> {
    if !is_chain(&w.block(p)) {
        return None;
    }
    Some(Shape {
        axis: ["x", "y", "z"].iter().position(|a| Some(*a) == w.prop(p, "axis").as_deref())?,
        first: w.prop(p, FIRST).as_deref() == Some("true"),
        part: Part::parse(&w.prop(p, "part")?)?,
    })
}

// shortcut: the chain is re-shaped at the end of the tick, not inside the placement; raise this cap if a build
// of chained drives longer than it fails to settle.
const MAX_STEPS: usize = 4096;

/// Chains the drives at `marks` and around them to their neighbours, and makes each gearshift follow its redstone.
/// Returns the positions whose network must be solved again: Create cuts the network for a tick when a gearshift's
/// signal changes (`detachKinetics`); here it is solved again from the new signal at once.
pub fn settle(w: &mut impl World, marks: &[Pos]) -> Vec<Pos> {
    let mut queue: VecDeque<Pos> = VecDeque::new();
    for &m in marks {
        queue.push_back(m);
        queue.extend(FACES.iter().map(|&(a, pos)| step(m, a, pos)));
    }
    let mut changed = Vec::new();
    let mut seen = Vec::new();
    let mut steps = 0;
    while let Some(p) = queue.pop_front() {
        steps += 1;
        if steps > MAX_STEPS {
            break;
        }
        let Some(old) = shape_at(w, p) else { continue };
        let mut now = old;
        for (axis, positive) in FACES {
            now = update_shape(now, axis, positive, shape_at(w, step(p, axis, positive)));
        }
        if now != old {
            w.set_prop(p, FIRST, if now.first { "true" } else { "false" });
            w.set_prop(p, "part", now.part.name());
            changed.push(p);
            queue.extend(FACES.iter().map(|&(a, pos)| step(p, a, pos)));
        }
        if !seen.contains(&p) {
            seen.push(p);
            if w.block(p) == GEARSHIFT {
                let live = w.signal(p);
                if live != w.stored_signal(p) {
                    w.store_signal(p, live);
                    changed.push(p);
                }
                let powered = if live > 0 { "true" } else { "false" };
                if w.prop(p, "powered").as_deref() != Some(powered) {
                    w.set_prop(p, "powered", powered);
                }
            }
        }
    }
    changed
}

// Positions whose block or neighbour changed (`native_block_changed`, `native_neighbor_changed`), settled at the
// end of their level's tick. Both run on the server thread.
static MARKS: std::sync::Mutex<Vec<(&'static str, Pos)>> = std::sync::Mutex::new(Vec::new());

/// Marks the chain drive (or the place of one) at `pos` in `dimension` (`minecraft:overworld`).
pub fn mark(dimension: &'static str, pos: Pos) {
    let mut marks = MARKS.lock().unwrap_or_else(|e| e.into_inner());
    if !marks.contains(&(dimension, pos)) {
        marks.push((dimension, pos));
    }
}

/// The positions marked in `dimension` since the last call.
pub fn take(dimension: &str) -> Vec<Pos> {
    let mut marks = MARKS.lock().unwrap_or_else(|e| e.into_inner());
    let mut out = Vec::new();
    marks.retain(|&(d, p)| {
        if d == dimension {
            out.push(p);
            false
        } else {
            true
        }
    });
    out
}

/// Every static model `static_look` can name, for `kinetics_chain_models` to register (they exist only with the look pack).
pub const STATIC_LOOK_MODELS: [&str; 15] = [
    "create:block/encased_chain_drive/single",
    "create:block/encased_chain_drive/middle_horizontal",
    "create:block/encased_chain_drive/middle_vertical",
    "create:block/encased_chain_drive/end_horizontal",
    "create:block/encased_chain_drive/end_vertical",
    "create:block/adjustable_chain_gearshift_single",
    "create:block/adjustable_chain_gearshift_middle_horizontal",
    "create:block/adjustable_chain_gearshift_middle_vertical",
    "create:block/adjustable_chain_gearshift_end_horizontal",
    "create:block/adjustable_chain_gearshift_end_vertical",
    "create:block/adjustable_chain_gearshift_single_powered",
    "create:block/adjustable_chain_gearshift_middle_horizontal_powered",
    "create:block/adjustable_chain_gearshift_middle_vertical_powered",
    "create:block/adjustable_chain_gearshift_end_horizontal_powered",
    "create:block/adjustable_chain_gearshift_end_vertical_powered",
];

/// Create's static models: `encased_chain_drive/<suffix>` for the drive, `adjustable_chain_gearshift_<suffix>` with
/// `_powered` for the gearshift (they exist only while the look pack is on). With the (x, y) turns of the block state
/// that `ChainDriveGenerator` gives it, applied x first.
pub fn static_look(name: &str, s: Shape, powered: bool) -> Option<(String, i32, i32)> {
    let suffix = if s.part == Part::None {
        "single".to_string()
    } else {
        let section = if s.part == Part::Middle { "middle" } else { "end" };
        format!("{section}_{}", if s.axis == 1 { "vertical" } else { "horizontal" })
    };
    let model = match name {
        DRIVE => format!("create:block/encased_chain_drive/{suffix}"),
        GEARSHIFT => format!("create:block/adjustable_chain_gearshift_{suffix}{}", if powered { "_powered" } else { "" }),
        _ => return None,
    };
    let (x, y) = if s.part == Part::None {
        (if s.axis == 1 { 90 } else { 0 }, if s.axis == 0 { 90 } else { 0 })
    } else {
        let start = s.part == Part::Start;
        let x = match s.axis {
            0 => (if s.first { 90 } else { 0 }) + if start { 180 } else { 0 },
            2 if s.first => 0,
            2 => if start { 270 } else { 90 },
            _ => 0,
        };
        let y = match s.axis {
            2 => if s.first && s.part == Part::End { 270 } else { 90 },
            1 => {
                let flip = (s.part == Part::End && !s.first) || (start && s.first);
                (if s.first { 90 } else { 0 }) + if flip { 180 } else { 0 }
            }
            _ => 0,
        };
        (x, y)
    };
    Some((model, x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const NONE: Shape = Shape { axis: 1, first: false, part: Part::None };

    fn at(axis: usize, first: bool, part: Part) -> Shape {
        Shape { axis, first, part }
    }

    #[test]
    fn the_connection_axis_is_the_first_or_second_other_axis() {
        assert_eq!(connection_axis(0, true), 1);
        assert_eq!(connection_axis(0, false), 2);
        assert_eq!(connection_axis(1, true), 0);
        assert_eq!(connection_axis(1, false), 2);
        assert_eq!(connection_axis(2, true), 0);
        assert_eq!(connection_axis(2, false), 1);
    }

    #[test]
    fn a_drive_beside_another_starts_a_chain_toward_it() {
        // Both vertical; the neighbour is east (x, positive): connected along the first coordinate.
        let s = update_shape(NONE, 0, true, Some(NONE));
        assert_eq!(s, at(1, true, Part::Start));
        let s = update_shape(NONE, 0, false, Some(NONE));
        assert_eq!(s, at(1, true, Part::End));
        // A neighbour on the drive's own axis does not chain.
        assert_eq!(update_shape(NONE, 1, true, Some(NONE)), NONE);
        // Neither does a block that is not a drive.
        assert_eq!(update_shape(NONE, 0, true, None), NONE);
    }

    #[test]
    fn a_second_neighbour_on_the_other_side_makes_a_middle_and_losing_one_ends_it() {
        let start = at(1, true, Part::Start);
        assert_eq!(update_shape(start, 0, false, Some(NONE)).part, Part::Middle);
        let mid = at(1, true, Part::Middle);
        assert_eq!(update_shape(mid, 0, true, None).part, Part::End);
        assert_eq!(update_shape(mid, 0, false, None).part, Part::Start);
        assert_eq!(update_shape(start, 0, true, None).part, Part::None);
        // A chain along another axis is left alone.
        assert_eq!(update_shape(start, 2, true, Some(NONE)), start);
    }

    /// A little world of drives: block name, axis, first, part, signal stored, signal live.
    struct Fake {
        blocks: HashMap<Pos, (String, [String; 3])>,
        stored: HashMap<Pos, i32>,
        live: HashMap<Pos, i32>,
    }

    impl Fake {
        fn new(drives: &[(Pos, &str, &str)]) -> Fake {
            let mut f = Fake { blocks: HashMap::new(), stored: HashMap::new(), live: HashMap::new() };
            for &(p, name, axis) in drives {
                f.blocks.insert(p, (name.to_string(), [axis.into(), "false".into(), "none".into()]));
            }
            f
        }
        fn part(&self, p: Pos) -> String {
            self.blocks[&p].1[2].clone()
        }
    }

    impl World for Fake {
        fn block(&self, p: Pos) -> String {
            self.blocks.get(&p).map(|b| b.0.clone()).unwrap_or("minecraft:air".into())
        }
        fn prop(&self, p: Pos, key: &str) -> Option<String> {
            let b = self.blocks.get(&p)?;
            match key {
                "axis" => Some(b.1[0].clone()),
                FIRST => Some(b.1[1].clone()),
                "part" => Some(b.1[2].clone()),
                _ => None,
            }
        }
        fn set_prop(&mut self, p: Pos, key: &str, value: &str) -> bool {
            let Some(b) = self.blocks.get_mut(&p) else { return false };
            match key {
                FIRST => b.1[1] = value.into(),
                "part" => b.1[2] = value.into(),
                _ => return false,
            }
            true
        }
        fn signal(&mut self, p: Pos) -> i32 {
            self.live.get(&p).copied().unwrap_or(0)
        }
        fn stored_signal(&mut self, p: Pos) -> i32 {
            self.stored.get(&p).copied().unwrap_or(0)
        }
        fn store_signal(&mut self, p: Pos, signal: i32) {
            self.stored.insert(p, signal);
        }
    }

    #[test]
    fn a_row_of_three_chains_start_middle_end_and_breaking_the_middle_unchains_the_ends() {
        let row = [([0, 0, 0], DRIVE, "y"), ([1, 0, 0], DRIVE, "y"), ([2, 0, 0], DRIVE, "y")];
        let mut w = Fake::new(&row);
        settle(&mut w, &[[1, 0, 0]]);
        assert_eq!([w.part([0, 0, 0]), w.part([1, 0, 0]), w.part([2, 0, 0])], ["start", "middle", "end"]);
        w.blocks.remove(&[1, 0, 0]);
        settle(&mut w, &[[1, 0, 0]]);
        assert_eq!([w.part([0, 0, 0]), w.part([2, 0, 0])], ["none", "none"]);
    }

    #[test]
    fn a_gearshift_stores_its_signal_and_reports_the_network_to_solve_again() {
        let mut w = Fake::new(&[([0, 0, 0], GEARSHIFT, "y")]);
        w.live.insert([0, 0, 0], 7);
        assert_eq!(settle(&mut w, &[[0, 0, 0]]), vec![[0, 0, 0]]);
        assert_eq!(w.stored[&[0, 0, 0]], 7);
        assert!(settle(&mut w, &[[0, 0, 0]]).is_empty());
    }

    #[test]
    fn the_look_follows_part_axis_and_power() {
        for name in [DRIVE, GEARSHIFT] {
            for axis in 0..3 {
                for first in [false, true] {
                    for part in [Part::None, Part::Start, Part::Middle, Part::End] {
                        for powered in [false, true] {
                            let (m, ..) = static_look(name, at(axis, first, part), powered).unwrap();
                            assert!(STATIC_LOOK_MODELS.contains(&m.as_str()), "{m}");
                        }
                    }
                }
            }
        }
        let (m, x, y) = static_look(DRIVE, NONE, false).unwrap();
        assert_eq!((m.as_str(), x, y), ("create:block/encased_chain_drive/single", 90, 0));
        let (m, x, y) = static_look(GEARSHIFT, at(2, false, Part::Start), true).unwrap();
        assert_eq!((m.as_str(), x, y), ("create:block/adjustable_chain_gearshift_end_horizontal_powered", 270, 90));
        let (m, ..) = static_look(DRIVE, at(1, true, Part::Middle), false).unwrap();
        assert_eq!(m, "create:block/encased_chain_drive/middle_vertical");
        assert_eq!(static_look("kinetics:shaft", NONE, false), None);
    }
}
