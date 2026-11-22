//! Triggered abilities that trigger an additional time because of what caused them to
//! trigger, or whose sources are described by alternatives (CR 603.2d).

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn isshin_doubles_attack_triggers_only() {
    cr!("603.2d");
    ruling!(
        "Isshin, Two Heavens as One",
        "It does not affect triggered abilities with other trigger conditions"
    );
    assert_supported("Isshin, Two Heavens as One");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isshin, Two Heavens as One");
    let paladin = t.battlefield(P0, "Accorder Paladin");
    let lookout = t.battlefield(P0, "Night Market Lookout");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (paladin, Entity::Player(P1)),
            (lookout, Entity::Player(P1)),
        ]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    // Battle cry twice; the lookout's "becomes tapped" ability once.
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.pt(lookout).0, 3);
}

#[test]
fn krang_doubles_abilities_triggered_by_drawing() {
    cr!("603.2d");
    assert_supported("Krang, the All-Powerful");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krang, the All-Powerful");
    t.battlefield(P0, "Underworld Dreams");
    t.g.draw_cards(P1, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn wayta_doubles_abilities_triggered_by_damage_to_your_creatures() {
    cr!("603.2d");
    assert_supported("Wayta, Trainer Prodigy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wayta, Trainer Prodigy");
    let reckoner = t.battlefield(P0, "Boros Reckoner");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(reckoner).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn gandalf_doubles_an_artifacts_own_enters_ability() {
    cr!("603.2d");
    ruling!(
        "Gandalf the White",
        "Gandalf the White affects a permanent's own enters-the-battlefield and leaves-the-battlefield triggered abilities"
    );
    assert_supported("Gandalf the White");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gandalf the White");
    let hand = t.hand_size(P0);
    t.enter(P0, "Ichor Wellspring");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // A nonlegendary, nonartifact permanent entering doesn't.
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn harmonic_prodigy_doubles_another_wizards_prowess() {
    cr!("603.2d", "702.108a");
    assert_supported("Harmonic Prodigy");
    let mut t = TestGame::new(2);
    // Harmonic Prodigy (a Human Wizard) has prowess too: its own triggers once.
    let prodigy = t.battlefield(P0, "Harmonic Prodigy");
    let hp = t.battlefield(P0, "Stormchaser Mage");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.resolve_all();
    // Prowess twice: 1/3 becomes 3/5.
    assert_eq!(t.pt(hp), (3, 5));
    assert_eq!(t.pt(prodigy), (2, 4));
}
