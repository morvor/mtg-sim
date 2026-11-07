//! Rulings batch S29 — abilities that trigger when counters are put on a permanent
//! (CR 122.6): they trigger both when a permanent enters with counters and when a player
//! puts counters on a permanent.

use crate::r_s01_common::supported;
use crate::r_s24_common::enter_together;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn counters_put_triggers_see_entering_with_counters_and_putting_counters() {
    cr!("122.6", "306.5b", "606.4");
    ruling!(
        "Bioessence Hydra",
        "Abilities that trigger when counters are put on a permanent trigger when a permanent enters the battlefield with counters and when a player puts counters on a permanent."
    );
    supported("Bioessence Hydra");
    supported("Jace Beleren");
    // Bioessence Hydra: "Whenever one or more loyalty counters are put on planeswalkers
    // you control, put that many +1/+1 counters on this creature." Jace Beleren enters
    // with three loyalty counters, then its +2 puts two more on it.
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Bioessence Hydra");
    let jace = cast_new(&mut t, P0, "Jace Beleren", &[]);
    t.resolve_all();
    assert_eq!(t.counters(jace, counters::LOYALTY), 3);
    assert_eq!(t.counters(hydra, counters::PLUS1), 3);
    t.activate(P0, t.g.current(jace), 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(jace, counters::LOYALTY), 5);
    assert_eq!(t.counters(hydra, counters::PLUS1), 5);
    // Loyalty counters on an opponent's planeswalker don't count.
    let theirs = t.battlefield(P1, "Jace Beleren");
    put_counters(&mut t, theirs, counters::LOYALTY, 2);
    t.resolve_all();
    assert_eq!(t.counters(hydra, counters::PLUS1), 5);
}

#[test]
fn a_planeswalker_entering_with_the_hydra_triggers_it_but_isnt_counted() {
    cr!("122.6", "603.6a", "614.12");
    ruling!(
        "Bioessence Hydra",
        "If Bioessence Hydra enters the battlefield at the same time as a planeswalker you control, the loyalty counters that planeswalker will receive won't be counted to determine how many counters Bioessence Hydra enters the battlefield with, but they will cause its last ability to trigger."
    );
    // "This creature enters with a +1/+1 counter on it for each loyalty counter on
    // planeswalkers you control."
    let mut t = TestGame::new(2);
    let ids = enter_together(
        &mut t,
        &[(P0, "Bioessence Hydra"), (P0, "Jace Beleren")],
    );
    let (hydra, jace) = (ids[0], ids[1]);
    assert_eq!(t.counters(jace, counters::LOYALTY), 3);
    assert_eq!(t.counters(hydra, counters::PLUS1), 0);
    t.resolve_all();
    // It entered with none, then its last ability put three on it.
    assert_eq!(t.counters(hydra, counters::PLUS1), 3);
    // Entering later, it counts them: three for the enters replacement, and nothing
    // triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jace Beleren");
    let hydra = t.enter(P0, "Bioessence Hydra");
    t.resolve_all();
    assert_eq!(t.counters(hydra, counters::PLUS1), 3);
}
