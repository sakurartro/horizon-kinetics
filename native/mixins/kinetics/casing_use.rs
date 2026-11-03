// A casing used on a shaft or cogwheel turns it into the cased block, its axis kept (Create's `EncasableBlock.tryEncase`).
// A cogwheel gets the shafts its neighbours along the axis have toward it. Sneaking places the casing as a block instead.
// shortcut: no placing sound, and sneaking is read from the host's keys, so a LAN guest's sneak doesn't count; add both if they matter.
fn kinetics_casing_use(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    use crate::game::kinetics as k;
    use crate::game::kinetics_casing as c;
    use horizon_world::chunk::{BlockProperties, BlockState};
    if *claimed || game.keys.contains(&crate::game::KeyCode::ShiftLeft) || !game.may_build() {
        return;
    }
    let Some(target) = c::encased_variant(&game.native_block(pos), &stack.item) else {
        return;
    };
    let Some(axis) = game.native_block_property(pos, "axis").as_deref().and_then(k::axis_of) else {
        return;
    };
    let mut properties = BlockProperties::new();
    properties.insert("axis".to_string(), ["x", "y", "z"][axis].to_string());
    if c::is_encased_cog(target) {
        let ends = k::shaft_ends(pos, axis, |p| {
            k::node_ends(
                &game.native_block(p),
                game.native_block_property(p, "axis").as_deref(),
                game.native_block_property(p, "bottom_shaft").as_deref(),
                game.native_block_property(p, "top_shaft").as_deref(),
            )
        });
        properties.insert(c::BOTTOM_SHAFT.to_string(), c::flag(ends[0]).to_string());
        properties.insert(c::TOP_SHAFT.to_string(), c::flag(ends[1]).to_string());
    }
    let state = BlockState { name: target.to_string(), properties };
    let Ok(id) = game.pack.assets.registry.resolve(&state) else {
        return;
    };
    let dimension = game.dim.id().to_owned();
    // UPDATE_ALL: the neighbours and the clients are told, and `native_block_changed` marks the network.
    if game.native_set_block(&dimension, pos, id, 3) {
        if c::CONSUMES_CASING && game.mode_id() != 1 {
            game.native_take_items(&stack.item, 1);
        }
        *claimed = true;
    }
}
