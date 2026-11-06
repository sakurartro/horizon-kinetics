// `/kinetics_contraption <from> <to> <dx> <dy> <dz>` (BUR.md phase 2, a debug command until the piston and the glue
// come): the blocks glued in the box from..to that touch its corner `from` become a contraption, which moves by
// (dx, dy, dz) and is put back there (contraption_tick.rs). Create's `Contraption.assemble`, then
// `removeBlocksFromWorld`: the snapshot is written into the marker before any block leaves the world, in the same
// tick, so a save never sees one without the other.
fn kinetics_contraption_command(
    game: &mut crate::game::Game<'_>,
    player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    use crate::game::{kinetics as k, kinetics_contraption as c};
    use horizon_world::nbt::{JavaString, Tag};
    let mut words = command.split_whitespace();
    if *ran || words.next() != Some(c::COMMAND) {
        return;
    }
    *ran = true;
    let base = player.position.map(|v| v.floor() as i32);
    let args: Vec<&str> = words.collect();
    let nums: Option<Vec<i32>> = (args.len() == 9)
        .then(|| args.iter().enumerate().map(|(i, t)| if i < 6 { c::coord(t, base[i % 3]) } else { t.parse().ok() }).collect())
        .flatten();
    let Some(n) = nums else {
        game.native_feedback(Text::literal(format!("/{} <from> <to> <dx> <dy> <dz>", c::COMMAND)));
        return;
    };
    let dimension = game.dim.id().to_owned();
    if dimension != player.dimension || !game.native_authoritative() {
        return;
    }
    let (from, to, by) = ([n[0], n[1], n[2]], [n[3], n[4], n[5]], [n[6], n[7], n[8]]);
    let glue = [c::Glue::new(from, to)];
    let found = c::collect(
        from,
        &glue,
        |p| {
            let facts = game.res.facts().get(game.world.block(p[0], p[1], p[2]));
            let foreign = matches!(game.world.block_entities.get(&(p[0], p[1], p[2])), Some(e) if !matches!(e, BlockEntity::Mod(_)));
            c::movable(&facts.name, facts.hardness, facts.has_class("LiquidBlock"), foreign)
        },
        c::LIMIT,
    );
    let blocks = match found {
        Ok(b) => b,
        Err(why) => {
            let text = match why {
                c::Refusal::NothingThere => format!("no block at {from:?} to start from"),
                c::Refusal::Unmovable(p) => format!("the block at {p:?} can't move"),
                c::Refusal::TooMany => format!("more than {} blocks", c::LIMIT),
            };
            game.native_feedback(Text::literal(text));
            return;
        }
    };
    let registry = game.pack.assets.registry;
    let Some(air) = registry.default_state("minecraft:air") else { return };
    // The snapshot: each block's place from the anchor, its state as `block_state` takes it, and our block entity's
    // data without its speed (the network is solved again where it lands).
    let mut pieces = Vec::new();
    for p in &blocks {
        let state = game.world.block(p[0], p[1], p[2]);
        let mut piece = Compound::new();
        piece.insert("pos".into(), Tag::IntArray((0..3).map(|i| p[i] - from[i]).collect()));
        piece.insert("state".into(), c_state_tag(registry, state));
        if let Some(BlockEntity::Mod(m)) = game.world.block_entities.get(&(p[0], p[1], p[2])) {
            let mut nbt = m.nbt.clone();
            nbt.remove(&JavaString::from(k::SPEED_KEY));
            piece.insert("nbt".into(), Tag::Compound(nbt));
        }
        pieces.push(Tag::Compound(piece));
    }
    let doubles = |v: [f64; 3]| Tag::List(v.iter().map(|x| Tag::Double(x.to_bits())).collect());
    let mut data = Compound::new();
    data.insert("origin".into(), Tag::IntArray(from.to_vec()));
    data.insert("offset".into(), doubles([0.0; 3]));
    data.insert("target".into(), doubles(by.map(f64::from)));
    data.insert("pieces".into(), Tag::List(pieces));
    let mut nbt = Compound::new();
    nbt.insert("Tags".into(), Tag::List(vec![Tag::String(c::MARKER_TAG.into())]));
    nbt.insert("data".into(), Tag::Compound(data));
    if game.native_summon(&dimension, "minecraft:marker", from.map(f64::from), nbt).is_none() {
        game.native_feedback(Text::literal("the marker couldn't be made here".to_string()));
        return;
    };
    // Its displays come at this tick's end (contraption_tick.rs). Create's flags: no drops, no shape updates (a torch on it stays put), moved by a piston.
    for p in &blocks {
        game.native_set_block(&dimension, *p, air, c::REMOVE_FLAGS);
    }
    game.native_feedback(Text::literal(format!("{} blocks move by {by:?}", blocks.len())));

    // `{Name, Properties}`, as a `block_display` and `state_from_nbt` read it.
    fn c_state_tag(registry: &BlockRegistry, state: BlockStateId) -> horizon_world::nbt::Tag {
        use horizon_world::nbt::Tag;
        let mut t = Compound::new();
        if let Some(s) = registry.state(state) {
            t.insert("Name".into(), Tag::String(s.name.as_str().into()));
            let props = s.properties.iter().map(|(k, v)| (k.as_str().into(), Tag::String(v.as_str().into()))).collect();
            t.insert("Properties".into(), Tag::Compound(props));
        }
        Tag::Compound(t)
    }
}
