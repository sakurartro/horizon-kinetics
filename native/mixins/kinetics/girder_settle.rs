// Settles the girders marked this tick: each takes the connections its surroundings give it (`GirderBlock.updateShape`).
// A girder that changes is a block change, so its neighbours are marked and settle at the next tick's end.
fn kinetics_girder_settle(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::kinetics_casing::flag as on;
    use crate::game::kinetics_girder as g;
    use horizon_world::chunk::{BlockProperties, BlockState};
    for pos in g::take(dimension) {
        let name = game.native_block(pos);
        if !g::is_girder(&name) {
            continue;
        }
        let side = |d: usize| {
            let p = [pos[0] + g::DIRS[d][0], pos[1] + g::DIRS[d][1], pos[2] + g::DIRS[d][2]];
            g::classify(&game.native_block(p), |k| game.native_block_property(p, k))
        };
        let flag = |key: &str| game.native_block_property(pos, key).as_deref() == Some("true");
        let mut properties = BlockProperties::new();
        if name == g::GIRDER {
            let axis = game.native_block_property(pos, "axis");
            let (x, z) = g::beam_flags(game.native_block_property(pos, "beam").as_deref().unwrap_or("none"));
            let before = g::GirderState {
                axis: ["x", "y", "z"].iter().position(|a| Some(*a) == axis.as_deref()).unwrap_or(1),
                x,
                z,
                top: flag("top"),
                bottom: flag("bottom"),
            };
            let after = g::update(before, side);
            if after == before {
                continue;
            }
            properties.insert("axis".to_string(), ["x", "y", "z"][after.axis].to_string());
            properties.insert("beam".to_string(), g::beam_name(after.x, after.z).to_string());
            properties.insert("top".to_string(), on(after.top).to_string());
            properties.insert("bottom".to_string(), on(after.bottom).to_string());
            // The template's own property, waterlogged, is kept.
            if let Some(w) = game.native_block_property(pos, "waterlogged") {
                properties.insert("waterlogged".to_string(), w);
            }
        } else {
            let (top, bottom) = g::update_shafted(flag("top"), flag("bottom"), side);
            if (top, bottom) == (flag("top"), flag("bottom")) {
                continue;
            }
            properties.insert("axis".to_string(), game.native_block_property(pos, "axis").unwrap_or_default());
            properties.insert("top".to_string(), on(top).to_string());
            properties.insert("bottom".to_string(), on(bottom).to_string());
        }
        let state = BlockState { name, properties };
        if let Ok(id) = game.pack.assets.registry.resolve(&state) {
            game.native_set_block(dimension, pos, id, 3);
        }
    }
}
