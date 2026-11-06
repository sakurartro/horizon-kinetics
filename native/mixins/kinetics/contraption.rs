// Kinetics: moving structures (contraptions), the rules with no game types. This file is both a test target of the
// crate and the module the bundle adds to the game as `game::kinetics_contraption`; the hooks are contraption_*.rs.
//
// After Create's `content/contraptions/Contraption.java` (searchMovedStructure, moveBlock, removeBlocksFromWorld,
// addBlocksToWorld) and `BlockMovementChecks`, https://github.com/Creators-of-Create/Create, branch mc1.21.1/dev.
//
// Copyright (c) The Create Team / The Creators of Create
// Licensed under the MIT License; the full notice is in runtime.rs.
//
// How a contraption lives in the world: one `marker` entity (MARKER_TAG) holds the whole snapshot in its `data`
// (origin, offset, target, every block's state and block entity), and a `block_display` (PIECE_TAG) shows each block.
// Entities are saved with their chunks, so the snapshot and the blocks taken out of the world are saved together:
// no file of our own that could get ahead of the world or fall behind it. The displays are only a picture: after a
// reload (new entity ids) or a chunk coming back, the hook removes the ones it doesn't know and makes them again
// from the marker.

pub type Pos = [i32; 3];

pub const MARKER_TAG: &str = "kinetics.contraption";
pub const PIECE_TAG: &str = "kinetics.contraption_piece";
/// A glued box is a `marker` entity with this tag and `min`, `max` in its `data` (saved with its chunk, like the contraptions).
pub const GLUE_TAG: &str = "kinetics.glue";
pub const PISTON: &str = "kinetics:mechanical_piston";
pub const POLE: &str = "kinetics:piston_extension_pole";
/// The piston's block entity key: 1 while its structure is at the far end of the poles.
pub const EXTENDED_KEY: &str = "Extended";
/// Poles a piston counts (Create's `maxPoleLength`-ish; each pole is a block of reach).
pub const MAX_POLES: i32 = 64;
pub const GLUE_ITEM: &str = "kinetics:super_glue";
/// Blocks in one contraption. Create allows 2048; each block is an entity here, so start low for weak PCs.
pub const LIMIT: usize = 256;
/// Blocks a tick the debug command moves at.
pub const SPEED: f64 = 1.0 / 16.0;
/// The debug command (contraption_command.rs).
pub const COMMAND: &str = "kinetics_contraption";
/// `Level.setBlock` flags, Create's: taking a block out is UPDATE_CLIENTS | UPDATE_IMMEDIATE | UPDATE_KNOWN_SHAPE |
/// UPDATE_SUPPRESS_DROPS | UPDATE_MOVE_BY_PISTON (nothing drops, a torch on it stays), putting one back
/// UPDATE_ALL | UPDATE_MOVE_BY_PISTON.
pub const REMOVE_FLAGS: i32 = 2 | 8 | 16 | 32 | 64;
pub const PLACE_FLAGS: i32 = 3 | 64;

/// A glued box (Create's super glue), corners included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glue {
    pub min: Pos,
    pub max: Pos,
}

impl Glue {
    pub fn new(a: Pos, b: Pos) -> Self {
        Glue { min: [0, 1, 2].map(|i| a[i].min(b[i])), max: [0, 1, 2].map(|i| a[i].max(b[i])) }
    }
    pub fn contains(&self, p: Pos) -> bool {
        (0..3).all(|i| (self.min[i]..=self.max[i]).contains(&p[i]))
    }
}

/// What assembly makes of a block (`BlockMovementChecks`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movable {
    /// Moves with the contraption.
    Yes,
    /// Left where it is and not followed: air, liquids (`isMovementNecessary` false).
    Skip,
    /// Glued in but can't move: the whole assembly is refused (`AssemblyException.unmovableBlock`).
    No,
}

/// `isMovementNecessary` / `isMovementAllowed`: air and liquids stay, unbreakable blocks (bedrock, hardness -1) and
/// blocks whose block entity isn't ours (chests: later) refuse.
pub fn movable(name: &str, hardness: f32, liquid: bool, foreign_entity: bool) -> Movable {
    if matches!(name, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air") || liquid {
        Movable::Skip
    } else if hardness < 0.0 || foreign_entity {
        Movable::No
    } else {
        Movable::Yes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The anchor itself doesn't move.
    NothingThere,
    Unmovable(Pos),
    TooMany,
}

const DIRS: [Pos; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];

/// The blocks that move with `anchor` (`searchMovedStructure`): from it, every neighbour glued to a block already in
/// (both in one glue box), anchor first. shortcut: glue only, none of Create's "sticky" links (a drill held by its
/// shaft, a chassis); add them with the parts that have them.
pub fn collect(anchor: Pos, glue: &[Glue], mut what: impl FnMut(Pos) -> Movable, limit: usize) -> Result<Vec<Pos>, Refusal> {
    match what(anchor) {
        Movable::Yes => {}
        Movable::Skip => return Err(Refusal::NothingThere),
        Movable::No => return Err(Refusal::Unmovable(anchor)),
    }
    let mut found = vec![anchor];
    let mut seen = std::collections::HashSet::from([anchor]);
    let mut i = 0;
    while i < found.len() {
        let p = found[i];
        i += 1;
        for d in DIRS {
            let n = [p[0] + d[0], p[1] + d[1], p[2] + d[2]];
            if seen.contains(&n) || !glue.iter().any(|g| g.contains(p) && g.contains(n)) {
                continue;
            }
            seen.insert(n);
            match what(n) {
                Movable::Skip => {}
                Movable::No => return Err(Refusal::Unmovable(n)),
                Movable::Yes if found.len() == limit => return Err(Refusal::TooMany),
                Movable::Yes => found.push(n),
            }
        }
    }
    Ok(found)
}

/// One tick of motion: `offset` goes toward `target` by at most `speed` blocks.
pub fn step(offset: [f64; 3], target: [f64; 3], speed: f64) -> [f64; 3] {
    let d = [0, 1, 2].map(|i| target[i] - offset[i]);
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len <= speed {
        return target;
    }
    [0, 1, 2].map(|i| offset[i] + d[i] / len * speed)
}

/// The cells a block whose corner is at `at` overlaps: two along an axis it is between cells on.
pub fn cells(at: [f64; 3]) -> Vec<Pos> {
    const EPS: f64 = 1e-6;
    let span = |v: f64| (v + EPS).floor() as i32..=(v + 1.0 - EPS).floor() as i32;
    let mut out = Vec::new();
    for x in span(at[0]) {
        for y in span(at[1]) {
            for z in span(at[2]) {
                out.push([x, y, z]);
            }
        }
    }
    out
}

/// `MechanicalPistonBlockEntity.getMovementSpeed`: blocks a tick from the RPM (`convertToLinear` is RPM / 512), at most .49.
pub fn piston_speed(rpm: f32) -> f64 {
    f64::from(rpm / 512.0).clamp(-0.49, 0.49)
}

/// A moving drill works only facing the way it goes (Create's `VecHelper.isVecPointingTowards`).
pub fn drills_toward(facing: Pos, motion: [f64; 3]) -> bool {
    (0..3).map(|i| f64::from(facing[i]) * motion[i]).sum::<f64>() > 0.0
}

/// A moving drill's break speed (`BlockBreakingMovementBehaviour.getBlockBreakingSpeed` from
/// `MovementContext.getAnimationSpeed`), as the RPM `kinetics_drill::step` takes (it breaks at RPM / 100).
pub fn drill_rpm(blocks_per_tick: f64) -> f32 {
    let len = blocks_per_tick.abs();
    let anim = if len < 1.0 / 512.0 { 0.0 } else { ((len * 1000.0 + 100.0) as i32 / 100 * 100) as f32 };
    (anim / 500.0).clamp(1.0 / 128.0, 16.0) * 100.0
}

/// What a block in the way is to a contraption: solid ones (`BlockState.isSolid`) stop it, breakable ones a drill
/// takes (`kinetics_drill::breakable`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InWay {
    pub solid: bool,
    pub breakable: bool,
}

/// The way of one step (Create's `ContraptionCollider.isCollidingWithWorld` and the drills' `visitNewPosition`).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Way {
    /// A solid block no drill takes: the contraption can't go.
    pub blocked: bool,
    /// (piece, cell): each drill's first block to break before the contraption goes on.
    pub drill: Vec<(usize, Pos)>,
}

/// `pieces`: each block's corner after the step and whether it is a drill working along this motion.
pub fn way(pieces: &[([f64; 3], bool)], mut world: impl FnMut(Pos) -> InWay) -> Way {
    let mut w = Way::default();
    for (i, &(at, drill)) in pieces.iter().enumerate() {
        let mut drilling = false;
        for c in cells(at) {
            let b = world(c);
            if drill && b.breakable {
                if !drilling {
                    w.drill.push((i, c));
                    drilling = true;
                }
            } else if b.solid {
                w.blocked = true;
            }
        }
    }
    w
}

/// Where a block lands when the contraption stops (`addBlocksToWorld`: the offset rounded).
pub fn landing(origin: Pos, offset: [f64; 3], rel: Pos) -> Pos {
    [0, 1, 2].map(|i| origin[i] + offset[i].round() as i32 + rel[i])
}

/// The chunk a position is in.
pub fn chunk_of(x: f64, z: f64) -> (i32, i32) {
    ((x.floor() as i32) >> 4, (z.floor() as i32) >> 4)
}

/// A coordinate typed in a command: `12`, `~` or `~-3` (from `base`, the player's block).
pub fn coord(token: &str, base: i32) -> Option<i32> {
    match token.strip_prefix('~') {
        Some("") => Some(base),
        Some(rest) => rest.parse::<i32>().ok().map(|v| base + v),
        None => token.parse().ok(),
    }
}

/// Entity ids of the contraptions in the levels this game has open: the marker's, then its displays' (empty until
/// made). Ids last as long as the level: `forget` when it opens.
pub struct Live {
    pub dimension: String,
    pub marker: u32,
    pub displays: Vec<u32>,
}

pub static LIVE: std::sync::Mutex<Vec<Live>> = std::sync::Mutex::new(Vec::new());

/// The first corner a player clicked with the glue, waiting for the second: (dimension, block).
pub static PENDING: std::sync::Mutex<Option<(String, Pos)>> = std::sync::Mutex::new(None);

/// A contraption to assemble at the next tick (made by the debug command and the piston): from `anchor`, moving by
/// `by` at `speed` blocks a tick, glued by the glue markers and `extra`.
pub struct Request {
    pub dimension: String,
    pub anchor: Pos,
    pub by: Pos,
    pub speed: f64,
    pub extra: Option<Glue>,
    pub feedback: bool,
    /// A piston and the `EXTENDED` value it gets once the request is taken (a refused one leaves it as it was).
    pub owner: Option<(Pos, i32)>,
}

pub static REQUESTS: std::sync::Mutex<Vec<Request>> = std::sync::Mutex::new(Vec::new());

pub fn forget(dimension: &str) {
    LIVE.lock().unwrap().retain(|l| l.dimension != dimension);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(blocks: &[Pos], bad: &[Pos]) -> impl FnMut(Pos) -> Movable {
        let (blocks, bad) = (blocks.to_vec(), bad.to_vec());
        move |p| {
            if bad.contains(&p) {
                Movable::No
            } else if blocks.contains(&p) {
                Movable::Yes
            } else {
                Movable::Skip
            }
        }
    }

    #[test]
    fn collects_the_glued_blocks_that_touch() {
        let g = [Glue::new([0, 0, 0], [2, 0, 2])];
        let mut blocks: Vec<Pos> = (0..3).flat_map(|x| (0..3).map(move |z| [x, 0, z])).collect();
        blocks.push([5, 0, 0]); // in no box with the rest
        blocks.push([0, -1, 0]); // the ground under it, not glued
        let got = collect([0, 0, 0], &g, world(&blocks, &[]), LIMIT).unwrap();
        assert_eq!(got.len(), 9);
        assert_eq!(got[0], [0, 0, 0]);
        assert!(!got.contains(&[0, -1, 0]));
    }

    #[test]
    fn a_gap_splits_the_box() {
        let g = [Glue::new([0, 0, 0], [4, 0, 0])];
        let got = collect([0, 0, 0], &g, world(&[[0, 0, 0], [1, 0, 0], [3, 0, 0]], &[]), LIMIT).unwrap();
        assert_eq!(got, vec![[0, 0, 0], [1, 0, 0]]);
    }

    #[test]
    fn two_boxes_join_where_they_overlap() {
        let g = [Glue::new([0, 0, 0], [1, 0, 0]), Glue::new([1, 0, 0], [1, 2, 0])];
        let got = collect([0, 0, 0], &g, world(&[[0, 0, 0], [1, 0, 0], [1, 1, 0], [1, 2, 0]], &[]), LIMIT).unwrap();
        assert_eq!(got.len(), 4);
    }

    #[test]
    fn refuses_unmovable_too_many_and_empty() {
        let g = [Glue::new([0, 0, 0], [2, 0, 0])];
        let blocks = [[0, 0, 0], [1, 0, 0], [2, 0, 0]];
        assert_eq!(collect([0, 0, 0], &g, world(&blocks, &[[2, 0, 0]]), LIMIT), Err(Refusal::Unmovable([2, 0, 0])));
        assert_eq!(collect([0, 0, 0], &g, world(&blocks, &[]), 2), Err(Refusal::TooMany));
        assert_eq!(collect([0, 0, 0], &g, world(&blocks, &[]), 3).map(|v| v.len()), Ok(3));
        assert_eq!(collect([9, 9, 9], &g, world(&blocks, &[]), LIMIT), Err(Refusal::NothingThere));
    }

    #[test]
    fn what_moves() {
        assert_eq!(movable("minecraft:stone", 1.5, false, false), Movable::Yes);
        assert_eq!(movable("minecraft:air", 0.0, false, false), Movable::Skip);
        assert_eq!(movable("minecraft:water", 100.0, true, false), Movable::Skip);
        assert_eq!(movable("minecraft:bedrock", -1.0, false, false), Movable::No);
        assert_eq!(movable("minecraft:chest", 2.5, false, true), Movable::No);
    }

    #[test]
    fn steps_reach_the_target_exactly() {
        let mut o = [0.0; 3];
        let mut n = 0;
        while o != [5.0, 0.0, 0.0] {
            o = step(o, [5.0, 0.0, 0.0], SPEED);
            n += 1;
        }
        assert_eq!(n, 80);
        assert_eq!(landing([10, 64, 10], o, [1, 0, -1]), [16, 64, 9]);
        assert_eq!(landing([0, 0, 0], [2.4, -0.6, 0.0], [0, 0, 0]), [2, -1, 0]);
    }

    #[test]
    fn cells_a_block_overlaps() {
        assert_eq!(cells([1.0, 2.0, 3.0]), vec![[1, 2, 3]]);
        assert_eq!(cells([1.0625, 2.0, -0.5]), vec![[1, 2, -1], [1, 2, 0], [2, 2, -1], [2, 2, 0]]);
    }

    #[test]
    fn drills_dig_and_the_rest_stops() {
        let stone = InWay { solid: true, breakable: true };
        let grass = InWay { solid: false, breakable: true };
        let bedrock = InWay { solid: true, breakable: false };
        let world = |rock: InWay| move |c: Pos| if c[0] >= 2 { rock } else { InWay::default() };
        // A drill at x 0 and a block behind it, moving +x by 1/16 into stone at x 2: free until it touches.
        let at = |dx: f64| [([dx, 0.0, 0.0], true), ([dx - 1.0, 0.0, 0.0], false)];
        assert_eq!(way(&at(1.0), world(stone)), Way::default());
        assert_eq!(way(&at(1.0625), world(stone)), Way { blocked: false, drill: vec![(0, [2, 0, 0])] });
        assert_eq!(way(&at(1.0625), world(bedrock)), Way { blocked: true, drill: vec![] });
        // No drill: stone stops it, grass doesn't.
        let plain = [([1.0625, 0.0, 0.0], false)];
        assert!(way(&plain, world(stone)).blocked);
        assert_eq!(way(&plain, world(grass)), Way::default());
        // A drill takes grass too (Create breaks whatever it enters).
        assert_eq!(way(&at(1.0625), world(grass)).drill, vec![(0, [2, 0, 0])]);
    }

    #[test]
    fn moving_drills() {
        assert!(drills_toward([1, 0, 0], [SPEED, 0.0, 0.0]));
        assert!(!drills_toward([-1, 0, 0], [SPEED, 0.0, 0.0]));
        assert!(!drills_toward([0, 1, 0], [SPEED, 0.0, 0.0]));
        // 1/16 block a tick: Create's animation speed 100, break speed 0.2.
        assert_eq!(drill_rpm(SPEED), 20.0);
        assert_eq!(drill_rpm(0.0), 100.0 / 128.0);
    }

    #[test]
    fn piston_speeds() {
        assert_eq!(piston_speed(16.0), 1.0 / 32.0);
        assert_eq!(piston_speed(-256.0), -0.49);
        assert_eq!(piston_speed(0.0), 0.0);
    }

    #[test]
    fn coords_and_chunks() {
        assert_eq!(coord("12", 5), Some(12));
        assert_eq!(coord("~", 5), Some(5));
        assert_eq!(coord("~-3", 5), Some(2));
        assert_eq!(coord("x", 5), None);
        assert_eq!(chunk_of(-0.5, 16.0), (-1, 1));
    }
}
