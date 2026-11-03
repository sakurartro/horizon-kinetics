// The placement helper's overlay (Create's `PlacementHelpers`): a shaft in hand aimed at a shaft, or a cogwheel in hand aimed
// at a cogwheel, shows the cell it will go in as a translucent box with an outline, and an arrow at the crosshair pointing
// there. Drawn in the HUD from the camera's own view (assist.rs `overlay`), so it has no depth test: it shows through
// blocks in front of it.
// shortcut: no partial-tick smoothing or view bobbing, and none in third person; a ghost made of flat shaded faces,
// not the model's texture. Add them if they matter.
fn kinetics_assist_hud(game: &mut crate::game::Game<'_>, layer: &str, batch: &mut HudBatch) {
    use crate::game::kinetics_assist as a;
    if layer != "minecraft:crosshair" || !game.native_window() {
        return;
    }
    let Some((item, _, _)) = game.native_held() else { return };
    let hand_cog = a::cog_kind(&item).filter(|_| item == "kinetics:cogwheel" || item == "kinetics:large_cogwheel");
    if item != a::SHAFT && hand_cog.is_none() {
        return;
    }
    let Some((distance, hit)) = game.native_view_hit() else { return };
    if distance > 6.0 {
        return;
    }
    let pos = [hit.block_x, hit.block_y, hit.block_z];
    let (lo, hi, target);
    if let Some(large) = hand_cog {
        let info = |p: [i32; 3]| {
            let name = game.native_block(p);
            a::Cell {
                cog: a::cog_kind(&name),
                axis: game.native_block_property(p, "axis").and_then(|x| a::axis_index(&x)),
                replaceable: crate::game::kinetics_water_wheel::replaceable(&name, game.native_block_property(p, "layers").as_deref()),
            }
        };
        let Some((cell, axis)) = a::cog_target(large, pos, [hit.x, hit.y, hit.z], info) else { return };
        (lo, hi) = a::cog_box(large, axis, cell);
        target = cell;
    } else {
        if hit.block != a::SHAFT {
            return;
        }
        let Some(axis) = game.native_block_property(pos, "axis") else { return };
        let Some(index) = a::axis_index(&axis) else { return };
        let is_pole = |p: [i32; 3]| {
            game.native_block(p) == a::SHAFT && game.native_block_property(p, "axis").as_deref() == Some(axis.as_str())
        };
        let replaceable = |p: [i32; 3]| {
            crate::game::kinetics_water_wheel::replaceable(&game.native_block(p), game.native_block_property(p, "layers").as_deref())
        };
        let Some(cell) = a::pole_target(index, pos, [hit.x, hit.y, hit.z], is_pole, replaceable) else { return };
        (lo, hi) = a::shaft_box(index, cell);
        target = cell;
    }
    let Some(eye) = game.eye_position() else { return };
    let fov = game.fov_degrees(game.clock.camera_fraction() as f32, eye);
    // ARGB to the batch's ABGR.
    let abgr = |argb: u32| (argb & 0xFF00_FF00) | (argb >> 16 & 0xFF) | (argb & 0xFF) << 16;
    for (quad, argb) in a::overlay(eye, game.yaw, game.pitch, fov, [batch.width, batch.height], lo, hi, pos, target) {
        batch.fill_quad(&game.gui.atlas, quad, abgr(argb));
    }
}
