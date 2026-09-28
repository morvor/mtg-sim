//! Rulings batch S28 — energy counters (CR 107.14, 122.1): {E} stands for one energy
//! counter a player gets; paying {E} removes one. Energy counters aren't mana and aren't
//! associated with any permanent, and effects that interact with the counters a player
//! has (proliferate) interact with them.

use crate::r_s01_common::supported;
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s04_common::next_upkeep;
use crate::r_s20_common::tap_for_mana;
use crate::r_s21_common::castable;
use crate::r_s28_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn each_energy_symbol_is_one_energy_counter() {
    cr!("107.14", "122.1");
    ruling!(
        "Aether Hub",
        "{E} is the energy symbol. It represents one energy counter."
    );
    supported("Aether Hub");
    supported("Chthonian Nightmare");
    // Aether Hub: "When this land enters, you get {E}." Chthonian Nightmare: "When this
    // enchantment enters, you get {E}{E}{E}."
    let mut t = TestGame::new(2);
    let hub = t.enter(P0, "Aether Hub");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 1);
    let nightmare = t.enter(P0, "Chthonian Nightmare");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 4);
    // The counters are the player's: none is on the permanents, and no mana was added.
    assert_eq!(t.counters(hub, "energy"), 0);
    assert_eq!(t.counters(nightmare, "energy"), 0);
    assert!(t.g.player(P0).mana_pool.is_empty());
    assert_eq!(energy(&t, P1), 0);
}

#[test]
fn energy_isnt_mana_and_doesnt_empty_between_steps() {
    cr!("107.14", "106.4", "500.5");
    ruling!(
        "Solar Transformer",
        "Energy counters aren't mana. They don't go away as steps, phases, and turns end, and effects that add mana \"of any type\" can't give you energy counters."
    );
    supported("Solar Transformer");
    supported("Reflecting Pool");
    let mut t = TestGame::new(2);
    t.enter(P0, "Solar Transformer");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 3);
    // Energy can't pay a mana cost.
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!castable(&mut t, P0, bears));
    // Reflecting Pool, with Aether Hub (which makes energy) the only other land: "any type
    // that a land you control could produce" adds mana, never energy.
    t.battlefield(P0, "Aether Hub");
    let pool = t.battlefield(P0, "Reflecting Pool");
    assert!(tap_for_mana(&mut t, P0, pool, "Add"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    assert_eq!(energy(&t, P0), 3);
    // Mana empties as steps end; energy stays through steps, phases, and turns.
    next_upkeep(&mut t, P1);
    assert!(t.g.player(P0).mana_pool.is_empty());
    next_upkeep(&mut t, P0);
    assert_eq!(energy(&t, P0), 3);
}

#[test]
fn energy_isnt_associated_with_the_permanent_that_gave_it() {
    cr!("107.14", "122.1", "118.3");
    ruling!(
        "Aether Hub",
        "Energy counters are a kind of counter that a player may have. They're not associated with any specific permanents."
    );
    supported("Solar Transformer");
    let mut t = TestGame::new(2);
    let hub = t.enter(P0, "Aether Hub");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 1);
    // The land leaves: the energy stays with the player.
    destroy(&mut t, hub);
    assert!(t.in_graveyard(P0, "Aether Hub"));
    assert_eq!(energy(&t, P0), 1);
    // Another permanent's "Pay {E}" uses it: "{T}, Pay {E}: Add one mana of any color."
    let transformer = t.battlefield(P0, "Solar Transformer");
    assert!(tap_for_mana(&mut t, P0, transformer, "Pay {E}"));
    assert_eq!(energy(&t, P0), 0);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn assaultron_dominators_energy_stays_after_it_dies() {
    cr!("107.14", "122.1");
    ruling!(
        "Assaultron Dominator",
        "Energy counters are a kind of counter that a player may have. They’re not associated with any specific permanents."
    );
    supported("Assaultron Dominator");
    supported("Automated Assembly Line");
    // "When this creature enters, you get {E}{E}."
    let mut t = TestGame::new(2);
    let dominator = t.enter(P0, "Assaultron Dominator");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 2);
    destroy(&mut t, dominator);
    assert!(t.in_graveyard(P0, "Assaultron Dominator"));
    assert_eq!(energy(&t, P0), 2);
    // Automated Assembly Line ("Pay {E}{E}{E}: Create a tapped 3/3 ... Robot") can spend
    // it along with energy from elsewhere.
    let line = t.battlefield(P0, "Automated Assembly Line");
    assert!(!can_activate(&mut t, P0, line));
    t.g.add_counters(Entity::Player(P0), "energy", 1, None);
    t.activate(P0, line, 0, &[]).expect("pay three energy");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 0);
    assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 1);
}

#[test]
fn helios_ones_energy_isnt_mana_and_stays_between_turns() {
    cr!("107.14", "106.4", "500.5");
    ruling!(
        "HELIOS One",
        "Energy counters aren’t mana. They don’t go away as steps, phases, and turns end, and effects that add mana “of any type” can’t give you energy counters."
    );
    supported("HELIOS One");
    // "{1}, {T}: You get {E}."
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 1);
    let helios = t.battlefield(P0, "HELIOS One");
    t.activate(P0, helios, 1, &[]).expect("activate for energy");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 1);
    // A second HELIOS One can't use the energy to pay its {1}.
    let other = t.battlefield(P0, "HELIOS One");
    t.g.recompute();
    assert!(t.activate(P0, other, 1, &[]).is_err());
    assert_eq!(energy(&t, P0), 1);
    // The energy is still there turns later.
    next_upkeep(&mut t, P1);
    next_upkeep(&mut t, P0);
    assert_eq!(energy(&t, P0), 1);
}

#[test]
fn paying_energy_removes_counters_and_proliferate_adds_them() {
    cr!("107.14", "118.3", "701.34a");
    ruling!(
        "Automated Assembly Line",
        "If an effect says you get one or more {E}, you get that many energy counters. To pay one or more {E}, you lose that many energy counters. You can’t pay more energy counters than you have. Any effects that interact with counters a player gets, has, or loses can interact with energy counters."
    );
    supported("Automated Assembly Line");
    supported("Steady Progress");
    let mut t = TestGame::new(2);
    let line = t.battlefield(P0, "Automated Assembly Line");
    t.g.add_counters(Entity::Player(P0), "energy", 2, None);
    // Two energy can't pay {E}{E}{E}.
    assert!(!can_activate(&mut t, P0, line));
    assert!(t.activate(P0, line, 0, &[]).is_err());
    assert_eq!(energy(&t, P0), 2);
    // Proliferate gives P0 another energy counter.
    t.answer_choose(P0, &[Entity::Player(P0)]);
    cast_card(&mut t, P0, "Steady Progress");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 3);
    // Now the three counters are lost to pay the cost.
    t.activate(P0, line, 0, &[]).expect("pay three energy");
    assert_eq!(energy(&t, P0), 0);
    t.resolve_all();
    assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 1);
}
