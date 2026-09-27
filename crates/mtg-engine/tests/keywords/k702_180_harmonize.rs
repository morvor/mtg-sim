//! CR 702.180 Harmonize (`src/kw/harmonize.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const HARMONIZE: CastMethod = CastMethod::Keyword(KeywordKind::Harmonize);

#[test]
fn harmonize_cards_compile() {
    assert_supported(&[
        "Wild Ride",
        "Unending Whisper",
        "Channeled Dragonfire",
        "Nature's Rhythm",
        "Songcrafter Mage",
        "Mammoth Bellow",
    ]);
}

/// Answers the harmonize choice of the creature to tap with none.
fn tap_nothing(t: &mut TestGame) {
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![]));
}

#[test]
fn cast_from_the_graveyard_for_the_harmonize_cost_then_exiled() {
    cr!("702.180a");
    ruling!(
        "Unending Whisper",
        "A spell cast using harmonize will always be exiled afterward"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let w = t.graveyard(P0, "Unending Whisper");
    let hand = t.hand_size(P0);
    t.cast(P0, w).method(HARMONIZE).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_exile("Unending Whisper"));
    assert!(!t.in_graveyard(P0, "Unending Whisper"));
    // The full cost was paid: {5}{U}.
    let untapped = t
        .g
        .permanents()
        .filter(|o| o.name() == "Island" && !o.tapped)
        .count();
    assert_eq!(untapped, 0);
}

#[test]
fn harmonize_works_only_from_the_graveyard() {
    cr!("702.180a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let w = t.hand(P0, "Unending Whisper");
    assert!(t.cast(P0, w).method(HARMONIZE).try_go().is_err());
    // Not enough mana for the harmonize cost: it can't be cast that way.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let w = t.graveyard(P0, "Unending Whisper");
    assert!(t.cast(P0, w).method(HARMONIZE).try_go().is_err());
    assert!(t.in_graveyard(P0, "Unending Whisper"));
}

#[test]
fn tapping_a_creature_reduces_the_generic_cost_by_its_power() {
    cr!("702.180a", "702.180b");
    ruling!(
        "Unending Whisper",
        "Tapping a creature won’t reduce colored mana components of harmonize costs."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 3);
    let w = t.graveyard(P0, "Unending Whisper");
    let asked = t.asked().len();
    choose_objects(&mut t, P0, &[giant]);
    t.cast(P0, w).method(HARMONIZE).go();
    // The creature was chosen as the harmonize cost was chosen, and tapped as the total
    // cost ({2}{U}) was paid.
    assert!(t.obj_now(giant).tapped);
    assert!(asked_since(&t, asked).iter().any(|(_, d)| matches!(
        d,
        Decision::ChooseEntities { prompt, .. } if prompt.contains("harmonize")
    )));
    assert!(t.g.permanents().all(|o| o.name() != "Island" || o.tapped));
    t.resolve_all();
    assert!(t.in_exile("Unending Whisper"));
    // A creature with more power than the generic part: only the colored mana is paid.
    let mut t = TestGame::new(2);
    let big = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ride = t.graveyard(P0, "Wild Ride");
    choose_objects(&mut t, P0, &[big]);
    t.cast(P0, ride).method(HARMONIZE).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(big).tapped);
    assert_eq!(t.pt(bears), (5, 2));
    // Without the creature, {4}{R} can't be paid with one land.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ride = t.graveyard(P0, "Wild Ride");
    tap_nothing(&mut t);
    assert!(t
        .cast(P0, ride)
        .method(HARMONIZE)
        .target(bears)
        .try_go()
        .is_err());
}

#[test]
fn tapping_a_creature_is_optional_and_can_be_summoning_sick() {
    cr!("702.180a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield_sick(P0, "Hill Giant");
    t.lands(P0, "Island", 6);
    let w = t.graveyard(P0, "Unending Whisper");
    tap_nothing(&mut t);
    t.cast(P0, w).method(HARMONIZE).go();
    assert!(!t.obj_now(giant).tapped);
    assert!(t.g.permanents().all(|o| o.name() != "Island" || o.tapped));
    // A summoning-sick creature can be tapped for it.
    let mut t = TestGame::new(2);
    let giant = t.battlefield_sick(P0, "Hill Giant");
    t.lands(P0, "Island", 3);
    let w = t.graveyard(P0, "Unending Whisper");
    choose_objects(&mut t, P0, &[giant]);
    t.cast(P0, w).method(HARMONIZE).go();
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn a_harmonized_spell_that_is_countered_is_exiled() {
    cr!("702.180a");
    ruling!(
        "Unending Whisper",
        "whether it resolves, is countered, or leaves the stack in some other way"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    t.lands(P1, "Island", 2);
    let w = t.graveyard(P0, "Unending Whisper");
    let spell = t.cast(P0, w).method(HARMONIZE).go();
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_exile("Unending Whisper"));
    assert!(!t.in_graveyard(P0, "Unending Whisper"));
}

#[test]
fn harmonize_follows_timing_rules_and_keeps_the_mana_value() {
    cr!("702.180a");
    ruling!(
        "Unending Whisper",
        "You must still follow any timing restrictions and permissions"
    );
    ruling!(
        "Unending Whisper",
        "The mana value of the spell is determined only by its mana cost"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let w = t.graveyard(P0, "Unending Whisper");
    // A sorcery can't be cast during another player's turn.
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, w).method(HARMONIZE).try_go().is_err());
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.cast(P0, w).method(HARMONIZE).go();
    assert_eq!(t.g.mana_value_of(spell), 1);
}

#[test]
fn harmonize_with_x_in_its_cost() {
    cr!("702.180a");
    // "Search your library for a creature card with mana value X or less, put it onto the
    // battlefield, then shuffle. Harmonize {X}{G}{G}{G}{G}"
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 4);
    t.library_top(P0, "Grizzly Bears");
    let r = t.graveyard(P0, "Nature's Rhythm");
    choose_objects(&mut t, P0, &[giant]);
    // X = 2: {2}{G}{G}{G}{G}, reduced by the Giant's power to {G}{G}{G}{G}.
    t.cast(P0, r).method(HARMONIZE).x(2).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_exile("Nature's Rhythm"));
}

#[test]
fn a_granted_harmonize_ability_costs_the_card_mana_cost() {
    cr!("702.180a");
    // Songcrafter Mage: "target instant or sorcery card in your graveyard gains harmonize
    // until end of turn. Its harmonize cost is equal to its mana cost."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).method(HARMONIZE).target(P1).try_go().is_err());
    t.clear_answers();
    let mage = t.enter(P0, "Songcrafter Mage");
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.resolve_all();
    assert!(t.obj_now(bolt).chars.has_keyword(KeywordKind::Harmonize));
    t.lands(P0, "Mountain", 1);
    tap_nothing(&mut t);
    t.cast(P0, bolt).method(HARMONIZE).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.on_battlefield(mage));
    assert_eq!(t.zone(bolt), Zone::Exile);
}
