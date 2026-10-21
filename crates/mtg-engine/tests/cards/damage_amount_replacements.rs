//! Damage replacements that change the amount (CR 614.1a): for sources with a quality
//! ("If a source you control with an odd mana value would deal damage to a permanent or
//! player, it deals double that damage ... instead."), setting the amount ("would deal an
//! amount of noncombat damage less than ~'s power to an opponent, that source deals damage
//! equal to ~'s power instead"), and for a target for a duration ("Until your next turn,
//! if that creature would deal combat damage to one of your opponents, it deals triple
//! that damage to that player instead.").

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn obosh_doubles_damage_from_sources_with_odd_mana_value() {
    cr!("614.1a", "202.3");
    assert_supported("Obosh, the Preypiercer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Obosh, the Preypiercer");
    t.lands(P0, "Mountain", 3);
    // Lightning Bolt: mana value 1.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 14);
    // Shock: mana value 1 too; to a creature.
    let giant = t.battlefield(P1, "Craw Wurm");
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(giant).go();
    t.resolve();
    assert_eq!(t.g.obj(giant).damage, 4);
    // Grizzly Bears (mana value 2) deals its damage unchanged.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 12);
    // An opponent's odd source isn't doubled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Obosh, the Preypiercer");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn ojer_axonil_raises_small_noncombat_damage_from_red_sources_to_its_power() {
    cr!("614.1a", "120.2b");
    ruling!(
        "Ojer Axonil, Deepest Might // Temple of Power",
        "occurs only once, when the damage would be dealt"
    );
    // (Its back face's activation condition isn't supported; the replacement is.)
    let def = card("Ojer Axonil, Deepest Might // Temple of Power");
    assert!(def
        .unsupported_text()
        .iter()
        .all(|t| !t.contains("would deal an amount")));
    let mut t = TestGame::new(2);
    let ojer = t.battlefield(P0, "Ojer Axonil, Deepest Might // Temple of Power");
    assert_eq!(t.pt(ojer), (4, 4));
    t.lands(P0, "Mountain", 4);
    // Shock (2) to an opponent: 4 instead.
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
    // To a creature: unchanged.
    let wurm = t.battlefield(P1, "Craw Wurm");
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(wurm).go();
    t.resolve();
    assert_eq!(t.g.obj(wurm).damage, 2);
    // Combat damage from a red creature: unchanged.
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.attack(&[(goblin, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn jeska_triples_combat_damage_to_opponents_until_your_next_turn() {
    cr!("614.1a", "611.2c", "606.3");
    assert_supported("Jeska, Thrice Reborn");
    let mut t = TestGame::new(2);
    let jeska = t.battlefield(P0, "Jeska, Thrice Reborn");
    t.g.add_counters(Entity::Object(jeska), "loyalty", 3, None);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // The 0 ability (the first loyalty ability).
    t.activate(P0, jeska, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    t.attack(
        &[(bears, Entity::Player(P1)), (wurm, Entity::Player(P1))],
        &[],
    );
    // 2 × 3 from the Bears, 6 from the Wurm.
    assert_eq!(t.life(P1), 8);
}
