// Solves the networks touched in `dimension` this tick and stores each part's speed.
fn kinetics_settle(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::kinetics as k;
    use horizon_world::nbt::{JavaString, Tag};
    let changed = k::take(dimension);
    if changed.is_empty() {
        return;
    }
    let speeds = k::solve(&changed, |p| {
        let name = game.native_block(p);
        if name == crate::game::kinetics_crank::BLOCK {
            // A hand crank: its facing, and the speed its turner gives it.
            let turn = crate::game::kinetics_crank::speed(dimension, p);
            return k::crank_node(&game.native_block_property(p, "facing")?, turn);
        }
        k::node(&name, game.native_block_property(p, "axis").as_deref())
    });
    let key = JavaString::from(k::SPEED_KEY);
    for (pos, speed) in speeds {
        let tag = Tag::Float(speed.to_bits());
        let moved = game.native_with_mod_block_entity(pos, |be| {
            if be.nbt.get(&key) == Some(&tag) {
                return false;
            }
            be.nbt.insert(key.clone(), tag.clone());
            true
        });
        if moved == Some(true) {
            game.native_block_entity_sync(pos);
        }
    }
}
