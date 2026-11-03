// Draws a water wheel turned by its speed, in the planks it is made of; the small one in its frame.
fn kinetics_water_wheel_submit(
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
    block: &str,
    state: BlockStateId,
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) {
    use crate::game::{kinetics as k, kinetics_water_wheel as w};
    use horizon_world::nbt::{JavaString, Tag};
    if !w::is_wheel(block) {
        return;
    }
    let large = block == w::LARGE;
    let Some(axis) = game.native_block_state_property(state, if large { "axis" } else { "facing" }).and_then(w::axis_index) else {
        return;
    };
    let speed = match entity.nbt.get(&JavaString::from(k::SPEED_KEY)) {
        Some(Tag::Float(bits)) => f32::from_bits(*bits),
        _ => 0.0,
    };
    let material = match entity.nbt.get(&JavaString::from(w::MATERIAL_KEY)) {
        Some(Tag::String(s)) => s.to_string_lossy(),
        _ => w::DEFAULT_MATERIAL.to_string(),
    };
    let ticks = draw.game_time as f64 + f64::from(draw.partial);
    let centre = [0.5; 3];
    let around = [[0, 0, 0], [1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
    draw.light = k::part_light(around.map(|d| game.native_light([pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]])));
    if !large {
        // Create's frame over our placeholder, a hair larger so it covers it (only with the look pack on).
        draw.push();
        draw.translate(centre);
        draw.scale([1.001; 3]);
        draw.translate([-0.5; 3]);
        if let Some((turn, degrees)) = k::lay_along(axis) {
            draw.rotate_about(turn, degrees, centre);
        }
        game.native_draw_model(draw, w::LOOK_FRAME, 0xFF_FFFF);
        draw.pop();
    }
    draw.push();
    draw.rotate_about(k::axis_vector(axis), k::angle(speed, ticks), centre);
    if let Some((turn, degrees)) = k::lay_along(axis) {
        draw.rotate_about(turn, degrees, centre);
    }
    game.native_draw_model(draw, &w::model(large, &material), 0xFF_FFFF);
    draw.pop();
}
