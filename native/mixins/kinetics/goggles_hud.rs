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
    if block == crate::game::kinetics_speedometer::BLOCK {
        // Create's gauge information, in a tooltip box beside the crosshair (`GoggleOverlayRenderer`: offset 20, 20).
        let lines = crate::game::kinetics_speedometer::tooltip(speed);
        let width = lines.iter().map(|(t, _)| game.native_text_width(t)).fold(0.0, f32::max);
        let (x, y) = (batch.width / 2.0 + 20.0, batch.height / 2.0 + 20.0);
        let (w, h) = (width + 6.0, 10.0 * lines.len() as f32 + 6.0);
        game.native_fill(batch, [x - 3.0, y - 3.0, w, h], 0xF0_100010);
        for rect in [[x - 3.0, y - 3.0, w, 1.0], [x - 3.0, y - 3.0 + h - 1.0, w, 1.0], [x - 3.0, y - 2.0, 1.0, h - 2.0], [x - 3.0 + w - 1.0, y - 2.0, 1.0, h - 2.0]] {
            game.native_fill(batch, rect, 0x50_5000FF);
        }
        for (i, (text, rgb)) in lines.iter().enumerate() {
            game.native_text(batch, text, [x, y + 10.0 * i as f32], *rgb, true);
        }
        return;
    }
    let text = g::label(speed);
    let width = game.native_text_width(&text);
    let (x, y) = (batch.width / 2.0 - width / 2.0, batch.height / 2.0 + 32.0);
    game.native_fill(batch, [x - 4.0, y - 3.0, width + 8.0, 15.0], 0xB0_000000);
    game.native_text(batch, &text, [x, y], 0xFF_FFFF, true);
}
