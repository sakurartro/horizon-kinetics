// A cogwheel is placed on the axis of the clicked face, as the log it stands on in the data did: its template is
// glass (a log would hide the ground face under its invisible model), whose `axis` is declared.
fn kinetics_cog_placement(
    level: &GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    use horizon_core::geometry::Direction as D;
    if !matches!(level.native_state_block(state), Some("kinetics:cogwheel" | "kinetics:large_cogwheel")) {
        return;
    }
    let axis = match context.face {
        D::West | D::East => "x",
        D::Down | D::Up => "y",
        D::North | D::South => "z",
    };
    if let Some(placed) = level.native_state_with_property(state, "axis", axis) {
        *answer = Some(Some(placed));
    }
}
