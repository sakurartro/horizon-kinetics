// Kinetics: the rotation speed controller's pure rules, with no game types: the bundle adds this file to the game as
// `game::kinetics_speed_controller`, and `cargo test` runs its tests (native/Cargo.toml `[[test]] speed_controller`).
//
// Ported from Create's `content/kinetics/speedController/SpeedControllerBlock.java` (`getStateForPlacement`) and
// `SpeedControllerBlockEntity.java` (DEFAULT_SPEED 16, the target between -256 and 256, `ControllerValueBoxTransform`),
// https://github.com/Creators-of-Create/Create, branch mc1.21.1/dev, commit 4148fd8. The link to the large
// cogwheel and the speed it conveys are in runtime.rs (`controller_partner`, `conveyed`).
//
// Copyright (c) The Create Team / The Creators of Create
// Licensed under the MIT License; the full notice is in runtime.rs.
//
// What differs from Create: no placement helper (cogwheel on the controller by clicking it), no preferred axis from
// neighbouring shafts; the target is set with the creative motor's wheel (1 per step, 16 with Shift) and its message,
// queue and value box (motor.rs); the target is saved under the same `rpm` key.

pub type Pos = [i32; 3];

pub const BLOCK: &str = "kinetics:rotation_speed_controller";

/// Whether the value box sits on `side` (`isSideActive`): the four sides, but not the two the shaft comes out of.
/// `axis` is the controller's (0 = x, 2 = z).
pub fn box_side(axis: usize, side: Pos) -> bool {
    side[1] == 0 && side[axis] == 0
}

/// The axis of a controller placed by a player looking along `yaw` (degrees), given the axis of the large cogwheel
/// above the spot if there is one: across that cogwheel's, else across the way the player faces (`getStateForPlacement`).
pub fn placement_axis(above_cog: Option<usize>, yaw: f32) -> usize {
    match above_cog {
        Some(a) if a != 1 => 2 - a,
        _ => {
            let (sin, cos) = f64::from(yaw).to_radians().sin_cos();
            // Facing z (the bigger cosine): the shaft goes across, along x.
            if cos.abs() >= sin.abs() { 0 } else { 2 }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_value_box_is_on_the_four_sides_across_the_shaft() {
        // A controller on x: its shaft ends are east and west.
        assert!(box_side(0, [0, 0, 1]) && box_side(0, [0, 0, -1]));
        assert!(!box_side(0, [1, 0, 0]) && !box_side(0, [-1, 0, 0]));
        assert!(!box_side(0, [0, 1, 0]) && !box_side(0, [0, -1, 0]));
        assert!(box_side(2, [1, 0, 0]) && !box_side(2, [0, 0, 1]));
    }

    #[test]
    fn it_is_placed_across_the_cogwheel_above_or_the_player() {
        assert_eq!(placement_axis(Some(0), 0.0), 2);
        assert_eq!(placement_axis(Some(2), 0.0), 0);
        assert_eq!(placement_axis(Some(1), 0.0), 0); // a vertical cogwheel doesn't count
        assert_eq!(placement_axis(None, 0.0), 0); // facing south: shaft along x
        assert_eq!(placement_axis(None, 90.0), 2); // facing west
        assert_eq!(placement_axis(None, -90.0), 2);
        assert_eq!(placement_axis(None, 180.0), 0);
    }
}
