//! CR 702.35 Madness.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use crate::common_k702_027_037::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const MADNESS: &str = "Madness";

/// Empties `p`'s hand into the library (so tests control exactly what's in hand).
fn empty_hand(t: &mut TestGame, p: PlayerId) {
    for c in t.g.player(p).hand.clone() {
        t.g.players[p.idx()].hand.retain(|x| *x != c);
        t.g.players[p.idx()].library.push(c);
        t.g.objects[c.0 as usize].zone = Zone::Library(p);
    }
}

/// In P1's main phase, P1 casts Mind Rot targeting P0 (who discards two cards) and it
/// resolves.
fn mind_rot_p0(t: &mut TestGame) {
    if t.g.turn.active != P1 {
        t.set_step(P1, Step::PrecombatMain);
    }
    t.lands(P1, "Swamp", 3);
    let rot = t.hand(P1, "Mind Rot");
    t.g.turn.priority = Some(P1);
    t.cast(P1, rot).target(P0).go();
    t.resolve();
}

/// The sources of the triggered abilities on the stack, bottom first.
fn stack_trigger_sources(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .filter_map(|s| match &t.g.obj(*s).stack.as_ref()?.kind {
            StackKind::Triggered { source, .. } => Some(*source),
            _ => None,
        })
        .collect()
}

#[test]
fn a_discarded_card_with_madness_is_exiled_and_can_be_cast_for_its_madness_cost() {
    cr!("702.35", "702.35a");
    ruling!(
        "Fiery Temper",
        "A spell cast for its madness cost is put onto the stack like any other spell."
    );
    assert_supported("Fiery Temper");
    assert_supported("Mind Rot");
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    t.lands(P0, "Mountain", 1);
    let temper = t.hand(P0, "Fiery Temper");
    t.hand(P0, "Grizzly Bears");
    mind_rot_p0(&mut t);
    // Discarded into exile; the other card went to the graveyard.
    assert!(t.in_exile("Fiery Temper"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.settle();
    assert_eq!(triggers_on_stack(&t, MADNESS), 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    // Cast for {R} rather than {1}{R}{R}: a spell on the stack.
    assert_eq!(t.zone(temper), Zone::Stack);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Fiery Temper"));
}

#[test]
fn if_it_isnt_cast_the_card_is_put_into_the_graveyard() {
    cr!("702.35a");
    ruling!(
        "Fiery Temper",
        "If you choose not to cast a card with madness when the madness triggered ability resolves, it's put into your graveyard. Madness doesn't give you another chance to cast it later."
    );
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    t.lands(P0, "Mountain", 1);
    let temper = t.hand(P0, "Fiery Temper");
    mind_rot_p0(&mut t);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(t.in_graveyard(P0, "Fiery Temper"));
    assert_eq!(untapped_lands(&t, P0), 1);
    assert!(!can_cast(&mut t, P0, temper, CastMethod::Keyword(KeywordKind::Madness)));
    // It can't be paid: it's put into the graveyard too.
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    t.hand(P0, "Fiery Temper");
    mind_rot_p0(&mut t);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.in_graveyard(P0, "Fiery Temper"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_card_discarded_into_exile_still_counts_as_discarded() {
    cr!("702.35a");
    ruling!(
        "Fiery Temper",
        "A card with madness that's discarded counts as having been discarded even though it's put into exile rather than a graveyard. If it was discarded to pay a cost, that cost is still paid. Abilities that trigger when a card is discarded will still trigger."
    );
    assert_supported("Olivia's Dragoon");
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    let haven = t.battlefield(P0, "Drake Haven");
    let dragoon = t.battlefield(P0, "Olivia's Dragoon");
    let temper = t.hand(P0, "Fiery Temper");
    // "Discard a card: ~ gains flying until end of turn": paid with Fiery Temper.
    t.answer_choose(P0, &[Entity::Object(temper)]);
    t.activate(P0, dragoon, 0, &[]).unwrap();
    t.settle();
    assert!(t.in_exile("Fiery Temper"));
    let sources = stack_trigger_sources(&t);
    assert!(sources.contains(&haven));
    assert_eq!(triggers_on_stack(&t, MADNESS), 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.obj_now(dragoon).has_keyword(KeywordKind::Flying));
}

#[test]
fn madness_ignores_timing_based_on_card_type() {
    cr!("702.35a");
    ruling!(
        "Fiery Temper",
        "Casting a spell with madness ignores the timing rules based on the card's card type. For example, you can cast a sorcery with madness if you discard it during an opponent's turn."
    );
    assert_supported("Murderous Compulsion");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    empty_hand(&mut t, P0);
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.hand(P0, "Murderous Compulsion");
    mind_rot_p0(&mut t);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Murderous Compulsion"));
}

#[test]
fn a_card_discarded_to_pay_a_cost_is_cast_before_the_spell_it_paid_for_resolves() {
    cr!("702.35a");
    ruling!(
        "Fiery Temper",
        "If you discard a card with madness to pay the cost of a spell or activated ability, that card's madness triggered ability (and the spell that card becomes, if you choose to cast it) will resolve before the spell or ability the discard paid for."
    );
    assert_supported("Tormenting Voice");
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    t.lands(P0, "Mountain", 3);
    let voice = t.hand(P0, "Tormenting Voice");
    let temper = t.hand(P0, "Fiery Temper");
    t.answer_choose(P0, &[Entity::Object(temper)]);
    t.cast(P0, voice).go();
    t.settle();
    // The madness trigger is above Tormenting Voice.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_on_stack(&t, MADNESS), 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Tormenting Voice still resolves afterward.
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn a_card_discarded_while_a_spell_resolves_is_in_exile_until_that_spell_is_done() {
    cr!("702.35a");
    ruling!(
        "Fiery Temper",
        "If you discard a card with madness while a spell or ability is resolving, it moves immediately to exile. Continue resolving that spell or ability, noting that the card you discarded is not in your graveyard at this time."
    );
    assert_supported("Faithless Looting");
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    t.lands(P0, "Mountain", 2);
    let looting = t.hand(P0, "Faithless Looting");
    let temper = t.hand(P0, "Fiery Temper");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(temper), Entity::Object(bears)]);
    t.cast(P0, looting).go();
    t.g.resolve_top();
    // Faithless Looting finished resolving; the madness trigger hasn't been put on the
    // stack yet, and the card is in exile, not the graveyard.
    assert!(t.in_exile("Fiery Temper"));
    assert!(!t.in_graveyard(P0, "Fiery Temper"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(triggers_on_stack(&t, MADNESS), 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn madness_works_for_discards_during_cleanup() {
    cr!("702.35a");
    ruling!(
        "Fiery Temper",
        "Madness works independently of why you're discarding the card. You could discard it to pay a cost, because a spell or ability tells you to, or because you have too many cards in your hand during your cleanup step."
    );
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    t.lands(P0, "Mountain", 1);
    let temper = t.hand(P0, "Fiery Temper");
    for _ in 0..7 {
        t.hand(P0, "Grizzly Bears");
    }
    t.answer_choose(P0, &[Entity::Object(temper)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P0, "Fiery Temper"));
}

#[test]
fn madness_is_an_alternative_cost() {
    cr!("702.35b");
    ruling!(
        "Fiery Temper",
        "To determine the total cost of a spell, start with the mana cost or alternative cost (such as a madness cost) you're paying, add any cost increases, then apply any cost reductions. The mana value of the spell is determined by only its mana cost"
    );
    ruling!(
        "Blast from the Past",
        "If you cast Blast from the Past using madness, you may also pay for kicker or buyback."
    );
    assert_supported("Blast from the Past");
    // Thalia's increase applies to the madness cost {R}.
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Mountain", 2);
    // Mind Rot costs {1} more too.
    t.lands(P1, "Swamp", 1);
    let temper = t.hand(P0, "Fiery Temper");
    mind_rot_p0(&mut t);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.obj_now(temper).chars.mana_value(), 3);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Blast from the Past cast for its madness cost, kicked and with buyback.
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    t.lands(P0, "Mountain", 9);
    t.hand(P0, "Blast from the Past");
    mind_rot_p0(&mut t);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    // {R} + kicker {2}{R} + buyback {4}{R}.
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(tokens(&t, P0), 1);
    assert!(t.in_hand(P0, "Blast from the Past"));
}

#[test]
fn a_madness_card_that_isnt_cast_can_be_found_in_the_graveyard_afterward() {
    cr!("702.35c", "400.7k");
    assert_supported("Pitchstone Wall");
    // The madness trigger resolves first: the card goes to the graveyard, where Pitchstone
    // Wall's ability (referring to the discarded card) finds it.
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    let wall = t.battlefield(P0, "Pitchstone Wall");
    t.hand(P0, "Fiery Temper");
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    mind_rot_p0(&mut t);
    t.settle();
    let sources = stack_trigger_sources(&t);
    assert_eq!(sources.len(), 2);
    assert_eq!(sources[0], wall, "madness trigger on top");
    t.answer_yes(P0, false);
    t.resolve();
    assert!(t.in_graveyard(P0, "Fiery Temper"));
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.on_battlefield(wall));
    assert!(t.in_hand(P0, "Fiery Temper"));
    // If Pitchstone Wall's ability resolves first, the card is still in exile: it can't
    // be returned from the graveyard.
    let mut t = TestGame::new(2);
    empty_hand(&mut t, P0);
    let wall = t.battlefield(P0, "Pitchstone Wall");
    t.hand(P0, "Fiery Temper");
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    mind_rot_p0(&mut t);
    t.settle();
    assert_eq!(*stack_trigger_sources(&t).last().unwrap(), wall);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.on_battlefield(wall));
    assert!(t.in_exile("Fiery Temper"));
    t.answer_yes(P0, false);
    t.resolve();
    assert!(t.in_graveyard(P0, "Fiery Temper"));
    assert!(!t.in_hand(P0, "Fiery Temper"));
}
