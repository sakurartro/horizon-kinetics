// The right button on a hand crank turns it, as `HandCrankBlock.useItemOn`: the click repeats while the button is held.
fn kinetics_crank_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    use crate::game::{kinetics as k, kinetics_crank as c};
    // The window's copy of a server's world only predicts; the server turns the crank.
    if *canceled || !game.native_authoritative() || game.native_block(pos) != c::BLOCK || game.spectator() {
        return;
    }
    let sneaking = game.keys.contains(&KeyCode::ShiftLeft);
    // Sneaking with something in hand puts it against the crank, as against any block.
    if sneaking && (game.stack_in(game.selected).is_some() || game.offhand.is_some()) {
        return;
    }
    let dimension = game.dim.id();
    if c::turn(dimension, pos, sneaking) {
        k::mark(dimension, pos);
    }
    if !game.creative
        && let Some(p) = game.player.as_mut()
    {
        p.cause_food_exhaustion(c::EXHAUSTION);
    }
    *canceled = true;
}
