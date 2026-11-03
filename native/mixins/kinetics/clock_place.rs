// A cuckoo clock is placed with its back to a shaft beside it, else facing the player (`getStateForPlacement`).
fn kinetics_clock_place(
    level: &GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    use crate::game::{kinetics as k, kinetics_clock as c};
    if !level.native_state_block(state).is_some_and(c::is_clock) {
        return;
    }
    // A neighbour has a shaft toward the clock when it turns on the axis that joins them.
    let mut shaft = [false; 4];
    for (i, (d, _)) in c::SIDES.iter().enumerate() {
        let at = [context.pos.x + d[0], context.pos.y, context.pos.z + d[2]];
        let neighbour = level.world.block(at[0], at[1], at[2]);
        if level.native_state_block(neighbour).and_then(k::kind_of).is_none() {
            continue;
        }
        let axis = if d[0] != 0 { "x" } else { "z" };
        shaft[i] = level.native_state_with_property(neighbour, "axis", axis) == Some(neighbour);
    }
    let facing = c::placement_facing(shaft, c::looking(context.yaw));
    if let Some(placed) = level.native_state_with_property(state, "facing", facing) {
        *answer = Some(Some(placed));
    }
}
