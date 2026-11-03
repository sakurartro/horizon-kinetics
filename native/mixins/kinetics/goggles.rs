// Kinetics: the engineer's goggles' rules, with no game types: the bundle adds this file to the game as
// `game::kinetics_goggles`, and `cargo test` runs its tests (native/Cargo.toml `[[test]] goggles`).
//
// Ported from Create's `content/equipment/goggles/GogglesItem.java` (`isWearingGoggles`: the helmet slot holds the
// goggles) and `GoggleOverlayRenderer.java` (looking at a kinetic block shows its speed),
// https://github.com/Creators-of-Create/Create, branch mc1.21.1/dev, commit 4148fd8.
//
// Copyright (c) The Create Team / The Creators of Create
// Licensed under the MIT License; the full notice is in runtime.rs.
//
// What differs from Create: only the speed is shown (stress and impact do not exist in this mod yet), as a line
// under the crosshair like the creative motor's value box.

pub const GOGGLES: &str = "kinetics:goggles";

/// The speed to show for a block entity's two saved speeds (a clutch or gearshift has a second half): the one
/// that turns faster, so a half that still turns is not hidden by one that stopped.
pub fn pick(a: f32, b: Option<f32>) -> f32 {
    match b {
        Some(b) if b.abs() > a.abs() => b,
        _ => a,
    }
}

/// The overlay line: the size in RPM and the way (negative: clockwise), like the motor's value box.
pub fn label(speed: f32) -> String {
    format!("{} RPM {}", speed.abs().round() as i32, if speed < 0.0 { "\u{27f3}" } else { "\u{27f2}" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_faster_half() {
        assert_eq!(pick(16.0, None), 16.0);
        assert_eq!(pick(0.0, Some(-32.0)), -32.0);
        assert_eq!(pick(8.0, Some(-4.0)), 8.0);
    }

    #[test]
    fn label_shows_size_and_way() {
        assert_eq!(label(16.0), "16 RPM \u{27f2}");
        assert_eq!(label(-256.0), "256 RPM \u{27f3}");
        assert_eq!(label(7.6), "8 RPM \u{27f2}");
        assert_eq!(label(0.0), "0 RPM \u{27f2}");
    }
}
