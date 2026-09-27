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
