// Kinetics: the gearbox and its vertical item. Which axis a gearbox is placed with, which half shafts it
// draws and how they are laid, with no game types: this file is a module of the bundle added to the game
// as `game::kinetics_gearbox`, and is included by the crate's test `tests/gearbox.rs`. How the gearbox
// passes speed on (`has_shaft_toward`, `gearbox_sign`) is in runtime.rs.
//
// The rules are ported from Create's `content/kinetics/gearbox/{GearboxBlock,VerticalGearboxItem,GearboxRenderer}.java`,
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
// Create's `vertical_gearbox` is an item that places the gearbox block lying on its side. Horizon's items
// place their own block, so here it is a block of its own (`kinetics:vertical_gearbox`) that turns into the
// gearbox, on its side, at the end of the tick it is placed in.

pub const GEARBOX: &str = "kinetics:gearbox";
pub const VERTICAL: &str = "kinetics:vertical_gearbox";

/// The half shaft the block entity draws, built along +z (from the middle of the block to its face).
pub const PART_MODELS: [&str; 1] = ["kinetics:block/gearbox_shaft_half"];
/// Create's own, drawn first while the pack from `tools/create_look.py` is on.
pub const LOOK_MODELS: [&str; 1] = ["create:block/shaft_half"];
/// Create's box, built along y, drawn over our placeholder (the blockstate model) while the pack is on.
pub const STATIC_LOOK: &str = "create:block/gearbox/block";

/// The axis (0 = x, 1 = y, 2 = z) a vertical gearbox is first placed with: the one across the way the player
/// faces, `player.getDirection().getClockWise().getAxis()`. `yaw` is the placer's, in degrees (0 faces +z).
pub fn placed_axis(yaw: f32) -> usize {
    let facing = ((yaw / 90.0 + 0.5).floor() as i32) & 3; // 0 south, 1 west, 2 north, 3 east
    if facing % 2 == 0 { 0 } else { 2 }
}

/// The axis the vertical gearbox ends up with (`VerticalGearboxItem.updateCustomBlockEntityTag`). `shafts` says,
/// for the neighbours +x, -x, +z, -z, whether one has a shaft toward the gearbox. All of them on one axis: the
/// gearbox takes the other horizontal axis, so its shafts meet them. None, or both axes: `first`, the axis
/// `placed_axis` gave.
pub fn vertical_axis(shafts: [bool; 4], first: usize) -> usize {
    let along_x = shafts[0] || shafts[1];
    let along_z = shafts[2] || shafts[3];
    match (along_x, along_z) {
        (true, false) => 2,
        (false, true) => 0,
        _ => first,
    }
}

/// The faces with a half shaft of a gearbox standing on `box_axis`, as (axis, positive side).
pub fn faces(box_axis: usize) -> impl Iterator<Item = (usize, bool)> {
    (0..3)
        .filter(move |&a| a != box_axis)
        .flat_map(|a| [(a, true), (a, false)])
}

/// The turn (axis, degrees) that lays a half shaft built along +z out of the face on `axis`.
pub fn lay_face(axis: usize, positive: bool) -> Option<([f32; 3], f32)> {
    match (axis, positive) {
        (2, true) => None,
        (2, false) => Some(([0.0, 1.0, 0.0], 180.0)),
        (0, true) => Some(([0.0, 1.0, 0.0], 90.0)),
        (0, false) => Some(([0.0, 1.0, 0.0], -90.0)),
        (1, true) => Some(([1.0, 0.0, 0.0], -90.0)),
        _ => Some(([1.0, 0.0, 0.0], 90.0)),
    }
}

// Vertical gearboxes placed (`native_block_changed`), turned into gearboxes at the end of their level's tick.
static PENDING: std::sync::Mutex<Vec<(&'static str, [i32; 3])>> = std::sync::Mutex::new(Vec::new());

pub fn mark(dimension: &'static str, pos: [i32; 3]) {
    PENDING
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((dimension, pos));
}

/// The positions marked in `dimension` since the last call.
pub fn take(dimension: &str) -> Vec<[i32; 3]> {
    let mut all = PENDING.lock().unwrap_or_else(|e| e.into_inner());
    let mut out = Vec::new();
    all.retain(|&(d, p)| {
        if d == dimension {
            out.push(p);
            false
        } else {
            true
        }
    });
    out
}
