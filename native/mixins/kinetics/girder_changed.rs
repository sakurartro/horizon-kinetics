// A block came or went at `pos`: the girders in it and around it are settled at the end of the level's tick.
fn kinetics_girder_changed(level: &GameLevel<'_>, dimension: LevelDimension, pos: BlockPos, new: BlockStateId) {
    use crate::game::kinetics_girder as g;
    let girder = |at: [i32; 3]| {
        level.native_state_block(level.world.block(at[0], at[1], at[2])).is_some_and(g::is_girder)
    };
    let here = [pos.x, pos.y, pos.z];
    if level.native_state_block(new).is_some_and(g::is_girder) {
        g::mark(dimension_id(dimension), here);
    }
    for d in g::DIRS {
        let at = [here[0] + d[0], here[1] + d[1], here[2] + d[2]];
        if girder(at) {
            g::mark(dimension_id(dimension), at);
        }
    }
}
