// Kinetics: the andesite casing. Which block a casing turns a shaft or cogwheel into, and the names of the
// models the casing blocks are drawn with, with no game types: this file is a module of the bundle added to
// the game as `game::kinetics_casing`, and a test target of the crate (`Cargo.toml`).
//
// The rules are ported from Create's `content/decoration/encasing/{EncasableBlock,EncasedBlock,CasingBlock}.java`
// and `content/kinetics/simpleRelays/encased/{EncasedShaftBlock,EncasedCogwheelBlock}.java`,
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

/// The casing block, and the item that encases with it.
pub const ANDESITE_CASING: &str = "kinetics:andesite_casing";

/// Whether encasing uses up the casing. Create's `tryEncase` does not shrink the stack, so neither do we; the
/// cased block drops the bare shaft or cogwheel, never the casing (their loot tables).
pub const CONSUMES_CASING: bool = false;

/// The block state properties of a cased cogwheel: whether a shaft pokes out of its end on the positive / negative
/// side of the axis.
pub const TOP_SHAFT: &str = "top_shaft";
pub const BOTTOM_SHAFT: &str = "bottom_shaft";

/// (bare block, casing item, cased block): `EncasingRegistry`'s variants.
const ENCASINGS: [(&str, &str, &str); 3] = [
    ("kinetics:shaft", ANDESITE_CASING, "kinetics:andesite_encased_shaft"),
    ("kinetics:cogwheel", ANDESITE_CASING, "kinetics:andesite_encased_cogwheel"),
    ("kinetics:large_cogwheel", ANDESITE_CASING, "kinetics:andesite_encased_large_cogwheel"),
];

/// The block that `block` turns into when `item` is used on it (`EncasableBlock.tryEncase`), none if it doesn't.
pub fn encased_variant(block: &str, item: &str) -> Option<&'static str> {
    ENCASINGS
        .iter()
        .find(|e| e.0 == block && e.1 == item)
        .map(|e| e.2)
}

pub fn is_casing(block: &str) -> bool {
    block == ANDESITE_CASING
}

/// A block cased with a casing: a shaft or cogwheel inside it.
pub fn is_encased(block: &str) -> bool {
    ENCASINGS.iter().any(|e| e.2 == block)
}

/// A cased cogwheel, small or large: the ones with `top_shaft` and `bottom_shaft`.
pub fn is_encased_cog(block: &str) -> bool {
    is_encased(block) && block.contains("cogwheel")
}

/// A block state property flag as the game spells it.
pub fn flag(on: bool) -> &'static str {
    if on { "true" } else { "false" }
}

/// Whether a block gets a block entity for the casing look and what turns inside it: the casing and every cased part.
pub fn has_block_entity(block: &str) -> bool {
    is_casing(block) || is_encased(block)
}

/// The turning part of a cased block, built along y: ours and Create's look, drawn instead if its pack is on
/// (`tools/create_look.py`). A cased shaft turns a whole shaft; a cased cogwheel the cogwheel with no shaft, whose
/// halves come out of the ends that have one (`SHAFT_HALF`).
pub fn spinner(block: &str) -> Option<(&'static str, &'static str)> {
    match block.strip_prefix("kinetics:")? {
        "andesite_encased_shaft" => Some(("kinetics:block/shaft_part", "create:block/shaft")),
        "andesite_encased_cogwheel" => Some((PART_MODELS[0], LOOK_MODELS[0])),
        "andesite_encased_large_cogwheel" => Some((PART_MODELS[1], LOOK_MODELS[1])),
        _ => None,
    }
}

/// The half of a shaft that comes out of a cased cogwheel's end, built along z from the centre to +z.
pub const HALF_SHAFT: (&str, &str) = (PART_MODELS[2], LOOK_MODELS[2]);

/// The standalone models the casing blocks draw, built along y: ours.
pub const PART_MODELS: [&str; 3] = [
    "kinetics:block/cogwheel_shaftless_part",
    "kinetics:block/large_cogwheel_shaftless_part",
    "kinetics:block/shaft_half_part",
];

/// Create's models for the same, as `LOOK_MODELS` in `runtime.rs`: they exist only while the look pack is on.
pub const LOOK_MODELS: [&str; 3] = [
    "create:block/cogwheel_shaftless",
    "create:block/large_cogwheel_shaftless",
    "create:block/shaft_half",
];

/// Create's static models of the casing blocks, drawn over our placeholders, the blockstate models, when the look
/// pack is on; all built along y.
pub const STATIC_LOOK_MODELS: [&str; 10] = [
    "create:block/andesite_casing",
    "create:block/encased_shaft/block_andesite",
    "create:block/andesite_encased_cogwheel",
    "create:block/andesite_encased_cogwheel_top",
    "create:block/andesite_encased_cogwheel_bottom",
    "create:block/andesite_encased_cogwheel_top_bottom",
    "create:block/andesite_encased_large_cogwheel",
    "create:block/andesite_encased_large_cogwheel_top",
    "create:block/andesite_encased_large_cogwheel_bottom",
    "create:block/andesite_encased_large_cogwheel_top_bottom",
];

/// Create's static model of `block`, with the shafts a cased cogwheel has at its ends.
pub fn static_look(block: &str, bottom: bool, top: bool) -> Option<&'static str> {
    let first = match block.strip_prefix("kinetics:")? {
        "andesite_casing" => return Some(STATIC_LOOK_MODELS[0]),
        "andesite_encased_shaft" => return Some(STATIC_LOOK_MODELS[1]),
        "andesite_encased_cogwheel" => 2,
        "andesite_encased_large_cogwheel" => 6,
        _ => return None,
    };
    // none, top, bottom, top and bottom
    Some(STATIC_LOOK_MODELS[first + usize::from(top) + 2 * usize::from(bottom)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_casing_turns_each_bare_part_into_its_cased_variant() {
        assert_eq!(
            encased_variant("kinetics:shaft", ANDESITE_CASING),
            Some("kinetics:andesite_encased_shaft")
        );
        assert_eq!(
            encased_variant("kinetics:cogwheel", ANDESITE_CASING),
            Some("kinetics:andesite_encased_cogwheel")
        );
        assert_eq!(
            encased_variant("kinetics:large_cogwheel", ANDESITE_CASING),
            Some("kinetics:andesite_encased_large_cogwheel")
        );
    }

    #[test]
    fn nothing_else_is_cased() {
        // Another item, a block that is cased already, the casing and the source.
        assert_eq!(encased_variant("kinetics:shaft", "minecraft:stone"), None);
        assert_eq!(
            encased_variant("kinetics:andesite_encased_shaft", ANDESITE_CASING),
            None
        );
        assert_eq!(encased_variant(ANDESITE_CASING, ANDESITE_CASING), None);
        assert_eq!(encased_variant("kinetics:source", ANDESITE_CASING), None);
    }

    #[test]
    fn block_kinds() {
        assert!(is_casing(ANDESITE_CASING));
        assert!(is_encased("kinetics:andesite_encased_large_cogwheel"));
        assert!(!is_encased("kinetics:cogwheel"));
        assert!(is_encased_cog("kinetics:andesite_encased_cogwheel"));
        assert!(is_encased_cog("kinetics:andesite_encased_large_cogwheel"));
        assert!(!is_encased_cog("kinetics:andesite_encased_shaft"));
        assert!(has_block_entity(ANDESITE_CASING));
        assert!(!has_block_entity("kinetics:shaft"));
    }

    #[test]
    fn every_cased_block_has_something_to_draw() {
        for (_, _, cased) in ENCASINGS {
            assert!(static_look(cased, false, false).is_some(), "{cased}");
            assert!(is_encased_cog(cased) == (spinner(cased).unwrap().0 != "kinetics:block/shaft_part"));
        }
        assert_eq!(static_look(ANDESITE_CASING, true, true), Some("create:block/andesite_casing"));
        assert_eq!(static_look("kinetics:shaft", false, false), None);
    }

    #[test]
    fn a_cased_cogwheels_look_follows_its_shafts() {
        let look = |b, bottom, top| static_look(b, bottom, top).unwrap();
        let cog = "kinetics:andesite_encased_cogwheel";
        assert_eq!(look(cog, false, false), "create:block/andesite_encased_cogwheel");
        assert_eq!(look(cog, false, true), "create:block/andesite_encased_cogwheel_top");
        assert_eq!(look(cog, true, false), "create:block/andesite_encased_cogwheel_bottom");
        assert_eq!(look(cog, true, true), "create:block/andesite_encased_cogwheel_top_bottom");
        let large = "kinetics:andesite_encased_large_cogwheel";
        assert_eq!(look(large, true, true), "create:block/andesite_encased_large_cogwheel_top_bottom");
        assert_eq!(look(large, false, false), "create:block/andesite_encased_large_cogwheel");
    }

    #[test]
    fn flags_are_spelled_as_the_game_does() {
        assert_eq!(flag(true), "true");
        assert_eq!(flag(false), "false");
    }
}
