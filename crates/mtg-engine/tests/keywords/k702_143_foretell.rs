//! CR 702.143 Foretell: effects referring to foretelling and foretold cards, cards that
//! become foretold, and revealing foretold cards.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_052_066::run_effect;
use crate::common_k702_140_152::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Decision, SpecialAction};
use mtg_engine::events::Event;
use mtg_engine::facedown::REVEALED;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::foretell::face_down_foretold;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FORETELL: CastMethod = CastMethod::Keyword(KeywordKind::Foretell);

/// Foretells `card` from `p`'s hand (paying {2}) and returns it in exile.
fn foretell(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    add_mana(t, p, ManaType::C, 2);
    t.g.turn.priority = Some(p);
    t.g.take_action(p, Action::Special(SpecialAction::Foretell { card }));
    t.g.flush_events();
    let new = t.g.current(card);
    assert_eq!(t.obj(new).zone, Zone::Exile);
    assert!(t.obj(new).face_down);
    new
}

fn scried(t: &TestGame) -> bool {
    t.asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::Scry { .. }))
}

fn revealed(t: &TestGame, c: ObjectId) -> bool {
    t.g.turn_events.iter().chain(t.g.events.iter()).any(|e| {
        matches!(e, Event::Custom { name, obj: Some(o), .. } if name == REVEALED && *o == c)
    })
}

#[test]
fn whenever_you_foretell_a_card_triggers_on_the_special_action() {
    cr!("702.143", "702.143c");
    // Dream Devourer's "Whenever you foretell a card, this creature gets +2/+0 until end of
    // turn."
    let def = custom_card(
        "Foretelling Devourer",
        "Creature — Demon Cleric",
        Some((0, 3)),
        "Whenever you foretell a card, ~ gets +2/+0 until end of turn.",
    );
    let mut t = TestGame::new(2);
    let devourer = t.custom(P0, def, Zone::Battlefield);
    let bolt = t.hand(P0, "Demon Bolt");
    foretell(&mut t, P0, bolt);
    t.resolve_all();
    assert_eq!(t.pt(devourer), (2, 3));
    // A card that becomes foretold through an effect wasn't foretold by the special
    // action: that isn't foretelling a card.
    let soldier = t.battlefield(P0, "The Foretold Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    run_effect(
        &mut t,
        Some(soldier),
        P0,
        Effect::Fight {
            a: Sel::This,
            b: Sel::Target(0),
        },
        &[Entity::Object(bears)],
    );
    t.resolve_all();
    assert_eq!(face_down_foretold(&t.g, Some(P0)).len(), 2);
    assert_eq!(t.pt(devourer), (2, 3));
}

#[test]
fn a_spell_that_was_foretold() {
    cr!("702.143c");
    assert_supported("Poison the Cup");
    let mut t = TestGame::new(2);
    // Poison the Cup: "Destroy target creature. If this spell was foretold, scry 2."
    // Foretell {1}{B}.
    let a = t.battlefield(P1, "Grizzly Bears");
    let cup = t.hand(P0, "Poison the Cup");
    let foretold = foretell(&mut t, P0, cup);
    t.advance_to(P1, Step::Upkeep);
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, foretold).method(FORETELL).target(a).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(scried(&t));
    // Cast from the hand, it wasn't foretold.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let cup = t.hand(P0, "Poison the Cup");
    add_mana(&mut t, P0, ManaType::B, 2);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, cup).target(a).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(!scried(&t));
}

#[test]
fn a_foretold_card_cast_for_another_cost_was_still_foretold() {
    cr!("702.143c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let cup = t.hand(P0, "Poison the Cup");
    let foretold = foretell(&mut t, P0, cup);
    t.advance_to(P1, Step::Upkeep);
    // An effect lets P0 cast it without paying its mana cost.
    t.answer_targets(P0, &[Entity::Object(a)]);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(foretold)],
    );
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(scried(&t));
}

#[test]
fn a_card_that_becomes_foretold_may_be_cast_for_its_foretell_cost() {
    cr!("702.143d");
    assert_supported("The Foretold Soldier");
    let mut t = TestGame::new(2);
    // The Foretold Soldier: 6/6, "Whenever this creature deals damage, exile it face down.
    // It becomes foretold." Foretell {1}{G}.
    let soldier = t.battlefield(P0, "The Foretold Soldier");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(soldier, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    let card = t.g.current(soldier);
    assert_eq!(t.obj(card).zone, Zone::Exile);
    assert!(t.obj(card).face_down);
    assert_eq!(face_down_foretold(&t.g, Some(P0)), vec![card]);
    // Its owner may look at it.
    assert!(mtg_engine::zones::may_look(&t.g, P0, card));
    // Not this turn.
    t.advance_to(P0, Step::PostcombatMain);
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(!castable(&mut t, P0, card, FORETELL));
    // After this turn, for its foretell cost {1}{G}.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::G, 2);
    assert!(castable(&mut t, P0, card, FORETELL));
    t.cast(P0, card).method(FORETELL).go();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    assert!(t.on_battlefield(card));
}

#[test]
fn an_effect_may_give_a_foretold_card_a_foretell_cost() {
    cr!("702.143d");
    assert_supported("Ethereal Valkyrie");
    let mut t = TestGame::new(2);
    // Ethereal Valkyrie: "Whenever this creature enters or attacks, draw a card, then
    // exile a card from your hand face down. It becomes foretold. Its foretell cost is its
    // mana cost reduced by {2}."
    let giant = t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Ethereal Valkyrie");
    t.resolve_all();
    let card = t.g.current(giant);
    assert_eq!(t.obj(card).zone, Zone::Exile);
    assert!(t.obj(card).face_down);
    // Hill Giant ({3}{R}, no foretell) may be cast for {1}{R} after this turn.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::R, 2);
    assert!(castable(&mut t, P0, card, FORETELL));
    t.cast(P0, card).method(FORETELL).go();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    assert!(t.on_battlefield(card));
}

#[test]
fn foretold_cards_are_kept_apart_with_their_order_and_costs() {
    cr!("702.143e");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Demon Bolt");
    let first = foretell(&mut t, P0, bolt);
    let giant = t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Ethereal Valkyrie");
    t.resolve_all();
    let second = t.g.current(giant);
    let cup = t.hand(P0, "Poison the Cup");
    let third = foretell(&mut t, P0, cup);
    // In the order they were exiled, each its own card.
    assert_eq!(
        face_down_foretold(&t.g, Some(P0)),
        vec![first, second, third]
    );
    assert!(face_down_foretold(&t.g, Some(P1)).is_empty());
    for c in [first, second, third] {
        assert!(mtg_engine::zones::may_look(&t.g, P0, c));
        assert!(!mtg_engine::zones::may_look(&t.g, P1, c));
    }
    // Each keeps its own foretell cost: {R}, {1}{R} (not printed on the card), {1}{B}.
    t.advance_to(P1, Step::Upkeep);
    let target = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, first).method(FORETELL).target(target).go();
    assert_eq!(pool(&t, P0), 0);
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, third).method(FORETELL).target(target).go();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, second, FORETELL));
    add_mana(&mut t, P0, ManaType::C, 1);
    assert!(castable(&mut t, P0, second, FORETELL));
}

#[test]
fn face_down_foretold_cards_are_revealed_when_their_owner_leaves_and_at_game_end() {
    cr!("702.143f");
    let mut t = TestGame::new(3);
    let bolt = t.hand(P0, "Demon Bolt");
    let mine = foretell(&mut t, P0, bolt);
    t.set_step(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Demon Bolt");
    let theirs = foretell(&mut t, P1, bolt);
    t.set_step(P2, Step::PrecombatMain);
    let bolt = t.hand(P2, "Demon Bolt");
    let other = foretell(&mut t, P2, bolt);
    assert!(!revealed(&t, mine) && !revealed(&t, theirs) && !revealed(&t, other));
    // P1 leaves the game: their face-down foretold card is revealed to all players.
    t.g.player_loses(P1);
    t.g.flush_events();
    assert!(revealed(&t, theirs));
    assert!(!revealed(&t, mine));
    assert!(!revealed(&t, other));
    // At the end of the game, all of them are revealed.
    t.g.player_loses(P2);
    t.g.flush_events();
    assert!(t.g.is_over());
    assert!(revealed(&t, mine));
    assert!(revealed(&t, other));
}
