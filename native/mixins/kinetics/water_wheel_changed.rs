// A large wheel came or went, or a structure block of its body did: the body is built, or the wheel or the
// leftover blocks go, at the end of the level's tick (water_wheel_settle.rs).
fn kinetics_water_wheel_changed(
    level: &GameLevel<'_>,
    dimension: LevelDimension,
    pos: BlockPos,
    old: BlockStateId,
    new: BlockStateId,
) {
    use crate::game::kinetics_water_wheel as w;
    let (old, new) = (level.native_state_block(old), level.native_state_block(new));
    let (dim, at) = (dimension_id(dimension), [pos.x, pos.y, pos.z]);
    if new == Some(w::LARGE) && old != Some(w::LARGE) {
        w::queue(dim, at, w::Task::Build);
    }
    if old == Some(w::LARGE) && new != Some(w::LARGE) {
        w::queue(dim, at, w::Task::Removed);
    }
    if old == Some(w::STRUCTURE) && new != Some(w::STRUCTURE) {
        w::queue(dim, at, w::Task::Broken);
    }
}
