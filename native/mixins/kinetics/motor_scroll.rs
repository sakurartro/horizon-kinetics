// The wheel over a side of a creative motor changes its speed (1 a step, 16 with Shift) instead of the hotbar:
// the window tells the server, which decides.
fn kinetics_motor_scroll(
    game: &mut crate::game::Game<'_>,
    accumulated: [i32; 2],
    modifiers: u32,
    canceled: &mut bool,
) {
    use crate::game::kinetics_motor as m;
    if !game.native_window() || accumulated[1] == 0 {
        return;
    }
    let Some((distance, hit)) = game.native_view_hit() else { return };
    let controller = hit.block == crate::game::kinetics_speed_controller::BLOCK;
    if (hit.block != m::BLOCK && !controller) || distance > 5.0 {
        return;
    }
    let pos = [hit.block_x, hit.block_y, hit.block_z];
    let local = [hit.x - f64::from(pos[0]), hit.y - f64::from(pos[1]), hit.z - f64::from(pos[2])];
    let Some(side) = m::hit_side(local) else { return };
    // A controller's value box is on its four sides across the shaft; a motor's by its facing.
    let active = if controller {
        let axis = game.native_block_property(pos, "axis").as_deref().and_then(crate::game::kinetics::axis_of);
        axis.is_some_and(|a| crate::game::kinetics_speed_controller::box_side(a, side))
    } else {
        let facing = game.native_block_property(pos, "facing").as_deref().and_then(m::dir_vec);
        facing.is_some_and(|f| m::box_side(f, side))
    };
    if !active {
        return;
    }
    let change = accumulated[1] * if modifiers & 1 != 0 { m::BIG_STEP } else { m::STEP };
    if game.native_net_send_to_server(m::CHANNEL, &m::encode(pos, change)).is_ok() {
        *canceled = true;
    }
}
