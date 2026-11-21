//! Parts of an effect that happen only under a condition (`src/oracle/patterns/
//! conditional_parts.rs`): "[effect] unless [condition]", "[effect] if [condition]", and
//! "If [condition], instead [effect]" where the replacement has targets of its own (chosen
//! only if the optional cost it depends on was paid, CR 601.2c). Also named characters'
//! pronouns ("he fights", "put a +1/+1 counter on him").

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The target decisions asked since `from`: (text, candidates).
fn target_choices(t: &TestGame, from: usize) -> Vec<(String, Vec<Entity>)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets {
                text, candidates, ..
            } => Some((text.clone(), candidates.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn kicked_instead_exile_target_nonland_permanent() {
    cr!("601.2c", "702.33g");
    ruling!(
        "Tear Asunder",
        "If Tear Asunder is kicked, it can target any nonland permanent"
    );
    // Tear Asunder ({1}{G} instant): "Kicker {1}{B}. Exile target artifact or
    // enchantment. If this spell was kicked, exile target nonland permanent instead."
    assert_compiles(&["Tear Asunder"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Swamp", 2);
    let rock = t.battlefield(P1, "Mind Stone");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Tear Asunder");
    let from = t.asked().len();
    t.cast(P0, c).kicked(true).target(wurm).go();
    // Only the kicked part's target is chosen.
    let choices = target_choices(&t, from);
    assert_eq!(choices.len(), 1);
    assert!(choices[0].1.contains(&Entity::Object(wurm)));
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert!(t.on_battlefield(rock));
    // Not kicked: an artifact or enchantment only.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let rock = t.battlefield(P1, "Mind Stone");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Tear Asunder");
    let from = t.asked().len();
    t.cast(P0, c).kicked(false).target(rock).go();
    let choices = target_choices(&t, from);
    assert_eq!(choices.len(), 1);
    assert!(!choices[0].1.contains(&Entity::Object(wurm)));
    t.resolve_all();
    assert!(!t.on_battlefield(rock));
    assert!(t.on_battlefield(wurm));
}

#[test]
fn kicked_it_deals_divided_damage_instead() {
    cr!("601.2c", "702.33g");
    ruling!(
        "Fight with Fire",
        "If Fight with Fire is kicked, it can target creatures, players, and planeswalkers."
    );
    // Fight with Fire ({2}{R} sorcery): "Kicker {5}{R}. Fight with Fire deals 5 damage to
    // target creature. If this spell was kicked, it deals 10 damage divided as you choose
    // among any number of targets instead."
    assert_compiles(&["Fight with Fire"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 9);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Fight with Fire");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![4, 6]));
    t.cast(P0, c)
        .kicked(true)
        .targets(&[Entity::Object(wurm), Entity::Player(P1)])
        .go();
    t.resolve_all();
    // The spell deals the damage (not the creature).
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P1), 14);
    // Not kicked: 5 damage to one creature.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Fight with Fire");
    t.cast(P0, c).kicked(false).target(wurm).go();
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn then_discard_a_card_unless_you_attacked_this_turn() {
    cr!("608.2c");
    // Chart a Course ({1}{U} sorcery): "Draw two cards. Then discard a card unless you
    // attacked this turn."
    assert_compiles(&["Chart a Course"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let c = t.hand(P0, "Chart a Course");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // After attacking, no discard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, "Chart a Course");
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn he_fights_and_counters_on_him() {
    cr!("608.2c");
    // Wolverine, Fierce Fighter: "When Wolverine enters, he fights up to one other target
    // creature."; Zuko, Seeking Honor: "Whenever Zuko deals combat damage to a player, put
    // a +1/+1 counter on him."
    assert_compiles(&["Wolverine, Fierce Fighter", "Zuko, Seeking Honor"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Forest", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Wolverine, Fierce Fighter");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    let w = t.named_on_battlefield("Wolverine, Fierce Fighter")[0];
    assert_eq!(t.obj(w).damage, 2);
    let mut t = TestGame::new(2);
    let zuko = t.battlefield(P0, "Zuko, Seeking Honor");
    t.attack(&[(zuko, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(zuko, counters::PLUS1), 1);
}
