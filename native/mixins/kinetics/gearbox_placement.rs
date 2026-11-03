// A gearbox is placed standing on y whatever face was clicked; a vertical gearbox on the axis across the way
// the player faces (the neighbours' shafts may still turn it, gearbox_settle.rs).
fn kinetics_gearbox_placement(
    level: &GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    use crate::game::kinetics_gearbox as g;
    let axis = match level.native_state_block(state) {
        Some(g::GEARBOX) => 1,
        Some(g::VERTICAL) => g::placed_axis(context.yaw),
        _ => return,
    };
    if let Some(placed) = level.native_state_with_property(state, "axis", ["x", "y", "z"][axis]) {
        *answer = Some(Some(placed));
    }
}
