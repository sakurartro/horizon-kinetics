// The placement helper's overlay (Create's `PlacementHelpers`): a shaft in hand, aimed at a shaft, shows the cell it will
// go in as a translucent shaft with an outline, and an arrow at the crosshair pointing there. Drawn in the HUD from the
// camera's own view (assist.rs `project`), so it has no depth test: it shows through blocks in front of it.
// shortcut: no partial-tick smoothing or view bobbing, and none in third person; a ghost made of flat shaded faces,
// not the model's texture. Add them if they matter.
fn kinetics_assist_hud(game: &mut crate::game::Game<'_>, layer: &str, batch: &mut HudBatch) {
    use crate::game::kinetics_assist as a;
    if layer != "minecraft:crosshair" || !game.native_window() {
        return;
    }
    if !game.native_held().is_some_and(|(item, _, _)| item == a::SHAFT) {
        return;
    }
    let Some((distance, hit)) = game.native_view_hit() else { return };
    if distance > 6.0 || hit.block != a::SHAFT {
        return;
    }
    let pos = [hit.block_x, hit.block_y, hit.block_z];
    let Some(axis) = game.native_block_property(pos, "axis") else { return };
    let Some(index) = a::axis_index(&axis) else { return };
    let is_pole = |p: [i32; 3]| {
        game.native_block(p) == a::SHAFT && game.native_block_property(p, "axis").as_deref() == Some(axis.as_str())
    };
    let replaceable = |p: [i32; 3]| {
        crate::game::kinetics_water_wheel::replaceable(&game.native_block(p), game.native_block_property(p, "layers").as_deref())
    };
    let Some(target) = a::pole_target(index, pos, [hit.x, hit.y, hit.z], is_pole, replaceable) else { return };
    let Some(eye) = game.eye_position() else { return };
    let fov = game.fov_degrees(game.clock.camera_fraction() as f32, eye);
    let size = [batch.width, batch.height];
    let project = |p: [f64; 3]| a::project(eye, game.yaw, game.pitch, fov, size, p);
    // ARGB to the batch's ABGR.
    let abgr = |argb: u32| (argb & 0xFF00_FF00) | (argb >> 16 & 0xFF) | (argb & 0xFF) << 16;
    let (lo, hi) = a::shaft_box(index, target);
    let corner = |i: usize| [if i & 1 == 0 { lo[0] } else { hi[0] }, if i & 2 == 0 { lo[1] } else { hi[1] }, if i & 4 == 0 { lo[2] } else { hi[2] }];
    let screen: Vec<Option<[f32; 2]>> = (0..8).map(|i| project(corner(i))).collect();
    // The faces turned to the camera, each with its corner indexes in order round it and its shade.
    let faces: [([usize; 4], usize, bool, f32); 6] = [
        ([0, 2, 6, 4], 0, false, 0.6), ([1, 3, 7, 5], 0, true, 0.6),
        ([0, 1, 5, 4], 1, false, 0.5), ([2, 3, 7, 6], 1, true, 1.0),
        ([0, 1, 3, 2], 2, false, 0.8), ([4, 5, 7, 6], 2, true, 0.8),
    ];
    for (ids, face_axis, high, shade) in faces {
        let toward = if high { eye[face_axis] > hi[face_axis] } else { eye[face_axis] < lo[face_axis] };
        let corners: Option<Vec<[f32; 2]>> = ids.iter().map(|i| screen[*i]).collect();
        let Some(c) = corners.filter(|_| toward) else { continue };
        let grey = (0xB8 as f32 * shade) as u32;
        batch.fill_quad(&game.gui.atlas, [c[0], c[1], c[2], c[3]], abgr(0x90_000000 | grey << 16 | grey << 8 | grey));
    }
    for (from, to) in (0..8usize).flat_map(|i| [(i, i | 1), (i, i | 2), (i, i | 4)]).filter(|(i, j)| i != j && i & (i ^ j) == 0) {
        if let (Some(p), Some(q)) = (screen[from], screen[to]) {
            let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
            let len = (dx * dx + dy * dy).sqrt().max(0.001);
            let n = [-dy / len * 0.5, dx / len * 0.5];
            batch.fill_quad(&game.gui.atlas, [[p[0] + n[0], p[1] + n[1]], [q[0] + n[0], q[1] + n[1]], [q[0] - n[0], q[1] - n[1]], [p[0] - n[0], p[1] - n[1]]], abgr(0xC0_101010));
        }
    }
    // The arrow at the crosshair, pointing from the aimed shaft to the cell.
    let centre = |c: [i32; 3]| project([f64::from(c[0]) + 0.5, f64::from(c[1]) + 0.5, f64::from(c[2]) + 0.5]);
    let (Some(from), Some(to)) = (centre(pos), centre(target)) else { return };
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return;
    }
    let (d, n) = ([dx / len, dy / len], [-dy / len, dx / len]);
    let (cx, cy) = (batch.width / 2.0 + d[0] * 14.0, batch.height / 2.0 + d[1] * 14.0);
    let at = |along: f32, across: f32| [cx + d[0] * along + n[0] * across, cy + d[1] * along + n[1] * across];
    for (grow, color) in [(1.0, 0xFF_101010_u32), (0.0, 0xFF_FFFFFF)] {
        batch.fill_quad(&game.gui.atlas, [at(-5.0 - grow, -1.5 - grow), at(1.0, -1.5 - grow), at(1.0, 1.5 + grow), at(-5.0 - grow, 1.5 + grow)], abgr(color));
        batch.fill_quad(&game.gui.atlas, [at(0.0, -4.0 - grow), at(5.0 + grow * 1.5, 0.0), at(5.0 + grow * 1.5, 0.0), at(0.0, 4.0 + grow)], abgr(color));
    }
}
