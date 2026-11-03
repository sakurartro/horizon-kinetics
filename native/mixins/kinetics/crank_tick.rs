// Each tick of a level: the cranks nobody turns any more stop, and the ones being turned click.
fn kinetics_crank_tick(game: &mut crate::game::Game<'_>, dimension: &str) {
    use crate::game::{kinetics as k, kinetics_crank as c};
    let (stopped, sounds) = c::tick(dimension);
    // `mark` keeps the dimension for later: its ids are the three vanilla's.
    let id = match dimension {
        "minecraft:overworld" => "minecraft:overworld",
        "minecraft:the_nether" => "minecraft:the_nether",
        "minecraft:the_end" => "minecraft:the_end",
        _ => return,
    };
    for pos in stopped {
        k::mark(id, pos);
    }
    if !sounds.is_empty() {
        // SoundSource 4 is `block`.
        game.with_level(|_, level, _| {
            for (pos, pitch) in sounds {
                let at = [f64::from(pos[0]) + 0.5, f64::from(pos[1]) + 0.5, f64::from(pos[2]) + 0.5];
                level.native_play_sound("minecraft:block.lever.click", at, 4, 0.4, pitch);
            }
        });
    }
}
