//! Delayed and "this turn" triggered abilities created by effects
//! (cards3-triggers-durations; CR 603.7).

use crate::r_s01_common::supported;
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn suppress_returns_the_cards_at_the_end_step_of_that_players_next_turn() {
    cr!("603.7a", "603.7c");
    supported("Suppress");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Island");
    t.library_top(P1, "Forest");
    cast_new(&mut t, P0, "Suppress", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    // Not at P0's end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    // At the end step of P1's next turn (P1 drew one card for the turn).
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Island"));
}

#[test]
fn reincarnation_returns_a_creature_card_from_its_owners_graveyard() {
    cr!("603.7a", "603.7c", "603.7b");
    ruling!(
        "Reincarnation",
        "The creature card that will be returned to the battlefield isn't chosen until the delayed triggered ability resolves."
    );
    supported("Reincarnation");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    t.graveyard(P0, "Serra Angel");
    cast_new(&mut t, P0, "Reincarnation", &[Entity::Object(bears)]);
    t.resolve_all();
    // The Bears die: P0 chooses the Hill Giant from P1's graveyard; it enters under P1's
    // control (its owner's).
    t.answer_choose(P0, &[Entity::Object(giant)]);
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.obj_now(g[0]).controller, P1);
    assert!(t.in_graveyard(P0, "Serra Angel"));
}

#[test]
fn elkin_lair_card_not_played_goes_to_the_graveyard() {
    cr!("603.7a", "603.4");
    interp!("delayed-if-not-played-means-still-exiled");
    ruling!(
        "Elkin Lair",
        "You can play the card this turn, not just during the resolution of the ability."
    );
    supported("Elkin Lair");
    for play in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Elkin Lair");
        t.hand(P1, "Forest");
        t.library_top(P1, "Island");
        t.set_step(P0, Step::End);
        t.advance_to(P1, Step::PrecombatMain);
        t.resolve_all();
        let exiled: Vec<ObjectId> = t
            .g
            .exile
            .iter()
            .copied()
            .filter(|o| t.g.obj(*o).chars.name == "Forest")
            .collect();
        assert_eq!(exiled.len(), 1);
        if play {
            t.play_land(P1, exiled[0]).expect("play it this turn");
        }
        t.advance_to(P1, Step::End);
        t.resolve_all();
        assert_eq!(t.in_graveyard(P1, "Forest"), !play, "played: {play}");
        assert_eq!(t.named_on_battlefield("Forest").len(), play as usize);
    }
}

#[test]
fn mnemonic_betrayal_returns_the_cards_still_exiled() {
    cr!("603.7a", "603.7c", "603.4");
    ruling!(
        "Mnemonic Betrayal",
        "Only the cards that remain in exile are returned to their owners’ graveyards."
    );
    supported("Mnemonic Betrayal");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.graveyard(P1, "Grizzly Bears");
    let bolt = t.graveyard(P1, "Lightning Bolt");
    cast_new(&mut t, P0, "Mnemonic Betrayal", &[]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Mnemonic Betrayal"));
    // P0 casts the Bolt from exile with mana of any type.
    t.lands(P0, "Island", 1);
    let bolt = t.g.current(bolt);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(!t.in_exile("Grizzly Bears"));
}

#[test]
fn rescue_from_the_underworld_returns_both_cards_at_your_next_upkeep() {
    cr!("603.7a", "400.7j");
    ruling!(
        "Rescue from the Underworld",
        "Rescue from the Underworld is exiled as it resolves, not later as its delayed trigger resolves."
    );
    supported("Rescue from the Underworld");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    cast_new(&mut t, P0, "Rescue from the Underworld", &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_exile("Rescue from the Underworld"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn rescue_from_the_underworld_a_sacrificed_token_doesnt_return() {
    cr!("603.7a", "400.7j", "111.8");
    ruling!(
        "Rescue from the Underworld",
        "If you sacrifice a creature token to cast Rescue from the Underworld, it won't return to the battlefield, although the target creature card will."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let token = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(token)]);
    cast_new(&mut t, P0, "Rescue from the Underworld", &[Entity::Object(giant)]);
    t.resolve_all();
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 0);
}

#[test]
fn great_train_heist_treasure_for_combat_damage_to_that_player_this_turn() {
    cr!("603.7b", "603.7c", "702.172a");
    supported("Great Train Heist");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 6);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.hand(P0, "Great Train Heist");
    t.cast(P0, c).modes(&[2]).target(Entity::Player(P1)).go();
    t.resolve_all();
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P2))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 17);
    // Only the damage to the chosen opponent.
    let treasures = crate::r_s01_common::with_subtype(&t, P0, "Treasure");
    assert_eq!(treasures.len(), 1);
    assert!(t.obj_now(treasures[0]).tapped);
}

#[test]
fn surge_to_victory_copies_the_exiled_card_for_each_combat_damage_trigger() {
    cr!("603.7b", "603.7c", "707.12");
    ruling!(
        "Surge to Victory",
        "You cast the copies while the delayed triggered ability of Surge to Victory is resolving."
    );
    supported("Surge to Victory");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    cast_new(&mut t, P0, "Surge to Victory", &[Entity::Object(bolt)]);
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(t.pt(bears), (3, 2));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    // 3 combat damage, then the copy of Lightning Bolt.
    assert_eq!(t.life(P1), 14);
    assert!(t.in_exile("Lightning Bolt"));
}

#[test]
fn lucius_returns_from_exile_when_that_creature_leaves() {
    cr!("603.7a", "603.7c", "400.7");
    supported("Lucius the Eternal");
    let mut t = TestGame::new(2);
    let lucius = t.battlefield(P0, "Lucius the Eternal");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    crate::r_s02_common::destroy(&mut t, lucius);
    t.resolve_all();
    assert!(t.in_exile("Lucius the Eternal"));
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    let back = t.named_on_battlefield("Lucius the Eternal");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).controller, P0);
}
