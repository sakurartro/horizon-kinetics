// The large wheels' bodies (`LargeWaterWheelBlock.tick`, `WaterWheelStructuralBlock.onRemove` / `tick`): a placed wheel
// fills the eight blocks round it with structure blocks (or goes if one of them is taken); a structure block that is
// gone takes its wheel with it; the structure blocks of a wheel that is gone go too.
fn kinetics_water_wheel_settle(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::kinetics_water_wheel as w;
    use horizon_world::chunk::{BlockProperties, BlockState};
    fn destroy(game: &mut crate::game::Game<'_>, p: [i32; 3], drop: bool) {
        let at = horizon_core::position::BlockPos::new(p[0], p[1], p[2]);
        game.with_level(|b, level, ticks| b.destroy_block(level, ticks, at, drop));
        let _ = game.process_level_events();
    }
    fn add(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
        [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
    }
    let near = || (-1..=1).flat_map(|x| (-1..=1).flat_map(move |y| (-1..=1).map(move |z| [x, y, z]))).filter(|d| *d != [0; 3]);
    for (pos, task) in w::take(dimension) {
        match task {
            w::Task::Build => {
                if game.native_block(pos) != w::LARGE {
                    continue;
                }
                let Some(axis) = game.native_block_property(pos, "axis").as_deref().and_then(w::axis_index) else {
                    continue;
                };
                let mut missing = Vec::new();
                let mut blocked = false;
                for (off, facing) in w::body(axis) {
                    let p = add(pos, off);
                    let name = game.native_block(p);
                    if name == w::STRUCTURE && game.native_block_property(p, "facing").as_deref() == Some(facing) {
                        continue;
                    }
                    if !w::replaceable(&name, game.native_block_property(p, "layers").as_deref()) {
                        blocked = true;
                        break;
                    }
                    missing.push((p, facing));
                }
                if blocked {
                    // Create: `destroyBlock(pos, false)`.
                    destroy(game, pos, false);
                    continue;
                }
                for (p, facing) in missing {
                    let mut properties = BlockProperties::new();
                    properties.insert("facing".to_string(), facing.to_string());
                    let state = BlockState { name: w::STRUCTURE.to_string(), properties };
                    if let Ok(id) = game.pack.assets.registry.resolve(&state) {
                        game.native_set_block(dimension, p, id, 3);
                    }
                }
            }
            w::Task::Broken => {
                if game.native_block(pos) == w::STRUCTURE {
                    continue;
                }
                let mut wheels = Vec::new();
                for d in near() {
                    let m = add(pos, d);
                    if game.native_block(m) != w::LARGE {
                        continue;
                    }
                    let Some(axis) = game.native_block_property(m, "axis").as_deref().and_then(w::axis_index) else {
                        continue;
                    };
                    if w::body(axis).iter().any(|(off, _)| add(m, *off) == pos) {
                        wheels.push(m);
                    }
                }
                let drop = !game.creative;
                for m in wheels {
                    destroy(game, m, drop);
                }
            }
            w::Task::Removed => {
                let get = |q: [i32; 3]| w::part_of(&game.native_block(q), |key| game.native_block_property(q, key));
                let stale: Vec<[i32; 3]> = near()
                    .map(|d| add(pos, d))
                    .filter(|&p| matches!(get(p), w::Part::Structure(f) if !w::still_valid(&get, p, f, false)))
                    .collect();
                let air = BlockState { name: "minecraft:air".to_string(), properties: BlockProperties::new() };
                if let Ok(id) = game.pack.assets.registry.resolve(&air) {
                    for p in stale {
                        game.native_set_block(dimension, p, id, 3);
                    }
                }
            }
        }
    }
}
