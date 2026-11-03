// Kinetics: the wooden and metal brackets. Which blocks take one, how it sits around them and which models draw it,
// with no game types: this file is a module of the bundle added to the game as `game::kinetics_bracket`, and a test
// target of the crate (`Cargo.toml`).
//
// The rules are ported from Create's `content/decoration/bracket/{BracketBlock,BracketBlockItem,BracketGenerator}.java`
// and `foundation/data/DirectionalAxisBlockStateGen.java`,
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

/// The block entity key of a bracketed shaft or cogwheel: 0 / absent none, else `1 + material * 6 + facing`.
pub const BRACKET_KEY: &str = "bracket";

/// Vanilla's `Direction.values()` order: the `facing` index used everywhere here.
pub const DIRS: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

/// The item of each material (Create's `wooden_bracket`, `metal_bracket`), and the part of the model names.
pub const ITEMS: [&str; 2] = ["kinetics:wooden_bracket", "kinetics:metal_bracket"];
const MATERIALS: [&str; 2] = ["wooden", "metal"];

/// `BracketType`: a cogwheel's bracket is wider than a shaft's (a large cogwheel takes the cog one too).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Cog,
    Shaft,
}

/// The bracket material an item is, if it is one.
pub fn material_of_item(item: &str) -> Option<usize> {
    ITEMS.iter().position(|i| *i == item)
}

/// What a block takes a bracket as: only the bare shaft and cogwheels (Create's `AbstractSimpleShaftBlock`;
/// the encased ones don't).
pub fn type_of(block: &str) -> Option<Type> {
    match block {
        "kinetics:shaft" => Some(Type::Shaft),
        "kinetics:cogwheel" | "kinetics:large_cogwheel" => Some(Type::Cog),
        _ => None,
    }
}

fn axis_of_dir(d: usize) -> usize {
    (0..3).find(|&a| DIRS[d][a] != 0).unwrap_or(1)
}

/// `AXIS_ALONG_FIRST_COORDINATE` of the bracket on a block turning on `axis`, set from face `facing`
/// (`getSuitableBracket`), none when the face is on the axis: no bracket fits there.
pub fn along_first(axis: usize, facing: usize) -> Option<bool> {
    let face_axis = axis_of_dir(facing);
    if face_axis == axis {
        return None;
    }
    // Create: `alongFirst = faceAxis != Z ? axis == Z : axis == Y`, and the state stores its negation.
    let along = if face_axis != 2 { axis == 2 } else { axis == 1 };
    Some(!along)
}

/// The facing a click on face `face` sets (the face's own direction) if a bracket fits there.
pub fn facing_for_click(axis: usize, face: [i32; 3]) -> Option<usize> {
    let facing = DIRS.iter().position(|d| *d == face)?;
    along_first(axis, facing).map(|_| facing)
}

/// The value saved in the block entity.
pub fn encode(material: usize, facing: usize) -> i32 {
    1 + (material * 6 + facing) as i32
}

/// (material, facing) of a saved value, none for no bracket or a corrupt one.
pub fn decode(value: i32) -> Option<(usize, usize)> {
    let v = usize::try_from(value.checked_sub(1)?).ok()?;
    (v < 12).then_some((v / 6, v % 6))
}

/// The blockstate rotations Create's generator gives the bracket model: (x, y) in degrees, as the game's variants
/// spell them (x turns the model first, then y).
pub fn rotations(facing: usize, along_first: bool) -> (i32, i32) {
    let d = DIRS[facing];
    let vertical = d[1] != 0;
    let x = if facing == 0 {
        180
    } else if facing == 1 {
        0
    } else if (d[0] != 0) == along_first {
        90
    } else {
        0
    };
    // `toYRot`: south 0, west 90, north 180, east 270.
    let to_y_rot = match facing {
        3 => 0,
        4 => 90,
        2 => 180,
        _ => 270,
    };
    let y = if vertical { if along_first { 90 } else { 0 } } else { (to_y_rot + 90) % 360 };
    (x, y)
}

/// Create's model of the bracket (it exists only while the look pack is on) and ours, for the model's
/// vertical (`ground`) or horizontal (`wall`) facing.
pub fn models(material: usize, ty: Type, facing: usize) -> (String, String) {
    let kind = if DIRS[facing][1] != 0 { "ground" } else { "wall" };
    let m = MATERIALS[material];
    let t = if ty == Type::Cog { "cog" } else { "shaft" };
    (format!("create:block/bracket/{t}/{kind}_{m}"), format!("kinetics:block/bracket_{kind}_{m}"))
}

/// Every model name the bracket draws, for the bundle to register: Create's and ours.
pub fn all_models() -> Vec<String> {
    let mut out = Vec::new();
    for material in 0..2 {
        for facing in [0, 2] {
            for ty in [Type::Cog, Type::Shaft] {
                let (look, ours) = models(material, ty, facing);
                out.push(look);
                if ty == Type::Cog {
                    out.push(ours);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_bare_shafts_and_cogwheels_take_brackets() {
        assert_eq!(type_of("kinetics:shaft"), Some(Type::Shaft));
        assert_eq!(type_of("kinetics:large_cogwheel"), Some(Type::Cog));
        assert_eq!(type_of("kinetics:andesite_encased_shaft"), None);
        assert_eq!(type_of("kinetics:gearbox"), None);
    }

    #[test]
    fn no_bracket_on_a_face_along_the_axis() {
        assert_eq!(facing_for_click(1, [0, 1, 0]), None);
        assert_eq!(facing_for_click(0, [1, 0, 0]), None);
        assert_eq!(facing_for_click(2, [0, 0, -1]), None);
        assert_eq!(facing_for_click(1, [1, 0, 0]), Some(5));
        assert_eq!(facing_for_click(0, [0, 1, 0]), Some(1));
        assert_eq!(facing_for_click(1, [0, 0, 0]), None);
    }

    #[test]
    fn along_first_follows_create() {
        // Face on x/y, shaft on z: alongFirst true, stored false.
        assert_eq!(along_first(2, 5), Some(false));
        assert_eq!(along_first(2, 1), Some(false));
        // Face on x/y, shaft on the other of x/y: alongFirst false, stored true.
        assert_eq!(along_first(1, 5), Some(true));
        assert_eq!(along_first(0, 1), Some(true));
        // Face on z: alongFirst when the shaft is on y.
        assert_eq!(along_first(1, 3), Some(false));
        assert_eq!(along_first(0, 3), Some(true));
    }

    #[test]
    fn rotations_match_the_generator() {
        // Up on a shaft along x (along_first stored true): y 90, x 0.
        assert_eq!(rotations(1, true), (0, 90));
        assert_eq!(rotations(1, false), (0, 0));
        assert_eq!(rotations(0, false), (180, 0));
        // A wall tips onto its side (x 90) when "facing on x" equals along_first.
        assert_eq!(rotations(3, false), (90, 90));
        assert_eq!(rotations(3, true), (0, 90));
        assert_eq!(rotations(5, true), (90, 0)); // east: toYRot 270 + 90
        assert_eq!(rotations(4, false), (0, 180)); // west: 90 + 90
        assert_eq!(rotations(2, true), (0, 270)); // north: 180 + 90
    }

    #[test]
    fn saved_values_round_trip() {
        for material in 0..2 {
            for facing in 0..6 {
                assert_eq!(decode(encode(material, facing)), Some((material, facing)));
            }
        }
        assert_eq!(decode(0), None);
        assert_eq!(decode(13), None);
        assert_eq!(decode(-5), None);
    }

    #[test]
    fn models_are_named_by_facing_kind() {
        assert_eq!(models(0, Type::Cog, 1).0, "create:block/bracket/cog/ground_wooden");
        assert_eq!(models(1, Type::Shaft, 5).0, "create:block/bracket/shaft/wall_metal");
        assert_eq!(models(1, Type::Shaft, 5).1, "kinetics:block/bracket_wall_metal");
        assert!(all_models().contains(&"create:block/bracket/shaft/ground_metal".to_string()));
    }
}
