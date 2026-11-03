// A rotation speed controller is placed across the large cogwheel above the spot, else across the way the player faces.
fn kinetics_speed_controller_place(
    level: &GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    use crate::game::{kinetics as k, kinetics_speed_controller as c};
    if level.native_state_block(state) != Some(c::BLOCK) {
        return;
    }
    let above = level.world.block(context.pos.x, context.pos.y + 1, context.pos.z);
    let cog = level.native_state_block(above).filter(|b| matches!(k::kind_of(b), Some(k::Kind::LargeCog)));
    let cog_axis = cog.and_then(|_| ["x", "y", "z"].iter().position(|a| level.native_state_with_property(above, "axis", a) == Some(above)));
    let axis = c::placement_axis(cog_axis, context.yaw);
    if let Some(placed) = level.native_state_with_property(state, "axis", ["x", "y", "z"][axis]) {
        *answer = Some(Some(placed));
    }
}
