// A kinetic block came or went at `pos`: its network is solved at the end of the level's tick.
fn kinetics_block_changed(
    level: &GameLevel<'_>,
    dimension: LevelDimension,
    pos: BlockPos,
    old: BlockStateId,
    new: BlockStateId,
) {
    use crate::game::kinetics as k;
    let kinetic = |state| level.native_state_block(state).is_some_and(|b| k::kind_of(b).is_some());
    if kinetic(old) || kinetic(new) {
        k::mark(dimension_id(dimension), [pos.x, pos.y, pos.z]);
    }
}
