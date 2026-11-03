// A girder is placed on the axis of the face that was clicked (`GirderBlock.getStateForPlacement`); its sides are
// settled at the end of the tick (girder_settle.rs).
fn kinetics_girder_placement(
    level: &GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    use crate::game::kinetics_girder as g;
    use horizon_core::geometry::Direction as D;
    if level.native_state_block(state) != Some(g::GIRDER) {
        return;
    }
    let axis = match context.face {
        D::West | D::East => 0,
        D::Down | D::Up => 1,
        D::North | D::South => 2,
    };
    let girder = g::placed(axis);
    let placed = level
        .native_state_with_property(state, "axis", ["x", "y", "z"][axis])
        .and_then(|s| level.native_state_with_property(s, "beam", g::beam_name(girder.x, girder.z)));
    if let Some(placed) = placed {
        *answer = Some(Some(placed));
    }
}
