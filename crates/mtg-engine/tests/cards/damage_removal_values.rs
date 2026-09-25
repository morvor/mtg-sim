//! Damage and removal amounts that depend on values: the sacrificed creature's power
//! (Fling), "-X/-X where X is ...", "for each" (patterns in
//! `src/oracle/patterns/damage_removal_*.rs` and the shared value grammar).

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn sacrificed_creature_cards_compile() {
    assert_compiles(&[
        "Fling",
        "Thud",
        "Rite of Consumption",
        "Bushmeat Poacher",
    ]);
}

#[test]
fn fling_uses_the_last_known_power_of_the_sacrificed_creature() {
    cr!("118.8", "608.2h");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 2);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    let fling = t.hand(P0, "Fling");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, fling).target(P1).go();
    assert!(!t.on_battlefield(bears), "sacrificed as a cost");
    t.resolve();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn a_copy_of_fling_deals_the_same_damage() {
    cr!("707.10");
    ruling!(
        "Melek, Izzet Paragon",
        "if a player sacrifices a 3/3 creature to cast Fling, and you copy it"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let fling = t.hand(P0, "Fling");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let fling = t.cast(P0, fling).target(P1).go();
    // A copy (e.g. from Melek) isn't cast: no creature is sacrificed for it, but it uses
    // the original's sacrificed creature.
    mtg_engine::copy::copy_spell(&mut t.g, fling, P0, false).expect("copied");
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "2 from the copy, 2 from the original");
}

#[test]
fn bushmeat_poacher_gains_life_equal_to_the_sacrificed_creatures_toughness() {
    cr!("118.8", "602.2b");
    let mut t = TestGame::new(2);
    let poacher = t.battlefield(P0, "Bushmeat Poacher");
    let wall = t.battlefield(P0, "Wall of Stone");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[Entity::Object(wall)]);
    t.activate(P0, poacher, 0, &[]).unwrap();
    t.resolve();
    // Wall of Stone is a 0/8.
    assert_eq!(t.life(P0), 28);
}
