// A large wheel reaches two and a half blocks out of its block.
fn kinetics_water_wheel_bounds(block: &str, bounds: &mut [f32; 6]) {
    if block == crate::game::kinetics_water_wheel::LARGE {
        *bounds = [-3.0, -3.0, -3.0, 4.0, 4.0, 4.0];
    }
}
