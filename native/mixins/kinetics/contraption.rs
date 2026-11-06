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
    fn coords_and_chunks() {
        assert_eq!(coord("12", 5), Some(12));
        assert_eq!(coord("~", 5), Some(5));
        assert_eq!(coord("~-3", 5), Some(2));
        assert_eq!(coord("x", 5), None);
        assert_eq!(chunk_of(-0.5, 16.0), (-1, 1));
    }
}
