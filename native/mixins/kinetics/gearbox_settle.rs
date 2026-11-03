// Turns the vertical gearboxes placed this tick into gearboxes on their side (Create's vertical_gearbox item
// places the gearbox block itself): standing across the shafts of the neighbours that point at it, else across
// the player's facing, the axis the placement gave it.
fn kinetics_gearbox_settle(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::kinetics as k;
    use crate::game::kinetics_gearbox as g;
    use horizon_world::chunk::{BlockProperties, BlockState};
    for pos in g::take(dimension) {
        if game.native_block(pos) != g::VERTICAL {
            continue;
        }
        let first = game.native_block_property(pos, "axis").as_deref().and_then(k::axis_of).unwrap_or(0);
        let shaft_toward = |d: [i32; 3], axis: usize| {
            let p = [pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]];
            k::node_ends(
                &game.native_block(p),
                game.native_block_property(p, "axis").as_deref(),
                game.native_block_property(p, "bottom_shaft").as_deref(),
                game.native_block_property(p, "top_shaft").as_deref(),
            )
            .is_some_and(|n| k::has_shaft_toward(n, axis, d[axis] < 0))
        };
        let shafts = [
            shaft_toward([1, 0, 0], 0),
            shaft_toward([-1, 0, 0], 0),
            shaft_toward([0, 0, 1], 2),
            shaft_toward([0, 0, -1], 2),
        ];
        let mut properties = BlockProperties::new();
        properties.insert("axis".to_string(), ["x", "y", "z"][g::vertical_axis(shafts, first)].to_string());
        let state = BlockState { name: g::GEARBOX.to_string(), properties };
        let Ok(id) = game.pack.assets.registry.resolve(&state) else {
            continue;
        };
        // UPDATE_ALL: the neighbours and the clients are told, and `native_block_changed` marks the network.
        game.native_set_block(dimension, pos, id, 3);
    }
}
