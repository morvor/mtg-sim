//! Equinox (hand-written, `src/cards/equinox.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn setup() -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Plains");
    let eq = t.battlefield(P0, "Equinox");
    t.g.attach(eq, Entity::Object(land));
    t.g.recompute();
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    (t, land, eq)
}

#[test]
fn counters_a_spell_that_would_destroy_your_land() {
    cr!("701.6a", "608.2b");
    ruling!("Equinox", "determine whether the targeted spell would destroy a land if it resolved right then");
    let (mut t, land, _) = setup();
    t.lands(P1, "Mountain", 3);
    let rain = t.hand(P1, "Stone Rain");
    let spell = t.cast(P1, rain).target(land).go();
    t.activate(P0, land, 1, &[Entity::Object(spell)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(land));
    assert!(t.in_graveyard(P1, "Stone Rain"));
}

#[test]
fn other_spells_arent_countered() {
    cr!("608.2b");
    ruling!("Equinox", "The ability can target any spell, even one that would not destroy a land");
    let (mut t, land, _) = setup();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    t.activate(P0, land, 1, &[Entity::Object(spell)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn a_destroy_all_lands_spell_is_countered() {
    cr!("608.2b");
    let (mut t, land, _) = setup();
    // Armageddon: {3}{W}, "Destroy all lands."
    t.lands(P1, "Mountain", 3);
    t.lands(P1, "Plains", 1);
    let arm = t.hand(P1, "Armageddon");
    let spell = t.cast(P1, arm).go();
    t.activate(P0, land, 1, &[Entity::Object(spell)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(land));
}
