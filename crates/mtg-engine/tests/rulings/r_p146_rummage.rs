//! Rulings batch P146 — rummaging and repeatable card draw: discarding as a cost
//! (CR 118.3, 602.2b), "you may discard ... If you do" while resolving (CR 118.12),
//! random discards (CR 701.9b), "for as long as you control" durations (CR 611.2b),
//! last known information (CR 113.7a, 608.2h), and triggers that count events, not
//! amounts (CR 603.2c).

use crate::r_p146_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The activated abilities of the object now.
fn activated(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
}

/// `who` gains control of `id` indefinitely (as a resolving effect would).
fn gain_control(t: &mut TestGame, id: ObjectId, who: PlayerId) {
    let id = t.g.current(id);
    run_from(
        t,
        who,
        None,
        Effect::GainControl {
            what: Sel::All(Filter::Objects(vec![id])),
            who: PlayerRef::Player(who),
            duration: Duration::Permanent,
        },
        &[],
    );
}

/// Quicksmith Spy enters under P0's control targeting a Darksteel Relic P0 controls; the
/// trigger is on the stack.
fn spy_enters() -> (TestGame, ObjectId, ObjectId) {
    supported("Quicksmith Spy");
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Darksteel Relic");
    t.answer_targets(P0, &[obj(relic)]);
    let spy = t.enter(P0, "Quicksmith Spy");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (t, spy, relic)
}

#[test]
fn quicksmith_spy_leaving_before_its_trigger_resolves() {
    cr!("611.2b");
    ruling!(
        "Quicksmith Spy",
        "If Quicksmith Spy leaves the battlefield while its ability is on the stack, the artifact never gains the card-drawing ability."
    );
    // "When this creature enters, target artifact you control gains "{T}: Draw a card"
    // for as long as you control this creature."
    let (mut t, spy, relic) = spy_enters();
    move_to(&mut t, spy, Zone::Exile);
    t.resolve_all();
    assert_eq!(activated(&mut t, relic), 0);
    // It stays that way even if the duration "begins again" later.
    t.answer_targets(P0, &[]);
    t.battlefield(P0, "Quicksmith Spy");
    assert_eq!(activated(&mut t, relic), 0);
}

#[test]
fn quicksmith_spy_leaving_ends_the_effect() {
    cr!("611.2b", "108.4");
    ruling!(
        "Quicksmith Spy",
        "If Quicksmith Spy leaves the battlefield, you no longer control it."
    );
    let (mut t, spy, relic) = spy_enters();
    t.resolve_all();
    assert_eq!(activated(&mut t, relic), 1);
    let hand = t.hand_size(P0);
    activate_resolve(&mut t, P0, relic, 0, &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    destroy(&mut t, spy);
    assert_eq!(activated(&mut t, relic), 0);
}

#[test]
fn quicksmith_spy_draw_ability_on_the_stack_resolves_anyway() {
    cr!("113.7a");
    ruling!(
        "Quicksmith Spy",
        "If Quicksmith Spy or the artifact leaves the battlefield while the card-drawing ability is on the stack, that ability resolves as normal."
    );
    for remove_relic in [false, true] {
        let (mut t, spy, relic) = spy_enters();
        t.resolve_all();
        t.activate(P0, relic, 0, &[]).unwrap();
        move_to(&mut t, spy, Zone::Exile);
        if remove_relic {
            move_to(&mut t, relic, Zone::Exile);
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1);
    }
}

#[test]
fn quicksmith_spy_losing_control_ends_the_effect_for_good() {
    cr!("611.2b");
    ruling!(
        "Quicksmith Spy",
        "If another player gains control of Quicksmith Spy, the artifact will no longer have the card-drawing ability, even if you regain control of Quicksmith Spy."
    );
    let (mut t, spy, relic) = spy_enters();
    t.resolve_all();
    assert_eq!(activated(&mut t, relic), 1);
    gain_control(&mut t, spy, P1);
    assert_eq!(activated(&mut t, relic), 0);
    gain_control(&mut t, spy, P0);
    assert_eq!(t.obj_now(spy).controller, P0);
    assert_eq!(activated(&mut t, relic), 0);
}

#[test]
fn library_larcenist_cast_after_drawing_before_blockers() {
    cr!("508.2", "117.3a", "603.3b");
    ruling!(
        "Library Larcenist",
        "You may cast spells and activate abilities after the card has been drawn but before blockers are declared."
    );
    supported("Library Larcenist");
    // "Whenever this creature attacks, draw a card."
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Library Larcenist");
    t.lands(P0, "Forest", 1);
    let growth = t.library_top(P0, "Giant Growth");
    attack_with(&mut t, &[(l, Entity::Player(P1))]);
    t.resolve_all();
    let growth = t.g.current(growth);
    assert_eq!(t.zone(growth), Zone::Hand(P0));
    // Still the declare attackers step: P0 casts the drawn card.
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    t.cast(P0, growth).target(l).go();
    t.resolve_all();
    assert_eq!(t.pt(l), (4, 5));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn rummagers_cant_be_activated_without_cards_to_discard() {
    cr!("118.3", "602.2b", "701.9a");
    ruling!(
        "Mad Prophet",
        "Discarding a card is part of the ability’s activation cost. You can’t activate the ability if you have no cards in hand."
    );
    ruling!(
        "Rummaging Goblin",
        "Discarding a card is part of the cost to activate Rummaging Goblin's ability. If you don't have a card in your hand, you can't pay this part of the cost and you can't activate the ability."
    );
    ruling!(
        "Pumpkin Bombs",
        "Discarding two cards is part of the cost of the ability, so you can't activate the ability if you don't have at least two cards in hand."
    );
    // Mad Prophet and Rummaging Goblin: "{T}, Discard a card: Draw a card."
    for name in ["Mad Prophet", "Rummaging Goblin"] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        assert!(!can_activate(&mut t, P0, c), "{name}");
        let card = t.hand(P0, "Grizzly Bears");
        assert!(can_activate(&mut t, P0, c), "{name}");
        activate_resolve(&mut t, P0, c, 0, &[]);
        assert_eq!(t.zone(card), Zone::Graveyard(P0));
        assert_eq!(t.hand_size(P0), 1);
    }
    // Pumpkin Bombs: "{T}, Discard two cards: Draw three cards, ..."
    supported("Pumpkin Bombs");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Pumpkin Bombs");
    t.hand(P0, "Grizzly Bears");
    assert!(!can_activate(&mut t, P0, b));
    t.hand(P0, "Grizzly Bears");
    assert!(can_activate(&mut t, P0, b));
    t.activate(P0, b, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn pumpkin_bombs_uses_its_last_known_fuse_counters() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Pumpkin Bombs",
        "If Pumpkin Bombs is no longer on the battlefield as its ability resolves, use the number of fuse counters on it as it last existed on the battlefield to determine how much damage is dealt."
    );
    // "... Draw three cards, then put a fuse counter on this artifact. It deals damage
    // equal to the number of fuse counters on it to target opponent. They gain control of
    // this artifact."
    let setup = || {
        let mut t = TestGame::new(2);
        let b = t.battlefield(P0, "Pumpkin Bombs");
        put_counters(&mut t, b, "fuse", 2);
        fill_hand(&mut t, P0, 2);
        t.activate(P0, b, 0, &[Entity::Player(P1)]).unwrap();
        (t, b)
    };
    let (mut t, b) = setup();
    move_to(&mut t, b, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), 3);
    // Still there: three counters, 3 damage, and P1 gets it.
    let (mut t, b) = setup();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.obj_now(b).controller, P1);
}

#[test]
fn irreverent_gremlin_checks_power_only_as_the_creature_enters() {
    cr!("603.2", "603.6a", "614.1c");
    ruling!(
        "Irreverent Gremlin",
        "Irreverent Gremlin's second ability checks the power of the other creature only at the moment that creature enters."
    );
    supported("Irreverent Gremlin");
    supported("Heirloom Auntie");
    // "Whenever another creature you control with power 2 or less enters, you may discard
    // a card. If you do, draw a card. Do this only once each turn."
    // A 2/2 that becomes 4/4 while the trigger is on the stack: P0 may still rummage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Irreverent Gremlin");
    let card = t.hand(P0, "Hill Giant");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    pump(&mut t, bears, 2, 2);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(card)]);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 1);
    // Counters it enters with count: Heirloom Auntie (4/4, "enters with two -1/-1
    // counters on it") enters as a 2/2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Irreverent Gremlin");
    t.enter(P0, "Heirloom Auntie");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "discard"), 1);
    // A 3/3 doesn't trigger it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Irreverent Gremlin");
    t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

/// Gallia of the Endless Dance attacks with two Grizzly Bears; the trigger is on the
/// stack. Returns the game and the attackers.
fn gallia_attack(hand: usize) -> (TestGame, Vec<ObjectId>) {
    supported("Gallia of the Endless Dance");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gallia of the Endless Dance");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    fill_hand(&mut t, P0, hand);
    attack_with(
        &mut t,
        &[
            (g, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
    );
    assert_eq!(triggers_on_stack(&t, "discard"), 1);
    (t, vec![g, a, b])
}

#[test]
fn gallia_resolves_even_if_the_attackers_left() {
    cr!("603.2", "608.2c");
    ruling!(
        "Gallia of the Endless Dance",
        "Once three or more creatures you control have attacked, you’ll be given the option to discard a card at random, even if some or all of those attackers leave the battlefield before Gallia’s triggered ability resolves."
    );
    // "Whenever you attack with three or more creatures, you may discard a card at
    // random. If you do, draw two cards."
    let (mut t, attackers) = gallia_attack(2);
    for a in attackers {
        destroy(&mut t, a);
    }
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.graveyard_size(P0), 4);
}

#[test]
fn gallia_discards_only_one_card() {
    cr!("608.2c", "701.9b");
    ruling!(
        "Gallia of the Endless Dance",
        "While resolving Gallia’s last ability, you can’t discard multiple cards to draw more than two cards."
    );
    let (mut t, _) = gallia_attack(3);
    yes(&mut t, P0);
    yes(&mut t, P0);
    t.resolve_all();
    // One card discarded at random, two drawn.
    assert_eq!(t.hand_size(P0), 4);
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn gallia_random_discard_needs_a_card_in_hand() {
    cr!("118.12", "701.9b");
    ruling!(
        "Gallia of the Endless Dance",
        "You can’t choose to discard a card at random if you have no cards in hand. If you have one card in hand, you may discard it at random, even though that’s not very random."
    );
    // No cards: nothing is discarded, so nothing is drawn.
    let (mut t, _) = gallia_attack(0);
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.graveyard_size(P0)), (0, 0));
    // One card: that card.
    let (mut t, _) = gallia_attack(1);
    let card = t.g.player(P0).hand[0];
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 2);
}

/// Daretti, Scrap Savant on P0's battlefield with `extra` more loyalty counters.
fn daretti(t: &mut TestGame, extra: u32) -> ObjectId {
    supported("Daretti, Scrap Savant");
    let d = t.battlefield(P0, "Daretti, Scrap Savant");
    if extra > 0 {
        put_counters(t, d, "loyalty", extra);
    }
    d
}

#[test]
fn daretti_emblem_returns_only_the_card_still_in_the_graveyard() {
    cr!("603.7", "400.7", "114.4");
    ruling!(
        "Daretti, Scrap Savant",
        "The ability of Daretti’s emblem will return the artifact card only if it’s still in your graveyard when the delayed triggered ability resolves at the beginning of the next end step."
    );
    // "−10: You get an emblem with "Whenever an artifact is put into your graveyard from
    // the battlefield, return that card to the battlefield at the beginning of the next
    // end step.""
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        let d = daretti(&mut t, 7);
        activate_resolve(&mut t, P0, d, 2, &[]);
        let o = t.battlefield(P0, "Ornithopter");
        destroy(&mut t, o);
        t.resolve_all();
        let card = t.g.current(o);
        assert_eq!(t.zone(card), Zone::Graveyard(P0));
        if leave {
            // It leaves and comes back: a new object.
            let back = move_to(&mut t, card, Zone::Exile).unwrap();
            move_to(&mut t, back, Zone::Graveyard(P0));
        }
        end_step(&mut t, P0);
        assert_eq!(
            t.named_on_battlefield("Ornithopter").len(),
            if leave { 0 } else { 1 }
        );
    }
}

#[test]
fn daretti_minus_two_sacrifices_whatever_artifact_is_left() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Daretti, Scrap Savant",
        "You choose which artifact to sacrifice as the second ability resolves. You must sacrifice an artifact if you control at least one at that time."
    );
    // "−2: Sacrifice an artifact. If you do, return target artifact card from your
    // graveyard to the battlefield."
    let mut t = TestGame::new(2);
    let d = daretti(&mut t, 0);
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Darksteel Relic");
    let card = t.graveyard(P0, "Millstone");
    let from = n_asked(&t);
    t.activate(P0, d, 1, &[obj(card)]).unwrap();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    // The intended Ornithopter is destroyed in response; the Relic must be sacrificed.
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.zone(b), Zone::Graveyard(P0));
    assert_eq!(t.named_on_battlefield("Millstone").len(), 1);
}

#[test]
fn daretti_plus_two_may_discard_zero() {
    cr!("107.1c", "608.2c");
    ruling!(
        "Daretti, Scrap Savant",
        "You may choose to discard zero cards as the first ability resolves. In that case, you won’t draw any cards."
    );
    // "+2: Discard up to two cards, then draw that many cards."
    let mut t = TestGame::new(2);
    let d = daretti(&mut t, 0);
    fill_hand(&mut t, P0, 2);
    t.activate(P0, d, 0, &[]).unwrap();
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.graveyard_size(P0)), (2, 0));
    assert_eq!(t.counters(d, "loyalty"), 5);
    // Discarding two draws two.
    let mut t = TestGame::new(2);
    let d = daretti(&mut t, 0);
    let cards = fill_hand(&mut t, P0, 2);
    t.activate(P0, d, 0, &[]).unwrap();
    t.answer_choose(P0, &[obj(cards[0]), obj(cards[1])]);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.graveyard_size(P0)), (2, 2));
}

#[test]
fn academy_raider_triggers_once_per_damage_event() {
    cr!("603.2c", "510.2");
    ruling!(
        "Academy Raider",
        "The triggered ability triggers just once each time Academy Raider deals combat damage to a player, regardless of how much damage it deals."
    );
    supported("Academy Raider");
    // "Whenever this creature deals combat damage to a player, you may discard a card. If
    // you do, draw a card."
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Academy Raider");
    pump(&mut t, r, 3, 0);
    attack_with(&mut t, &[(r, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 16);
    assert_eq!(triggers_on_stack(&t, "discard"), 1);
}

#[test]
fn teferis_imp_with_no_cards_to_discard() {
    cr!("702.26a", "609.3");
    ruling!(
        "Teferi's Imp",
        "There is no negative effect if you can’t discard when it phases out. You still get to draw a card when it phases in."
    );
    supported("Teferi's Imp");
    // "Whenever this creature phases out, discard a card. Whenever this creature phases
    // in, draw a card."
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Teferi's Imp");
    // P0's next untap step: it phases out; the discard trigger finds an empty hand.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj(imp).phased_out);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert!(!t.has_lost(P0));
    // The one after: it phases in and P0 draws (before the draw step).
    t.advance_to(P1, Step::Upkeep);
    // Empty P0's hand (the card drawn in the draw step).
    let hand = t.g.player(P0).hand.clone();
    for c in hand {
        move_to(&mut t, c, Zone::Exile);
    }
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj(imp).phased_out);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn drowned_rusalka_discards_before_drawing() {
    cr!("608.2c");
    ruling!(
        "Drowned Rusalka",
        "Unlike many similar cards, Drowned Rusalka's ability causes you to discard a card first and then draw a card. If your hand is empty, you will just draw a card."
    );
    supported("Drowned Rusalka");
    // "{U}, Sacrifice a creature: Discard a card, then draw a card."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let r = t.battlefield(P0, "Drowned Rusalka");
    let bolt = t.hand(P0, "Lightning Bolt");
    let top = t.library_top(P0, "Giant Growth");
    t.answer_choose(P0, &[obj(r)]);
    t.activate(P0, r, 0, &[]).unwrap();
    let from = n_asked(&t);
    t.resolve_all();
    // The drawn card wasn't available to discard.
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert_eq!(t.zone(t.g.current(top)), Zone::Hand(P0));
    for (_, d) in &t.asked()[from..] {
        if let Decision::ChooseEntities { candidates, .. } = d {
            assert_eq!(candidates.len(), 1);
        }
    }
    // Empty hand: just a draw.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let r = t.battlefield(P0, "Drowned Rusalka");
    t.answer_choose(P0, &[obj(r)]);
    t.activate(P0, r, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.graveyard_size(P0)), (1, 1));
}

#[test]
fn merchant_of_the_vale_cost_versus_haggle_choice() {
    cr!("118.3", "118.12", "608.2c", "715.3");
    ruling!(
        "Merchant of the Vale // Haggle",
        "You choose which card, if any, to discard while Haggle is resolving. In contrast, you must discard a card to activate the ability of Merchant of the Vale."
    );
    supported("Merchant of the Vale // Haggle");
    // Merchant: "{2}{R}, Discard a card: Draw a card."
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let m = t.battlefield(P0, "Merchant of the Vale // Haggle");
    assert!(!can_activate(&mut t, P0, m));
    let card = t.hand(P0, "Grizzly Bears");
    assert!(can_activate(&mut t, P0, m));
    t.activate(P0, m, 0, &[]).unwrap();
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    // Haggle: "You may discard a card. If you do, draw a card."
    for discard in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 1);
        let card = t.hand(P0, "Grizzly Bears");
        let h = t.hand(P0, "Merchant of the Vale // Haggle");
        t.cast(P0, h).method(CastMethod::Half(1)).go();
        // Nothing discarded while casting.
        assert_eq!(t.zone(card), Zone::Hand(P0));
        t.answer_yes(P0, discard);
        t.answer_choose(P0, &[obj(card)]);
        t.resolve_all();
        assert_eq!(
            t.zone(card),
            if discard {
                Zone::Graveyard(P0)
            } else {
                Zone::Hand(P0)
            }
        );
        assert_eq!(t.hand_size(P0), 1);
    }
}

#[test]
fn lorehold_the_card_is_drawn_whether_or_not_miracle_is_used() {
    cr!("702.94a", "121.1", "603.11");
    ruling!(
        "Lorehold, the Historian",
        "You still draw the card whether you use the miracle ability or not."
    );
    supported("Lorehold, the Historian");
    supported("Psychosis Crawler");
    // "Each instant and sorcery card in your hand has miracle {2}." Psychosis Crawler:
    // "Whenever you draw a card, each opponent loses 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lorehold, the Historian");
    // (Psychosis Crawler's toughness is the number of cards in P0's hand.)
    t.hand(P0, "Grizzly Bears");
    t.battlefield(P0, "Psychosis Crawler");
    t.lands(P0, "Mountain", 2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    // P1's upkeep: Lorehold's "you may discard a card. If you do, draw a card": no.
    t.answer_yes(P0, false);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    t.answer_yes(P0, true); // reveal it
    t.answer_yes(P0, false); // don't cast it
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    // The miracle trigger happened and wasn't used; the card stays in hand, and the
    // draw trigger happened too.
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
    assert_eq!(crate::r_s11_common::triggered_from(&t, bolt), 1);
    assert_eq!(t.life(P1), 19);
}
