// A creative motor is placed with its shaft toward a shaft beside it, else toward the player (away when sneaking).
fn kinetics_motor_place(
    level: &GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    use crate::game::{kinetics as k, kinetics_motor as m};
    if level.native_state_block(state) != Some(m::BLOCK) {
        return;
    }
    // A neighbour has a shaft toward the motor when it turns on the axis that joins them.
    let mut shaft = [false; 6];
    for (i, d) in m::DIRS.iter().enumerate() {
        let at = [context.pos.x + d[0], context.pos.y + d[1], context.pos.z + d[2]];
        let neighbour = level.world.block(at[0], at[1], at[2]);
        if level.native_state_block(neighbour).and_then(k::kind_of).is_none() {
            continue;
        }
        let axis = ["x", "y", "z"][m::axis(*d)];
        shaft[i] = level.native_state_with_property(neighbour, "axis", axis) == Some(neighbour);
    }
    let look = m::nearest_looking(context.yaw, context.pitch);
    let facing = m::placement_facing(look, context.sneaking, shaft);
    if let Some(placed) = level.native_state_with_property(state, "facing", m::dir_name(facing)) {
        *answer = Some(Some(placed));
    }
}
