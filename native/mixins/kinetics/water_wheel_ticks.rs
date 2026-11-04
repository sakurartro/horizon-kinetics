// The wheels' block entities tick (the game only ticks those a mod asks for): that is how they look at the water.
fn kinetics_water_wheel_ticks(block: &str, ticks: &mut bool) {
    if crate::game::kinetics_water_wheel::is_wheel(block) {
        *ticks = true;
    }
}
