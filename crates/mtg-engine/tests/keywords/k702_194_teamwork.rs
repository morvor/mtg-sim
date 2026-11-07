//! CR 702.194 Teamwork (`src/kw/teamwork.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Repulsor Blast ({3}{R} sorcery): "Teamwork 2. Repulsor Blast deals 5 damage to target
/// creature. If this spell was cast using teamwork, it also deals 2 damage to that
/// creature's controller."
const BLAST: &str = "Repulsor Blast";

#[test]
fn teamwork_cards_compile() {
    assert_supported(&[
        BLAST,
        "Go Nuts!",
        "Widow's Bite",
        "Quantum Reduction",
        "Team Tactics",
        "Heroic Teamwork",
    ]);
}

#[test]
fn an_optional_additional_cost_tapping_creatures_with_total_power_n() {
    cr!("702.194a", "702.194b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, BLAST);
    t.cast(P0, c).kicked(true).target(wurm).go();
    assert!(t.obj(bears).tapped);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P1), 18);
    // Without teamwork: no creature is tapped and only the creature is dealt damage.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, BLAST);
    t.cast(P0, c).kicked(false).target(wurm).go();
    assert!(!t.obj(bears).tapped);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn any_number_of_untapped_creatures_you_control_with_enough_total_power() {
    cr!("702.194a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let tapped = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[tapped.0 as usize].tapped = true;
    t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, BLAST);
    // Total power 1 among the untapped creatures its caster controls: teamwork 2 can't be
    // paid, so it's cast without teamwork.
    t.cast(P0, c).kicked(true).target(wurm).go();
    assert!(!t.obj(elves).tapped);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, BLAST);
    t.lands(P0, "Mountain", 4);
    // Two creatures with power 1 together can.
    let elves2 = t.battlefield(P0, "Llanowar Elves");
    t.cast(P0, c).kicked(true).target(wurm).go();
    assert!(t.obj(elves).tapped && t.obj(elves2).tapped);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn choose_both_modes_if_cast_using_teamwork() {
    cr!("702.194b");
    // Go Nuts! ({G} sorcery): "Teamwork 3. Choose one. If this spell was cast using
    // teamwork, choose both instead. • Put a +1/+1 counter on target creature. • Target
    // creature you control fights target creature an opponent controls."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Go Nuts!");
    t.cast(P0, c)
        .kicked(true)
        .modes(&[0, 1])
        .targets(&[Entity::Object(giant)])
        .targets(&[Entity::Object(giant)])
        .targets(&[Entity::Object(bears)])
        .go();
    assert!(t.obj(giant).tapped);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
    assert!(!t.on_battlefield(bears));
    // Without teamwork, only one mode.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Go Nuts!");
    let spell = t
        .cast(P0, c)
        .kicked(false)
        .modes(&[0, 1])
        .targets(&[Entity::Object(giant)])
        .targets(&[Entity::Object(giant)])
        .targets(&[Entity::Object(bears)])
        .try_go();
    if let Ok(s) = spell {
        assert_eq!(t.g.obj(s).stack.as_ref().unwrap().chosen.len(), 1);
    }
    t.clear_answers();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn targets_of_a_part_that_needs_teamwork_are_chosen_only_with_teamwork() {
    cr!("702.194c");
    let def = custom_card(
        "Joint Demolition",
        "{1}{R}",
        "Sorcery",
        None,
        "Teamwork 2\nDestroy target artifact. If this spell was cast using teamwork, destroy target enchantment.",
    );
    // Without teamwork, it's cast as if it had no enchantment target: castable with none
    // around.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.battlefield(P0, "Grizzly Bears");
    let rock = t.battlefield(P1, "Mind Stone");
    let c = put(&mut t, P0, def.clone(), Zone::Hand(P0));
    let spell = t.cast(P0, c).kicked(false).target(rock).go();
    assert_eq!(
        t.g.obj(spell).stack.as_ref().unwrap().chosen[0]
            .targets
            .iter()
            .filter(|v| !v.is_empty())
            .count(),
        1
    );
    t.resolve_all();
    assert!(!t.on_battlefield(rock));
    // With teamwork, the enchantment target is chosen too.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.battlefield(P0, "Grizzly Bears");
    let rock = t.battlefield(P1, "Mind Stone");
    let ench = t.battlefield(P1, "Glorious Anthem");
    let c = put(&mut t, P0, def, Zone::Hand(P0));
    t.cast(P0, c)
        .kicked(true)
        .targets(&[Entity::Object(rock)])
        .targets(&[Entity::Object(ench)])
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(rock));
    assert!(!t.on_battlefield(ench));
}

#[test]
fn flash_if_it_is_cast_using_teamwork() {
    cr!("702.194b");
    // Quantum Reduction ({1}{U} Aura): "Teamwork 2. You may cast this spell as though it
    // had flash if it's cast using teamwork."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Quantum Reduction");
    t.set_step(P1, Step::BeginningOfCombat);
    // Not at instant speed without teamwork.
    assert!(t.cast(P0, c).kicked(false).target(wurm).try_go().is_err());
    t.clear_answers();
    t.cast(P0, c)
        .method(CastMethod::Keyword(KeywordKind::Teamwork))
        .target(wurm)
        .go();
    assert!(t.obj(bears).tapped);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (1, 4));
}

/// The target decisions asked since `from`: (text, candidates).
fn target_choices(t: &TestGame, from: usize) -> Vec<(String, Vec<Entity>)> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets {
                text, candidates, ..
            } => Some((text, candidates)),
            _ => None,
        })
        .collect()
}

#[test]
fn instead_a_part_with_its_own_target_cruel_alliance() {
    cr!("702.194b", "702.194c");
    // Cruel Alliance ({2}{B} sorcery): "Teamwork 2. Exile target creature with mana value
    // 3 or less. If this spell was cast using teamwork, instead exile target creature and
    // you gain 3 life."
    assert_supported(&["Cruel Alliance"]);
    // With teamwork, only the replacement's target is chosen: any creature.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Cruel Alliance");
    let from = t.asked().len();
    t.cast(P0, c).kicked(true).target(wurm).go();
    assert!(t.obj(bears).tapped);
    let choices = target_choices(&t, from);
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].0, "target creature");
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P0), 23);
    // Without teamwork, only a creature with mana value 3 or less can be the target, and
    // no life is gained.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Cruel Alliance");
    let from = t.asked().len();
    t.cast(P0, c).kicked(false).target(theirs).go();
    assert!(!t.obj(bears).tapped);
    let choices = target_choices(&t, from);
    assert_eq!(choices.len(), 1);
    assert!(!choices[0].1.contains(&Entity::Object(wurm)));
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn the_chosen_card_is_whichever_target_was_chosen() {
    cr!("702.194c");
    // Too Evil to Stay Dead ({2}{B} sorcery): "Teamwork 4. Choose target creature card in
    // your graveyard with mana value 4 or less. If this spell was cast using teamwork,
    // instead choose target creature card in your graveyard. Return the chosen card to
    // the battlefield."
    assert_supported(&["Too Evil to Stay Dead"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let wurm = t.graveyard(P0, "Craw Wurm");
    t.graveyard(P0, "Grizzly Bears");
    let c = t.hand(P0, "Too Evil to Stay Dead");
    t.cast(P0, c).kicked(true).target(wurm).go();
    assert!(t.obj(giant).tapped && t.obj(elves).tapped);
    t.resolve_all();
    assert_eq!(named(&t, "Craw Wurm").len(), 1);
    // Without teamwork: a creature card with mana value 4 or less.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let wurm = t.graveyard(P0, "Craw Wurm");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let c = t.hand(P0, "Too Evil to Stay Dead");
    let from = t.asked().len();
    t.cast(P0, c).kicked(false).target(bears).go();
    let choices = target_choices(&t, from);
    assert_eq!(choices.len(), 1);
    assert!(!choices[0].1.contains(&Entity::Object(wurm)));
    t.resolve_all();
    assert_eq!(named(&t, "Grizzly Bears").len(), 1);
    assert!(named(&t, "Craw Wurm").is_empty());
}

#[test]
fn unless_this_spell_was_cast_using_teamwork() {
    cr!("702.194b");
    // Timeline Inquiry ({3}{U} instant): "Teamwork 2. Draw three cards. Then discard a
    // card unless this spell was cast using teamwork."
    assert_supported(&["Timeline Inquiry"]);
    for teamwork in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 4);
        t.battlefield(P0, "Grizzly Bears");
        for _ in 0..5 {
            t.library_top(P0, "Island");
        }
        let c = t.hand(P0, "Timeline Inquiry");
        t.cast(P0, c).kicked(teamwork).go();
        t.resolve_all();
        let hand = t.g.player(P0).hand.len();
        assert_eq!(hand, if teamwork { 3 } else { 2 }, "teamwork: {teamwork}");
    }
}

#[test]
fn also_if_this_spell_was_cast_using_teamwork() {
    cr!("702.194b");
    // Beast Mode ({1}{G} instant): "Teamwork 1. Target creature gets +2/+2 and gains
    // trample until end of turn. Also put a +1/+1 counter on that creature if this spell
    // was cast using teamwork."
    assert_supported(&["Beast Mode"]);
    for teamwork in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        t.battlefield(P0, "Llanowar Elves");
        let giant = t.battlefield(P0, "Hill Giant");
        let c = t.hand(P0, "Beast Mode");
        t.cast(P0, c).kicked(teamwork).target(giant).go();
        t.resolve_all();
        let n = t.counters(giant, counters::PLUS1);
        assert_eq!(n, if teamwork { 1 } else { 0 }, "teamwork: {teamwork}");
        assert_eq!(t.pt(giant).0, 5 + n as i32);
    }
}
