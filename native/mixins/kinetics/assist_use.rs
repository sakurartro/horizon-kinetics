// A shaft used on a shaft goes in the cell the placement helper shows (assist.rs `pole_target`), the end of the row it is
// aimed along; a cogwheel used on a cogwheel goes in the meshing cell with the axis Create gives it (`cog_target`), as
// Create's `PlacementHelpers` do before the normal placement.
// shortcut: the aim is the acting player's view ray (as the host reads it); if that does not hit the clicked block the
// click places as usual. Sneaking does not turn the helper off. Cased cogwheels in hand are not helped.
fn kinetics_assist_use(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    use crate::game::kinetics_assist as a;
    use horizon_world::chunk::{BlockProperties, BlockState};
    let hand = &*stack.item;
    let hand_cog = a::cog_kind(hand).filter(|_| hand == "kinetics:cogwheel" || hand == "kinetics:large_cogwheel");
    if *claimed || (hand != a::SHAFT && hand_cog.is_none()) || !game.may_build() {
        return;
    }
    let Some((distance, hit)) = game.native_view_hit() else { return };
    if distance > 8.0 || [hit.block_x, hit.block_y, hit.block_z] != pos {
        return;
    }
    let (target, axis_index) = if let Some(large) = hand_cog {
        let info = |p: [i32; 3]| {
            let name = game.native_block(p);
            a::Cell {
                cog: a::cog_kind(&name),
                axis: game.native_block_property(p, "axis").and_then(|x| a::axis_index(&x)),
                replaceable: crate::game::kinetics_water_wheel::replaceable(&name, game.native_block_property(p, "layers").as_deref()),
            }
        };
        let Some(t) = a::cog_target(large, pos, [hit.x, hit.y, hit.z], info) else { return };
        t
    } else {
        if game.native_block(pos) != a::SHAFT {
            return;
        }
        let Some(axis) = game.native_block_property(pos, "axis") else { return };
        let Some(index) = a::axis_index(&axis) else { return };
        let is_pole = |p: [i32; 3]| {
            game.native_block(p) == a::SHAFT && game.native_block_property(p, "axis").as_deref() == Some(axis.as_str())
        };
        let replaceable = |p: [i32; 3]| {
            crate::game::kinetics_water_wheel::replaceable(&game.native_block(p), game.native_block_property(p, "layers").as_deref())
        };
        let Some(t) = a::pole_target(index, pos, [hit.x, hit.y, hit.z], is_pole, replaceable) else { return };
        (t, index)
    };
    let mut properties = BlockProperties::new();
    properties.insert("axis".to_string(), ["x", "y", "z"][axis_index].to_string());
    let state = BlockState { name: hand.to_string(), properties };
    let Ok(id) = game.pack.assets.registry.resolve(&state) else { return };
    let dimension = game.dim.id().to_owned();
    if game.native_set_block(&dimension, target, id, 3) {
        if game.mode_id() != 1 {
            game.native_take_items(&stack.item, 1);
        }
        *claimed = true;
    }
}
