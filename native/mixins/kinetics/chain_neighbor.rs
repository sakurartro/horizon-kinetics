// A neighbour of a chain drive or gearshift changed (a block beside it, a lever, a wire): it is chained again and its
// redstone is read at the end of the level's tick.
fn kinetics_chain_neighbor(
    level: &GameLevel<'_>,
    dimension: LevelDimension,
    pos: BlockPos,
    state: BlockStateId,
) {
    use crate::game::kinetics_chain as c;
    if level.native_state_block(state).is_some_and(c::is_chain) {
        c::mark(dimension_id(dimension), [pos.x, pos.y, pos.z]);
    }
}
