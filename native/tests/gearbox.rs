// The gearbox: how it passes speed on (runtime.rs, with the solver) and the pure rules of gearbox.rs.
// Expected values come from Create's `RotationPropagator.getAxisModifier`, worked out by hand.
use horizon_mod_kinetics::*;

#[allow(dead_code)]
#[path = "../mixins/kinetics/gearbox.rs"]
mod gearbox;

const X: &str = "x";
const Y: &str = "y";
const Z: &str = "z";

fn world<'a>(blocks: &'a [(Pos, &'a str, &'a str)]) -> impl Fn(Pos) -> Option<Node> + 'a {
    move |p| {
        blocks
            .iter()
            .find(|b| b.0 == p)
            .and_then(|b| node(&format!("kinetics:{}", b.1), Some(b.2)))
    }
}

fn speed(w: &[(Pos, &str, &str)], changed: Pos, at: Pos) -> f32 {
    solve(&[changed], world(w)).get(&at).copied().unwrap_or(0.0)
}

#[test]
fn a_gearbox_reverses_the_way_straight_through_and_splits_the_other_axis() {
    // Source -x of a gearbox standing on y, shafts out of its other three sides.
    let w = [
        ([-1, 0, 0], "source", X),
        ([0, 0, 0], "gearbox", Y),
        ([1, 0, 0], "shaft", X),
        ([0, 0, 1], "shaft", Z),
        ([0, 0, -1], "shaft", Z),
    ];
    assert_eq!(speed(&w, [0, 0, 0], [0, 0, 0]).abs(), 16.0);
    assert_eq!(speed(&w, [0, 0, 0], [1, 0, 0]), -16.0);
    // Create: the +z face has the source's direction sign flipped relative to +x, the -z face is the opposite.
    assert_eq!(speed(&w, [0, 0, 0], [0, 0, 1]), 16.0);
    assert_eq!(speed(&w, [0, 0, 0], [0, 0, -1]), -16.0);
}

#[test]
fn the_output_does_not_depend_on_which_side_is_the_input() {
    // Source on +z instead: its face-to-face signs follow from the table (+z in, -z out reversed, +x/-x opposite).
    let w = [
        ([0, 0, 1], "source", Z),
        ([0, 0, 0], "gearbox", Y),
        ([0, 0, -1], "shaft", Z),
        ([1, 0, 0], "shaft", X),
        ([-1, 0, 0], "shaft", X),
    ];
    assert_eq!(speed(&w, [0, 0, 0], [0, 0, -1]), -16.0);
    // Create: input +z (S = +z), output +x: same axis direction as S -> -1; -x: +1.
    assert_eq!(speed(&w, [0, 0, 0], [1, 0, 0]), -16.0);
    assert_eq!(speed(&w, [0, 0, 0], [-1, 0, 0]), 16.0);
}

#[test]
fn a_gearbox_has_no_shaft_on_its_own_axis() {
    let w = [
        ([0, -1, 0], "source", Y),
        ([0, 0, 0], "gearbox", Y),
        ([0, 1, 0], "shaft", Y),
    ];
    assert_eq!(speed(&w, [0, 0, 0], [0, 1, 0]), 0.0);
    assert_eq!(speed(&w, [0, 0, 0], [0, 0, 0]), 0.0);
    // A shaft pointing across a face, not along it, doesn't connect either.
    let w = [
        ([-1, 0, 0], "source", X),
        ([0, 0, 0], "gearbox", Y),
        ([0, 0, 1], "shaft", X),
    ];
    assert_eq!(speed(&w, [0, 0, 0], [0, 0, 1]), 0.0);
}

#[test]
fn a_chain_of_gearboxes_multiplies_the_signs() {
    // Two gearboxes, side by side along x, each on y: inline the speed flips twice.
    let w = [
        ([-1, 0, 0], "source", X),
        ([0, 0, 0], "gearbox", Y),
        ([1, 0, 0], "gearbox", Y),
        ([2, 0, 0], "shaft", X),
        ([1, 0, 1], "shaft", Z),
    ];
    assert_eq!(speed(&w, [0, 0, 0], [2, 0, 0]), 16.0);
    // The second gearbox turns +16; its +z face is the second axis's positive side: reversed.
    assert_eq!(speed(&w, [0, 0, 0], [1, 0, 1]), -16.0);
}

#[test]
fn a_gearbox_standing_on_x_has_its_shafts_on_y_and_z() {
    let w = [
        ([0, -1, 0], "source", Y),
        ([0, 0, 0], "gearbox", X),
        ([0, 1, 0], "shaft", Y),
        ([0, 0, 1], "shaft", Z),
    ];
    assert_eq!(speed(&w, [0, 0, 0], [0, 1, 0]), -16.0);
    // y is the first of its two axes, z the second: perpendicular faces differ in sign.
    assert_eq!(speed(&w, [0, 0, 0], [0, 0, 1]).abs(), 16.0);
}

#[test]
fn two_sources_pulling_a_gearbox_against_each_other_lock_it() {
    // Both inputs turn "+" about their axes; the faces want opposite latent speeds.
    let w = [
        ([-1, 0, 0], "source", X),
        ([0, 0, 0], "gearbox", Y),
        ([1, 0, 0], "source", X),
    ];
    assert_eq!(speed(&w, [0, 0, 0], [0, 0, 0]), 0.0);
    assert_eq!(speed(&w, [0, 0, 0], [1, 0, 0]), 0.0);
}

#[test]
fn a_gearbox_is_a_kinetic_part_with_a_block_entity() {
    assert!(has_block_entity("kinetics:gearbox"));
    assert!(!has_block_entity("kinetics:vertical_gearbox"));
    assert_eq!(node("kinetics:vertical_gearbox", Some("x")), None);
}

#[test]
fn the_vertical_gearbox_is_placed_across_the_players_facing() {
    // Facing south (+z, yaw 0) or north: shafts along x would point sideways, so the box is on x.
    assert_eq!(gearbox::placed_axis(0.0), 0);
    assert_eq!(gearbox::placed_axis(180.0), 0);
    assert_eq!(gearbox::placed_axis(-170.0), 0);
    // West (90), east (270 / -90): on z.
    assert_eq!(gearbox::placed_axis(90.0), 2);
    assert_eq!(gearbox::placed_axis(-90.0), 2);
    assert_eq!(gearbox::placed_axis(270.0), 2);
}

#[test]
fn a_vertical_gearbox_lines_its_shafts_up_with_the_neighbours_it_finds() {
    // [+x, -x, +z, -z]
    assert_eq!(gearbox::vertical_axis([true, false, false, false], 0), 2);
    assert_eq!(gearbox::vertical_axis([false, true, false, false], 0), 2);
    assert_eq!(gearbox::vertical_axis([false, false, true, true], 2), 0);
    // Both axes ask for it, or none does: the placement axis stays.
    assert_eq!(gearbox::vertical_axis([true, false, true, false], 0), 0);
    assert_eq!(gearbox::vertical_axis([false; 4], 2), 2);
}

#[test]
fn half_shafts_are_laid_out_of_the_faces_they_belong_to() {
    // A +z model turned by `lay_face` must point out of the face.
    let turn = |(axis, deg): ([f32; 3], f32), v: [f32; 3]| {
        let t = deg.to_radians();
        let (s, c) = (t.sin(), t.cos());
        let [x, y, z] = v;
        let out = if axis[0] != 0.0 {
            [x, y * c - z * s, y * s + z * c]
        } else if axis[1] != 0.0 {
            [x * c + z * s, y, -x * s + z * c]
        } else {
            [x * c - y * s, x * s + y * c, z]
        };
        out.map(|n| n.round())
    };
    for (axis, positive) in gearbox::faces(1) {
        let mut want = [0.0; 3];
        want[axis] = if positive { 1.0 } else { -1.0 };
        let got = match gearbox::lay_face(axis, positive) {
            Some(t) => turn(t, [0.0, 0.0, 1.0]),
            None => [0.0, 0.0, 1.0],
        };
        assert_eq!(got, want, "{axis} {positive}");
    }
    assert_eq!(gearbox::faces(1).count(), 4);
    // y faces too (a gearbox on x or z).
    for positive in [true, false] {
        let got = turn(gearbox::lay_face(1, positive).unwrap(), [0.0, 0.0, 1.0]);
        assert_eq!(got, [0.0, if positive { 1.0 } else { -1.0 }, 0.0]);
    }
}

#[test]
fn vertical_gearboxes_are_marked_per_dimension() {
    gearbox::mark("test:g1", [1, 2, 3]);
    gearbox::mark("test:g2", [4, 5, 6]);
    assert_eq!(gearbox::take("test:g1"), vec![[1, 2, 3]]);
    assert!(gearbox::take("test:g1").is_empty());
    assert_eq!(gearbox::take("test:g2"), vec![[4, 5, 6]]);
}
