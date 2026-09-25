//! CR 702.10 Haste.

use super::k702_001_010_common::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn haste_is_a_static_ability_that_works_when_granted() {
    cr!("702.10a", "702.10b");
    let mut t = TestGame::new(2);
    assert_eq!(printed_keywords("Raging Goblin", KeywordKind::Haste).len(), 1);
    // Fervor's static ability gives creatures its controller controls haste, including
    // ones that come under that player's control later.
    t.battlefield(P0, "Fervor");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bears));
}

#[test]
fn creature_with_haste_can_attack_the_turn_it_comes_under_your_control() {
    cr!("702.10b", "302.6");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield_sick(P0, "Raging Goblin");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, goblin));
    assert!(!can_attack(&mut t, bears), "summoning sick without haste");
    declare(&mut t, &[(goblin, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(is_attacking(&t, goblin));
    assert!(!is_attacking(&t, bears));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn haste_lets_a_creature_whose_control_changed_this_turn_attack() {
    cr!("702.10b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    // Act of Treason: gain control of target creature until end of turn, untap it, it
    // gains haste.
    let treason = t.hand(P0, "Act of Treason");
    t.cast(P0, treason).target(bears).go();
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bears));
    // Without haste it couldn't: it hasn't been under P0's control since the turn began.
    remove_kw(&mut t, bears, KeywordKind::Haste);
    assert!(!can_attack(&mut t, bears));
}

#[test]
fn haste_allows_tap_abilities_of_a_new_creature() {
    cr!("702.10c");
    let mut t = TestGame::new(2);
    // Prodigal Pyromancer: "{T}: This creature deals 1 damage to any target."
    let pyro = t.battlefield_sick(P0, "Prodigal Pyromancer");
    assert!(t.activate(P0, pyro, 0, &[Entity::Player(P1)]).is_err());
    assert_eq!(t.stack_len(), 0);
    grant(&mut t, pyro, Keyword::new(KeywordKind::Haste));
    assert!(t.activate(P0, pyro, 0, &[Entity::Player(P1)]).is_ok());
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.obj_now(pyro).tapped);
}

#[test]
fn haste_allows_untap_symbol_abilities_of_a_new_creature() {
    cr!("702.10c");
    let mut t = TestGame::new(2);
    // Order of Whiteclay: "{1}{W}{W}, {Q}: Return target creature card with mana value 3
    // or less from your graveyard to the battlefield."
    let order = t.battlefield_sick(P0, "Order of Whiteclay");
    t.g.objects[order.0 as usize].tapped = true;
    t.lands(P0, "Plains", 3);
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(t
        .activate(P0, order, 0, &[Entity::Object(bears)])
        .is_err());
    t.clear_answers();
    t.battlefield(P0, "Fervor");
    assert!(t.activate(P0, order, 0, &[Entity::Object(bears)]).is_ok());
    assert!(!t.obj_now(order).tapped, "untapped as a cost");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn a_creature_without_haste_can_use_tap_abilities_once_controlled_since_turn_start() {
    cr!("702.10c", "302.6");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    assert!(!t.obj_now(pyro).has_keyword(KeywordKind::Haste));
    assert!(t.activate(P0, pyro, 0, &[Entity::Player(P1)]).is_ok());
}

#[test]
fn multiple_instances_of_haste_are_redundant() {
    cr!("702.10d");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fervor");
    let goblin = t.battlefield_sick(P0, "Raging Goblin");
    assert_eq!(instances(&t, goblin, KeywordKind::Haste), 2);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(goblin, Entity::Player(P1))]);
    go_to(&mut t, Step::EndOfCombat);
    // It attacks once and deals its damage once.
    assert_eq!(t.life(P1), 19);
    // Removing haste removes every instance: it can no longer attack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fervor");
    let goblin = t.battlefield_sick(P0, "Raging Goblin");
    remove_kw(&mut t, goblin, KeywordKind::Haste);
    assert_eq!(instances(&t, goblin, KeywordKind::Haste), 0);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, goblin));
}

#[test]
fn losing_haste_after_attacking_doesnt_remove_from_combat() {
    cr!("702.10b", "506.4");
    ruling!(
        "Fervor",
        "If an attacking creature loses haste, perhaps because Fervor leaves the battlefield after attackers have been declared, it won't be removed from combat."
    );
    let mut t = TestGame::new(2);
    let fervor = t.battlefield(P0, "Fervor");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(is_attacking(&t, bears));
    t.g.destroy(fervor, None);
    t.settle();
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Haste));
    assert!(is_attacking(&t, bears));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_new_creature_that_loses_haste_before_attacking_cant_attack() {
    cr!("702.10b");
    ruling!(
        "Lightning Greaves",
        "If a creature enters the battlefield under your control and gains haste, but then loses it before attacking, it won't be able to attack that turn."
    );
    let mut t = TestGame::new(2);
    let greaves = t.battlefield(P0, "Lightning Greaves");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let goblin = t.battlefield_sick(P0, "Goblin Piker");
    // Equip {0} to the Bears, then move it to the Goblin Piker.
    t.activate(P0, greaves, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
    t.activate(P0, greaves, 0, &[Entity::Object(goblin)]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Haste));
    assert!(t.obj_now(goblin).has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, bears));
    assert!(can_attack(&mut t, goblin));
}

#[test]
fn untap_symbol_abilities_need_haste_on_a_new_creature() {
    cr!("702.10c");
    ruling!(
        "Order of Whiteclay",
        "If a creature with an {Q} ability hasn't been under your control since your most recent turn began, you can't activate that ability, unless the creature has haste."
    );
    let mut t = TestGame::new(2);
    let order = t.battlefield_sick(P0, "Order of Whiteclay");
    t.g.objects[order.0 as usize].tapped = true;
    t.lands(P0, "Plains", 3);
    let bears = t.graveyard(P0, "Grizzly Bears");
    grant(&mut t, order, Keyword::new(KeywordKind::Haste));
    assert!(t.activate(P0, order, 0, &[Entity::Object(bears)]).is_ok());
}
