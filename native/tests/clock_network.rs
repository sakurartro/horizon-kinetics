// A cuckoo clock in the rotation network: a consumer with its shaft at the back (a crank at rest, runtime.rs).
use horizon_mod_kinetics::{Kind, Node, Pos, crank_node, kind_of, solve};

#[test]
fn a_clock_is_turned_through_the_shaft_at_its_back_and_gives_nothing() {
    assert_eq!(kind_of("kinetics:cuckoo_clock"), Some(Kind::Crank { toward: 1, rpm: 0 }));
    assert_eq!(kind_of("kinetics:mysterious_cuckoo_clock"), Some(Kind::Crank { toward: 1, rpm: 0 }));
    let shaft = Node { kind: Kind::Shaft, axis: 2, shafts: [true; 2] };
    let source = Node { kind: Kind::Source, axis: 2, shafts: [true; 2] };
    let run = |facing| {
        let blocks: Vec<(Pos, Node)> = vec![([0, 0, 0], crank_node(facing, 0).unwrap()), ([0, 0, 1], shaft), ([0, 0, 2], source)];
        let speeds = solve(&[[0, 0, 2]], |p| blocks.iter().find(|b| b.0 == p).map(|b| b.1));
        speeds.get(&[0, 0, 0]).copied().unwrap_or(0.0)
    };
    // Facing north (-z), its back and its shaft are on the south side, where the shaft comes from.
    assert_eq!(run("north"), 16.0);
    // Facing south, its shaft is on the north side: nothing reaches it.
    assert_eq!(run("south"), 0.0);
}
