// Chain drives in the solver (runtime.rs): they pass speed along their chain, a gearshift scales it by its signal.
// Expected values come from Create's `ChainDriveBlock.getRotationSpeedModifier` / `getModifierForSignal`.
use horizon_mod_kinetics::*;

/// (pos, name, axis, part, signal): a chain drive's node, or any other block with just its axis.
type B = (Pos, &'static str, &'static str, &'static str, i32);

fn world<'a>(blocks: &'a [B]) -> impl Fn(Pos) -> Option<Node> + 'a {
    move |p| {
        let b = blocks.iter().find(|b| b.0 == p)?;
        let name = format!("kinetics:{}", b.1);
        chain_node(&name, Some(b.2), Some(b.3), Some("true"), b.4).or_else(|| node(&name, Some(b.2)))
    }
}

fn speed(w: &[B], at: Pos) -> f32 {
    solve(&[w[0].0], world(w)).get(&at).copied().unwrap_or(0.0)
}

const D: &str = "encased_chain_drive";
const G: &str = "adjustable_chain_gearshift";

// With `axis_along_first` true a vertical drive is chained along x: `start` toward +x, `end` toward -x.

#[test]
fn a_chain_carries_the_speed_sideways_in_the_same_direction() {
    // Source under a drive on y; chained east to another drive whose shaft goes on up.
    let w = [
        ([0, 0, 0], "source", "y", "none", 0),
        ([0, 1, 0], D, "y", "start", 0),
        ([1, 1, 0], D, "y", "end", 0),
        ([1, 2, 0], "shaft", "y", "none", 0),
    ];
    assert_eq!(speed(&w, [1, 2, 0]), 16.0);
    assert_eq!(speed(&w, [1, 1, 0]), 16.0);
}

#[test]
fn drives_with_no_chain_between_them_do_not_connect() {
    let w = [
        ([0, 0, 0], "source", "y", "none", 0),
        ([0, 1, 0], D, "y", "none", 0),
        ([1, 1, 0], D, "y", "none", 0),
    ];
    assert_eq!(speed(&w, [1, 1, 0]), 0.0);
    // The `end` part reaches toward -x, not the drive at +x.
    let w = [
        ([0, 0, 0], "source", "y", "none", 0),
        ([0, 1, 0], D, "y", "end", 0),
        ([1, 1, 0], D, "y", "none", 0),
    ];
    assert_eq!(speed(&w, [1, 1, 0]), 0.0);
}

#[test]
fn a_gearshift_scales_the_speed_by_its_signal() {
    for (signal, factor) in [(0, 1.0), (15, 2.0), (7, 1.5)] {
        // Driven from the plain drive: the gearshift's speed is divided by its factor.
        let w = [
            ([0, 0, 0], "source", "y", "none", 0),
            ([0, 1, 0], D, "y", "start", 0),
            ([1, 1, 0], G, "y", "end", signal),
        ];
        assert!((speed(&w, [1, 1, 0]) - 16.0 / factor).abs() < 1e-3, "signal {signal}");
        // Driven from the gearshift: the drive turns by its factor.
        let w = [
            ([0, 0, 0], "source", "y", "none", 0),
            ([0, 1, 0], G, "y", "start", signal),
            ([1, 1, 0], D, "y", "end", 0),
        ];
        assert!((speed(&w, [1, 1, 0]) - 16.0 * factor).abs() < 1e-3, "signal {signal}");
    }
}

#[test]
fn a_gearshift_passes_the_speed_straight_along_its_shaft() {
    let w = [
        ([0, 0, 0], "source", "y", "none", 0),
        ([0, 1, 0], G, "y", "none", 15),
        ([0, 2, 0], "shaft", "y", "none", 0),
    ];
    assert_eq!(speed(&w, [0, 2, 0]), 16.0);
}

#[test]
fn chain_nodes_are_named_and_read_their_state() {
    assert!(has_block_entity("kinetics:encased_chain_drive"));
    let n = chain_node("kinetics:adjustable_chain_gearshift", Some("z"), Some("middle"), Some("false"), 3).unwrap();
    assert_eq!(n.kind, Kind::Chain { signal: 3, conn: 1, ends: [true, true] });
    assert_eq!(chain_node("kinetics:shaft", Some("z"), None, None, 0), None);
    assert_eq!(chain_factor(0), 1.0);
    assert_eq!(chain_factor(15), 2.0);
}
