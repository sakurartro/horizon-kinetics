// Each tick of a level: the clocks that turn tick, chime, smoke and, the mysterious ones, go off
// (`CuckooClockBlockEntity.tick`). The sounds and particles are the server's; the clients draw from the saved start.
// shortcut: walks every block entity of the level each tick; index the clocks if that ever shows.
fn kinetics_clock_tick(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::{kinetics as k, kinetics_clock as c};
    use horizon_world::chunk::{BlockProperties, BlockState};
    use horizon_world::nbt::{JavaString, Tag};
    let turning: Vec<([i32; 3], bool, i32, i64)> = game
        .world
        .block_entities
        .iter()
        .filter_map(|(p, be)| {
            let BlockEntity::Mod(m) = be else { return None };
            if !c::is_clock(&m.id) {
                return None;
            }
            match m.nbt.get(&JavaString::from(k::SPEED_KEY)) {
                Some(Tag::Float(bits)) if f32::from_bits(*bits) != 0.0 => {}
                _ => return None,
            }
            let anim = match m.nbt.get(&JavaString::from(c::ANIM_KEY)) {
                Some(Tag::Int(v)) => *v,
                _ => c::NONE,
            };
            let start = match m.nbt.get(&JavaString::from(c::START_KEY)) {
                Some(Tag::Long(v)) => *v,
                _ => 0,
            };
            Some(([p.0, p.1, p.2], m.id == c::MYSTERIOUS, anim, start))
        })
        .collect();
    let (now, day) = (game.world.game_time, game.native_day_time());
    for (pos, mysterious, anim, start) in turning {
        let s = c::step(anim, start, now, day, c::natural(dimension), mysterious);
        if s.changed {
            game.native_with_mod_block_entity(pos, |be| {
                be.nbt.insert(JavaString::from(c::ANIM_KEY), Tag::Int(s.anim));
                be.nbt.insert(JavaString::from(c::START_KEY), Tag::Long(s.start));
            });
            game.native_block_entity_sync(pos);
        }
        let at = [f64::from(pos[0]) + 0.5, f64::from(pos[1]) + 0.5, f64::from(pos[2]) + 0.5];
        if !s.sounds.is_empty() || s.smoke {
            // SoundSource 4 is `block`.
            game.with_level(|_, level, _| {
                for x in &s.sounds {
                    level.native_play_sound(x.event, at, 4, x.volume, x.pitch);
                }
                if s.smoke {
                    use horizon_client::particles::{ParticleOptions, ParticleType};
                    level.native_send_particles(&ParticleOptions::simple(ParticleType::LargeSmoke), None, at, [0.3; 3], 0.0, 1);
                }
            });
        }
        if s.explode {
            // `destroyBlock(pos, false)`, then a block-breaking blast of radius 3 (`ExplosionInteraction.BLOCK`).
            let air = BlockState { name: "minecraft:air".to_string(), properties: BlockProperties::new() };
            if let Ok(id) = game.pack.assets.registry.resolve(&air) {
                game.native_set_block(dimension, pos, id, 3);
            }
            game.native_explode(dimension, at, 3.0, false, crate::game::native_explosions::NativeBlast::Block);
        }
    }
}
