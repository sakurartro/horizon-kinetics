// A comparator reads the needle (`GaugeBlock.getAnalogOutputSignal`): the dial target times 14, 0 to 15.
fn kinetics_stressometer_analog(level: &GameLevel<'_>, pos: BlockPos, state: BlockStateId, output: &mut i32) {
    use crate::game::{kinetics_stress as st, kinetics_stressometer as s};
    use horizon_world::nbt::{JavaString, Tag};
    if level.native_state_block(state) != Some(s::BLOCK) {
        return;
    }
    let read = |key: &str| match level.world.block_entities.get(&(pos.x, pos.y, pos.z)) {
        Some(BlockEntity::Mod(be)) => match be.nbt.get(&JavaString::from(key)) {
            Some(Tag::Float(bits)) => Some(f32::from_bits(*bits)),
            _ => None,
        },
        _ => None,
    };
    let target = st::read(read).map_or(0.0, |n| s::dial_target(n.stress.capacity, n.stress.stress, n.theoretical_speed, st::DISABLE_STRESS));
    *output = s::analog(target);
}
