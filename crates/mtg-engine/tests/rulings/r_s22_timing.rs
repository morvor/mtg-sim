//! Rulings batch S22 — permissions to play cards from other zones and alternative costs
//! don't change the timing rules or the costs of the spells: a creature spell is still
//! cast only during its controller's main phase while the stack is empty (CR 307.1,
//! 601.3, 601.2f), a land is played only as a special action during the main phase
//! (CR 305.2), and all costs — additional costs included — are paid (CR 601.2f–h);
//! alternative costs may be paid (CR 118.9).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, can_play_land};
use crate::r_s07_common::cast_methods;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const OVERLOAD: CastMethod = CastMethod::Keyword(KeywordKind::Overload);

/// Whether `p` could cast the card `card` (normally) in each situation: during their
/// main phase with an empty stack, with a spell on the stack, in their combat, and during
/// the opponent's turn.
fn creature_timing(t: &mut TestGame, p: PlayerId, card: ObjectId) -> [bool; 4] {
    let main = can_cast(t, p, card, CastMethod::Normal);
    // A spell on the stack (P0's Lightning Bolt at P1).
    t.lands(p, "Mountain", 1);
    let bolt = t.hand(p, "Lightning Bolt");
    let spell = t.cast(p, bolt).target(Entity::Player(P1)).go();
    let with_stack = can_cast(t, p, card, CastMethod::Normal);
    t.resolve_all();
    assert!(!t.g.stack.contains(&spell));
    t.set_step(p, Step::DeclareAttackers);
    let combat = can_cast(t, p, card, CastMethod::Normal);
    let other = if p == P0 { P1 } else { P0 };
    t.g.combat = None;
    t.set_step(other, Step::PrecombatMain);
    let their_turn = can_cast(t, p, card, CastMethod::Normal);
    t.set_step(p, Step::PrecombatMain);
    [main, with_stack, combat, their_turn]
}

#[test]
fn heroes_hangout_the_exiled_card_is_played_following_the_timing_rules() {
    cr!("601.3", "307.1", "305.2", "601.2f");
    ruling!(
        "Heroes' Hangout",
        "You must follow all normal timing rules when playing a land or casting a spell this way."
    );
    supported("Heroes' Hangout");
    // Date Night — exile the top two cards (Grizzly Bears and Forest) and choose Grizzly
    // Bears (the first one exiled, P0's default choice): P0 may cast it until the end of
    // their next turn, but only as a creature spell normally can be cast, and paying for
    // it.
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Forest"]);
    t.lands(P0, "Mountain", 1);
    let hangout = t.hand(P0, "Heroes' Hangout");
    t.cast(P0, hangout).modes(&[0]).go();
    t.resolve_all();
    let bears = t.g.current(cards[0]);
    assert_eq!(t.zone(bears), Zone::Exile);
    // No mana: it can't be cast.
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.lands(P0, "Forest", 2);
    assert_eq!(
        creature_timing(&mut t, P0, bears),
        [true, false, false, false]
    );
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);

    // Choosing the Forest: P0 may play it only as a land play during their main phase.
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    t.lands(P0, "Mountain", 1);
    let hangout = t.hand(P0, "Heroes' Hangout");
    t.cast(P0, hangout).modes(&[0]).go();
    t.resolve_all();
    let forest = t.g.current(cards[0]);
    assert_eq!(t.zone(forest), Zone::Exile);
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, forest));
    t.set_step(P0, Step::DeclareAttackers);
    assert!(!can_play_land(&mut t, P0, forest));
    t.g.combat = None;
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, forest));
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(forest));
}

#[test]
fn act_on_impulse_the_exiled_cards_are_played_following_the_normal_rules() {
    cr!("601.3", "307.1", "601.2f", "601.2h");
    ruling!(
        "Act on Impulse",
        "Playing a card this way follows the normal rules for playing the card. You must pay its costs, and you must follow all applicable timing rules. For example, if one of the cards is a creature card, you can cast that card only during your main phase while the stack is empty."
    );
    supported("Act on Impulse");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Island"]);
    t.lands(P0, "Mountain", 3);
    let impulse = t.hand(P0, "Act on Impulse");
    t.cast(P0, impulse).go();
    t.resolve_all();
    let bears = t.g.current(cards[0]);
    assert_eq!(t.zone(bears), Zone::Exile);
    // Its costs must be paid: no mana left.
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.lands(P0, "Forest", 2);
    // Only in the main phase with an empty stack (this turn: "until end of turn").
    let timing = creature_timing(&mut t, P0, bears);
    assert_eq!(&timing[..3], &[true, false, false]);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Act on Impulse, Lightning Bolt, and Grizzly Bears.
    assert_eq!(tapped_lands(&t, P0), 3 + 1 + 2);
}

#[test]
fn hakoda_ally_spells_from_the_top_follow_the_timing_rules_and_are_paid_for() {
    cr!("601.3", "307.1", "601.2f");
    ruling!(
        "Hakoda, Selfless Commander",
        "You must pay all costs and follow all timing rules for spells cast from the top of your library this way."
    );
    supported("Hakoda, Selfless Commander");
    supported("Kor Bladewhirl");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hakoda, Selfless Commander");
    let ally = t.library_top(P0, "Kor Bladewhirl");
    // No mana: it can't be cast.
    assert!(!can_cast(&mut t, P0, ally, CastMethod::Normal));
    t.lands(P0, "Plains", 2);
    assert_eq!(
        creature_timing(&mut t, P0, ally),
        [true, false, false, false]
    );
    // Not a creature spell of another creature type.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hakoda, Selfless Commander");
    t.lands(P0, "Forest", 2);
    let bears = t.library_top(P0, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
}

/// P0 casts Tormenting Voice ("As an additional cost to cast this spell, discard a card.
/// Draw two cards.") from where `place` put it, with or without a card in hand (a
/// Forest); whether it could be cast, and whether the Forest was discarded.
fn tormenting_voice_from(place: fn(&mut TestGame) -> ObjectId, with_card: bool) -> (bool, bool) {
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let voice = place(&mut t);
    if with_card {
        t.hand(P0, "Forest");
    }
    assert_eq!(t.hand_size(P0), with_card as usize);
    let ok = t.cast(P0, voice).try_go().is_ok();
    t.resolve_all();
    (ok, t.in_graveyard(P0, "Forest"))
}

/// P0 casts Cyclonic Rift from where `place` put it, overloaded ({6}{U}: "Return each
/// nonland permanent you don't control to its owner's hand"). Returns the number of P1's
/// permanents returned.
fn overloaded_rift_from(place: fn(&mut TestGame) -> ObjectId) -> usize {
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 7);
    let rift = place(&mut t);
    assert!(cast_methods(&mut t, P0, rift).contains(&OVERLOAD));
    t.cast(P0, rift).method(OVERLOAD).go();
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 7);
    t.hand_size(P1)
}

fn top_voice(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Magus of the Future");
    t.library_top(P0, "Tormenting Voice")
}

fn top_rift(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Magus of the Future");
    t.library_top(P0, "Cyclonic Rift")
}

#[test]
fn magus_of_the_future_spells_from_the_library_pay_all_costs_and_may_use_alternative_costs() {
    cr!("601.3", "601.2b", "601.2f", "118.8a", "118.9");
    ruling!(
        "Magus of the Future",
        "You still pay all costs for a spell you cast from your library, including additional costs. You may also pay alternative costs."
    );
    supported("Magus of the Future");
    assert_eq!(tormenting_voice_from(top_voice, true), (true, true));
    assert_eq!(tormenting_voice_from(top_voice, false), (false, false));
    assert_eq!(overloaded_rift_from(top_rift), 2);
}

fn channeled(t: &mut TestGame, name: &str) -> ObjectId {
    let channeler = t.battlefield(P0, "Magmatic Channeler");
    // The card discarded to activate it.
    t.hand(P0, "Island");
    // The card is the first exiled: P0's default choice.
    let cards = stack_library(t, P0, &[name, "Hill Giant"]);
    t.activate(P0, channeler, 0, &[]).expect("activate Magmatic Channeler");
    t.resolve_all();
    t.clear_answers();
    let c = t.g.current(cards[0]);
    assert_eq!(t.zone(c), Zone::Exile);
    c
}

fn channeled_voice(t: &mut TestGame) -> ObjectId {
    channeled(t, "Tormenting Voice")
}

fn channeled_rift(t: &mut TestGame) -> ObjectId {
    channeled(t, "Cyclonic Rift")
}

#[test]
fn magmatic_channeler_the_exiled_card_is_paid_for_and_may_use_alternative_costs() {
    cr!("601.3", "601.2b", "601.2f", "118.8a", "118.9");
    ruling!(
        "Magmatic Channeler",
        "You’ll still pay all costs for a spell cast this way, including additional costs. You may also pay alternative costs if any are available."
    );
    supported("Magmatic Channeler");
    // "{T}, Discard a card: Exile the top two cards of your library, then choose one of
    // them. You may play that card this turn."
    assert_eq!(tormenting_voice_from(channeled_voice, true), (true, true));
    assert_eq!(tormenting_voice_from(channeled_voice, false), (false, false));
    assert_eq!(overloaded_rift_from(channeled_rift), 2);
}

/// The alternative-cost methods `p` could cast `card` with now.
fn alternative_methods(t: &mut TestGame, p: PlayerId, card: ObjectId) -> usize {
    cast_methods(t, p, card)
        .into_iter()
        .filter(|m| matches!(m, CastMethod::Alternative(_)))
        .count()
}

#[test]
fn allosaurus_rider_needs_two_other_green_cards_for_its_alternative_cost() {
    cr!("118.9", "601.2b", "601.2h");
    ruling!(
        "Allosaurus Rider",
        "If you don’t have two cards of the right color in your hand, you can’t choose to cast the spell using the alternative cost."
    );
    supported("Allosaurus Rider");
    // "You may exile two green cards from your hand rather than pay this spell's mana
    // cost." One other green card (and a red one): not available.
    let mut t = TestGame::new(2);
    let rider = t.hand(P0, "Allosaurus Rider");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Lightning Bolt");
    assert_eq!(alternative_methods(&mut t, P0, rider), 0);
    // The Rider itself doesn't count: it's on the stack as the cost is paid.
    // A second green card: available, and both are exiled.
    let elves = t.hand(P0, "Llanowar Elves");
    assert_eq!(alternative_methods(&mut t, P0, rider), 1);
    let m = cast_methods(&mut t, P0, rider)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .unwrap();
    t.cast(P0, rider).method(m).go();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.zone(elves), Zone::Exile);
    assert!(t.in_hand(P0, "Lightning Bolt"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Allosaurus Rider").len(), 1);
}

#[test]
fn allosaurus_rider_cast_for_its_alternative_cost_follows_creature_timing() {
    cr!("118.9", "307.1", "601.3");
    ruling!(
        "Allosaurus Rider",
        "Paying the alternative cost doesn’t change when you can cast the spell. A creature spell you cast this way, for example, can still only be cast during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    let rider = t.hand(P0, "Allosaurus Rider");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Llanowar Elves");
    assert_eq!(alternative_methods(&mut t, P0, rider), 1);
    // A spell on the stack.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert_eq!(alternative_methods(&mut t, P0, rider), 0);
    t.resolve_all();
    // During combat.
    t.set_step(P0, Step::DeclareAttackers);
    assert_eq!(alternative_methods(&mut t, P0, rider), 0);
    // During the opponent's turn.
    t.g.combat = None;
    t.set_step(P1, Step::PrecombatMain);
    assert_eq!(alternative_methods(&mut t, P0, rider), 0);
    t.set_step(P0, Step::PostcombatMain);
    assert_eq!(alternative_methods(&mut t, P0, rider), 1);
    let _ = CardType::Creature;
}
