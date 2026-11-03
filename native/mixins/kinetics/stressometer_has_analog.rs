// A stressometer has a comparator output.
fn kinetics_stressometer_has_analog(level: &GameLevel<'_>, state: BlockStateId, output: &mut bool) {
    if level.native_state_block(state) == Some(crate::game::kinetics_stressometer::BLOCK) {
        *output = true;
    }
}
