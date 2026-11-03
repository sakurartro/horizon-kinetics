// The creative motor's value box: its speed and way under the crosshair while a side of it is aimed at.
fn kinetics_motor_hud(game: &mut crate::game::Game<'_>, layer: &str, batch: &mut HudBatch) {
    use crate::game::kinetics_motor as m;
    use horizon_world::nbt::{JavaString, Tag};
    if layer != "minecraft:crosshair" || !game.native_window() {
        return;
    }
    let Some((distance, hit)) = game.native_view_hit() else { return };
    if hit.block != m::BLOCK || distance > 5.0 {
        return;
    }
    let pos = [hit.block_x, hit.block_y, hit.block_z];
    let local = [hit.x - f64::from(pos[0]), hit.y - f64::from(pos[1]), hit.z - f64::from(pos[2])];
    let Some(side) = m::hit_side(local) else { return };
    let Some(facing) = game.native_block_property(pos, "facing").as_deref().and_then(m::dir_vec) else {
        return;
    };
    if !m::box_side(facing, side) {
        return;
    }
    let rpm = match game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(be)) => match be.nbt.get(&JavaString::from(crate::game::kinetics::RPM_KEY)) {
            Some(Tag::Int(v)) => *v,
            _ => m::DEFAULT_RPM,
        },
        _ => m::DEFAULT_RPM,
    };
    let text = m::label(rpm);
    let width = game.native_text_width(&text);
    let (x, y) = (batch.width / 2.0 - width / 2.0, batch.height / 2.0 + 14.0);
    game.native_fill(batch, [x - 4.0, y - 3.0, width + 8.0, 15.0], 0xB0_000000);
    game.native_text(batch, &text, [x, y], 0xFF_FFFF, true);
}
