// Kinetics: the water wheels' rules, with no game types. This file is both a test target of the crate and the
// module the bundle adds to the game as `game::kinetics_water_wheel`.
//
// Ported from Create's `content/kinetics/waterwheel/` (WaterWheelBlockEntity.determineAndApplyFlowScore,
// getFlowVectorAtPosition, getGeneratedSpeed, SMALL_OFFSETS / LARGE_OFFSETS; LargeWaterWheelBlock.tick and
// getStateForPlacement; WaterWheelStructuralBlock.getMaster / stillValid) and vanilla's `FlowingFluid.getFlow`,
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
// shortcut: the API has no "is this block solid / replaceable" query, so `replaceable` is a list of vanilla's
// replaceable blocks and "solid" is "not replaceable"; add real queries when the API has them.

use std::collections::HashMap;
use std::sync::Mutex;

pub type Pos = [i32; 3];

pub const SMALL: &str = "kinetics:water_wheel";
pub const LARGE: &str = "kinetics:large_water_wheel";
pub const STRUCTURE: &str = "kinetics:water_wheel_structure";
/// The block entity's saved keys: the flow score (an int) and the planks it is made of (an item name).
pub const FLOW_KEY: &str = "flow";
pub const MATERIAL_KEY: &str = "material";
/// `Blocks.SPRUCE_PLANKS`.
pub const DEFAULT_MATERIAL: &str = "minecraft:spruce_planks";
/// Ticks between two looks at the water (Create: a lazy tick every 60 and one tick after each shape update).
pub const PERIOD: u8 = 10;

pub fn is_wheel(name: &str) -> bool {
    name == SMALL || name == LARGE
}

/// `getSize()`: 1 for the small wheel, 2 for the large one.
pub fn size(name: &str) -> i32 {
    if name == LARGE { 2 } else { 1 }
}

/// `getGeneratedSpeed`: the flow score clamped to one way or the other, 8 RPM, halved for the large wheel.
pub fn generated(score: i32, size: i32) -> i32 {
    score.clamp(-1, 1) * 8 / size
}

pub fn axis_index(value: &str) -> Option<usize> {
    match value {
        "x" | "east" | "west" => Some(0),
        "y" | "up" | "down" => Some(1),
        "z" | "north" | "south" => Some(2),
        _ => None,
    }
}

// ---- Fluids -------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Liquid {
    Water,
    Lava,
}

/// A `FluidState`: `amount` 1 to 8 (a source is 8), `falling` is the flowing fluid above another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fluid {
    pub liquid: Liquid,
    pub amount: u8,
    pub falling: bool,
}

impl Fluid {
    pub const fn source(liquid: Liquid) -> Self {
        Fluid { liquid, amount: 8, falling: false }
    }
    /// `getOwnHeight`.
    fn height(self) -> f64 {
        f64::from(self.amount) / 9.0
    }
}

/// What the wheel needs to know of one block.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    pub fluid: Option<Fluid>,
    /// Stands in for `blocksMotion` / `isFaceSturdy`.
    pub solid: bool,
    /// A bubble column: `Some(true)` drags down.
    pub drag: Option<bool>,
    pub ice: bool,
}

/// Whether a block can be built over (`canBeReplaced`): vanilla's air, fluids and plants. `layers` is a snow
/// layer's height.
pub fn replaceable(name: &str, layers: Option<&str>) -> bool {
    let Some(name) = name.strip_prefix("minecraft:") else { return false };
    match name {
        "air" | "cave_air" | "void_air" | "water" | "lava" | "short_grass" | "tall_grass" | "fern" | "large_fern"
        | "seagrass" | "tall_seagrass" | "vine" | "dead_bush" | "fire" | "soul_fire" | "bubble_column"
        | "glow_lichen" | "hanging_roots" | "light" | "structure_void" | "sculk_vein" | "nether_sprouts"
        | "crimson_roots" | "warped_roots" | "short_dry_grass" | "tall_dry_grass" => true,
        "snow" => layers == Some("1"),
        _ => false,
    }
}

/// The `Cell` of block `name`, `prop` reading its state's properties (`level`, `waterlogged`, `drag`, `layers`).
pub fn classify(name: &str, prop: impl Fn(&str) -> Option<String>) -> Cell {
    let level = |liquid| {
        // `LiquidBlock.getFluidState`: level 0 a source, 1 to 7 flowing (amount 8 - level), 8 and up falling.
        let l: u8 = prop("level").and_then(|v| v.parse().ok()).unwrap_or(0);
        Some(match l {
            0 => Fluid::source(liquid),
            1..=7 => Fluid { liquid, amount: 8 - l, falling: false },
            _ => Fluid { liquid, amount: 8, falling: true },
        })
    };
    let (fluid, drag) = match name {
        "minecraft:water" => (level(Liquid::Water), None),
        "minecraft:lava" => (level(Liquid::Lava), None),
        "minecraft:bubble_column" => (
            Some(Fluid::source(Liquid::Water)),
            Some(prop("drag").as_deref() == Some("true")),
        ),
        "minecraft:kelp" | "minecraft:kelp_plant" | "minecraft:seagrass" | "minecraft:tall_seagrass" => {
            (Some(Fluid::source(Liquid::Water)), None)
        }
        _ if prop("waterlogged").as_deref() == Some("true") => (Some(Fluid::source(Liquid::Water)), None),
        _ => (None, None),
    };
    Cell {
        fluid,
        solid: !replaceable(name, prop("layers").as_deref()),
        drag,
        ice: matches!(name, "minecraft:ice" | "minecraft:packed_ice" | "minecraft:blue_ice" | "minecraft:frosted_ice"),
    }
}

fn add(a: Pos, b: Pos) -> Pos {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

const HORIZONTAL: [Pos; 4] = [[0, 0, -1], [1, 0, 0], [0, 0, 1], [-1, 0, 0]];

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1.0e-4 { [0.0; 3] } else { [v[0] / len, v[1] / len, v[2] / len] }
}

/// `FlowingFluid.getFlow` of the fluid at `pos`: the way it runs downhill, length 1 (or zero on the flat).
pub fn flow(get: &impl Fn(Pos) -> Cell, pos: Pos) -> [f64; 3] {
    let Some(own) = get(pos).fluid else { return [0.0; 3] };
    let affects = |f: Option<Fluid>| f.is_none_or(|f| f.liquid == own.liquid);
    let (mut x, mut z) = (0.0, 0.0);
    for d in HORIZONTAL {
        let p = add(pos, d);
        let cell = get(p);
        if !affects(cell.fluid) {
            continue;
        }
        let mut f = cell.fluid.map_or(0.0, Fluid::height);
        let mut g = 0.0;
        if f == 0.0 {
            if !cell.solid
                && let Some(below) = get(add(p, [0, -1, 0])).fluid
                && below.liquid == own.liquid
            {
                f = below.height();
                if f > 0.0 {
                    g = own.height() - (f - 0.888_888_9);
                }
            }
        } else {
            g = own.height() - f;
        }
        if g != 0.0 {
            x += f64::from(d[0]) * g;
            z += f64::from(d[2]) * g;
        }
    }
    let mut v = [x, 0.0, z];
    if own.falling {
        // `isSolidFace`: not the same fluid, not ice, and a solid block.
        let face = |p: Pos| {
            let c = get(p);
            c.fluid.is_none_or(|f| f.liquid != own.liquid) && !c.ice && c.solid
        };
        for d in HORIZONTAL {
            let p = add(pos, d);
            if face(p) || face(add(p, [0, 1, 0])) {
                let n = normalize(v);
                v = [n[0], n[1] - 6.0, n[2]];
                break;
            }
        }
    }
    normalize(v)
}

/// `getFlowVectorAtPosition`: a bubble column pushes straight up or down.
pub fn flow_at(get: &impl Fn(Pos) -> Cell, pos: Pos) -> [f64; 3] {
    match get(pos).drag {
        Some(down) => [0.0, if down { -1.0 } else { 1.0 }, 0.0],
        None => flow(get, pos),
    }
}

// ---- The flow score -----------------------------------------------------------------------------------------

/// Directions are `[x, y, z]` units in vanilla's `Iterate.directions` order (the order the sets are built in doesn't
/// matter: they are sets).
const DIRECTIONS: [Pos; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

fn axis_of(d: Pos) -> usize {
    (0..3).find(|&a| d[a] != 0).unwrap_or(1)
}

/// `SMALL_OFFSETS` / `LARGE_OFFSETS`: the blocks beside the wheel (and, for the large one, the ring outside its
/// 3x3 body) whose water drives it.
pub fn offsets(axis: usize, size: i32) -> Vec<Pos> {
    let mut out: Vec<Pos> = Vec::new();
    for d in DIRECTIONS.into_iter().filter(|d| axis_of(*d) != axis) {
        if size == 1 {
            out.push(d);
            continue;
        }
        let centre = [d[0] * 2, d[1] * 2, d[2] * 2];
        out.push(centre);
        for d2 in DIRECTIONS.into_iter().filter(|d2| axis_of(*d2) != axis && axis_of(*d2) != axis_of(d)) {
            out.push(add(centre, d2));
        }
    }
    out
}

/// `VecHelper.rotate(v, 90, axis)`.
fn quarter_turn(v: [f64; 3], axis: usize) -> [f64; 3] {
    match axis {
        0 => [v[0], -v[2], v[1]],
        1 => [v[2], v[1], -v[0]],
        _ => [-v[1], v[0], v[2]],
    }
}

/// `determineAndApplyFlowScore`: how many of the `offsets` push the wheel one way (+1) or the other (-1), and
/// whether one of them is lava. `get` reads the world around `centre`.
pub fn flow_score(get: &impl Fn(Pos) -> Cell, centre: Pos, axis: usize, size: i32) -> (i32, bool) {
    let mut plane = [1.0; 3];
    plane[axis] = 0.0;
    let (mut score, mut lava) = (0, false);
    for off in offsets(axis, size) {
        let at = add(centre, off);
        let f = flow_at(get, at);
        lava |= get(at).fluid.is_some_and(|f| f.liquid == Liquid::Lava);
        let f = [f[0] * plane[0], f[1] * plane[1], f[2] * plane[2]];
        if f == [0.0; 3] {
            continue;
        }
        let f = normalize(f);
        let n = normalize([f64::from(off[0]), f64::from(off[1]), f64::from(off[2])]);
        let motion = quarter_turn(n, axis);
        let dot = f[0] * motion[0] + f[1] * motion[1] + f[2] * motion[2];
        if dot.abs() > 0.5 {
            score += dot.signum() as i32;
        }
    }
    (score, lava)
}

// ---- The large wheel's body ---------------------------------------------------------------------------------

/// `Direction.getClockWise(axis)`.
fn clockwise(d: Pos, axis: usize) -> Pos {
    if axis_of(d) == axis {
        return d;
    }
    match axis {
        // getClockWiseX: up -> south -> down -> north -> up.
        0 => match d {
            [0, 1, 0] => [0, 0, 1],
            [0, 0, -1] => [0, 1, 0],
            [0, -1, 0] => [0, 0, -1],
            _ => [0, -1, 0],
        },
        // getClockWise: north -> east -> south -> west -> north.
        1 => match d {
            [0, 0, -1] => [1, 0, 0],
            [1, 0, 0] => [0, 0, 1],
            [0, 0, 1] => [-1, 0, 0],
            _ => [0, 0, -1],
        },
        // getClockWiseZ: up -> east -> down -> west -> up.
        _ => match d {
            [0, 1, 0] => [1, 0, 0],
            [1, 0, 0] => [0, -1, 0],
            [0, -1, 0] => [-1, 0, 0],
            _ => [0, 1, 0],
        },
    }
}

pub fn dir_name(v: Pos) -> &'static str {
    match v {
        [0, -1, 0] => "down",
        [0, 1, 0] => "up",
        [0, 0, -1] => "north",
        [0, 0, 1] => "south",
        [-1, 0, 0] => "west",
        _ => "east",
    }
}

pub fn dir_of(name: &str) -> Option<Pos> {
    DIRECTIONS.into_iter().find(|d| dir_name(*d) == name)
}

/// The eight blocks a large wheel on `axis` fills round itself (`LargeWaterWheelBlock.tick`), each with the `facing`
/// of the structure block there: toward the next block on the way to the wheel.
pub fn body(axis: usize) -> Vec<(Pos, &'static str)> {
    let mut out = Vec::new();
    for side in DIRECTIONS.into_iter().filter(|d| axis_of(*d) != axis) {
        for secondary in [false, true] {
            let target = if secondary { clockwise(side, axis) } else { side };
            let base = if secondary { side } else { [0; 3] };
            out.push((add(base, target), dir_name([-target[0], -target[1], -target[2]])));
        }
    }
    out
}

/// What `stillValid` / `getMaster` see in a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Other,
    /// A large wheel on this axis.
    Wheel(usize),
    /// A structure block facing this way.
    Structure(Pos),
}

/// The `Part` of block `name`, `prop` reading its state's `axis` / `facing`.
pub fn part_of(name: &str, prop: impl Fn(&str) -> Option<String>) -> Part {
    if name == LARGE {
        prop("axis").as_deref().and_then(axis_index).map_or(Part::Other, Part::Wheel)
    } else if name == STRUCTURE {
        prop("facing").as_deref().and_then(dir_of).map_or(Part::Other, Part::Structure)
    } else {
        Part::Other
    }
}

/// `WaterWheelStructuralBlock.stillValid` for the structure block at `pos` facing `facing`.
pub fn still_valid(get: &impl Fn(Pos) -> Part, pos: Pos, facing: Pos, directly_adjacent: bool) -> bool {
    let target = add(pos, facing);
    let part = get(target);
    if !directly_adjacent
        && let Part::Structure(next) = part
        && still_valid(get, target, next, true)
    {
        return true;
    }
    matches!(part, Part::Wheel(axis) if axis != axis_of(facing))
}

/// `getMaster`: where the structure block at `pos` facing `facing` leads to.
pub fn master(get: &impl Fn(Pos) -> Part, pos: Pos, facing: Pos) -> Pos {
    let target = add(pos, facing);
    match get(target) {
        Part::Structure(next) => master(get, target, next),
        _ => target,
    }
}

/// Whether the large wheel's body fits: every block round it on its plane can be built over.
pub fn fits(axis: usize, replaceable: impl Fn(Pos) -> bool) -> bool {
    body(axis).into_iter().all(|(off, _)| replaceable(off))
}

// ---- Materials ----------------------------------------------------------------------------------------------

/// The wood types with a wheel model (`assets/kinetics/models/block/*_water_wheel_<wood>`; Create takes any planks).
pub const WOODS: [&str; 11] =
    ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "mangrove", "cherry", "pale_oak", "bamboo", "crimson"];
pub const WARPED: &str = "warped";

/// The wood of an item (`minecraft:oak_planks`: oak); none for anything that isn't a vanilla planks.
pub fn wood_of(item: &str) -> Option<&'static str> {
    let wood = item.strip_prefix("minecraft:")?.strip_suffix("_planks")?;
    WOODS.iter().copied().chain([WARPED]).find(|w| *w == wood)
}

/// The standalone model of a wheel made of `material`: oak's for a material with no model.
pub fn model(large: bool, material: &str) -> String {
    let wood = wood_of(material).unwrap_or("oak");
    format!("kinetics:block/{}water_wheel_{wood}", if large { "large_" } else { "" })
}

/// Every wheel model, for the bake.
pub fn models() -> Vec<String> {
    WOODS.iter().copied().chain([WARPED]).flat_map(|w| {
        [false, true].map(|large| model(large, &format!("minecraft:{w}_planks")))
    }).collect()
}

/// Create's own static model of the small wheel (the frame), drawn over our placeholder with the look pack.
pub const LOOK_FRAME: &str = "create:block/water_wheel/block";

// ---- Per-wheel state ----------------------------------------------------------------------------------------

// Ticks since a wheel last looked at the water, by dimension and position; every call runs on the server thread.
// shortcut: entries of removed wheels stay, a few bytes each; prune them if worlds with thousands of wheels appear.
static AGES: Mutex<Option<HashMap<(String, Pos), u8>>> = Mutex::new(None);

/// One tick of the wheel at `pos`: whether it is time to look at the water (the first tick, then every `PERIOD`).
pub fn due(dimension: &str, pos: Pos) -> bool {
    let mut all = AGES.lock().unwrap_or_else(|e| e.into_inner());
    let age = all.get_or_insert_with(HashMap::new).entry((dimension.to_string(), pos)).or_insert(PERIOD);
    *age += 1;
    if *age > PERIOD {
        *age = 1;
        return true;
    }
    false
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Task {
    /// A large wheel was placed: fill its body.
    Build,
    /// A structure block went: the wheel it belonged to goes too.
    Broken,
    /// A large wheel went: its structure blocks go.
    Removed,
}

static QUEUE: Mutex<Vec<(&'static str, Pos, Task)>> = Mutex::new(Vec::new());

pub fn queue(dimension: &'static str, pos: Pos, task: Task) {
    QUEUE.lock().unwrap_or_else(|e| e.into_inner()).push((dimension, pos, task));
}

/// The tasks queued for `dimension` since the last call.
pub fn take(dimension: &str) -> Vec<(Pos, Task)> {
    let mut all = QUEUE.lock().unwrap_or_else(|e| e.into_inner());
    let mut out = Vec::new();
    all.retain(|&(d, p, t)| {
        if d == dimension {
            out.push((p, t));
            false
        } else {
            true
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny world: `water` cells with their level, `solid` cells.
    fn world(water: &[(Pos, &'static str)], solid: &[Pos]) -> impl Fn(Pos) -> Cell {
        let water: Vec<_> = water.to_vec();
        let solid: Vec<_> = solid.to_vec();
        move |p| {
            if let Some((_, level)) = water.iter().find(|(q, _)| *q == p) {
                return classify("minecraft:water", |k| (k == "level").then(|| level.to_string()));
            }
            Cell { solid: solid.contains(&p), ..Cell::default() }
        }
    }

    #[test]
    fn levels_make_fluid_states() {
        let c = |l: &'static str| classify("minecraft:water", |k| (k == "level").then(|| l.to_string())).fluid.unwrap();
        assert_eq!(c("0"), Fluid::source(Liquid::Water));
        assert_eq!((c("3").amount, c("3").falling), (5, false));
        assert_eq!((c("9").amount, c("9").falling), (8, true));
        let wl = classify("minecraft:oak_fence", |k| (k == "waterlogged").then(|| "true".to_string()));
        assert_eq!(wl.fluid, Some(Fluid::source(Liquid::Water)));
        assert!(wl.solid);
    }

    #[test]
    fn water_runs_downhill_away_from_the_source() {
        // A source at the origin with a flowing level-1 block east of it: the flow at the east block points east.
        let w = world(&[([0, 0, 0], "0"), ([1, 0, 0], "1")], &[]);
        let v = flow(&w, [1, 0, 0]);
        // The source is higher on the west, so the lower block east of it is pulled east (+x).
        assert!(v[0] > 0.99 && v[2].abs() < 1e-9, "{v:?}");
        // A lone source is flat.
        assert_eq!(flow(&world(&[([0, 0, 0], "0")], &[]), [0, 0, 0]), [0.0; 3]);
    }

    #[test]
    fn a_stream_going_one_way_turns_a_small_wheel_one_way() {
        // Wheel at the origin on y; a flow running east on its north side (z = -1) and the same on its south side.
        // Going round counter-clockwise seen from +y (VecHelper.rotate(normal, 90, y)): the north side must go west.
        let on_north = world(&[([0, 0, -2], "0"), ([0, 0, -1], "1"), ([1, 0, -1], "2")], &[]);
        let (score, lava) = flow_score(&on_north, [0, 0, 0], 1, 1);
        assert!(!lava);
        // Flow at (0,0,-1) runs +x (from the source side), normal = (0,0,-1), rotate 90 about y -> (-1,0,0): dot -1.
        assert_eq!(score, -1);
        // The same stream on the south side runs the other way round the wheel: the sign flips.
        let on_south = world(&[([0, 0, 2], "0"), ([0, 0, 1], "1"), ([1, 0, 1], "2")], &[]);
        assert_eq!(flow_score(&on_south, [0, 0, 0], 1, 1).0, 1);
        // Both together cancel.
        let both = |p| on_north(p).fluid.map_or_else(|| on_south(p), |_| on_north(p));
        assert_eq!(flow_score(&both, [0, 0, 0], 1, 1).0, 0);
    }

    #[test]
    fn a_bubble_column_pushes_straight_up_or_down() {
        let col = |down: bool| move |p: Pos| {
            if p == [1, 0, 0] {
                classify("minecraft:bubble_column", |k| (k == "drag").then(|| down.to_string()))
            } else {
                Cell::default()
            }
        };
        // Wheel on z, column east of it: normal +x, rotate 90 about z -> +y, so an upward column scores +1.
        assert_eq!(flow_score(&col(false), [0, 0, 0], 2, 1).0, 1);
        assert_eq!(flow_score(&col(true), [0, 0, 0], 2, 1).0, -1);
        // Wheel on y: the column's push is along the axis, which the plane removes.
        assert_eq!(flow_score(&col(false), [0, 0, 0], 1, 1).0, 0);
    }

    #[test]
    fn lava_counts_and_is_seen() {
        let lava = |p: Pos| {
            if p == [0, 0, -1] {
                classify("minecraft:lava", |_| Some("0".to_string()))
            } else if p == [1, 0, -1] {
                classify("minecraft:lava", |_| Some("2".to_string()))
            } else {
                Cell::default()
            }
        };
        let (score, is_lava) = flow_score(&lava, [0, 0, 0], 1, 1);
        assert!(is_lava);
        assert!(score.abs() <= 1);
    }

    #[test]
    fn the_speed_is_eight_halved_for_the_large_wheel_and_ignores_strength() {
        assert_eq!((generated(0, 1), generated(1, 1), generated(-5, 1)), (0, 8, -8));
        assert_eq!((generated(3, 2), generated(-1, 2)), (4, -4));
    }

    #[test]
    fn the_offsets_are_the_neighbours_and_the_outer_ring() {
        for axis in 0..3 {
            assert_eq!(offsets(axis, 1).len(), 4);
            let large = offsets(axis, 2);
            assert_eq!(large.len(), 12);
            // Two blocks out in the plane (one of them 2, the other 0 or 1 either way), none on the axis.
            assert!(large.iter().all(|o| o[axis] == 0 && o.iter().map(|v| v.abs()).max() == Some(2)));
            let mut sorted = large.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), 12);
        }
    }

    #[test]
    fn the_body_is_the_ring_of_eight_each_leading_to_the_wheel() {
        for axis in 0..3 {
            let body = body(axis);
            assert_eq!(body.len(), 8);
            let mut cells: Vec<Pos> = body.iter().map(|b| b.0).collect();
            cells.sort();
            cells.dedup();
            assert_eq!(cells.len(), 8);
            assert!(cells.iter().all(|c| c[axis] == 0 && c.iter().all(|v| v.abs() <= 1) && *c != [0; 3]));
            // Following the facings from any cell reaches the wheel at the origin, and `still_valid` agrees.
            let get = |p: Pos| {
                if p == [0; 3] {
                    return Part::Wheel(axis);
                }
                body.iter().find(|b| b.0 == p).map_or(Part::Other, |b| Part::Structure(dir_of(b.1).unwrap()))
            };
            for (p, f) in &body {
                let f = dir_of(f).unwrap();
                assert!(still_valid(&get, *p, f, false), "{p:?}");
                assert_eq!(master(&get, *p, f), [0; 3], "{p:?}");
            }
        }
    }

    #[test]
    fn a_structure_block_dies_with_its_wheel_or_when_the_wheel_turns() {
        let body = body(1);
        let (p, f) = body[0];
        let f = dir_of(f).unwrap();
        // No wheel: invalid.
        let none = |q: Pos| body.iter().find(|b| b.0 == q).map_or(Part::Other, |b| Part::Structure(dir_of(b.1).unwrap()));
        assert!(!still_valid(&none, p, f, false));
        // A wheel on the same axis as the facing doesn't count: its body would not be here.
        let along = |q: Pos| if q == [0; 3] { Part::Wheel(axis_of(f)) } else { none(q) };
        assert!(!still_valid(&along, p, f, false));
    }

    #[test]
    fn the_body_needs_room_in_its_plane_only() {
        // Stone above the wheel doesn't matter on y; stone in the plane does.
        assert!(fits(1, |o| o != [0, 1, 0]));
        assert!(!fits(1, |o| o != [1, 0, 1]));
        assert!(replaceable("minecraft:water", None) && replaceable("minecraft:snow", Some("1")));
        assert!(!replaceable("minecraft:snow", Some("2")) && !replaceable("minecraft:stone", None));
    }

    #[test]
    fn planks_make_a_material_with_a_model() {
        assert_eq!(wood_of("minecraft:dark_oak_planks"), Some("dark_oak"));
        assert_eq!(wood_of("minecraft:warped_planks"), Some("warped"));
        assert_eq!(wood_of("minecraft:oak_log"), None);
        assert_eq!(wood_of("create:something_planks"), None);
        assert_eq!(model(true, "minecraft:birch_planks"), "kinetics:block/large_water_wheel_birch");
        assert_eq!(model(false, "weird"), "kinetics:block/water_wheel_oak");
        assert_eq!(models().len(), 24);
    }

    #[test]
    fn a_wheel_looks_at_the_water_on_its_first_tick_and_then_every_period() {
        let (d, p) = ("test:age", [5, 5, 5]);
        let looks: Vec<u32> = (1..=25).filter(|_| due(d, p)).collect();
        assert_eq!(looks.len(), 3);
        assert_eq!(looks[1] - looks[0], u32::from(PERIOD));
        assert_eq!(looks[0], 1);
    }

    #[test]
    fn tasks_are_taken_by_dimension() {
        queue("test:q1", [1, 1, 1], Task::Build);
        queue("test:q2", [2, 2, 2], Task::Broken);
        assert_eq!(take("test:q1"), vec![([1, 1, 1], Task::Build)]);
        assert!(take("test:q1").is_empty());
        assert_eq!(take("test:q2"), vec![([2, 2, 2], Task::Broken)]);
    }
}
