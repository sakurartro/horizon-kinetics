// Kinetics: the wrench's choices, with no game types: this file is both a test target of the crate
// and the module the bundle adds to the game as `game::kinetics_wrench`.
//
// The turning rule is ported from Create's `content/equipment/wrench/IWrenchable.java`
// (`getRotatedBlockState` for a block with an `axis`: the axis turns a quarter about the clicked face;
// `WrenchItem.useOn`: sneaking picks the block up, otherwise it turns),
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

/// The wrench item.
pub const WRENCH: &str = "kinetics:wrench";

/// What a wrench click does to a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Set the block's `axis` property to this value.
    Rotate(&'static str),
    /// Pick the block up.
    Remove,
    /// Nothing: not a kinetic block, or already turned that way.
    Nothing,
}

/// The axis (0 = x, 1 = y, 2 = z) a face's normal lies along, none for a vector that isn't a face.
pub fn face_axis(face: [i32; 3]) -> Option<usize> {
    let axes: Vec<usize> = (0..3).filter(|&a| face[a] != 0).collect();
    match axes[..] {
        [a] if face[a].abs() == 1 => Some(a),
        _ => None,
    }
}

/// The axis after a quarter turn about `face_axis`: the other two swap, the clicked one stays.
pub fn rotated_axis(axis: usize, face_axis: usize) -> usize {
    if axis == face_axis { axis } else { 3 - axis - face_axis }
}

/// What the wrench does to a block: `kinetic` says it is one of ours, `axis` is its `axis` property
/// (0, 1 or 2), `face` the face clicked.
pub fn action(kinetic: bool, sneaking: bool, axis: Option<usize>, face: [i32; 3]) -> Action {
    if !kinetic {
        return Action::Nothing;
    }
    if sneaking {
        return Action::Remove;
    }
    let (Some(axis), Some(face)) = (axis, face_axis(face)) else {
        return Action::Nothing;
    };
    match rotated_axis(axis, face) {
        turned if turned == axis => Action::Nothing,
        0 => Action::Rotate("x"),
        1 => Action::Rotate("y"),
        _ => Action::Rotate("z"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces() {
        assert_eq!(face_axis([0, 1, 0]), Some(1));
        assert_eq!(face_axis([-1, 0, 0]), Some(0));
        assert_eq!(face_axis([0, 0, -1]), Some(2));
        assert_eq!(face_axis([0, 0, 0]), None);
        assert_eq!(face_axis([1, 1, 0]), None);
    }

    #[test]
    fn a_quarter_turn_swaps_the_other_two_axes() {
        // Vertical shaft, clicked on its side (x face): it lies along z; along z, clicked on x: back to y.
        assert_eq!(rotated_axis(1, 0), 2);
        assert_eq!(rotated_axis(2, 0), 1);
        assert_eq!(rotated_axis(0, 1), 2);
        assert_eq!(rotated_axis(0, 2), 1);
        // Clicked on its own end: no change.
        for a in 0..3 {
            assert_eq!(rotated_axis(a, a), a);
        }
        // Two quarter turns about one axis are a half turn: the axis comes back.
        for a in 0..3 {
            for f in 0..3 {
                assert_eq!(rotated_axis(rotated_axis(a, f), f), a);
            }
        }
    }

    #[test]
    fn click_decides() {
        // Shaft along y, clicked on the east face: now along z.
        assert_eq!(action(true, false, Some(1), [1, 0, 0]), Action::Rotate("z"));
        assert_eq!(action(true, false, Some(2), [0, 1, 0]), Action::Rotate("x"));
        assert_eq!(action(true, false, Some(0), [0, 0, -1]), Action::Rotate("y"));
        // Its own end: nothing to turn.
        assert_eq!(action(true, false, Some(1), [0, -1, 0]), Action::Nothing);
        // Sneaking picks it up, wherever it was clicked.
        assert_eq!(action(true, true, Some(1), [0, 1, 0]), Action::Remove);
        // Other blocks are left alone.
        assert_eq!(action(false, false, Some(1), [1, 0, 0]), Action::Nothing);
        assert_eq!(action(false, true, None, [1, 0, 0]), Action::Nothing);
        assert_eq!(action(true, false, None, [1, 0, 0]), Action::Nothing);
    }
}
