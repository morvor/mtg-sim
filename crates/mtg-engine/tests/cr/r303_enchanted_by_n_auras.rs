//! "if ~ is enchanted by N or more Auras" as a condition (CR 303.4b, 603.4): it counts
//! the Auras attached to the object, not anything the object is attached to.

use crate::r703_common::oracle_card;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn intervening_if_counts_auras_attached_to_it() {
    cr!("303.4b", "603.4");
    let collector = oracle_card(
        "Aura Collector",
        "Creature — Human",
        "{2}{W}",
        Some((2, 2)),
        "At the beginning of your upkeep, if Aura Collector is enchanted by two or more Auras, you gain 3 life.",
    );
    let mut t = TestGame::new(2);
    let c = t.custom(P0, collector, Zone::Battlefield);
    let other = t.battlefield(P1, "Grizzly Bears");
    let first = t.battlefield(P1, "Pacifism");
    assert!(t.g.attach(first, Entity::Object(c)));
    // An Aura on another creature doesn't count.
    let elsewhere = t.battlefield(P1, "Pacifism");
    assert!(t.g.attach(elsewhere, Entity::Object(other)));
    t.g.recompute();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Two Auras on it: the ability triggers and resolves.
    let second = t.battlefield(P1, "Pacifism");
    assert!(t.g.attach(second, Entity::Object(c)));
    t.g.recompute();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}
