// A chain drive was placed (or a block changed beside one): it is chained to its neighbours at the end of the tick.
fn kinetics_chain_changed(
    level: &GameLevel<'_>,
    dimension: LevelDimension,
    pos: BlockPos,
    old: BlockStateId,
    new: BlockStateId,
) {
    use crate::game::kinetics_chain as c;
    let chain = |state| level.native_state_block(state).is_some_and(c::is_chain);
    if chain(old) || chain(new) {
        c::mark(dimension_id(dimension), [pos.x, pos.y, pos.z]);
    }
}
