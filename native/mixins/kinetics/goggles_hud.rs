// While goggles are on the head, the speed and stress lines (goggles.rs) of the kinetic block under the crosshair,
// below the creative motor's value box.
fn kinetics_goggles_hud(game: &mut crate::game::Game<'_>, layer: &str, batch: &mut HudBatch) {
    use crate::game::{kinetics as k, kinetics_goggles as g, kinetics_stress as st};
    use horizon_world::nbt::{JavaString, Tag};
    if layer != "minecraft:crosshair" || !game.native_window() {
        return;
    }
    if !game.armor[0].as_ref().is_some_and(|s| &*s.item == g::GOGGLES) {
        return;
    }
    let Some((distance, hit)) = game.native_view_hit() else { return };
    if distance > 5.0 {
        return;
    }
    let pos = [hit.block_x, hit.block_y, hit.block_z];
    let block = game.native_block(pos);
    let read = |key: &str| match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(be)) => match be.nbt.get(&JavaString::from(key)) {
            Some(Tag::Float(bits)) => Some(f32::from_bits(*bits)),
            _ => None,
        },
        _ => None,
    };
    let (speed, theoretical, overstressed) = if block == "kinetics:source" {
        (k::SOURCE_SPEED, k::SOURCE_SPEED, false)
    } else if k::has_block_entity(&block) || crate::game::kinetics_casing::has_block_entity(&block) {
        let speed = g::pick(read(k::SPEED_KEY).unwrap_or(0.0), read(crate::game::kinetics_clutch::SPEED_POS_KEY));
        let net = st::read(&read);
        (speed, net.map_or(speed, |n| n.theoretical_speed), net.is_some_and(|n| n.stress.overstressed()))
    } else {
        return;
    };
    let mut lines = vec![(g::label(speed), 0xFF_FFFF)];
    lines.extend(g::stress_lines(overstressed, st::impact(&block), st::capacity(&block), theoretical));
    for (i, (text, rgb)) in lines.iter().enumerate() {
        let width = game.native_text_width(text);
        let (x, y) = (batch.width / 2.0 - width / 2.0, batch.height / 2.0 + 32.0 + 15.0 * i as f32);
        game.native_fill(batch, [x - 4.0, y - 3.0, width + 8.0, 15.0], 0xB0_000000);
        game.native_text(batch, text, [x, y], *rgb, true);
    }
}
