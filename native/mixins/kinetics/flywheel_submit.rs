// Draws the flywheel's wheel, laid along its axis and turned by its eased speed. Its shaft is drawn by submit.rs
// (the flywheel is a shaft for the network), like Create's `FlywheelRenderer` over `KineticBlockEntityRenderer`.
fn kinetics_flywheel_submit(
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
    block: &str,
    state: BlockStateId,
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) {
    use crate::game::{kinetics as k, kinetics_flywheel as f};
    use horizon_world::nbt::{JavaString, Tag};
    if block != f::BLOCK {
        return;
    }
    let Some(axis) = game.native_block_state_property(state, "axis").and_then(k::axis_of) else { return };
    let speed = match entity.nbt.get(&JavaString::from(k::SPEED_KEY)) {
        Some(Tag::Float(bits)) => f32::from_bits(*bits),
        _ => 0.0,
    };
    let ticks = draw.game_time as f64 + f64::from(draw.partial);
    let centre = [0.5; 3];
    let around = [[0, 0, 0], [1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
    draw.light = k::part_light(around.map(|d| game.native_light([pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]])));
    draw.push();
    draw.rotate_about(k::axis_vector(axis), f::spin(pos, ticks, speed), centre);
    if let Some((turn, degrees)) = k::lay_along(axis) {
        draw.rotate_about(turn, degrees, centre);
    }
    game.native_draw_model(draw, f::WHEEL_MODEL, 0xFF_FFFF);
    draw.pop();
}
