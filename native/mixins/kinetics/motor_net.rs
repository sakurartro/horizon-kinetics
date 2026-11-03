// A client's wheel over a creative motor, queued for the end of the level's tick. The bytes are the player's own
// input: the block is checked there, and the reach here when the server knows where the player stands.
// shortcut: a player the server can't place (the host of a LAN world) isn't reach-checked; add it when their
// position is readable here.
fn kinetics_motor_net(game: &crate::game::Game<'_>, player: u128, channel: &str, data: &[u8]) {
    use crate::game::kinetics_motor as m;
    if channel != m::CHANNEL {
        return;
    }
    let Some((pos, change)) = m::decode(data) else { return };
    if let Some(p) = game.native_server_players().into_iter().find(|p| p.uuid == player) {
        if !m::in_reach(p.position, pos) {
            return;
        }
    }
    m::push(pos, m::Op::Add(change));
}
