// A shaft used on a metal girder goes through it: the girder becomes `metal_girder_encased_shaft` (Create's
// `GirderBlock.useItemOn`), the shaft item used up outside creative.
// shortcut: no placing sound, and the wrench doesn't turn the shafted girder back into a girder (Create's
// `getRotatedBlockState`); add them if they matter.
fn kinetics_girder_use(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    use crate::game::kinetics_girder as g;
    use horizon_world::chunk::{BlockProperties, BlockState};
    if *claimed || &*stack.item != g::SHAFT_ITEM || !game.may_build() || game.native_block(pos) != g::GIRDER {
        return;
    }
    let (x_beam, _) = g::beam_flags(game.native_block_property(pos, "beam").as_deref().unwrap_or("none"));
    let clicked = face.iter().position(|c| *c != 0).unwrap_or(1);
    let on = |key: &str| game.native_block_property(pos, key).unwrap_or_else(|| "false".to_string());
    let mut properties = BlockProperties::new();
    properties.insert("axis".to_string(), ["x", "y", "z"][g::shaft_axis(x_beam, clicked)].to_string());
    properties.insert("top".to_string(), on("top"));
    properties.insert("bottom".to_string(), on("bottom"));
    let state = BlockState { name: g::ENCASED.to_string(), properties };
    let Ok(id) = game.pack.assets.registry.resolve(&state) else {
        return;
    };
    let dimension = game.dim.id().to_owned();
    if game.native_set_block(&dimension, pos, id, 3) {
        if game.mode_id() != 1 {
            game.native_take_items(&stack.item, 1);
        }
        *claimed = true;
    }
}
