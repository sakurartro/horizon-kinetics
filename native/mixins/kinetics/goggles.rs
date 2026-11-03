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
// Stress lines: `KineticBlockEntity.addToTooltip` / `addToGoggleTooltip` / `addStressImpactStats` and
// `GeneratingKineticBlockEntity.addToGoggleTooltip`, with the values of stress.rs.
//
// What differs from Create: lines under the crosshair like the creative motor's value box, not a tooltip box;
// English only; a generator's speed is its block's speed.

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

const GOLD: u32 = 0xFFAA00;
const GRAY: u32 = 0xAAAAAA;
const AQUA: u32 = 0x55FFFF;

fn su(value: f32) -> String {
    format!("{} su", (value * 100.0).round() / 100.0)
}

/// The stress lines (text, RGB) for a block: its base `impact` and `capacity` (su per RPM, stress.rs), the speed
/// the network `theoretical`ly gives it, and whether the network `overstressed` (stopped). Overstressed: Create's
/// warning in place of the rest; else the impact of a block that has one and the capacity of a generator.
pub fn stress_lines(overstressed: bool, impact: f32, capacity: f32, theoretical: f32) -> Vec<(String, u32)> {
    let mut out: Vec<(String, u32)> = Vec::new();
    if overstressed {
        out.push(("Overstressed".into(), GOLD));
        for l in ["It appears that this contraption is overstressed.", "Add more sources or slow down the", "components with a high stress impact."] {
            out.push((l.into(), GRAY));
        }
        return out;
    }
    if impact != 0.0 {
        out.push(("Kinetic Stats:".into(), GRAY));
        out.push(("Kinetic Stress Impact:".into(), GRAY));
        out.push((format!(" {} at current speed", su(impact * theoretical.abs())), AQUA));
    }
    if capacity != 0.0 {
        out.push(("Generator Stats:".into(), GRAY));
        out.push(("Kinetic Stress Capacity:".into(), GRAY));
        out.push((format!(" {} at current speed", su(capacity * theoretical.abs())), AQUA));
    }
    out
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

    #[test]
    fn stress_lines_follow_create() {
        assert!(stress_lines(false, 0.0, 0.0, 16.0).is_empty());
        let l = stress_lines(false, 4.0, 0.0, -16.0);
        assert_eq!(l[2].0, " 64 su at current speed");
        let g = stress_lines(false, 0.0, 8.0, 16.0);
        assert_eq!((g[0].0.as_str(), g[2].0.as_str()), ("Generator Stats:", " 128 su at current speed"));
        assert_eq!(stress_lines(true, 4.0, 8.0, 16.0)[0].0, "Overstressed");
    }
}
