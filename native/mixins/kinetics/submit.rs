// Draws a shaft or cogwheel turned to its axis and by its speed; one standing
// still keeps its geometry, so it costs nothing each frame.
fn kinetics_submit(
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
    block: &str,
    state: BlockStateId,
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) {
    use crate::game::kinetics as k;
    use horizon_world::nbt::{JavaString, Tag};
    let Some(kind) = k::kind_of(block) else { return };
    let Some(model) = k::part_model(kind) else { return };
    let Some(axis) = game.native_block_state_property(state, "axis").and_then(k::axis_of) else {
        return;
    };
    let speed = match entity.nbt.get(&JavaString::from(k::SPEED_KEY)) {
        Some(Tag::Float(bits)) => f32::from_bits(*bits),
        _ => 0.0,
    };
    let ticks = draw.game_time as f64 + f64::from(draw.partial);
    let angle = k::angle(speed, ticks) + k::tooth_offset(kind, axis, pos);
    let centre = [0.5; 3];
    draw.push();
    draw.rotate_about(k::axis_vector(axis), angle, centre);
    if let Some((turn, degrees)) = k::lay_along(axis) {
        draw.rotate_about(turn, degrees, centre);
    }
    let look = k::look_model(kind).is_some_and(|m| game.native_draw_model(draw, m, 0xFF_FFFF));
    if !look {
        game.native_draw_model(draw, model, 0xFF_FFFF);
    }
    draw.pop();
    if speed == 0.0 {
        draw.keep();
    }
}
