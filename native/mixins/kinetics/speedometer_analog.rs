// A comparator reads the needle (`GaugeBlock.getAnalogOutputSignal`): the dial target times 14, 0 to 15.
fn kinetics_speedometer_analog(level: &GameLevel<'_>, pos: BlockPos, state: BlockStateId, output: &mut i32) {
    use horizon_world::nbt::{JavaString, Tag};
    if level.native_state_block(state) != Some(crate::game::kinetics_speedometer::BLOCK) {
        return;
    }
    let speed = match level.world.block_entities.get(&(pos.x, pos.y, pos.z)) {
        Some(BlockEntity::Mod(be)) => match be.nbt.get(&JavaString::from(crate::game::kinetics::SPEED_KEY)) {
            Some(Tag::Float(bits)) => f32::from_bits(*bits),
            _ => 0.0,
        },
        _ => 0.0,
    };
    *output = crate::game::kinetics_speedometer::analog(speed);
}
