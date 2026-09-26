//! CR 702.86 Annihilator.

use crate::common_k702_011_017::{assert_supported, attack_with, bf, custom_card};
use crate::common_k702_018_026::{declare_blocks, triggers_on_stack};
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn permanents_of(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents().filter(|o| o.controller == p).count()
}

#[test]
fn annihilator_makes_the_defending_player_sacrifice_n_permanents_before_blocks() {
    cr!("702.86", "702.86a");
    ruling!(
        "Ulamog's Crusher",
        "Annihilator abilities trigger and resolve during the declare attackers step. The defending player chooses and sacrifices the required number of permanents before they declare blockers. Any creatures sacrificed this way won't be able to block."
    );
    assert_supported("Ulamog's Crusher");
    let mut t = TestGame::new(2);
    let crusher = t.battlefield(P0, "Ulamog's Crusher");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    attack_with(&mut t, &[(crusher, Entity::Player(P1))]);
    t.settle();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(triggers_on_stack(&t, "Annihilator 2"), 1);
    // The defending player chooses which two permanents to sacrifice.
    t.answer_choose(P1, &[Entity::Object(bears), Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(land));
    // Nothing is left to block with.
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 12);
}

#[test]
fn annihilator_sacrifices_everything_if_there_are_fewer_than_n_permanents() {
    cr!("702.86a");
    let mut t = TestGame::new(2);
    let crusher = t.battlefield(P0, "Ulamog's Crusher");
    t.battlefield(P1, "Forest");
    attack_with(&mut t, &[(crusher, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(permanents_of(&t, P1), 0);
    // The attacking player's permanents are untouched.
    assert!(t.on_battlefield(crusher));
}

#[test]
fn each_instance_of_annihilator_triggers_separately() {
    cr!("702.86b");
    let def = custom_card(
        "Doubly Annihilating Eldrazi",
        "Creature — Eldrazi",
        Some((5, 5)),
        "Annihilator 1\nAnnihilator 2",
    );
    let mut t = TestGame::new(2);
    let eldrazi = bf(&mut t, P0, def);
    t.lands(P1, "Forest", 4);
    attack_with(&mut t, &[(eldrazi, Entity::Player(P1))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Annihilator 1"), 1);
    assert_eq!(triggers_on_stack(&t, "Annihilator 2"), 1);
    t.resolve_all();
    assert_eq!(permanents_of(&t, P1), 1);
}

#[test]
fn a_creature_attacking_a_sacrificed_planeswalker_keeps_attacking() {
    cr!("702.86a", "506.4c");
    ruling!(
        "Ulamog's Crusher",
        "If a creature with annihilator is attacking a planeswalker, and the defending player chooses to sacrifice that planeswalker, the attacking creature continues to attack. It may be blocked. If it isn't blocked, it simply won't deal combat damage to anything."
    );
    let mut t = TestGame::new(2);
    let crusher = t.battlefield(P0, "Ulamog's Crusher");
    let jace = t.battlefield(P1, "Jace Beleren");
    let land = t.battlefield(P1, "Island");
    assert!(t.g.obj(jace).is(CardType::Planeswalker));
    attack_with(&mut t, &[(crusher, Entity::Object(jace))]);
    t.answer_choose(P1, &[Entity::Object(jace), Entity::Object(land)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    assert!(t.g.is_attacking(crusher));
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    // It deals no combat damage to anything.
    assert_eq!(t.life(P1), 20);
}

/// With the attack multiple players option every opponent is a defending player, but an
/// attacking creature's ability refers to the one it attacks (CR 802.2a).
#[test]
fn the_defending_player_is_the_player_the_creature_attacks() {
    cr!("702.86a", "802.2a");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let crusher = t.battlefield(P0, "Ulamog's Crusher");
    t.lands(P1, "Forest", 2);
    t.lands(P2, "Forest", 2);
    attack_with(&mut t, &[(crusher, Entity::Player(P2))]);
    t.resolve_all();
    assert_eq!(permanents_of(&t, P1), 2);
    assert_eq!(permanents_of(&t, P2), 0);
}
