// The stress rules (mixins/kinetics/stress.rs) on networks solved by runtime.rs, as settle.rs wires them.
use horizon_mod_kinetics::{Node, Pos, generated, motor_node, node, solve_groups};

#[path = "../mixins/kinetics/stress.rs"]
#[allow(dead_code)]
mod stress;

fn world(len: i32) -> impl Fn(Pos) -> Option<Node> {
    move |p| {
        if p[1] != 0 || p[2] != 0 || !(0..len).contains(&p[0]) {
            None
        } else if p[0] == 0 {
            motor_node("east", 16)
        } else {
            node("kinetics:shaft", Some("x"))
        }
    }
}

fn eval(len: i32, impact: f32, cap: f32) -> stress::NetworkStress {
    let get = world(len);
    let (speeds, groups) = solve_groups(&[[1, 0, 0]], &get);
    assert_eq!(groups.len(), 1);
    stress::evaluate(
        &groups[0],
        |v| speeds[&v],
        |p| get(p).and_then(generated),
        |p| if p[0] == 0 { 0.0 } else { impact },
        |p| if p[0] == 0 { cap } else { 0.0 },
    )
}

#[test]
fn a_motor_with_shafts_has_its_capacity_and_their_stress() {
    // Motor 16 RPM x 100 su, three shafts x 4 su: 1600 vs 192.
    let ns = eval(4, 4.0, 100.0);
    assert_eq!((ns.capacity, ns.stress, ns.size), (1600.0, 192.0, 4));
    assert!(!ns.overstressed());
}

#[test]
fn more_load_than_capacity_overstresses() {
    let ns = eval(10, 4.0, 10.0);
    assert!(ns.overstressed());
    assert_eq!(stress::Level::of(ns.percent()), stress::Level::Overstressed);
}
