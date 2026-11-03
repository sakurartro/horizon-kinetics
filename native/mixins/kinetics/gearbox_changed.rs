// A vertical gearbox came to `pos`: it is made a gearbox at the end of the level's tick.
fn kinetics_gearbox_changed(level: &GameLevel<'_>, dimension: LevelDimension, pos: BlockPos, new: BlockStateId) {
    use crate::game::kinetics_gearbox as g;
    if level.native_state_block(new) == Some(g::VERTICAL) {
        g::mark(dimension_id(dimension), [pos.x, pos.y, pos.z]);
    }
}
