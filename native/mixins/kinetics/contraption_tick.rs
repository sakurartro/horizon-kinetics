// At the end of each level's tick, the contraptions in it (contraption.rs): each marker gets its displays (again,
// after a reload or its chunk coming back: entity ids change, so tagged displays no contraption holds are removed),
// then moves a step toward its target and, there, puts its blocks back into the world (Create's
// `Contraption.addBlocksToWorld`; a block whose place is taken drops as an item) and goes.
fn kinetics_contraption_tick(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::kinetics_contraption as c;
    use horizon_world::nbt::Tag;
    let tagged = |raw: Option<&Tag>, tag: &str| match raw.and_then(|t| t.get("Tags")) {
        Some(Tag::List(l)) => l.iter().any(|t| matches!(t, Tag::String(s) if s.to_string_lossy() == tag)),
        _ => false,
    };
    let mut markers = Vec::new();
    let mut pieces = std::collections::HashSet::new();
    for o in &game.others {
        if o.kind == "minecraft:marker" && tagged(game.persist.raw.get(&o.id), c::MARKER_TAG) {
            markers.push(o.id);
        } else if o.kind == "minecraft:block_display" && tagged(game.persist.raw.get(&o.id), c::PIECE_TAG) {
            pieces.insert(o.id);
        }
    }
    let mut mine: Vec<c::Live> = {
        let mut all = c::LIVE.lock().unwrap();
        let (mine, rest): (Vec<c::Live>, Vec<c::Live>) = all.drain(..).partition(|l| l.dimension == dimension);
        *all = rest;
        mine
    };
    if markers.is_empty() && pieces.is_empty() && mine.is_empty() {
        return;
    }
    mine.retain(|l| markers.contains(&l.marker));
    for &m in &markers {
        if !mine.iter().any(|l| l.marker == m) {
            mine.push(c::Live { dimension: dimension.to_owned(), marker: m, displays: Vec::new() });
        }
    }
    for id in &pieces {
        if !mine.iter().any(|l| l.displays.contains(id)) {
            game.native_remove_entity(dimension, *id);
        }
    }
    mine.retain_mut(|l| c_run(game, dimension, l, &pieces));
    c::LIVE.lock().unwrap().extend(mine);

    /// One contraption's tick: false once it is put back (or its marker can't be read).
    fn c_run(
        game: &mut crate::game::Game<'_>,
        dimension: &str,
        l: &mut crate::game::kinetics_contraption::Live,
        alive: &std::collections::HashSet<u32>,
    ) -> bool {
        use crate::game::kinetics_contraption as c;
        use horizon_world::nbt::Tag;
        let Some(data) = game.persist.raw.get(&l.marker).and_then(|t| t.get("data")).cloned() else {
            return false;
        };
        let ints = |key: &str| match data.get(key) {
            Some(Tag::IntArray(v)) if v.len() == 3 => Some([v[0], v[1], v[2]]),
            _ => None,
        };
        let doubles = |key: &str| match data.get(key) {
            Some(Tag::List(v)) if v.len() == 3 => {
                let d = |t: &Tag| if let Tag::Double(b) = t { f64::from_bits(*b) } else { 0.0 };
                Some([d(&v[0]), d(&v[1]), d(&v[2])])
            }
            _ => None,
        };
        let (Some(origin), Some(offset), Some(target), Some(Tag::List(list))) =
            (ints("origin"), doubles("offset"), doubles("target"), data.get("pieces"))
        else {
            return false;
        };
        let rel = |p: &Tag| match p.get("pos") {
            Some(Tag::IntArray(v)) if v.len() == 3 => [v[0], v[1], v[2]],
            _ => [0; 3],
        };
        let at = |off: [f64; 3], r: [i32; 3]| [0, 1, 2].map(|i| f64::from(origin[i] + r[i]) + off[i]);
        // Its displays: made again when one is missing.
        if l.displays.is_empty() || l.displays.iter().any(|id| !alive.contains(id)) {
            for id in l.displays.drain(..) {
                game.native_remove_entity(dimension, id);
            }
            for p in list {
                let mut nbt = Compound::new();
                nbt.insert("block_state".into(), p.get("state").cloned().unwrap_or(Tag::Compound(Compound::new())));
                nbt.insert("Tags".into(), Tag::List(vec![Tag::String(c::PIECE_TAG.into())]));
                // Clients ease each tick's move over the tick.
                nbt.insert("teleport_duration".into(), Tag::Int(1));
                match game.native_summon(dimension, "minecraft:block_display", at(offset, rel(p)), nbt) {
                    Some(id) => l.displays.push(id),
                    // A chunk that isn't loaded: try again next tick.
                    None => {
                        for id in l.displays.drain(..) {
                            game.native_remove_entity(dimension, id);
                        }
                        return true;
                    }
                }
            }
        }
        if offset == target {
            return c_place(game, dimension, l, origin, offset, list);
        }
        let next = c::step(offset, target, c::SPEED);
        // Stop at the edge of the loaded world: an entity in a chunk that isn't loaded is never saved.
        if !list.iter().all(|p| {
            let q = at(next, rel(p));
            game.world.chunks.contains_key(&c::chunk_of(q[0], q[2]))
        }) {
            return true;
        }
        game.native_move_entity(dimension, l.marker, at(next, [0; 3]), 0.0, 0.0);
        for (id, p) in l.displays.iter().zip(list) {
            game.native_move_entity(dimension, *id, at(next, rel(p)), 0.0, 0.0);
        }
        if let Some(Tag::Compound(d)) = game.persist.raw.get_mut(&l.marker).and_then(|t| t.get_mut("data")) {
            d.insert("offset".into(), Tag::List(next.iter().map(|x| Tag::Double(x.to_bits())).collect()));
        }
        true
    }

    /// Puts the blocks back where the offset rounds to, then removes the displays and the marker: false.
    fn c_place(
        game: &mut crate::game::Game<'_>,
        dimension: &str,
        l: &mut crate::game::kinetics_contraption::Live,
        origin: [i32; 3],
        offset: [f64; 3],
        list: &[horizon_world::nbt::Tag],
    ) -> bool {
        use crate::game::kinetics_contraption as c;
        use horizon_world::nbt::Tag;
        for p in list {
            let r = match p.get("pos") {
                Some(Tag::IntArray(v)) if v.len() == 3 => [v[0], v[1], v[2]],
                _ => continue,
            };
            let Some(state) = p.get("state").and_then(|s| game.state_from_nbt(s)) else { continue };
            let at = c::landing(origin, offset, r);
            let facts = game.res.facts().get(game.world.block(at[0], at[1], at[2]));
            let free = c::movable(&facts.name, facts.hardness, facts.has_class("LiquidBlock"), false) == c::Movable::Skip;
            if free && game.native_set_block(dimension, at, state, c::PLACE_FLAGS) {
                if let (Some(Tag::Compound(saved)), Some(BlockEntity::Mod(m))) =
                    (p.get("nbt"), game.world.block_entities.get_mut(&(at[0], at[1], at[2])))
                {
                    m.nbt.extend(saved.clone());
                }
                continue;
            }
            // shortcut: the item is the block's own id (true for our blocks and most others, not for a wall torch
            // or redstone wire), with no loot table; use the block's loot when Horizon has a helper for it.
            let name = game.res.facts().get(state).name.to_string();
            let mut item = Compound::new();
            item.insert("id".into(), Tag::String(name.as_str().into()));
            item.insert("count".into(), Tag::Int(1));
            let mut nbt = Compound::new();
            nbt.insert("Item".into(), Tag::Compound(item));
            game.native_summon(dimension, "minecraft:item", at.map(|v| f64::from(v) + 0.5), nbt);
        }
        for id in l.displays.drain(..) {
            game.native_remove_entity(dimension, id);
        }
        game.native_remove_entity(dimension, l.marker);
        false
    }
}
