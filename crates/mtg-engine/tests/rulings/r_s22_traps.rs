//! Rulings batch S22 — Traps: "If [condition], you may pay [cost] rather than pay this
//! spell's mana cost." An alternative cost the player may choose while its condition is
//! true, or ignore (CR 118.9, 601.2b); cost increases and reductions apply to whichever
//! cost is paid (CR 601.2f).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s07_common::cast_methods;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The Trap's alternative-cost method, if P0 could cast it that way now.
fn alternative(t: &mut TestGame, card: ObjectId) -> Option<CastMethod> {
    cast_methods(t, P0, card)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
}

#[test]
fn needlebite_trap_can_be_cast_for_its_mana_cost_even_when_its_condition_is_met() {
    cr!("118.9", "601.2b");
    ruling!(
        "Needlebite Trap",
        "You may ignore a Trap's alternative cost condition and simply cast it for its normal mana cost. This is true even if its alternative cost condition has been met."
    );
    supported("Needlebite Trap");
    // "If an opponent gained life this turn, you may pay {B} rather than pay this spell's
    // mana cost. Target player loses 5 life and you gain 5 life."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    let trap = t.hand(P0, "Needlebite Trap");
    assert_eq!(alternative(&mut t, trap), None);
    assert!(cast_methods(&mut t, P0, trap).contains(&CastMethod::Normal));
    // P1 gains life: the alternative cost is available, and so is the mana cost.
    t.g.gain_life(P1, 1);
    assert!(alternative(&mut t, trap).is_some());
    assert!(cast_methods(&mut t, P0, trap).contains(&CastMethod::Normal));
    t.cast(P0, trap).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 7);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (25, 16));
    // Or for {B}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    let trap = t.hand(P0, "Needlebite Trap");
    t.g.gain_life(P1, 1);
    let alt = alternative(&mut t, trap).unwrap();
    t.cast(P0, trap).method(alt).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn baloth_cage_trap_can_be_cast_for_its_mana_cost_even_when_its_condition_is_met() {
    cr!("118.9", "601.2b");
    ruling!(
        "Baloth Cage Trap",
        "You may ignore a Trap’s alternative cost condition and simply cast it for its normal mana cost. This is true even if its alternative cost condition has been met."
    );
    supported("Baloth Cage Trap");
    // "If an opponent had an artifact enter the battlefield under their control this
    // turn, you may pay {1}{G} rather than pay this spell's mana cost."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let trap = t.hand(P0, "Baloth Cage Trap");
    // An artifact entering under P0's control doesn't count.
    t.enter(P0, "Ornithopter");
    t.settle();
    assert_eq!(alternative(&mut t, trap), None);
    t.enter(P1, "Ornithopter");
    t.settle();
    assert!(alternative(&mut t, trap).is_some());
    t.cast(P0, trap).go();
    assert_eq!(tapped_lands(&t, P0), 5);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Beast").len(), 1);
}

#[test]
fn runeflare_trap_cost_changes_apply_to_whichever_cost_is_paid() {
    cr!("601.2f", "118.9");
    ruling!(
        "Runeflare Trap",
        "Effects that increase or reduce the cost to cast a Trap will apply to whichever cost you chose to pay."
    );
    supported("Runeflare Trap");
    // "If an opponent drew three or more cards this turn, you may pay {R} rather than pay
    // this spell's mana cost." Thalia, Guardian of Thraben: "Noncreature spells cost {1}
    // more to cast."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Mountain", 8);
    let trap = t.hand(P0, "Runeflare Trap");
    t.g.draw_cards(P1, 2);
    assert_eq!(alternative(&mut t, trap), None);
    t.g.draw_cards(P1, 1);
    let alt = alternative(&mut t, trap).expect("P1 drew three cards");
    // {R} + {1}.
    t.cast(P0, trap).method(alt).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Its mana cost {4}{R}{R} + {1}; and with Goblin Electromancer ("Instant and sorcery
    // spells you cast cost {1} less to cast.") the alternative cost {R} + {1} - {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Mountain", 8);
    let trap = t.hand(P0, "Runeflare Trap");
    t.cast(P0, trap).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 7);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.battlefield(P0, "Goblin Electromancer");
    t.lands(P0, "Mountain", 8);
    let trap = t.hand(P0, "Runeflare Trap");
    t.g.draw_cards(P1, 3);
    let alt = alternative(&mut t, trap).unwrap();
    t.cast(P0, trap).method(alt).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn trap_conditions_look_at_what_opponents_did_and_at_attacking_creatures() {
    cr!("118.9", "601.2b");
    supported("Whiplash Trap");
    supported("Lethargy Trap");
    supported("Pitfall Trap");
    // Whiplash Trap: "If an opponent had two or more creatures enter the battlefield under
    // their control this turn". Creatures that left since still count.
    let mut t = TestGame::new(2);
    let trap = t.hand(P0, "Whiplash Trap");
    // (Two creatures to target: "Return two target creatures to their owners' hands.")
    t.battlefield(P0, "Llanowar Elves");
    let bears = t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(alternative(&mut t, trap), None);
    crate::r_s02_common::destroy(&mut t, bears);
    t.enter(P1, "Hill Giant");
    t.settle();
    add_mana(&mut t, P0, ManaType::U, 1);
    assert!(alternative(&mut t, trap).is_some());
    // Lethargy Trap: "If three or more creatures are attacking"; Pitfall Trap: "If exactly
    // one creature is attacking".
    let mut t = TestGame::new(2);
    let lethargy = t.hand(P0, "Lethargy Trap");
    let pitfall = t.hand(P0, "Pitfall Trap");
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::W, 1);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Llanowar Elves");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(a, Entity::Player(P0))]);
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::W, 1);
    assert_eq!(alternative(&mut t, lethargy), None);
    assert!(alternative(&mut t, pitfall).is_some());
    let mut t = TestGame::new(2);
    let lethargy = t.hand(P0, "Lethargy Trap");
    let pitfall = t.hand(P0, "Pitfall Trap");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Hill Giant");
    let c2 = t.battlefield(P1, "Llanowar Elves");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[
            (a, Entity::Player(P0)),
            (b2, Entity::Player(P0)),
            (c2, Entity::Player(P0)),
        ],
    );
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::W, 1);
    assert!(alternative(&mut t, lethargy).is_some());
    assert_eq!(alternative(&mut t, pitfall), None);
    let _ = (b, c);
}
