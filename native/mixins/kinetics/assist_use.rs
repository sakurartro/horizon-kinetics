// A shaft used on a shaft goes in the cell the placement helper shows (assist.rs `pole_target`), the end of the row it is
// aimed along, as Create's `PlacementHelpers` do before the normal placement.
// shortcut: the aim is the acting player's view ray (as the host reads it); if that does not hit the clicked block the
// click places as usual. Sneaking does not turn the helper off.
fn kinetics_assist_use(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    use crate::game::kinetics_assist as a;
    use horizon_world::chunk::{BlockProperties, BlockState};
    if *claimed || &*stack.item != a::SHAFT || !game.may_build() || game.native_block(pos) != a::SHAFT {
        return;
    }
    let Some(axis) = game.native_block_property(pos, "axis") else { return };
    let Some(index) = a::axis_index(&axis) else { return };
    let Some((distance, hit)) = game.native_view_hit() else { return };
    if distance > 8.0 || [hit.block_x, hit.block_y, hit.block_z] != pos {
        return;
    }
    let is_pole = |p: [i32; 3]| {
        game.native_block(p) == a::SHAFT && game.native_block_property(p, "axis").as_deref() == Some(axis.as_str())
    };
    let replaceable = |p: [i32; 3]| {
        crate::game::kinetics_water_wheel::replaceable(&game.native_block(p), game.native_block_property(p, "layers").as_deref())
    };
    let Some(target) = a::pole_target(index, pos, [hit.x, hit.y, hit.z], is_pole, replaceable) else { return };
    let mut properties = BlockProperties::new();
    properties.insert("axis".to_string(), axis);
    let state = BlockState { name: a::SHAFT.to_string(), properties };
    let Ok(id) = game.pack.assets.registry.resolve(&state) else { return };
    let dimension = game.dim.id().to_owned();
    if game.native_set_block(&dimension, target, id, 3) {
        if game.mode_id() != 1 {
            game.native_take_items(&stack.item, 1);
        }
        *claimed = true;
    }
}
