// A stressometer is placed on the end of a shaft it is clicked on (facing away, its shaft along that one), else on the
// clicked face with its shaft toward a neighbouring shaft or across the way the player faces (`GaugeBlock.getStateForPlacement`).
fn kinetics_stressometer_place(
    level: &GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    use crate::game::{kinetics as k, kinetics_stressometer as s};
    use horizon_core::geometry::Direction as D;
    if level.native_state_block(state) != Some(s::BLOCK) {
        return;
    }
    let face = match context.face {
        D::West => [-1, 0, 0],
        D::East => [1, 0, 0],
        D::Down => [0, -1, 0],
        D::Up => [0, 1, 0],
        D::North => [0, 0, -1],
        D::South => [0, 0, 1],
    };
    // Whether the block at `at` has a shaft toward `side` (the way from it to the gauge or clicked face).
    let shaft_toward = |at: [i32; 3], side: [i32; 3]| {
        let neighbour = level.world.block(at[0], at[1], at[2]);
        let Some(kind) = level.native_state_block(neighbour).and_then(k::kind_of) else { return false };
        let axis = ["x", "y", "z"][(0..3).find(|&a| side[a] != 0).unwrap_or(1)];
        let along = level.native_state_with_property(neighbour, "axis", axis) == Some(neighbour);
        // A gearbox's axis is the one without a shaft.
        if kind == k::Kind::Gearbox { !along } else { along }
    };
    let pos = [context.pos.x, context.pos.y, context.pos.z];
    let back = [pos[0] - face[0], pos[1] - face[1], pos[2] - face[2]];
    let (facing, along) = s::placed(
        face,
        s::nearest_looking(context.yaw, context.pitch),
        context.yaw,
        shaft_toward(back, face),
        |side| shaft_toward([pos[0] + side[0], pos[1] + side[1], pos[2] + side[2]], [-side[0], -side[1], -side[2]]),
    );
    let placed = level
        .native_state_with_property(state, "facing", s::dir_name(facing))
        .and_then(|st| level.native_state_with_property(st, "axis_along_first", if along { "true" } else { "false" }));
    if let Some(placed) = placed {
        *answer = Some(Some(placed));
    }
}
