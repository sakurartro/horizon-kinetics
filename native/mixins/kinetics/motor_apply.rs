// Applies what players asked of the creative motors and rotation speed controllers (the wheel, the right button) to this level's motors, saves the
// setting in their block entities and has their networks solved again.
fn kinetics_motor_apply(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::{kinetics as k, kinetics_motor as m};
    use horizon_world::nbt::{JavaString, Tag};
    let asked = m::take(|p| game.native_block(p) == m::BLOCK || game.native_block(p) == crate::game::kinetics_speed_controller::BLOCK);
    if asked.is_empty() {
        return;
    }
    let level: &'static str = match dimension {
        "minecraft:the_nether" => "minecraft:the_nether",
        "minecraft:the_end" => "minecraft:the_end",
        _ => "minecraft:overworld",
    };
    let key = JavaString::from(k::RPM_KEY);
    for (pos, op) in asked {
        let moved = game.native_with_mod_block_entity(pos, |be| {
            let old = match be.nbt.get(&key) {
                Some(Tag::Int(v)) => *v,
                _ => m::DEFAULT_RPM,
            };
            let new = m::apply(old, op);
            if new == old {
                return false;
            }
            be.nbt.insert(key.clone(), Tag::Int(new));
            true
        });
        if moved == Some(true) {
            k::mark(level, pos);
            game.native_block_entity_sync(pos);
        }
    }
}
