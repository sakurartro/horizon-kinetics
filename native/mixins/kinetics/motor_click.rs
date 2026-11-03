// The right button on a creative motor with both hands empty reverses it (Create sets the way on its value box's
// board; a block in hand is placed against the motor as usual). The server decides, at the end of the level's tick.
fn kinetics_motor_click(game: &crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    use crate::game::kinetics_motor as m;
    if game.native_window() || game.native_block(pos) != m::BLOCK {
        return;
    }
    if game.stack_in(game.selected).is_some() || game.offhand.is_some() {
        return;
    }
    m::push(pos, m::Op::Reverse);
    *canceled = true;
}
