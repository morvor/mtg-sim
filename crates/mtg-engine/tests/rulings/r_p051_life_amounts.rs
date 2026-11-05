//! Rulings batch P051 — how much life a drain makes players lose and gain: life loss isn't
//! damage (CR 119.3, 615.1), "life equal to the damage dealt this way" counts the damage
//! actually dealt (CR 120.3a, 615.1), "for each card exiled this way" counts the cards
//! actually exiled (CR 608.2c), a fixed gain doesn't scale with the number of opponents,
//! and the life lost by a player whose life total can't change or who can't gain life.

use crate::r_p051_common::*;
use crate::r_p057_common::into_upkeep;
use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

const BEARS: &str = "Grizzly Bears";

#[test]
fn subversion_life_loss_isnt_damage_and_cant_be_prevented() {
    cr!("119.3", "615.1a");
    ruling!(
        "Subversion",
        "This is loss of life, not damage. It can’t be prevented."
    );
    supported("Subversion");
    supported("Solitary Confinement");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Subversion");
    // "Prevent all damage that would be dealt to you."
    t.battlefield(P1, "Solitary Confinement");
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 19]);
}

/// "You gain life equal to the life lost this way" gains the life the opponents actually
/// lost: all of it together, none from an opponent whose life total can't change, twice
/// as much when the loss is doubled, and none when you can't gain life.
#[test]
fn subversion_gains_the_life_actually_lost() {
    cr!("119.3", "119.8", "614.1a", "608.2c");
    supported("Subversion");
    // Three opponents; one of them (P2) has Platinum Emperion ("Your life total can't
    // change.").
    let mut t = TestGame::new(4);
    t.battlefield(P0, "Subversion");
    t.battlefield(P2, "Platinum Emperion");
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(lives(&t, 4), vec![22, 19, 20, 19]);
    // Bloodletter of Aclazotz: "If an opponent would lose life during your turn, they lose
    // twice that much life instead."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Subversion");
    t.battlefield(P0, "Bloodletter of Aclazotz");
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![22, 18]);
    // Leyline of Punishment: "Players can't gain life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Subversion");
    t.battlefield(P1, "Leyline of Punishment");
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![20, 19]);
}

#[test]
fn corrupt_gains_the_damage_dealt_unbounded_by_toughness_or_life() {
    cr!("120.3a", "120.3e", "120.4");
    ruling!(
        "Corrupt",
        "The amount of damage dealt to a creature is not bounded by its toughness, and the amount of damage dealt to a player is not bounded by that player's life total. For example, if Corrupt deals 6 damage to a 2/2 creature, you'll gain 6 life."
    );
    // A creature.
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 6);
    let bears = t.battlefield(P1, BEARS);
    cast_pool(&mut t, P0, "Corrupt", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, BEARS));
    assert_eq!(t.life(P0), 26);
    // A player at 2 life.
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 6);
    t.g.players[1].life = 2;
    cast_pool(&mut t, P0, "Corrupt", &[pl(P1)]);
    t.resolve_all();
    assert!(t.has_lost(P1));
    assert_eq!(t.life(P0), 26);
}

#[test]
fn corrupt_gains_the_damage_actually_dealt() {
    cr!("615.1a", "614.1a", "120.3a");
    ruling!(
        "Corrupt",
        "The life you gain is equal to the damage dealt by Corrupt, not necessarily to the number of Swamps you control (if some or all of the damage is prevented, for example)."
    );
    // All of it prevented: Solitary Confinement ("Prevent all damage that would be dealt
    // to you").
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.battlefield(P1, "Solitary Confinement");
    cast_pool(&mut t, P0, "Corrupt", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![20, 20]);
    // Some of it prevented: Healing Salve's second mode ("Prevent the next 3 damage that
    // would be dealt to any target this turn") on P1.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve).modes(&[1]).target(pl(P1)).go();
    t.resolve_all();
    cast_pool(&mut t, P0, "Corrupt", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 19]);
    // More than the Swamps: Fiery Emancipation ("If a source you control would deal
    // damage to a permanent or player, it deals triple that damage to that permanent or
    // player instead.").
    supported("Fiery Emancipation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.battlefield(P0, "Fiery Emancipation");
    cast_pool(&mut t, P0, "Corrupt", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![26, 14]);
}

#[test]
fn corrupt_counts_the_swamps_on_resolution() {
    cr!("608.2h");
    ruling!("Corrupt", "The swamps are counted on resolution.");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    cast_pool(&mut t, P0, "Corrupt", &[pl(P1)]);
    // A Swamp in response.
    t.lands(P0, "Swamp", 1);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![23, 17]);
}

#[test]
fn suffer_the_past_counts_cards_actually_exiled_as_one_life_gain() {
    cr!("608.2b", "608.2c", "119.9");
    ruling!(
        "Suffer the Past",
        "The life that’s lost and the life that’s gained are based on the number of cards that are actually exiled, not on the value of X (in case any of the targeted cards somehow leave the graveyard in response and thus can’t be exiled this way)."
    );
    ruling!(
        "Suffer the Past",
        "Suffer the Past’s effect causes a single life-gain event, not a number of separate ones. This matters for cards such as Cradle of Vitality."
    );
    supported("Suffer the Past");
    let mut t = TestGame::new(2);
    // "Whenever you gain life, put a +1/+1 counter on this creature."
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    let cards: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P1, BEARS)).collect();
    crate::r_p046_common::fill_pool(&mut t, P0, 10);
    let s = t.hand(P0, "Suffer the Past");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(cards[0]), obj(cards[1]), obj(cards[2])]);
    t.cast(P0, s).x(3).go();
    // One of the cards leaves the graveyard in response.
    move_to(&mut t, cards[1], Zone::Exile);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![22, 18]);
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.counters(pridemate, "+1/+1"), 1, "one life-gain event");
}

#[test]
fn lampad_gains_one_life_however_many_opponents_lose_life() {
    cr!("119.3", "608.2c");
    ruling!(
        "Lampad of Death's Vigil",
        "You gain only 1 life, no matter how many opponents lose life."
    );
    supported("Lampad of Death's Vigil");
    let mut t = TestGame::new(4);
    let lampad = t.battlefield(P0, "Lampad of Death's Vigil");
    let bears = t.battlefield(P0, BEARS);
    t.answer_choose(P0, &[obj(bears)]);
    assert!(activate_text(&mut t, P0, lampad, "Sacrifice", &[]));
    t.resolve_all();
    assert_eq!(lives(&t, 4), vec![21, 19, 19, 19]);
}

#[test]
fn brutal_hordechief_drains_each_attackers_defending_player() {
    cr!("506.2", "508.5a", "603.2c");
    ruling!(
        "Brutal Hordechief",
        "The defending player affected by Brutal Hordechief’s triggered ability is determined relative to the creature that attacked. For example, if Brutal Hordechief attacked one opponent and two other creatures you control attacked another opponent, the first opponent would lose 1 life and the second opponent would lose 2 life. You’d gain a total of 3 life."
    );
    supported("Brutal Hordechief");
    let mut t = TestGame::new(3);
    let chief = t.battlefield(P0, "Brutal Hordechief");
    let a = t.battlefield(P0, BEARS);
    let b = t.battlefield(P0, BEARS);
    attack_with(&mut t, &[(chief, pl(P1)), (a, pl(P2)), (b, pl(P2))]);
    t.resolve_all();
    assert_eq!(lives(&t, 3), vec![23, 19, 18]);
}

#[test]
fn certain_death_destroys_before_the_life_changes() {
    cr!("608.2c");
    ruling!(
        "Certain Death",
        "The targeted creature will have already left the battlefield before its controller loses 2 life and you gain 2 life. If it has any abilities that care about gaining or losing life, they won’t apply."
    );
    supported("Certain Death");
    let mut t = TestGame::new(2);
    // Your own Ajani's Pridemate ("Whenever you gain life, put a +1/+1 counter on this
    // creature."): you're its controller, so you lose 2 and gain 2.
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    cast_pool(&mut t, P0, "Certain Death", &[obj(pridemate)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Ajani's Pridemate"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 0);
    assert!(t.g.stack.is_empty());
}
