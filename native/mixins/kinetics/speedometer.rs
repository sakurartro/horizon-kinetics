// Kinetics: the speedometer's pure rules, with no game types: the bundle adds this file to the game as
// `game::kinetics_speedometer`, and `cargo test` runs its tests (native/Cargo.toml `[[test]] speedometer`).
//
// Ported from Create's `content/kinetics/gauge/` (`GaugeBlock.java`: `getStateForPlacement`, `shouldRenderHeadOnFace`,
// `getAnalogOutputSignal`; `SpeedGaugeBlockEntity.java`: `getDialTarget`, the colour, the goggles text;
// `GaugeBlockEntity.java`: the dial's easing; `GaugeRenderer.java`: the dial's pivot and turn), `base/DirectionalAxisKineticBlock.java`
// (`getRotationAxis`) and `base/IRotate.java` (`SpeedLevel`), and the blockstate Create's datagen wrote for it,
// https://github.com/Creators-of-Create/Create, branch mc1.21.1/dev, commit 4148fd8.
//
// Copyright (c) The Create Team / The Creators of Create
// Licensed under the MIT License; the full notice is in runtime.rs.
//
// What differs from Create: Create's server config is fixed at its defaults (medium 30, fast 100, max 256 RPM); the
// dust particles of a turning gauge (`animateTick`) are not made (no particle API for them); a head is also drawn on a
// face a solid neighbour covers (hidden inside that block anyway); the goggles text is in English and drawn by our
// own box, not Create's tooltip.

pub type Pos = [i32; 3];

pub const BLOCK: &str = "kinetics:speedometer";

/// Create's defaults for `kinetics.mediumSpeed`, `fastSpeed` and `maxRotationSpeed`.
pub const MEDIUM: f32 = 30.0;
pub const FAST: f32 = 100.0;
pub const MAX: f32 = 256.0;

/// Create's models (only with the pack from `tools/create_look.py` on): the two bodies, the dial and the head.
pub const LOOK_BASE: &str = "create:block/gauge/base";
pub const LOOK_WALL: &str = "create:block/gauge/base_wall";
pub const LOOK_DIAL: &str = "create:block/gauge/dial";
pub const LOOK_HEAD: &str = "create:block/gauge/speedometer/head";
pub const LOOK_MODELS: [&str; 4] = [LOOK_BASE, LOOK_WALL, LOOK_DIAL, LOOK_HEAD];

const NORTH: Pos = [0, 0, -1];
const SOUTH: Pos = [0, 0, 1];
const WEST: Pos = [-1, 0, 0];
const EAST: Pos = [1, 0, 0];
const UP: Pos = [0, 1, 0];
const DOWN: Pos = [0, -1, 0];
/// The four horizontal sides, as `Direction.toYRot` has them (south 0, west 90, north 180, east 270).
pub const HORIZONTAL: [Pos; 4] = [SOUTH, WEST, NORTH, EAST];

pub fn dir_vec(name: &str) -> Option<Pos> {
    Some(match name {
        "north" => NORTH,
        "south" => SOUTH,
        "west" => WEST,
        "east" => EAST,
        "up" => UP,
        "down" => DOWN,
        _ => return None,
    })
}

pub fn dir_name(v: Pos) -> &'static str {
    match v {
        NORTH => "north",
        SOUTH => "south",
        WEST => "west",
        EAST => "east",
        UP => "up",
        _ => "down",
    }
}

fn axis(v: Pos) -> usize {
    (0..3).find(|&a| v[a] != 0).unwrap_or(1)
}

fn opposite(v: Pos) -> Pos {
    v.map(|c| -c)
}

/// The axis the shaft goes along (`DirectionalAxisKineticBlock.getRotationAxis`), 0 = x, 1 = y, 2 = z.
pub fn rotation_axis(facing: Pos, along_first: bool) -> usize {
    match (axis(facing), along_first) {
        (0, true) => 1,
        (0, false) => 2,
        (1, true) => 0,
        (1, false) => 2,
        (_, true) => 0,
        (_, false) => 1,
    }
}

/// `GaugeBlock.shouldRenderHeadOnFace` without the neighbour check: whether a dial and head sit on the horizontal side `face`.
pub fn head_on_face(facing: Pos, along_first: bool, face: Pos) -> bool {
    if face[1] != 0 || face == opposite(facing) {
        return false;
    }
    let rot = rotation_axis(facing, along_first);
    axis(face) != rot && (rot != 1 || face == facing)
}

/// The yaw (degrees about up) `GaugeRenderer.rotateBufferTowards` turns a model built on the east side to `face`.
pub fn face_yaw(face: Pos) -> f32 {
    // -toYRot - 90
    match face {
        EAST => 0.0,
        SOUTH => -90.0,
        WEST => -180.0,
        _ => 90.0,
    }
}

/// Create's blockstate for the gauge: the body model (`true`: the wall one) and its `x` and `y` rotations (blockstate
/// degrees, clockwise).
pub fn variant(facing: Pos, along_first: bool) -> (bool, i32, i32) {
    match (facing, along_first) {
        (DOWN, false) => (false, 180, 0),
        (EAST, false) => (true, 0, 0),
        (NORTH, false) => (true, 90, 270),
        (SOUTH, false) => (true, 90, 90),
        (UP, false) => (false, 0, 0),
        (WEST, false) => (true, 0, 180),
        (DOWN, true) => (false, 180, 90),
        (EAST, true) => (true, 90, 0),
        (NORTH, true) => (true, 0, 270),
        (SOUTH, true) => (true, 0, 90),
        (UP, true) => (false, 0, 90),
        _ => (true, 90, 180),
    }
}

/// The turns, outermost first, that put Create's body model in the blockstate's place: a draw's `rotate_about`
/// about the block's centre. The blockstate turns x first, then y, both clockwise.
pub fn look_turns(facing: Pos, along_first: bool) -> [([f32; 3], f32); 2] {
    let (_, x, y) = variant(facing, along_first);
    [([0.0, 1.0, 0.0], -(y as f32)), ([1.0, 0.0, 0.0], -(x as f32))]
}

/// The horizontal side a player looking along `yaw` (degrees) faces (`getHorizontalDirection`).
pub fn horizontal(yaw: f32) -> Pos {
    let (sin, cos) = f64::from(yaw).to_radians().sin_cos();
    if cos.abs() >= sin.abs() {
        if cos >= 0.0 { SOUTH } else { NORTH }
    } else if sin < 0.0 {
        EAST
    } else {
        WEST
    }
}

/// The side a player looking along `yaw` and `pitch` looks to the most (`getNearestLookingDirection`).
pub fn nearest_looking(yaw: f32, pitch: f32) -> Pos {
    let (yaw, pitch) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    let look = [-yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos()];
    let a = (0..3).max_by(|&i, &j| look[i].abs().total_cmp(&look[j].abs())).unwrap_or(1);
    let mut v = [0; 3];
    v[a] = if look[a] < 0.0 { -1 } else { 1 };
    v
}

/// The state a gauge is placed in (`GaugeBlock.getStateForPlacement` over `DirectionalAxisKineticBlock`'s): the
/// facing and `axis_along_first`. `face` is the face clicked, `look` the nearest looking direction, `yaw` the player's;
/// `shaft_back` says the clicked block has a shaft toward `face`, `shaft(side)` whether the neighbour on `side` has
/// one toward the new gauge.
pub fn placed(face: Pos, look: Pos, yaw: f32, shaft_back: bool, shaft: impl Fn(Pos) -> bool) -> (Pos, bool) {
    let horizontal = horizontal(yaw);
    if shaft_back {
        let positive = look.iter().sum::<i32>() > 0;
        return match axis(face) {
            0 => (if positive { NORTH } else { SOUTH }, true),
            1 => (opposite(horizontal), axis(horizontal) == 0),
            _ => (if positive { WEST } else { EAST }, false),
        };
    }
    // `getFacingForPlacement` is the clicked face.
    let a = axis(face);
    if a != 1 {
        let along = a == 2;
        // A shaft above or below (the sideways ones never count: `prefersConnectionTo(.., false)`).
        return (face, if shaft(UP) || shaft(DOWN) { a == 0 } else { along });
    }
    // `getAxisAlignmentForPlacement`: the player's axis is not X.
    let mut along = axis(horizontal) != 0;
    let mut preferred: Option<Pos> = None;
    for side in HORIZONTAL {
        if !shaft(side) {
            continue;
        }
        if preferred.is_some_and(|p| axis(p) != axis(side)) {
            preferred = None;
            break;
        }
        preferred = Some(side);
    }
    if let Some(p) = preferred {
        along = axis(p) == 0;
    }
    (face, along)
}

/// How far the needle goes for `speed` (`SpeedGaugeBlockEntity.getDialTarget`): 0 to 1.125.
pub fn dial_target(speed: f32) -> f32 {
    let speed = speed.abs();
    let lerp = |t: f32, a: f32, b: f32| a + (b - a) * t;
    if speed == 0.0 {
        0.0
    } else if speed < MEDIUM {
        lerp(speed / MEDIUM, 0.0, 0.45)
    } else if speed < FAST {
        lerp((speed - MEDIUM) / (FAST - MEDIUM), 0.45, 0.75)
    } else {
        lerp((speed - FAST) / (MAX - FAST), 0.75, 1.125)
    }
}

/// The comparator's signal (`getAnalogOutputSignal`): the dial target times 14, rounded up, at most 15.
pub fn analog(speed: f32) -> i32 {
    (dial_target(speed) * 14.0).clamp(0.0, 15.0).ceil() as i32
}

/// `IRotate.SpeedLevel.of`: 0 none, 1 slow, 2 medium, 3 fast.
pub fn level(speed: f32) -> usize {
    let speed = speed.abs();
    if speed >= FAST {
        3
    } else if speed >= MEDIUM {
        2
    } else if speed >= 1.0 {
        1
    } else {
        0
    }
}

/// The level's colour (`SpeedLevel.getColor`).
pub fn level_color(level: usize) -> u32 {
    [0x000000, 0x22FF22, 0x0084FF, 0xFF55FF][level]
}

/// The gauge's colour for `speed` (`Color.mixColors(level colour, white, .25)`): what its particles are.
pub fn color(speed: f32) -> u32 {
    let c = level_color(level(speed));
    let mix = |shift: u32| {
        let a = ((c >> shift) & 0xFF) as f32;
        (a + (255.0 - a) * 0.25).round() as u32
    };
    (mix(16) << 16) | (mix(8) << 8) | mix(0)
}

/// The level's text colour (`SpeedLevel.getTextColor`: dark gray, green, aqua, light purple).
pub fn text_color(level: usize) -> u32 {
    [0x555555, 0x55FF55, 0x55FFFF, 0xFF55FF][level]
}

/// `CreateLang.number`: at most two decimals, none when whole.
pub fn number(v: f32) -> String {
    let s = format!("{:.2}", v.abs());
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The goggles' lines for a gauge turning at `speed` (`GaugeBlockEntity` / `SpeedGaugeBlockEntity.addToGoggleTooltip`),
/// each with its 0xRRGGBB colour: white header, gray title, and the speed line in its level's colour with the
/// 3-wide bar (`TooltipHelper.makeProgressBar`).
pub fn tooltip(speed: f32) -> [(String, u32); 3] {
    let l = level(speed);
    let bar = format!(" {}{} ", "\u{2588}".repeat(l), "\u{2592}".repeat(3 - l));
    let name = ["None", "Slow", "Moderate", "Fast"][l];
    [
        ("Gauge Information:".to_string(), 0xFFFFFF),
        ("Rotation Speed".to_string(), 0xAAAAAA),
        (format!("{bar}{name} ({} RPM) ", number(speed)), text_color(l)),
    ]
}

/// The needle's easing (`GaugeBlockEntity.tick`), run once per game tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Dial {
    pub state: f32,
    pub prev: f32,
}

impl Dial {
    /// One tick toward `target`; `rand` is a number in 0..1 (the jitter of a needle past the end).
    pub fn tick(&mut self, target: f32, rand: impl Fn() -> f32) {
        self.prev = self.state;
        self.state += (target - self.state) * 0.125;
        if self.state > 1.0 && rand() < 0.5 {
            self.state -= (self.state - 1.0) * rand();
        }
    }

    /// The needle's place `partial` of the way to the next tick.
    pub fn progress(&self, partial: f32) -> f32 {
        self.prev + (self.state - self.prev) * partial
    }
}

/// The turn of the dial about its pivot, in degrees (`PI / 2 * -progress` about east).
pub fn dial_degrees(progress: f32) -> f32 {
    -90.0 * progress
}

/// The dial's pivot, in blocks along y and z (`dialPivot`).
pub const DIAL_PIVOT: f32 = 5.75 / 16.0;

/// A cheap number in 0..1 from a seed (splitmix64), for the needle's jitter.
pub fn noise(seed: u64) -> f32 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) >> 40) as f32 / (1u64 << 24) as f32
}

// The dials seen so far on the client, each with the game tick it was last eased at.
static DIALS: std::sync::Mutex<Vec<(Pos, Dial, i64)>> = std::sync::Mutex::new(Vec::new());

/// The dial at `pos`, eased up to game tick `now` toward `target`. shortcut: the client's list is never pruned and
/// is searched linearly; prune it (or key it by the block entity) if worlds hold hundreds of gauges.
pub fn dial_at(pos: Pos, now: i64, target: f32) -> Dial {
    let mut dials = DIALS.lock().unwrap_or_else(|e| e.into_inner());
    let i = dials.iter().position(|d| d.0 == pos).unwrap_or_else(|| {
        dials.push((pos, Dial::default(), now));
        dials.len() - 1
    });
    let (_, dial, last) = &mut dials[i];
    // At most a second of catching up after a gap.
    for t in (*last + 1).max(now - 20)..=now {
        let seed = (t as u64) ^ ((pos[0] as u64) << 40) ^ ((pos[1] as u64) << 20) ^ (pos[2] as u64);
        let n = std::cell::Cell::new(seed);
        dial.tick(target, || {
            n.set(n.get().wrapping_add(1));
            noise(n.get())
        });
    }
    *last = now;
    *dial
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(v: [f32; 3], axis: [f32; 3], degrees: f32) -> [f32; 3] {
        // Rodrigues, counterclockwise seen from the axis' positive end.
        let t = degrees.to_radians();
        let d = axis[0] * v[0] + axis[1] * v[1] + axis[2] * v[2];
        let c = [axis[1] * v[2] - axis[2] * v[1], axis[2] * v[0] - axis[0] * v[2], axis[0] * v[1] - axis[1] * v[0]];
        [0, 1, 2].map(|i| v[i] * t.cos() + c[i] * t.sin() + axis[i] * d * (1.0 - t.cos()))
    }

    fn through(turns: [([f32; 3], f32); 2], v: [f32; 3]) -> [i32; 3] {
        // The last turn is applied to the vertex first.
        let v = turn(v, turns[1].0, turns[1].1);
        let v = turn(v, turns[0].0, turns[0].1);
        v.map(|c| c.round() as i32)
    }

    #[test]
    fn the_body_models_face_and_hold_the_shaft_like_the_blockstate_says() {
        for facing in [NORTH, SOUTH, WEST, EAST, UP, DOWN] {
            for along in [false, true] {
                let (wall, ..) = variant(facing, along);
                assert_eq!(wall, axis(facing) != 1);
                // The wall body faces east and the standing one up, both with the shaft along z, unturned.
                let at_rest = if wall { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
                let turns = look_turns(facing, along);
                assert_eq!(through(turns, at_rest), facing, "{facing:?} {along}");
                let shaft = through(turns, [0.0, 0.0, 1.0]).map(i32::abs);
                assert_eq!(axis(shaft), rotation_axis(facing, along), "{facing:?} {along}");
            }
        }
    }

    #[test]
    fn the_rotation_axis_follows_the_facing() {
        assert_eq!(rotation_axis(UP, false), 2);
        assert_eq!(rotation_axis(UP, true), 0);
        assert_eq!(rotation_axis(EAST, false), 2);
        assert_eq!(rotation_axis(EAST, true), 1);
        assert_eq!(rotation_axis(NORTH, true), 0);
        assert_eq!(rotation_axis(NORTH, false), 1);
    }

    #[test]
    fn heads_sit_on_the_sides_that_show_them() {
        let sides = |f, a| HORIZONTAL.into_iter().filter(|&s| head_on_face(f, a, s)).count();
        // A gauge on a floor, shaft along z: heads on both x sides.
        assert!(head_on_face(UP, false, EAST) && head_on_face(UP, false, WEST));
        assert_eq!(sides(UP, false), 2);
        // On a wall facing east, shaft along z: a head on its face, none on the back.
        assert!(head_on_face(EAST, false, EAST) && !head_on_face(EAST, false, WEST));
        assert!(!head_on_face(EAST, false, NORTH)); // the shaft's end
        // A shaft along y: only the facing side.
        assert_eq!(sides(EAST, true), 1);
        assert!(head_on_face(EAST, true, EAST));
        assert_eq!(face_yaw(EAST), 0.0);
        assert_eq!(face_yaw(NORTH), 90.0);
    }

    #[test]
    fn it_is_placed_on_a_shaft_end_or_facing_the_clicked_face() {
        let none = |_: Pos| false;
        // On the side of a shaft along x (clicked the east face), looking east: facing south, along first.
        assert_eq!(placed(EAST, EAST, 270.0, true, none), (NORTH, true));
        assert_eq!(placed(EAST, WEST, 90.0, true, none), (SOUTH, true));
        assert_eq!(placed(NORTH, NORTH, 180.0, true, none), (EAST, false));
        // On top of a vertical shaft, the player looking east: faces west, shaft along y.
        assert_eq!(placed(UP, DOWN, 270.0, true, none), (WEST, true));
        // Plain: the clicked face.
        assert_eq!(placed(NORTH, NORTH, 180.0, false, none), (NORTH, true));
        assert_eq!(placed(EAST, EAST, 270.0, false, none), (EAST, false));
        assert_eq!(placed(EAST, EAST, 270.0, false, |s| s == UP), (EAST, true));
        // On a floor: across the player's axis, or toward a neighbouring shaft.
        assert_eq!(placed(UP, DOWN, 0.0, false, none), (UP, true));
        assert_eq!(placed(UP, DOWN, 270.0, false, none), (UP, false));
        assert_eq!(placed(UP, DOWN, 0.0, false, |s| s == WEST), (UP, true));
        assert_eq!(placed(UP, DOWN, 0.0, false, |s| s == NORTH), (UP, false));
        assert_eq!(placed(UP, DOWN, 0.0, false, |s| s == NORTH || s == EAST), (UP, true));
    }

    #[test]
    fn the_player_looks_as_minecraft_says() {
        assert_eq!(horizontal(0.0), SOUTH);
        assert_eq!(horizontal(90.0), WEST);
        assert_eq!(horizontal(180.0), NORTH);
        assert_eq!(horizontal(-90.0), EAST);
        assert_eq!(nearest_looking(0.0, 0.0), SOUTH);
        assert_eq!(nearest_looking(0.0, 80.0), DOWN);
        assert_eq!(nearest_looking(0.0, -80.0), UP);
    }

    #[test]
    fn the_needle_follows_the_speed() {
        assert_eq!(dial_target(0.0), 0.0);
        assert!((dial_target(15.0) - 0.225).abs() < 1e-6);
        assert!((dial_target(30.0) - 0.45).abs() < 1e-6);
        assert!((dial_target(-65.0) - 0.6).abs() < 1e-6);
        assert!((dial_target(100.0) - 0.75).abs() < 1e-6);
        assert!((dial_target(256.0) - 1.125).abs() < 1e-6);
        assert_eq!(dial_degrees(0.5), -45.0);
    }

    #[test]
    fn the_comparator_reads_the_needle() {
        assert_eq!(analog(0.0), 0);
        assert_eq!(analog(16.0), 4); // 0.2400 * 14 = 3.36
        assert_eq!(analog(100.0), 11); // 0.75 * 14 = 10.5
        assert_eq!(analog(256.0), 15); // 15.75 clamped
    }

    #[test]
    fn colours_and_levels_are_creates() {
        assert_eq!([0.0, 0.5, 1.0, 29.9, 30.0, 99.0, 100.0, -256.0].map(level), [0, 0, 1, 1, 2, 2, 3, 3]);
        assert_eq!(color(0.0), 0x404040);
        assert_eq!(color(16.0), 0x59FF59); // 0x22FF22 mixed a quarter toward white
        assert_eq!(color(50.0), 0x40A3FF);
        assert_eq!(color(200.0), 0xFF80FF);
    }

    #[test]
    fn the_goggles_say_what_create_says() {
        let t = tooltip(-16.0);
        assert_eq!(t[0], ("Gauge Information:".to_string(), 0xFFFFFF));
        assert_eq!(t[1].0, "Rotation Speed");
        assert_eq!(t[2], (" \u{2588}\u{2592}\u{2592} Slow (16 RPM) ".to_string(), 0x55FF55));
        assert_eq!(tooltip(0.0)[2].0, " \u{2592}\u{2592}\u{2592} None (0 RPM) ");
        assert_eq!(tooltip(128.5)[2], (" \u{2588}\u{2588}\u{2588} Fast (128.5 RPM) ".to_string(), 0xFF55FF));
        assert_eq!(number(7.256), "7.26");
    }

    #[test]
    fn the_dial_eases_toward_its_target() {
        let mut d = Dial::default();
        d.tick(0.8, || 0.9);
        assert!((d.state - 0.1).abs() < 1e-6 && d.prev == 0.0);
        assert!((d.progress(0.5) - 0.05).abs() < 1e-6);
        for _ in 0..200 {
            d.tick(0.8, || 0.9);
        }
        assert!((d.state - 0.8).abs() < 1e-4);
        // Past the end it falls back by a random part of the excess half the time.
        let mut d = Dial { state: 1.1, prev: 1.1 };
        d.tick(1.125, || 0.25);
        assert!(d.state < 1.1 + (1.125 - 1.1) * 0.125);
    }

    #[test]
    fn dials_are_remembered_per_position() {
        let a = dial_at([9000, 1, 1], 100, 0.5);
        let b = dial_at([9000, 1, 1], 100, 0.5);
        assert_eq!(a, b); // the same tick is not eased twice
        assert!(dial_at([9000, 1, 1], 101, 0.5).state > a.state);
        assert_eq!(dial_at([9001, 1, 1], 100, 0.5).state, a.state);
        assert!((0..100).all(|i| (0.0..1.0).contains(&noise(i))));
    }
}
