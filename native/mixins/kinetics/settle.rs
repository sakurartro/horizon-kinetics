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
    // The lookup reads the world, and a block entity can only be reached mutably: share the game through a cell.
    let cell = std::cell::RefCell::new(&mut *game);
    let mut speeds = k::solve_halves(&changed, |p| {
        if detached.contains(&p) {
            return None;
        }
        let name = cell.borrow().native_block(p);
        let nm = name.strip_prefix("kinetics:").unwrap_or("");
        if nm == "clutch" || nm == "gearshift" {
            let g = cell.borrow();
            return k::node_split(
                &name,
                g.native_block_property(p, "axis").as_deref(),
                g.native_block_property(p, "powered").as_deref(),
            );
        }
        if crate::game::kinetics_chain::is_chain(&name) {
            // A chain drive or gearshift: its chain, and the signal its block entity saved.
            let key = JavaString::from(crate::game::kinetics_chain::SIGNAL_KEY);
            let signal = match cell.borrow_mut().world.block_entities.get_mut(&(p[0], p[1], p[2])) {
                Some(BlockEntity::Mod(m)) => match m.nbt.get(&key) {
                    Some(Tag::Int(v)) => *v,
                    _ => 0,
                },
                _ => 0,
            };
            let g = cell.borrow();
            return k::chain_node(
                &name,
                g.native_block_property(p, "axis").as_deref(),
                g.native_block_property(p, "part").as_deref(),
                g.native_block_property(p, crate::game::kinetics_chain::FIRST).as_deref(),
                signal,
            );
        }
        if name == crate::game::kinetics_crank::BLOCK {
            // A hand crank: its facing, and the speed its turner gives it.
            let turn = crate::game::kinetics_crank::speed(dimension, p);
            return k::crank_node(&cell.borrow().native_block_property(p, "facing")?, turn);
        }
        if name == "kinetics:powered_shaft" {
            // A powered shaft: its generator's speed, saved in its block entity (0 until set).
            let rpm = match cell.borrow_mut().world.block_entities.get_mut(&(p[0], p[1], p[2])) {
                Some(BlockEntity::Mod(m)) => match m.nbt.get(&JavaString::from(k::RPM_KEY)) {
                    Some(Tag::Int(v)) => *v,
                    _ => 0,
                },
                _ => 0,
            };
            return k::powered_shaft_node(cell.borrow().native_block_property(p, "axis").as_deref(), rpm);
        }
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
