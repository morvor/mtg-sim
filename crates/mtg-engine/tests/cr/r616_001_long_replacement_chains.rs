//! However many replacement effects apply to an event, each is applied once, one after
//! another, until none is left (CR 614.5, 616.1f): with N Hardened Scales, a creature that
//! would get one +1/+1 counter gets 1 + N (Hardened Scales's ruling). The engine stops a
//! chain at a depth meant only to catch loops (`Game::replace_rec`): 33 effects are the
//! most it lets through, so 34 are cut short (the second test).

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls `n` Hardened Scales ("If one or more +1/+1 counters would be put on a
/// creature you control, that many plus one +1/+1 counters are put on it instead.") and
/// Grizzly Bears, and casts Battlegrowth ("Put a +1/+1 counter on target creature.") on the
/// Bears, taking the default answer to each choice of the Scales to apply next (CR 616.1).
/// Returns the Bears' +1/+1 counters.
fn battlegrowth_under_hardened_scales(n: usize) -> u32 {
    supported("Hardened Scales");
    let mut t = TestGame::new(2);
    for _ in 0..n {
        t.battlefield(P0, "Hardened Scales");
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Battlegrowth", &[obj(bears)]);
    plus_counters(&t, bears)
}

#[test]
fn thirty_three_hardened_scales_each_add_a_counter() {
    cr!("614.5", "616.1", "616.1f");
    ruling!(
        "Hardened Scales",
        "Each additional Hardened Scales you control will increase the number of +1/+1 counters placed on a creature you control by one."
    );
    // 33 is the longest chain the engine's depth guard lets through: a guard one level
    // lower would leave the Bears with 33 counters.
    assert_eq!(battlegrowth_under_hardened_scales(33), 34);
}

/// Fails until fix: the depth guard of `Game::replace_rec` (`depth > 32`) returns the event
/// as modified so far once 33 effects have been applied, so the 34th Hardened Scales adds
/// nothing (the Bears get 34 counters, not 35). CR 616.1f repeats the process "until there
/// are no more left to apply", and CR 614.5 already bounds the chain (each effect applies
/// once; `applied` keeps their keys).
#[test]
fn thirty_four_hardened_scales_each_add_a_counter() {
    cr!("614.5", "616.1f");
    ruling!(
        "Hardened Scales",
        "Each additional Hardened Scales you control will increase the number of +1/+1 counters placed on a creature you control by one."
    );
    assert_eq!(battlegrowth_under_hardened_scales(34), 35);
}
