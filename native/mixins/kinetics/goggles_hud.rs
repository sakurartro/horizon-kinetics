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
    if block == crate::game::kinetics_stressometer::BLOCK {
        // Create's gauge information in a tooltip box beside the crosshair (`GoggleOverlayRenderer`: offset 20, 20),
        // each line a row of coloured pieces.
        use crate::game::kinetics_stressometer as sm;
        let net = st::read(&read);
        let (capacity, stress) = net.map_or((0.0, 0.0), |n| (n.stress.capacity, n.stress.stress));
        let level = st::Level::of(stress / if capacity == 0.0 { 1.0 } else { capacity }) as usize;
        let Some(lines) = sm::tooltip(capacity, stress, net.map_or(0.0, |n| n.theoretical_speed), level, st::DISABLE_STRESS) else { return };
        let width = lines.iter().map(|l| l.iter().map(|(t, _)| game.native_text_width(t)).sum::<f32>()).fold(0.0, f32::max);
        let (x, y) = (batch.width / 2.0 + 20.0, batch.height / 2.0 + 20.0);
        let (w, h) = (width + 6.0, 10.0 * lines.len() as f32 + 6.0);
        game.native_fill(batch, [x - 3.0, y - 3.0, w, h], 0xF0_100010);
        for rect in [[x - 3.0, y - 3.0, w, 1.0], [x - 3.0, y - 3.0 + h - 1.0, w, 1.0], [x - 3.0, y - 2.0, 1.0, h - 2.0], [x - 3.0 + w - 1.0, y - 2.0, 1.0, h - 2.0]] {
            game.native_fill(batch, rect, 0x50_5000FF);
        }
        for (i, line) in lines.iter().enumerate() {
            let mut at = x;
            for (text, rgb) in line {
                game.native_text(batch, text, [at, y + 10.0 * i as f32], *rgb, true);
                at += game.native_text_width(text);
            }
        }
        return;
    }
    let mut lines = vec![(g::label(speed), 0xFF_FFFF)];
    lines.extend(g::stress_lines(overstressed, st::impact(&block), st::capacity(&block), theoretical));
    for (i, (text, rgb)) in lines.iter().enumerate() {
        let width = game.native_text_width(text);
        let (x, y) = (batch.width / 2.0 - width / 2.0, batch.height / 2.0 + 32.0 + 15.0 * i as f32);
        game.native_fill(batch, [x - 4.0, y - 3.0, width + 8.0, 15.0], 0xB0_000000);
        game.native_text(batch, text, [x, y], *rgb, true);
    }
}
