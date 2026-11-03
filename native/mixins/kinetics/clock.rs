// Kinetics: the cuckoo clocks' rules, with no game types. This file is both a test target of the crate and the
// module the bundle adds to the game as `game::kinetics_clock`.
//
// Ported from Create's `content/kinetics/clock/CuckooClockBlockEntity.java` (tick, moveHands), `CuckooClockBlock.java`
// (the mysterious clock) and `CuckooClockRenderer.java` (the doors and the figure),
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
// What differs from Create: the animation is stored as its start tick (`anim`, `anim_start` in the block entity) and
// every sound and particle is made by the server, so a clock that loses its speed mid-chime keeps its animation
// running on the game's clock (Create pauses it); there is no advancement and no display link source, and the
// surprise explosion has vanilla's damage type.

use std::collections::HashMap;
use std::sync::Mutex;

pub type Pos = [i32; 3];

pub const CLOCK: &str = "kinetics:cuckoo_clock";
pub const MYSTERIOUS: &str = "kinetics:mysterious_cuckoo_clock";
/// The block entity's saved animation (`NONE`, `PIG`, `CREEPER`, `SURPRISE` as 0 to 3) and the game tick it began at.
pub const ANIM_KEY: &str = "anim";
pub const START_KEY: &str = "anim_start";

pub const NONE: i32 = 0;
pub const PIG: i32 = 1;
pub const CREEPER: i32 = 2;
pub const SURPRISE: i32 = 3;

pub fn is_clock(name: &str) -> bool {
    name == CLOCK || name == MYSTERIOUS
}

/// Only the overworld is natural (`DimensionType.natural`): the Nether and the End tick instead of chiming.
pub fn natural(dimension: &str) -> bool {
    dimension == "minecraft:overworld"
}

/// The hour (0 to 23) and minute the clock shows at the world's `day_time`; time runs 24 times faster where it
/// isn't natural.
pub fn time(day_time: i64, natural: bool) -> (i32, i32) {
    let day = ((day_time * if natural { 1 } else { 24 }) % 24000) as i32;
    ((day / 1000 + 6) % 24, (day % 1000) * 60 / 1000)
}

/// The animation a clock shows by itself at that time: the pig at noon, the creeper at dusk, when players can sleep;
/// the mysterious clock always shows the surprise.
pub fn trigger(hours: i32, minutes: i32, mysterious: bool) -> i32 {
    let anim = if hours == 12 && minutes < 5 {
        PIG
    } else if hours == 18 && minutes < 36 && minutes > 31 {
        CREEPER
    } else {
        return NONE;
    };
    if mysterious { SURPRISE } else { anim }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sound {
    pub event: &'static str,
    pub volume: f32,
    pub pitch: f32,
}

const HAT: &str = "minecraft:block.note_block.hat";
const CHIME: &str = "minecraft:block.note_block.chime";
const fn sound(event: &'static str, volume: f32, pitch: f32) -> Sound {
    Sound { event, volume, pitch }
}

/// The ticking of a clock with nothing to announce: slow in the day's own time, fast where time is sped up.
pub fn tick_sound(ticks: i64, natural: bool) -> Option<Sound> {
    let (long, short) = if natural { (32, 16) } else { (6, 3) };
    if ticks % long == 0 {
        Some(sound(HAT, 1.0 / 16.0, 2.0))
    } else if ticks % short == 0 {
        Some(sound(HAT, 1.0 / 16.0, 1.5))
    } else {
        None
    }
}

/// Ticks between the figure's pops: three pops of a few ticks for the surprise, two slow ones for the others.
fn step_of(anim: i32) -> i32 {
    if anim == SURPRISE { 3 } else { 15 }
}

/// The sounds of an animation `value` ticks after it began.
pub fn animation_sounds(anim: i32, value: i32) -> Vec<Sound> {
    let mut out = Vec::new();
    if value == 1 {
        out.push(sound(CHIME, 2.0, 0.5));
    }
    if value == 21 {
        out.push(sound(CHIME, 2.0, 0.793701));
    }
    if value == 40 && anim == SURPRISE {
        out.push(sound("minecraft:entity.tnt.primed", 1.0, 1.0));
    }
    let step = step_of(anim);
    for phase in (30..=60).step_by(step as usize) {
        if value == phase - step / 3 {
            out.push(sound("minecraft:block.chest.open", 1.0 / 16.0, 2.0));
        }
        if value == phase {
            out.push(if anim == PIG {
                sound("minecraft:entity.pig.ambient", 0.25, 1.0)
            } else {
                sound("minecraft:entity.creeper.hurt", 0.25, 3.0)
            });
        }
        if value == phase + step / 3 {
            out.push(sound("minecraft:block.chest.close", 1.0 / 16.0, 2.0));
        }
    }
    out
}

/// What one tick of a turning clock does.
#[derive(Debug, Default, PartialEq)]
pub struct Step {
    /// The animation now (`NONE` to `SURPRISE`) and when it began.
    pub anim: i32,
    pub start: i64,
    /// Whether `anim` or `start` changed, so the clients must be told.
    pub changed: bool,
    pub sounds: Vec<Sound>,
    /// Large smoke around the clock (the surprise, from tick 30 on).
    pub smoke: bool,
    /// The clock blows up this tick.
    pub explode: bool,
}

/// One tick, at game time `now`, of a clock that turns (`CuckooClockBlockEntity.tick`).
pub fn step(anim: i32, start: i64, now: i64, day_time: i64, natural: bool, mysterious: bool) -> Step {
    let mut out = Step { anim, start, ..Step::default() };
    if !natural {
        out.anim = NONE;
        out.changed = anim != NONE;
        out.sounds.extend(tick_sound(now, false));
        return out;
    }
    if anim == NONE {
        let (hours, minutes) = time(day_time, true);
        let next = trigger(hours, minutes, mysterious);
        if next != NONE {
            return Step { anim: next, start: now, changed: true, ..out };
        }
        out.sounds.extend(tick_sound(now, true));
        return out;
    }
    let value = (now - start) as i32;
    // Create ends it once its progress passes 100, counted from the tick after it began.
    if value > 101 {
        return Step { anim: NONE, changed: true, ..out };
    }
    out.sounds = animation_sounds(anim, value);
    out.smoke = anim == SURPRISE && value > 30;
    out.explode = anim == SURPRISE && value == 50;
    out
}

/// The angle in degrees the doors are open at `value` ticks (partial included) into an animation
/// (`CuckooClockRenderer`): 135 at the most.
pub fn door_angle(anim: i32, value: f32) -> f32 {
    let step = step_of(anim);
    let mut angle = 0.0;
    for phase in (30..=60).step_by(step as usize) {
        let local = value - phase as f32;
        if local < -((step / 3) as f32) {
            continue;
        } else if local < 0.0 {
            angle = lerp((value - (phase - 5) as f32) / 5.0, 0.0, 135.0);
        } else if local < (step / 3) as f32 {
            angle = 135.0;
        } else if local < (2 * step / 3) as f32 {
            angle = lerp((value - (phase + 5) as f32) / 5.0, 135.0, 0.0);
        }
    }
    angle
}

/// How far the figure sticks out, along the clock's x in blocks: 10/16 (hidden in the clock) shut, 2/16 open.
pub fn figure_offset(angle: f32) -> f32 {
    -(angle / 135.0) / 2.0 + 10.0 / 16.0
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

/// The turn in degrees about y that takes Create's partial models (built on the west side) to a clock's `facing`
/// (`horizontalAngle(facing.getCounterClockWise())`), right-handed.
pub fn west_to(facing: &str) -> Option<f32> {
    Some(match facing {
        "west" => 0.0,
        "south" => 90.0,
        "east" => 180.0,
        "north" => -90.0,
        _ => return None,
    })
}

/// The turn in degrees about y that takes Create's block model (built facing north) to `facing`: the blockstate's
/// `y` is clockwise, this is right-handed.
pub fn north_to(facing: &str) -> Option<f32> {
    west_to(facing).map(|w| w + 90.0)
}

/// The sides a shaft may come from, with the facing name that points to each.
pub const SIDES: [([i32; 3], &str); 4] =
    [([0, 0, -1], "north"), ([0, 0, 1], "south"), ([-1, 0, 0], "west"), ([1, 0, 0], "east")];

fn opposite(name: &str) -> &'static str {
    match name {
        "north" => "south",
        "south" => "north",
        "east" => "west",
        _ => "east",
    }
}

/// The horizontal direction a player looks along (`getHorizontalDirection`) at `yaw` degrees: 0 is south, 90 west.
pub fn looking(yaw: f32) -> &'static str {
    match ((yaw.rem_euclid(360.0) + 45.0) / 90.0) as i32 % 4 {
        0 => "south",
        1 => "west",
        2 => "north",
        _ => "east",
    }
}

/// The facing a clock is placed with (`getPreferredHorizontalFacing`, `getStateForPlacement`): away from the side
/// with a shaft pointing at it, else toward the player. `shaft[i]` is for `SIDES[i]`; `looking` is the way the player
/// faces. Shafts on both axes decide nothing.
pub fn placement_facing(shaft: [bool; 4], looking: &str) -> &'static str {
    let mut found: Option<usize> = None;
    for i in (0..4).filter(|&i| shaft[i]) {
        if found.is_some_and(|f| (f < 2) != (i < 2)) {
            found = None;
            break;
        }
        found = Some(i);
    }
    opposite(found.map_or(looking, |i| SIDES[i].1))
}

// The hands as the window draws them (client side), by position.
#[derive(Clone, Copy, Debug, Default)]
struct Hand {
    value: f32,
    previous: f32,
}
#[derive(Clone, Copy, Debug)]
struct Hands {
    tick: i64,
    hour: Hand,
    minute: Hand,
}
static HANDS: Mutex<Option<HashMap<Pos, Hands>>> = Mutex::new(None);

fn shortest(current: f32, target: f32) -> f32 {
    (((target % 360.0 - current % 360.0) % 360.0 + 540.0) % 360.0) - 180.0
}

// `moveHands` for one hand: chase the target the short way, a fifth of the gap a tick (`Chaser.EXP`).
fn chase(hand: &mut Hand, target: f32) {
    let target = hand.value + shortest(hand.value, target);
    hand.previous = hand.value;
    let diff = target - hand.value;
    hand.value = if diff.abs() < 1.0 / 4096.0 { target } else { hand.value + diff * 0.2 };
}

fn at(hand: Hand, partial: f32) -> f32 {
    hand.previous + shortest(hand.previous, hand.value) * partial
}

/// The angles in degrees of the hour and minute hands of the clock at `pos` at `ticks` (game time plus the partial
/// tick) showing `hours`:`minutes`. They move only while `running` (the clock turns), a tick at a time.
pub fn hands(pos: Pos, ticks: f64, running: bool, hours: i32, minutes: i32) -> (f32, f32) {
    let whole = ticks.floor() as i64;
    let mut all = HANDS.lock().unwrap_or_else(|e| e.into_inner());
    let h = all
        .get_or_insert_with(HashMap::new)
        .entry(pos)
        .or_insert(Hands { tick: whole, hour: Hand::default(), minute: Hand::default() });
    if running {
        let (hour, minute) = ((30 * (hours % 12)) as f32, (6 * minutes) as f32);
        for _ in 0..(whole - h.tick).clamp(0, 100) {
            chase(&mut h.hour, hour);
            chase(&mut h.minute, minute);
        }
    }
    h.tick = whole;
    let partial = (ticks - whole as f64) as f32;
    (at(h.hour, partial), at(h.minute, partial))
}

/// The models the clock is drawn with: ours (a plain look in vanilla textures) and Create's (they exist only while
/// the pack from `tools/create_look.py` is on; the game logs the missing ones and draws ours).
pub const PART_MODELS: [&str; 7] = [
    "kinetics:block/clock_body_part",
    "kinetics:block/clock_hour_part",
    "kinetics:block/clock_minute_part",
    "kinetics:block/clock_left_door_part",
    "kinetics:block/clock_right_door_part",
    "kinetics:block/clock_pig_part",
    "kinetics:block/clock_creeper_part",
];
pub const LOOK_MODELS: [&str; 7] = [
    "create:block/cuckoo_clock/block",
    "create:block/cuckoo_clock/hour_hand",
    "create:block/cuckoo_clock/minute_hand",
    "create:block/cuckoo_clock/left_door",
    "create:block/cuckoo_clock/right_door",
    "create:block/cuckoo_clock/pig",
    "create:block/cuckoo_clock/creeper",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_day_starts_at_six_and_the_hours_are_a_thousand_ticks() {
        assert_eq!(time(0, true), (6, 0));
        assert_eq!(time(6000, true), (12, 0));
        assert_eq!(time(6500, true), (12, 30));
        assert_eq!(time(12000, true), (18, 0));
        assert_eq!(time(18999, true), (0, 59));
        // Elsewhere time runs 24 times faster: 125 ticks are three hours.
        assert_eq!(time(125, false), (9, 0));
        assert_eq!(time(1000, false), (6, 0));
    }

    #[test]
    fn noon_brings_the_pig_and_dusk_the_creeper_and_a_mysterious_clock_the_surprise() {
        assert_eq!(trigger(12, 0, false), PIG);
        assert_eq!(trigger(12, 4, false), PIG);
        assert_eq!(trigger(12, 5, false), NONE);
        assert_eq!(trigger(18, 31, false), NONE);
        assert_eq!(trigger(18, 32, false), CREEPER);
        assert_eq!(trigger(18, 35, false), CREEPER);
        assert_eq!(trigger(18, 36, false), NONE);
        assert_eq!(trigger(12, 0, true), SURPRISE);
        assert_eq!(trigger(11, 0, true), NONE);
    }

    #[test]
    fn a_clock_starts_chimes_and_ends() {
        // 6000 is noon.
        let s = step(NONE, 0, 100, 6000, true, false);
        assert_eq!((s.anim, s.start, s.changed), (PIG, 100, true));
        let mut heard = Vec::new();
        let mut cur = (s.anim, s.start);
        for now in 101..210 {
            let s = step(cur.0, cur.1, now, 6000, true, false);
            heard.extend(s.sounds.iter().map(|x| (now - 100, x.event)));
            assert!(!s.explode && !s.smoke);
            cur = (s.anim, s.start);
            if s.changed {
                assert_eq!(now - 100, 102);
                break;
            }
        }
        assert_eq!(cur.0, NONE);
        assert!(heard.contains(&(1, CHIME)) && heard.contains(&(21, CHIME)));
        let oinks: Vec<_> = heard.iter().filter(|s| s.1 == "minecraft:entity.pig.ambient").map(|s| s.0).collect();
        assert_eq!(oinks, vec![30, 45, 60]);
        // A creeper hurts instead.
        assert!(animation_sounds(CREEPER, 30).iter().any(|s| s.event == "minecraft:entity.creeper.hurt" && s.pitch == 3.0));
    }

    #[test]
    fn the_surprise_smokes_and_blows_up_once() {
        let mut cur = (SURPRISE, 0i64);
        let mut booms = Vec::new();
        for now in 1..60 {
            let s = step(cur.0, cur.1, now, 6000, true, true);
            assert_eq!(s.smoke, now > 30);
            if s.explode {
                booms.push(now);
            }
            cur = (s.anim, s.start);
        }
        assert_eq!(booms, vec![50]);
        assert!(animation_sounds(SURPRISE, 40).iter().any(|s| s.event == "minecraft:entity.tnt.primed"));
    }

    #[test]
    fn where_time_is_not_natural_the_clock_only_ticks() {
        let s = step(NONE, 0, 6, 6000, false, true);
        assert_eq!((s.anim, s.changed), (NONE, false));
        assert_eq!(s.sounds, vec![sound(HAT, 1.0 / 16.0, 2.0)]);
        assert_eq!(tick_sound(3, false).map(|s| s.pitch), Some(1.5));
        assert_eq!(tick_sound(4, false), None);
        assert_eq!(tick_sound(16, true).map(|s| s.pitch), Some(1.5));
        assert_eq!(tick_sound(32, true).map(|s| s.pitch), Some(2.0));
        assert_eq!(tick_sound(8, true), None);
    }

    #[test]
    fn the_doors_open_for_the_figure_and_shut_again() {
        // The pig: doors open from tick 25 to 30, stay till 35, shut at 40; the same again at 45.
        assert_eq!(door_angle(PIG, 20.0), 0.0);
        assert!((door_angle(PIG, 27.5) - 67.5).abs() < 1e-3);
        assert_eq!(door_angle(PIG, 32.0), 135.0);
        assert!((door_angle(PIG, 37.5) - 67.5).abs() < 1e-3);
        assert_eq!(door_angle(PIG, 40.0), 0.0);
        assert_eq!(door_angle(PIG, 50.0), 135.0);
        assert_eq!(figure_offset(0.0), 0.625);
        assert!((figure_offset(135.0) - 0.125).abs() < 1e-6);
        assert_eq!(door_angle(SURPRISE, 30.0), 135.0);
    }

    #[test]
    fn a_clock_faces_away_from_the_shaft_beside_it_else_toward_the_player() {
        // Shaft on the north side: the clock faces south, its back to the shaft.
        assert_eq!(placement_facing([true, false, false, false], "east"), "south");
        assert_eq!(placement_facing([false, false, false, true], "east"), "west");
        // Shafts on two axes: the player decides, and the clock faces them.
        assert_eq!(placement_facing([true, false, true, false], "north"), "south");
        // Two on one axis: the later wins.
        assert_eq!(placement_facing([true, true, false, false], "north"), "north");
        assert_eq!(placement_facing([false; 4], "west"), "east");
    }

    #[test]
    fn the_player_looks_along_the_nearest_side() {
        assert_eq!(looking(0.0), "south");
        assert_eq!(looking(44.0), "south");
        assert_eq!(looking(90.0), "west");
        assert_eq!(looking(-90.0), "east");
        assert_eq!(looking(180.0), "north");
        assert_eq!(looking(-179.0), "north");
        assert_eq!(looking(359.0), "south");
    }

    #[test]
    fn partial_models_turn_from_the_west_and_the_block_from_the_north() {
        // A right-handed turn of +90 about y takes west (-x) to south (+z).
        assert_eq!(west_to("south"), Some(90.0));
        assert_eq!(west_to("north"), Some(-90.0));
        assert_eq!(north_to("north"), Some(0.0));
        // Blockstate `y: 90` (clockwise, north to east) is -90 right-handed, 270 the same.
        assert_eq!(north_to("east"), Some(270.0));
        assert_eq!(west_to("up"), None);
    }

    #[test]
    fn hands_chase_the_time_a_tick_at_a_time_and_wait_when_stopped() {
        let p = [5, 5, 5];
        // The first look starts them at 0; at 3:00 the hour hand heads for 90 degrees, a fifth of the gap a tick,
        // and is drawn the tick before (partial 0 is the previous value).
        assert_eq!(hands(p, 100.0, true, 3, 0), (0.0, 0.0));
        assert_eq!(hands(p, 101.0, true, 3, 0).0, 0.0);
        assert!((hands(p, 102.0, true, 3, 0).0 - 18.0).abs() < 1e-3);
        assert!((hands(p, 103.0, true, 3, 0).0 - 32.4).abs() < 1e-3);
        // Stopped: they stay where they are however long it takes.
        assert!((hands(p, 150.0, false, 3, 0).0 - 32.4).abs() < 1e-3);
        for t in 151..400 {
            hands(p, t as f64, true, 3, 0);
        }
        let (h, m) = hands(p, 400.0, true, 3, 0);
        assert!((h - 90.0).abs() < 0.01 && m == 0.0, "{h}");
        // 59 minutes is 354 degrees, reached the short way round (-6) from 0.
        let q = [6, 6, 6];
        hands(q, 0.0, true, 12, 0);
        for t in 1..200 {
            hands(q, t as f64, true, 11, 59);
        }
        let (_, m) = hands(q, 200.0, true, 11, 59);
        assert!((m + 6.0).abs() < 0.01, "{m}");
    }
}
