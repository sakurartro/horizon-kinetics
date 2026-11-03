// Every tick of a water wheel's block entity: now and then it looks at the water around it
// (`WaterWheelBlockEntity.determineAndApplyFlowScore`); a new score is saved and the network solved again.
fn kinetics_water_wheel_tick(game: &mut crate::game::Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    use crate::game::{kinetics as k, kinetics_water_wheel as w};
    use horizon_world::nbt::{JavaString, Tag};
    let name: &str = &entity.id;
    if !w::is_wheel(name) || !game.native_authoritative() {
        return;
    }
    let dimension = game.dim.id();
    if !w::due(dimension, pos) {
        return;
    }
    let Some(axis) = game
        .native_block_property(pos, if name == w::LARGE { "axis" } else { "facing" })
        .as_deref()
        .and_then(w::axis_index)
    else {
        return;
    };
    // Each block is read once for this look.
    let memo = std::cell::RefCell::new(std::collections::HashMap::new());
    let get = |p: [i32; 3]| {
        *memo.borrow_mut().entry(p).or_insert_with(|| w::classify(&game.native_block(p), |key| game.native_block_property(p, key)))
    };
    let (score, _lava) = w::flow_score(&get, pos, axis, w::size(name));
    let key = JavaString::from(w::FLOW_KEY);
    let old = match entity.nbt.get(&key) {
        Some(Tag::Int(v)) => *v,
        _ => 0,
    };
    if score != old {
        entity.nbt.insert(key, Tag::Int(score));
        k::mark(dimension, pos);
    }
}
