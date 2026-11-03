// Draws the bracket saved in a shaft's or cogwheel's block entity: Create's model if its pack is on, ours otherwise.
// The model is turned like the block state rotations of Create's bracket generator (x first, then y).
fn kinetics_bracket_submit(
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
    block: &str,
    state: BlockStateId,
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) {
    use crate::game::{kinetics as k, kinetics_bracket as b};
    use horizon_world::nbt::{JavaString, Tag};
    let Some(ty) = b::type_of(block) else { return };
    let Some(Tag::Int(value)) = entity.nbt.get(&JavaString::from(b::BRACKET_KEY)) else { return };
    let Some((material, facing)) = b::decode(*value) else { return };
    let Some(axis) = game.native_block_state_property(state, "axis").and_then(k::axis_of) else { return };
    let Some(along) = b::along_first(axis, facing) else { return };
    let (x, y) = b::rotations(facing, along);
    let (look, ours) = b::models(material, ty, facing);
    let around = [[0, 0, 0], [1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
    draw.light = k::part_light(around.map(|d| game.native_light([pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]])));
    let centre = [0.5; 3];
    draw.push();
    // The later turn is applied to the model first: x, then y. The game's `y` runs clockwise seen from above.
    draw.rotate_about([0.0, 1.0, 0.0], -(y as f32), centre);
    draw.rotate_about([1.0, 0.0, 0.0], x as f32, centre);
    if !game.native_draw_model(draw, &look, 0xFF_FFFF) {
        game.native_draw_model(draw, &ours, 0xFF_FFFF);
    }
    draw.pop();
}
