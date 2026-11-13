//! "the number of different mana values among [objects]" (`Value::ManaValuesAmong`,
//! CR 202.3) and "you get an amount of {E} equal to [value]" (CR 107.14, 122.1).

use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn all_seeing_arbiter_counts_different_mana_values_in_the_graveyard() {
    cr!("202.3", "202.3a");
    compiles("All-Seeing Arbiter");
    // "Whenever you discard a card, target creature an opponent controls gets -X/-0 until
    // your next turn, where X is the number of different mana values among cards in your
    // graveyard." Graveyard after the discard: Forest (0), Shock (1), Lightning Bolt (1),
    // Hill Giant (4): three different mana values. (P1's graveyard doesn't count.)
    let mut t = TestGame::new(2);
    t.battlefield(P0, "All-Seeing Arbiter");
    let wurm = t.battlefield(P1, "Craw Wurm");
    for name in ["Forest", "Shock", "Lightning Bolt"] {
        t.graveyard(P0, name);
    }
    t.graveyard(P1, "Divination");
    let giant = t.hand(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.g.discard(P0, giant, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (3, 4));
}

#[test]
fn peema_aether_seer_gets_energy_equal_to_the_greatest_power() {
    cr!("107.14", "122.1");
    compiles("Peema Aether-Seer");
    compiles("Robobrain War Mind");
    // "When this creature enters, you get an amount of {E} equal to the greatest power
    // among creatures you control." Craw Wurm: 6.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    t.enter(P0, "Peema Aether-Seer");
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter("energy"), 6);
    // Robobrain War Mind: "... equal to the number of artifact creatures you control."
    // Itself and Ornithopter: 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.enter(P0, "Robobrain War Mind");
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter("energy"), 2);
}
