// While goggles are on the head, the speed of the kinetic block under the crosshair, in a line below the
// creative motor's value box.
fn kinetics_goggles_hud(game: &mut crate::game::Game<'_>, layer: &str, batch: &mut HudBatch) {
    use crate::game::{kinetics as k, kinetics_goggles as g};
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
    let speed = if block == "kinetics:source" {
        k::SOURCE_SPEED
    } else if k::has_block_entity(&block) || crate::game::kinetics_casing::has_block_entity(&block) {
        let read = |key: &str| match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
            Some(BlockEntity::Mod(be)) => match be.nbt.get(&JavaString::from(key)) {
                Some(Tag::Float(bits)) => Some(f32::from_bits(*bits)),
                _ => None,
            },
            _ => None,
        };
        g::pick(read(k::SPEED_KEY).unwrap_or(0.0), read(crate::game::kinetics_clutch::SPEED_POS_KEY))
    } else {
        return;
    };
    let text = g::label(speed);
    let width = game.native_text_width(&text);
    let (x, y) = (batch.width / 2.0 - width / 2.0, batch.height / 2.0 + 32.0);
    game.native_fill(batch, [x - 4.0, y - 3.0, width + 8.0, 15.0], 0xB0_000000);
    game.native_text(batch, &text, [x, y], 0xFF_FFFF, true);
}
