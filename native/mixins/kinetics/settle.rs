// Solves the networks touched in `dimension` this tick and stores each part's speed.
fn kinetics_settle(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::kinetics as k;
    use crate::game::kinetics_clutch as c;
    use horizon_world::nbt::{JavaString, Tag};
    let mut changed = k::take(dimension);
    // A clutch or gearshift that was placed, or whose neighbour changed, follows its redstone signal.
    // Like Create's `detachKinetics`, one that flips cuts its network for this tick; the flip marks it
    // again (block changed), so the network joins again, in its new state, at the next tick's end.
    let mut detached: Vec<[i32; 3]> = Vec::new();
    let mut checks = c::take(dimension);
    checks.extend(changed.iter().copied());
    for pos in checks {
        let name = game.native_block(pos);
        if !c::is_split(&name) || detached.contains(&pos) {
            continue;
        }
        let powered = game.native_block_property(pos, "powered").as_deref() == Some("true");
        let Some(flip) = c::flip(&name, powered, game.native_neighbor_signal(dimension, pos)) else {
            continue;
        };
        // Flag 2 is UPDATE_CLIENTS: nothing around it needs to hear.
        let value = if flip.powered { "true" } else { "false" };
        if game.native_set_block_property_with(pos, "powered", value, 2) {
            changed.push(pos);
            if flip.detach {
                detached.push(pos);
            }
        }
    }
    if changed.is_empty() {
        return;
    }
    let mut speeds = k::solve_halves(&changed, |p| {
        if detached.contains(&p) {
            return None;
        }
        k::node_split(
            &game.native_block(p),
            game.native_block_property(p, "axis").as_deref(),
            game.native_block_property(p, "powered").as_deref(),
        )
    });
    for &p in &detached {
        speeds.insert((p, 0), 0.0);
        speeds.insert((p, 1), 0.0);
    }
    let mut moved: Vec<[i32; 3]> = Vec::new();
    for ((pos, half), speed) in speeds {
        let key = JavaString::from(if half == 0 { k::SPEED_KEY } else { c::SPEED_POS_KEY });
        let tag = Tag::Float(speed.to_bits());
        let wrote = game.native_with_mod_block_entity(pos, |be| {
            if be.nbt.get(&key) == Some(&tag) {
                return false;
            }
            be.nbt.insert(key.clone(), tag.clone());
            true
        });
        if wrote == Some(true) && !moved.contains(&pos) {
            moved.push(pos);
        }
    }
    for pos in moved {
        game.native_block_entity_sync(pos);
    }
}
