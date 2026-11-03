// Kinetics: Create's metal girder and the girder with a shaft through it, with no game types: this file is a module
// of the bundle added to the game as `game::kinetics_girder`, and a test target of the crate (`Cargo.toml`).
//
// The rules are ported from Create's `content/decoration/girder/{GirderBlock,GirderEncasedShaftBlock}.java`,
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

pub const GIRDER: &str = "kinetics:metal_girder";
pub const ENCASED: &str = "kinetics:metal_girder_encased_shaft";
/// The item that turns a girder into the encased shaft (`AllBlocks.SHAFT.isIn(stack)`).
pub const SHAFT_ITEM: &str = "kinetics:shaft";

/// Create's girder has the boolean states `x` and `z` (a beam along that axis); a content block's shapes are chosen
/// by one property, so here they are one `beam` property: none, x, z or xz.
pub fn beam_name(x: bool, z: bool) -> &'static str {
    match (x, z) {
        (false, false) => "none",
        (true, false) => "x",
        (false, true) => "z",
        (true, true) => "xz",
    }
}

pub fn beam_flags(name: &str) -> (bool, bool) {
    (name == "x" || name == "xz", name == "z" || name == "xz")
}

pub fn is_girder(block: &str) -> bool {
    block == GIRDER || block == ENCASED
}

/// The six directions as the game's offsets, index / 2 being the axis (x, y, z): -x, +x, -y, +y, -z, +z.
pub const DIRS: [[i32; 3]; 6] = [[-1, 0, 0], [1, 0, 0], [0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1]];

/// What stands beside a girder, as far as its connections care.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Neighbour {
    Other,
    /// A block with something to hold on to (`isFaceFull` and solid in Create).
    Solid,
    Girder { x: bool, z: bool },
    /// A girder with a shaft through it, along `axis` (0 = x, 2 = z).
    Shafted { axis: usize },
    /// A wall post, a lantern, a vertical chain: they attach to the vertical ends.
    WallUp,
    ChainY,
    Lantern { hanging: bool },
}

/// `name` and its block state properties as a `Neighbour`.
/// shortcut: "solid" is any block that isn't air or a fluid (no collision shapes are readable here), so a plant
/// keeps a connection a bare-face check would drop; read the shape when the API gives one.
pub fn classify(name: &str, prop: impl Fn(&str) -> Option<String>) -> Neighbour {
    let on = |k: &str| prop(k).as_deref() == Some("true");
    match name {
        GIRDER => {
            let (x, z) = beam_flags(prop("beam").as_deref().unwrap_or("none"));
            Neighbour::Girder { x, z }
        }
        ENCASED => Neighbour::Shafted { axis: if prop("axis").as_deref() == Some("x") { 0 } else { 2 } },
        "minecraft:iron_chain" | "minecraft:chain" if prop("axis").as_deref() == Some("y") => Neighbour::ChainY,
        n if n.ends_with("_wall") && on("up") => Neighbour::WallUp,
        n if n.ends_with("lantern") => Neighbour::Lantern { hanging: on("hanging") },
        n if n.ends_with("air") || n.ends_with("water") || n.ends_with("lava") => Neighbour::Other,
        _ => Neighbour::Solid,
    }
}

/// A girder's connection states.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GirderState {
    pub axis: usize,
    pub x: bool,
    pub z: bool,
    pub top: bool,
    pub bottom: bool,
}

/// A girder placed against a face on `face_axis` (`getStateForPlacement`): its beam runs along that axis if it is
/// horizontal, a post if it is vertical. The sides are filled in by `update`.
pub fn placed(face_axis: usize) -> GirderState {
    GirderState { axis: face_axis, x: face_axis == 0, z: face_axis == 2, top: false, bottom: false }
}

/// Whether a vertical end at `d` (2 down, 3 up) is held (`updateVerticalProperty`); `own_post` is a girder standing on y.
fn attaches(own_post: bool, side: Neighbour, d: usize) -> bool {
    own_post
        || match side {
            Neighbour::Girder { .. } | Neighbour::Shafted { .. } | Neighbour::WallUp | Neighbour::ChainY => true,
            Neighbour::Lantern { hanging } => (d == 2) == hanging,
            _ => false,
        }
}

/// The state a girder settles into given what is around it, `GirderBlock.updateShape` for every side:
/// a beam across the girder's axis is dropped when nothing solid is at either end, a side joins when the girder
/// runs that way or a girder or shafted girder is lined up with it, a vertical end joins when `attaches`.
pub fn update(mut g: GirderState, side: impl Fn(usize) -> Neighbour) -> GirderState {
    for a in [0usize, 2] {
        if g.axis != a && side(2 * a) != Neighbour::Solid && side(2 * a + 1) != Neighbour::Solid {
            if a == 0 {
                g.x = false;
            } else {
                g.z = false;
            }
        }
    }
    if g.axis != 1 {
        g.bottom &= side(2) == Neighbour::Solid;
        g.top &= side(3) == Neighbour::Solid;
    }
    for d in 0..6 {
        let a = d / 2;
        if a == 1 {
            if attaches(g.axis == 1, side(d), d) {
                if d == 2 {
                    g.bottom = true;
                } else {
                    g.top = true;
                }
            }
            continue;
        }
        let join = g.axis == a
            || match side(d) {
                Neighbour::Shafted { axis } => axis != a,
                Neighbour::Girder { x, z } => {
                    if a == 0 {
                        x
                    } else {
                        z
                    }
                }
                _ => false,
            };
        if join {
            if a == 0 {
                g.x = true;
            } else {
                g.z = true;
            }
        }
    }
    g
}

/// The ends of a girder with a shaft through it (`GirderEncasedShaftBlock.updateShape`): only the vertical ones.
pub fn update_shafted(top: bool, bottom: bool, side: impl Fn(usize) -> Neighbour) -> (bool, bool) {
    let end = |on: bool, d: usize| (on && side(d) == Neighbour::Solid) || attaches(false, side(d), d);
    (end(top, 3), end(bottom, 2))
}

/// The axis (0 = x, 2 = z) of the shaft that a shaft item puts through a girder (`useItemOn`): along z for a girder
/// that has an x beam or when a z face was clicked, else along x.
pub fn shaft_axis(x_beam: bool, clicked_axis: usize) -> usize {
    if x_beam || clicked_axis == 2 {
        2
    } else {
        0
    }
}

// Girders next to a changed block (`native_block_changed`), settled at the end of their level's tick. Both run on
// the server thread.
static DIRTY: std::sync::Mutex<Vec<(&'static str, [i32; 3])>> = std::sync::Mutex::new(Vec::new());

pub fn mark(dimension: &'static str, pos: [i32; 3]) {
    DIRTY.lock().unwrap_or_else(|e| e.into_inner()).push((dimension, pos));
}

/// The positions marked in `dimension` since the last call, each once.
pub fn take(dimension: &str) -> Vec<[i32; 3]> {
    let mut dirty = DIRTY.lock().unwrap_or_else(|e| e.into_inner());
    let mut out: Vec<[i32; 3]> = dirty.iter().filter(|d| d.0 == dimension).map(|d| d.1).collect();
    dirty.retain(|d| d.0 != dimension);
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use Neighbour::*;

    fn around(list: &[(usize, Neighbour)]) -> impl Fn(usize) -> Neighbour + '_ {
        move |d| list.iter().find(|e| e.0 == d).map_or(Other, |e| e.1)
    }

    #[test]
    fn a_girder_runs_along_the_face_it_was_placed_on() {
        assert_eq!(update(placed(0), around(&[])), GirderState { axis: 0, x: true, z: false, top: false, bottom: false });
        assert!(update(placed(2), around(&[])).z);
        // A post holds on at both ends.
        let post = update(placed(1), around(&[]));
        assert!(post.top && post.bottom && !post.x && !post.z);
    }

    #[test]
    fn a_beam_joins_a_girder_lined_up_with_it_and_a_shaft_across() {
        let g = GirderState { axis: 0, x: true, z: false, top: false, bottom: false };
        assert!(update(g, around(&[(4, Neighbour::Girder { x: false, z: true })])).z);
        assert!(update(g, around(&[(5, Shafted { axis: 0 })])).z);
        assert!(!update(g, around(&[(5, Shafted { axis: 2 })])).z);
        assert!(!update(g, around(&[(4, Neighbour::Girder { x: true, z: false })])).z);
    }

    #[test]
    fn a_cross_beam_is_dropped_when_nothing_solid_is_at_its_ends() {
        let cross = GirderState { axis: 0, x: true, z: true, top: false, bottom: false };
        assert!(!update(cross, around(&[])).z);
        assert!(update(cross, around(&[(4, Solid)])).z);
        assert!(update(cross, around(&[(5, Solid)])).z);
        // The girder's own axis is never dropped.
        assert!(update(cross, around(&[])).x);
    }

    #[test]
    fn vertical_ends_need_support_or_an_attachment() {
        let g = GirderState { axis: 0, x: true, z: false, top: true, bottom: true };
        let r = update(g, around(&[]));
        assert!(!r.top && !r.bottom);
        let r = update(g, around(&[(3, Solid), (2, WallUp)]));
        assert!(r.top && r.bottom);
        let bare = GirderState { top: false, bottom: false, ..g };
        let r = update(bare, around(&[(3, Solid), (2, Lantern { hanging: true })]));
        assert!(!r.top, "a solid block doesn't make a new end");
        assert!(r.bottom, "a hanging lantern below attaches");
        assert!(!update(bare, around(&[(2, Lantern { hanging: false })])).bottom);
        assert!(update(bare, around(&[(3, ChainY)])).top);
    }

    #[test]
    fn a_shafted_girder_keeps_its_vertical_ends_the_same_way() {
        assert_eq!(update_shafted(true, true, around(&[])), (false, false));
        assert_eq!(update_shafted(true, false, around(&[(3, Solid), (2, Solid)])), (true, false));
        assert_eq!(update_shafted(false, false, around(&[(3, Neighbour::Girder { x: false, z: false })])), (true, false));
    }

    #[test]
    fn a_shaft_item_goes_through_across_the_beam() {
        assert_eq!(shaft_axis(true, 0), 2);
        assert_eq!(shaft_axis(false, 2), 2);
        assert_eq!(shaft_axis(false, 0), 0);
        assert_eq!(shaft_axis(false, 1), 0);
    }

    #[test]
    fn neighbours_are_read_from_block_names_and_properties() {
        let none = |_: &str| None;
        let with = |key: &'static str, value: &'static str| move |k: &str| (k == key).then(|| value.to_string());
        assert_eq!(classify("minecraft:stone", none), Solid);
        assert_eq!(classify("minecraft:air", none), Other);
        assert_eq!(classify("minecraft:water", none), Other);
        assert_eq!(classify(GIRDER, with("beam", "xz")), Neighbour::Girder { x: true, z: true });
        assert_eq!(classify(ENCASED, with("axis", "x")), Shafted { axis: 0 });
        assert_eq!(classify("minecraft:cobblestone_wall", with("up", "true")), WallUp);
        assert_eq!(classify("minecraft:lantern", with("hanging", "true")), Lantern { hanging: true });
        assert_eq!(classify("minecraft:iron_chain", with("axis", "y")), ChainY);
        assert_eq!(classify("minecraft:iron_chain", with("axis", "x")), Solid);
    }

    #[test]
    fn beam_names_round_trip() {
        for (x, z) in [(false, false), (true, false), (false, true), (true, true)] {
            assert_eq!(beam_flags(beam_name(x, z)), (x, z));
        }
    }

    #[test]
    fn marks_are_taken_once_per_dimension() {
        mark("test:a", [1, 2, 3]);
        mark("test:a", [1, 2, 3]);
        mark("test:b", [0, 0, 0]);
        assert_eq!(take("test:a"), vec![[1, 2, 3]]);
        assert!(take("test:a").is_empty());
        assert_eq!(take("test:b"), vec![[0, 0, 0]]);
    }
}
