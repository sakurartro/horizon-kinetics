// Solves the networks touched in `dimension` this tick and stores each part's speed.
fn kinetics_settle(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::kinetics as k;
    use horizon_world::nbt::{JavaString, Tag};
    let changed = k::take(dimension);
    if changed.is_empty() {
        return;
    }
    // The lookup reads the world, and a block entity can only be reached mutably: share the game through a cell.
    let cell = std::cell::RefCell::new(&mut *game);
    let speeds = k::solve(&changed, |p| {
        let name = cell.borrow().native_block(p);
        if name != crate::game::kinetics_motor::BLOCK {
            let g = cell.borrow();
            return k::node_ends(
                &name,
                g.native_block_property(p, "axis").as_deref(),
                g.native_block_property(p, "bottom_shaft").as_deref(),
                g.native_block_property(p, "top_shaft").as_deref(),
            );
        }
        // A motor's end and its setting (saved in its block entity, 16 until set).
        let rpm = match cell.borrow_mut().world.block_entities.get_mut(&(p[0], p[1], p[2])) {
            Some(BlockEntity::Mod(m)) => match m.nbt.get(&JavaString::from(k::RPM_KEY)) {
                Some(Tag::Int(v)) => *v,
                _ => crate::game::kinetics_motor::DEFAULT_RPM,
            },
            _ => crate::game::kinetics_motor::DEFAULT_RPM,
        };
        let facing = cell.borrow().native_block_property(p, "facing")?;
        k::motor_node(&facing, rpm)
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
