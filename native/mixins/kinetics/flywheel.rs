// Kinetics: the flywheel's rules, with no game types. This file is both a test target of the crate and the
// module the bundle adds to the game as `game::kinetics_flywheel`.
//
// Ported from Create's `content/kinetics/flywheel/FlywheelBlockEntity.java` (visualSpeed, angle, tick),
// `FlywheelRenderer.java` and `FlywheelBlock.java`,
// https://github.com/Creators-of-Create/Create, branch mc1.21.1/dev, commit 4148fd8.
//
// Copyright (c) The Create Team / The Creators of Create
// Licensed under the MIT License:
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// The flywheel is a shaft for the network (runtime.rs `kind_of`: `hasShaftTowards` along its axis only, no
// stress impact) with a wheel drawn over the shaft. Only the wheel's easing lives here.
//
// shortcut: `LerpedFloat`/`Chaser.EXP` (Create's catnip library, not in the repo) are written from memory:
// each tick the speed moves by 1/64 of the way to the target and snaps within 0.001; check against catnip
// if the spin-up ever looks off.

use std::collections::HashMap;
use std::sync::Mutex;

pub type Pos = [i32; 3];

pub const BLOCK: &str = "kinetics:flywheel";
/// The wheel, built along the y axis (Create's `flywheel/block`, an OBJ Horizon can't read: rebuilt in JSON).
pub const WHEEL_MODEL: &str = "kinetics:block/flywheel_part";

/// `visualSpeed.chase(.., 1 / 64f, Chaser.EXP)`.
const CHASE: f32 = 1.0 / 64.0;

#[derive(Clone, Copy, Debug, Default)]
struct Spin {
    tick: i64,
    angle: f32,
    prev: f32,
    value: f32,
}
static SPINS: Mutex<Option<HashMap<Pos, Spin>>> = Mutex::new(None);

/// The wheel's angle in degrees at `pos` at `ticks` (game time plus the partial tick) while its network gives it
/// `rpm`. `FlywheelBlockEntity.tick`: `visualSpeed` chases the speed, `angle += visualSpeed * 3 / 10`;
/// `FlywheelRenderer`: `angle + visualSpeed(partial) * 3 / 10 * partial`.
pub fn spin(pos: Pos, ticks: f64, rpm: f32) -> f32 {
    let whole = ticks.floor() as i64;
    let mut all = SPINS.lock().unwrap_or_else(|e| e.into_inner());
    let s = all
        .get_or_insert_with(HashMap::new)
        .entry(pos)
        .or_insert(Spin { tick: whole, ..Spin::default() });
    if whole < s.tick || whole - s.tick > 1200 {
        // Time jumped (another world, a long wait): no winding of the lost time.
        *s = Spin { tick: whole, angle: s.angle, prev: s.value, value: s.value };
    }
    while s.tick < whole {
        s.prev = s.value;
        let diff = rpm - s.value;
        s.value = if diff.abs() < 1e-3 { rpm } else { s.value + diff * CHASE };
        s.angle = (s.angle + s.value * 3.0 / 10.0) % 360.0;
        s.tick += 1;
    }
    let pt = (ticks - whole as f64) as f32;
    let speed = (s.prev + (s.value - s.prev) * pt) * 3.0 / 10.0;
    (s.angle + speed * pt) % 360.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_wheel_rests_and_winds_up_towards_the_network_speed() {
        let p = [1, 1, 1];
        assert_eq!(spin(p, 100.0, 64.0), 0.0);
        let first = spin(p, 101.0, 64.0);
        // One tick: the speed went 1/64 of the way to 64 rpm, that is 1 rpm = 0.3 degrees.
        assert!((first - 0.3).abs() < 1e-4, "{first}");
        let mut last = first;
        let mut step = 0.0;
        for t in 102..1500 {
            let a = spin(p, t as f64, 64.0);
            step = (a - last).rem_euclid(360.0);
            last = a;
        }
        // At speed: 64 rpm turns 19.2 degrees a tick.
        assert!((step - 19.2).abs() < 0.01, "{step}");
    }

    #[test]
    fn a_released_wheel_keeps_turning_a_while() {
        let p = [2, 2, 2];
        for t in 100..1500 {
            spin(p, t as f64, 64.0);
        }
        let a = spin(p, 1500.0, 0.0);
        let b = spin(p, 1501.0, 0.0);
        let c = spin(p, 1502.0, 0.0);
        let (d1, d2) = ((b - a).rem_euclid(360.0), (c - b).rem_euclid(360.0));
        assert!(d1 > 0.0 && d2 > 0.0 && d2 < d1, "{d1} {d2}");
    }

    #[test]
    fn the_partial_tick_blends_between_ticks() {
        let p = [3, 3, 3];
        for t in 100..400 {
            spin(p, t as f64, 32.0);
        }
        let a = spin(p, 400.0, 32.0);
        let mid = spin(p, 400.5, 32.0);
        let b = spin(p, 401.0, 32.0);
        let (h, f) = ((mid - a).rem_euclid(360.0), (b - a).rem_euclid(360.0));
        assert!(h > 0.0 && h < f, "{a} {mid} {b}");
    }
}
