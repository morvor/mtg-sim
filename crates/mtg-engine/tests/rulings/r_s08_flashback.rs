//! Rulings batch S08 — flashback (CR 702.34): "You may cast this card from your graveyard
//! [if the resulting spell is an instant or sorcery spell] by paying [cost] rather than
//! paying its mana cost" and "If the flashback cost was paid, exile this card instead of
//! putting it anywhere else any time it would leave the stack."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s03_common::*;
use crate::r_s04_common::*;
use crate::r_s07_common::*;
use crate::r_s08_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

/// Answers a priority decision by casting a spell with flashback, if one can be.
fn flashback_first(_g: &Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::Priority { actions } => actions
            .iter()
            .find(|a| matches!(a, Action::Cast { method, .. } if *method == FLASHBACK))
            .cloned()
            .map(Answer::Action),
        _ => None,
    }
}

/// Runs the game (P1 passing) until a spell cast with flashback is on the stack; the
/// players who got priority meanwhile.
fn run_until_flashed_back(t: &mut TestGame, from: usize) -> Vec<PlayerId> {
    respond(t, P0, flashback_first);
    let ok = t.g.run_until(1000, |g| {
        g.stack.iter().any(|id| {
            g.obj(*id)
                .stack
                .as_ref()
                .is_some_and(|si| si.cast.method == FLASHBACK)
        })
    });
    assert!(ok, "{}", t.dump_log());
    priority_asked_since(t, from)
}

#[test]
fn a_card_discarded_during_your_turn_can_be_flashed_back_before_anyone_else_acts() {
    cr!("702.34a", "117.3b");
    ruling!(
        "Think Twice",
        "If a card with flashback is put into your graveyard during your turn, you can cast it if it's legal to do so before any other player can take any actions."
    );
    supported("Faithless Looting");
    supported("Think Twice");
    // P0 casts Faithless Looting and discards Think Twice with it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Island", 3);
    let tt = t.hand(P0, "Think Twice");
    let bears = t.hand(P0, "Grizzly Bears");
    let looting = t.hand(P0, "Faithless Looting");
    t.cast(P0, looting).go();
    t.answer_choose(P0, &[Entity::Object(tt), Entity::Object(bears)]);
    let from = t.asked().len();
    let priority = run_until_flashed_back(&mut t, from);
    // P0 passed, P1 passed and Faithless Looting resolved; P0 then got priority first and
    // flashed Think Twice back.
    assert_eq!(priority, vec![P0, P1, P0]);
    assert!(t.in_graveyard(P0, "Faithless Looting"));
    t.resolve_all();
    assert!(t.in_exile("Think Twice"));
}

#[test]
fn a_resolved_instant_can_be_flashed_back_before_anyone_else_acts() {
    cr!("702.34a", "117.3b", "608.2n");
    ruling!(
        "Lava Dart",
        "If a card with flashback is put into your graveyard during your turn, you can cast it if it’s legal to do so before any other player can take any actions."
    );
    supported("Lava Dart");
    // Lava Dart: "deals 1 damage to any target. Flashback—Sacrifice a Mountain."
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let dart = t.hand(P0, "Lava Dart");
    t.cast(P0, dart).target(P1).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = t.asked().len();
    let priority = run_until_flashed_back(&mut t, from);
    assert_eq!(priority, vec![P0, P1, P0]);
    assert_eq!(t.life(P1), 19);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Lava Dart"));
}

#[test]
fn flashback_is_casting_from_the_graveyard_for_the_flashback_cost_then_exiling() {
    cr!("702.34a", "118.9");
    ruling!(
        "Strike It Rich",
        "\"Flashback [cost]\" means \"You may cast this card from your graveyard by paying [cost] rather than paying its mana cost\" and \"If the flashback cost was paid, exile this card instead of putting it anywhere else any time it would leave the stack.\""
    );
    supported("Strike It Rich");
    // Strike It Rich: {R} "Create a Treasure token. Flashback {2}{R}".
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let card = t.graveyard(P0, "Strike It Rich");
    // Two lands don't pay {2}{R}; its mana cost {R} isn't an option.
    assert!(!can_cast(&mut t, P0, card, FLASHBACK));
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, card, FLASHBACK));
    t.cast(P0, card).method(FLASHBACK).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    assert!(t.in_exile("Strike It Rich"));
    assert!(!t.in_graveyard(P0, "Strike It Rich"));
}

#[test]
fn flashback_casts_only_instant_or_sorcery_spells_and_exiles_them() {
    cr!("702.34a");
    ruling!(
        "Faithless Looting",
        "\"Flashback [cost]\" means \"You may cast this card from your graveyard if the resulting spell is an instant or sorcery spell by paying [cost] rather than paying its mana cost\""
    );
    // Faithless Looting from the graveyard for {2}{R}: exiled afterwards.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let looting = t.graveyard(P0, "Faithless Looting");
    t.cast(P0, looting).method(FLASHBACK).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.in_exile("Faithless Looting"));
    // A creature card with flashback can't be cast with it: the resulting spell wouldn't
    // be an instant or sorcery spell.
    t.lands(P0, "Mountain", 3);
    let golem = custom_card(
        "Flashback Golem",
        "Artifact Creature — Golem",
        "{1}",
        Some((2, 2)),
        "Flashback {1}",
    );
    let golem = t.custom(P0, golem, Zone::Graveyard(P0));
    assert!(!can_cast(&mut t, P0, golem, FLASHBACK));
}

#[test]
fn a_flashback_cost_can_be_a_sacrifice() {
    cr!("702.34a", "118.9");
    ruling!(
        "Lava Dart",
        "“Flashback [cost]” means “You may cast this card from your graveyard by paying [cost] rather than paying its mana cost” and “If the flashback cost was paid, exile this card instead of putting it anywhere else any time it would leave the stack.”"
    );
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    let dart = t.graveyard(P0, "Lava Dart");
    t.cast(P0, dart).method(FLASHBACK).target(P1).go();
    // No mana: the Mountain was sacrificed instead.
    assert!(!t.on_battlefield(mountain));
    assert!(t.in_graveyard(P0, "Mountain"));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.in_exile("Lava Dart"));
    // Without a Mountain to sacrifice, it can't be flashed back.
    let dart = t.graveyard(P0, "Lava Dart");
    t.lands(P0, "Island", 3);
    assert!(!can_cast(&mut t, P0, dart, FLASHBACK));
}

#[test]
fn a_flashback_cost_is_increased_like_any_cost_and_doesnt_change_mana_value() {
    cr!("702.34a", "601.2f", "202.3");
    ruling!(
        "Firebolt",
        "To determine the total cost of a spell, start with the mana cost or alternative cost (such as a flashback cost) you’re paying, add any cost increases, then apply any cost reductions. The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Firebolt");
    supported("Thalia, Guardian of Thraben");
    // Firebolt: {R}, "Flashback {4}{R}"; Thalia: noncreature spells cost {1} more.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Mountain", 5);
    let bolt = t.graveyard(P0, "Firebolt");
    assert!(!can_cast(&mut t, P0, bolt, FLASHBACK));
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, bolt, FLASHBACK));
    let spell = t.cast(P0, bolt).method(FLASHBACK).target(P1).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(mana_value(&t, spell), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_sorcery_can_be_flashed_back_only_at_sorcery_speed() {
    cr!("702.34a", "307.1");
    ruling!(
        "Firebolt",
        "You must still follow any timing restrictions and permissions, including those based on the card’s type. For instance, you can cast a sorcery using flashback only when you could normally cast a sorcery."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let bolt = t.graveyard(P0, "Firebolt");
    assert!(can_cast(&mut t, P0, bolt, FLASHBACK));
    // Not in combat, not in the opponent's turn, not with a spell on the stack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, bolt, FLASHBACK));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, bolt, FLASHBACK));
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Lightning Bolt");
    t.cast(P1, shock).target(P1).go();
    assert!(!can_cast(&mut t, P0, bolt, FLASHBACK));
    t.resolve_all();
    assert!(can_cast(&mut t, P0, bolt, FLASHBACK));
}

/// P0 casts Grizzly Fate (from its hand, or from its graveyard with flashback) with
/// `others` other cards in its graveyard. The number of Bears created.
fn grizzly_fate(others: usize, flashback: bool) -> usize {
    let mut t = TestGame::new(2);
    graveyard_n(&mut t, P0, "Grizzly Bears", others);
    t.lands(P0, "Forest", 7);
    let fate = if flashback {
        t.graveyard(P0, "Grizzly Fate")
    } else {
        t.hand(P0, "Grizzly Fate")
    };
    let method = if flashback {
        FLASHBACK
    } else {
        CastMethod::Normal
    };
    t.cast(P0, fate).method(method).go();
    t.resolve_all();
    with_subtype(&t, P0, "Bear").len()
}

#[test]
fn a_resolving_threshold_spell_doesnt_count_itself_in_the_graveyard() {
    cr!("702.34a", "608.2n", "207.2c");
    ruling!(
        "Grizzly Fate",
        "At the time this spell is resolving, it is on the stack and not in the graveyard."
    );
    supported("Grizzly Fate");
    // Seven other cards: threshold.
    assert_eq!(grizzly_fate(7, false), 4);
    // Six: no threshold as it resolves.
    assert_eq!(grizzly_fate(6, false), 2);
    // Flashed back with six others: the graveyard had seven cards when it was cast, but
    // only six while it resolves.
    assert_eq!(grizzly_fate(6, true), 2);
}
