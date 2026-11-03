// A small wheel is placed on the axis the player looks along; a large one on the axis of the clicked face, and only
// where the eight blocks round it can be built over (`LargeWaterWheelBlock.getStateForPlacement`).
// shortcut: no retry one block further out (Create's `LargeWaterWheelBlockItem.place`) and no outline of the missing room.
fn kinetics_water_wheel_place(
    level: &GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    use crate::game::{kinetics_motor as m, kinetics_water_wheel as w};
    use horizon_core::geometry::Direction as D;
    match level.native_state_block(state) {
        Some(w::SMALL) => {
            let facing = m::dir_name(m::nearest_looking(context.yaw, context.pitch));
            if let Some(placed) = level.native_state_with_property(state, "facing", facing) {
                *answer = Some(Some(placed));
            }
        }
        Some(w::LARGE) => {
            let axis = match context.face {
                D::West | D::East => 0,
                D::Down | D::Up => 1,
                D::North | D::South => 2,
            };
            let free = |off: [i32; 3]| {
                let at = [context.pos.x + off[0], context.pos.y + off[1], context.pos.z + off[2]];
                let there = level.world.block(at[0], at[1], at[2]);
                let thin = level.native_state_with_property(there, "layers", "1") == Some(there);
                level.native_state_block(there).is_some_and(|name| w::replaceable(name, thin.then_some("1")))
            };
            if !w::fits(axis, free) {
                *answer = Some(None);
            } else if let Some(placed) = level.native_state_with_property(state, "axis", ["x", "y", "z"][axis]) {
                *answer = Some(Some(placed));
            }
        }
        _ => {}
    }
}
