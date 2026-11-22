//! Rulings batch P214 — explore (CR 701.44): reveal the top card of your library; a land
//! card goes to your hand, otherwise put a +1/+1 counter on the exploring permanent and
//! you may put the revealed card into your graveyard.

use crate::r_p214_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s05_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::kwa::explore::{EXPLORED, REVEALED_LAND, REVEALED_NONLAND};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The explore events so far: (permanent, what was revealed).
fn explored(t: &TestGame) -> Vec<(Option<ObjectId>, i32)> {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::Custom {
                name, obj, amount, ..
            } if name == EXPLORED => Some((*obj, *amount)),
            _ => None,
        })
        .collect()
}

/// Path of Discovery ("Whenever a creature you control enters, it explores."), the
/// watcher `watcher` and Hill Giant on top of P0's library; Grizzly Bears enter and are
/// destroyed before Path's trigger resolves. Returns (watcher, Bears).
fn bears_leave_before_exploring(t: &mut TestGame, watcher: &str) -> (ObjectId, ObjectId) {
    supported(watcher);
    t.battlefield(P0, "Path of Discovery");
    let w = t.battlefield(P0, watcher);
    t.library_top(P0, "Hill Giant");
    let bears = enter(t, P0, "Grizzly Bears");
    assert_eq!(t.stack_len(), 1);
    destroy(t, bears);
    assert!(!t.on_battlefield(bears));
    t.answer_yes(P0, false);
    t.resolve_all();
    // The Bears explored (revealing a nonland card) although they had left.
    assert_eq!(explored(t), vec![(Some(bears), REVEALED_NONLAND)]);
    (w, bears)
}

#[test]
fn a_creature_that_left_still_explores_merfolk_cave_diver() {
    cr!("701.44a", "701.44c", "603.2");
    ruling!(
        "Merfolk Cave-Diver",
        "If a creature leaves the battlefield before an effect instructs it to explore, it still explores. Abilities like Merfolk Cave-Diver's that trigger whenever a creature you control explores will still trigger."
    );
    // Merfolk Cave-Diver (2/4): "Whenever a creature you control explores, this creature
    // gets +1/+0 until end of turn and can't be blocked this turn."
    let mut t = TestGame::new(2);
    let (diver, _) = bears_leave_before_exploring(&mut t, "Merfolk Cave-Diver");
    assert_eq!(t.pt(diver), (3, 4));
}

#[test]
fn a_creature_that_left_still_explores_nicanzil() {
    cr!("701.44a", "701.44c", "603.2");
    ruling!(
        "Nicanzil, Current Conductor",
        "If a creature leaves the battlefield before an effect instructs it to explore, it still explores. Abilities like Nicanzil's that trigger whenever a creature you control explores will still trigger."
    );
    // Nicanzil: "Whenever a creature you control explores a nonland card, put a +1/+1
    // counter on Nicanzil."
    let mut t = TestGame::new(2);
    let (nic, _) = bears_leave_before_exploring(&mut t, "Nicanzil, Current Conductor");
    assert_eq!(t.counters(nic, counters::PLUS1), 1);
}

#[test]
fn a_creature_that_left_still_explores_wildgrowth_walker() {
    cr!("701.44a", "701.44c", "603.2");
    ruling!(
        "Wildgrowth Walker",
        "If a creature leaves the battlefield before an effect instructs it to explore, it still explores. Effects that trigger when a creature you control explores, such as that of Wildgrowth Walker, trigger if appropriate."
    );
    // Wildgrowth Walker: "Whenever a creature you control explores, put a +1/+1 counter on
    // this creature and you gain 3 life."
    let mut t = TestGame::new(2);
    let (walker, _) = bears_leave_before_exploring(&mut t, "Wildgrowth Walker");
    assert_eq!(t.counters(walker, counters::PLUS1), 1);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn a_creature_that_left_still_explores_shadowed_caravel() {
    cr!("701.44a", "701.44c", "603.2");
    ruling!(
        "Shadowed Caravel",
        "If a resolving spell or ability instructs a specific creature to explore but that creature has left the battlefield, the creature still explores. Effects that trigger when a creature you control explores, such as that of Shadowed Caravel, trigger if appropriate."
    );
    // Shadowed Caravel: "Whenever a creature you control explores, put a +1/+1 counter on
    // this Vehicle."
    let mut t = TestGame::new(2);
    let (caravel, _) = bears_leave_before_exploring(&mut t, "Shadowed Caravel");
    assert_eq!(t.counters(caravel, counters::PLUS1), 1);
}

/// Hakbal of the Surging Soul and Coral Merfolk on the battlefield; the game advances to
/// P0's beginning of combat (Hakbal's trigger on the stack). Returns (Hakbal, Coral).
fn hakbal_setup(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Hakbal of the Surging Soul");
    let hakbal = t.battlefield(P0, "Hakbal of the Surging Soul");
    let coral = t.battlefield(P0, "Coral Merfolk");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    (hakbal, coral)
}

#[test]
fn hakbals_merfolk_all_explore_as_its_ability_resolves() {
    cr!("701.44a", "608.2", "117.3");
    ruling!(
        "Hakbal of the Surging Soul",
        "Each of your Merfolk creatures will explore during the resolution of Hakbal of the Surging Soul's first ability. Players won't be able to respond between the explores."
    );
    let mut t = TestGame::new(2);
    let (hakbal, coral) = hakbal_setup(&mut t);
    stack_library(&mut t, P0, &["Hill Giant", "Forest"]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve();
    // Both explored during the one resolution, with no priority in between.
    assert_eq!(explored(&t).len(), 2);
    assert_eq!(count_asked(&t, from, is_priority), 0);
    let c = t.counters(hakbal, counters::PLUS1) + t.counters(coral, counters::PLUS1);
    assert_eq!(c, 1);
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn hakbals_controller_chooses_the_order_the_merfolk_explore() {
    cr!("701.44a", "701.44d");
    ruling!(
        "Hakbal of the Surging Soul",
        "If you control multiple Merfolk, you choose the order in which they explore. Make sure it's clear which Merfolk is exploring before you reveal any cards."
    );
    // Hill Giant is on top: the first Merfolk to explore gets the counter and P0 leaves it
    // there; the second reveals it again.
    for first_coral in [true, false] {
        let mut t = TestGame::new(2);
        let (hakbal, coral) = hakbal_setup(&mut t);
        t.library_top(P0, "Hill Giant");
        let from = t.asked().len();
        let first = if first_coral { coral } else { hakbal };
        t.answer_choose(P0, &[Entity::Object(first)]);
        t.answer_yes(P0, false);
        t.answer_yes(P0, false);
        t.resolve();
        let asks = asked_since(&t, from);
        assert!(asks.iter().any(|(p, d)| *p == P0
            && matches!(d, Decision::ChooseEntities { candidates, .. }
                if candidates.contains(&Entity::Object(coral))
                    && candidates.contains(&Entity::Object(hakbal)))));
        let ex = explored(&t);
        assert_eq!(ex.len(), 2);
        assert_eq!(ex[0].0, Some(first));
    }
}

#[test]
fn jadelight_spelunker_with_x_0_doesnt_explore() {
    cr!("701.44a", "107.3m", "603.2");
    ruling!(
        "Jadelight Spelunker",
        "If Jadelight Spelunker enters the battlefield without being cast, or if it was cast with X equal to 0, the enters-the-battlefield ability will still trigger, but Jadelight Spelunker won't explore at all."
    );
    supported("Jadelight Spelunker");
    // Jadelight Spelunker: "When this creature enters, it explores X times."
    // Put onto the battlefield without being cast.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Hill Giant");
    let js = enter(&mut t, P0, "Jadelight Spelunker");
    assert_eq!(triggers_on_stack(&t, "explores X times"), 1);
    t.resolve_all();
    assert!(explored(&t).is_empty());
    assert_eq!(t.counters(js, counters::PLUS1), 0);
    // Cast with X = 0.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Forest", 1);
    let js = t.hand(P0, "Jadelight Spelunker");
    t.cast(P0, js).x(0).go();
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "explores X times"), 1);
    t.resolve_all();
    assert!(explored(&t).is_empty());
    // Cast with X = 2: it explores twice.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Forest", 3);
    let js = t.hand(P0, "Jadelight Spelunker");
    t.cast(P0, js).x(2).go();
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(explored(&t).len(), 2);
    assert_eq!(t.counters(js, counters::PLUS1), 2);
}

/// P0's Deadeye Tracker ability targets two cards in P1's graveyard; `remove` of them
/// leave the graveyard before it resolves.
fn deadeye(remove: usize) -> (TestGame, ObjectId, Vec<ObjectId>) {
    supported("Deadeye Tracker");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tracker = t.battlefield(P0, "Deadeye Tracker");
    let cards = vec![
        t.graveyard(P1, "Grizzly Bears"),
        t.graveyard(P1, "Hill Giant"),
    ];
    t.library_top(P0, "Craw Wurm");
    t.lands(P0, "Swamp", 2);
    t.answer_targets(
        P0,
        &[Entity::Object(cards[0]), Entity::Object(cards[1])],
    );
    activate_containing(&mut t, P0, tracker, "explores").expect("activate");
    for c in &cards[..remove] {
        t.g.move_object(*c, Zone::Hand(P1), MoveCause::Effect, None);
    }
    t.answer_yes(P0, false);
    t.resolve_all();
    (t, tracker, cards)
}

#[test]
fn deadeye_tracker_doesnt_explore_if_both_targets_are_illegal() {
    cr!("608.2b", "701.44a");
    ruling!(
        "Deadeye Tracker",
        "If each target card is an illegal target by the time Deadeye Tracker's ability resolves, the entire ability doesn't resolve. Deadeye Tracker won't explore."
    );
    let (t, tracker, _) = deadeye(2);
    assert!(explored(&t).is_empty());
    assert_eq!(t.counters(tracker, counters::PLUS1), 0);
}

#[test]
fn deadeye_tracker_explores_if_one_target_is_legal() {
    cr!("608.2b", "701.44a");
    ruling!(
        "Deadeye Tracker",
        "If one target card is an illegal target by the time Deadeye Tracker's ability resolves, the remaining legal target is exiled and Deadeye Tracker explores."
    );
    let (t, tracker, cards) = deadeye(1);
    assert_eq!(t.zone(cards[0]), Zone::Hand(P1));
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(explored(&t), vec![(Some(tracker), REVEALED_NONLAND)]);
    assert_eq!(t.counters(tracker, counters::PLUS1), 1);
}

#[test]
fn enter_the_unknown_with_an_illegal_target_does_nothing() {
    cr!("608.2b", "701.44a", "305.2");
    ruling!(
        "Enter the Unknown",
        "If the target creature is an illegal target by the time Enter the Unknown tries to resolve, the spell doesn't resolve. It won't explore, and you won't be able to play an additional land."
    );
    supported("Enter the Unknown");
    for respond in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.library_top(P0, "Hill Giant");
        let forest = t.hand(P0, "Forest");
        t.play_land(P0, forest).unwrap();
        let second = t.hand(P0, "Forest");
        let etu = t.hand(P0, "Enter the Unknown");
        t.cast(P0, etu).target(bears).go();
        if respond {
            destroy(&mut t, bears);
        }
        t.answer_yes(P0, false);
        t.resolve_all();
        assert_eq!(explored(&t).len(), usize::from(!respond));
        assert_eq!(can_play_land(&mut t, P0, second), !respond);
    }
}

#[test]
fn enter_the_unknown_in_an_opponents_turn() {
    cr!("701.44a", "305.2", "305.3");
    ruling!(
        "Enter the Unknown",
        "If you somehow manage to cast Enter the Unknown when it's not your turn, the target creature explores when it resolves, but you won't be able to play a land that turn."
    );
    supported("Leyline of Anticipation");
    // Leyline of Anticipation: "You may cast spells as though they had flash."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Anticipation");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.library_top(P0, "Hill Giant");
    let forest = t.hand(P0, "Forest");
    t.lands(P0, "Forest", 1);
    let etu = t.hand(P0, "Enter the Unknown");
    assert!(can_cast(&mut t, P0, etu, CastMethod::Normal));
    t.cast(P0, etu).target(bears).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(explored(&t), vec![(Some(bears), REVEALED_NONLAND)]);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // No land can be played in P1's turn.
    t.g.stack.clear();
    assert!(!can_play_land(&mut t, P0, forest));
    // The additional land drop lasted only that turn: in P0's turn, one land.
    t.advance_to(P0, Step::PrecombatMain);
    t.play_land(P0, forest).unwrap();
    let second = t.hand(P0, "Forest");
    assert!(!can_play_land(&mut t, P0, second));
}

/// Nicanzil and Path of Discovery on the battlefield, `top` on top of P0's library;
/// Grizzly Bears enter (Path's trigger on the stack).
fn nicanzil(t: &mut TestGame, top: &str) -> (ObjectId, ObjectId, ObjectId) {
    supported("Nicanzil, Current Conductor");
    let nic = t.battlefield(P0, "Nicanzil, Current Conductor");
    t.battlefield(P0, "Path of Discovery");
    let card = t.library_top(P0, top);
    let bears = enter(t, P0, "Grizzly Bears");
    (nic, bears, card)
}

#[test]
fn nicanzils_land_ability_waits_until_the_explore_is_done() {
    cr!("701.44a", "603.3", "305.4");
    ruling!(
        "Nicanzil, Current Conductor",
        "Nicanzil's first ability won't resolve until after you're done exploring. You may put any land card from your hand onto the battlefield, including the land card you explored (if it's still in your hand) or one that was already in your hand."
    );
    // Nicanzil: "Whenever a creature you control explores a land card, you may put a land
    // card from your hand onto the battlefield tapped."
    for explored_land in [true, false] {
        let mut t = TestGame::new(2);
        let other = t.hand(P0, "Island");
        let (_, _, forest) = nicanzil(&mut t, "Forest");
        t.resolve();
        // The explore is done (the Forest is in hand); Nicanzil's trigger waits.
        let forest = t.g.current(forest);
        assert_eq!(t.zone(forest), Zone::Hand(P0));
        assert_eq!(t.stack_len(), 1);
        let pick = if explored_land { forest } else { other };
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(pick)]);
        t.resolve_all();
        let pick = t.g.current(pick);
        assert!(t.on_battlefield(pick));
        assert!(t.obj_now(pick).tapped);
    }
}

#[test]
fn nicanzil_cares_about_the_card_as_it_was_revealed() {
    cr!("701.44a", "603.2", "603.10");
    ruling!(
        "Nicanzil, Current Conductor",
        "Nicanzil, Current Conductor's triggered abilities care about the characteristics of the card as it was revealed while a creature explored. It doesn't matter what happened to the card after that."
    );
    // A nonland card is revealed and put into the graveyard: Nicanzil still gets its
    // counter.
    let mut t = TestGame::new(2);
    let (nic, _, giant) = nicanzil(&mut t, "Hill Giant");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    assert_eq!(t.counters(nic, counters::PLUS1), 1);
    // A land card is revealed and put into the hand, then leaves the hand before the
    // trigger resolves: it still triggered (and P0 may put another land).
    let mut t = TestGame::new(2);
    let island = t.hand(P0, "Island");
    let (nic, _, forest) = nicanzil(&mut t, "Forest");
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    let forest = t.g.current(forest);
    move_to(&mut t, forest, Zone::Graveyard(P0));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(island)));
    assert_eq!(t.counters(nic, counters::PLUS1), 0);
    assert_eq!(
        explored(&t).iter().map(|e| e.1).collect::<Vec<_>>(),
        vec![REVEALED_LAND]
    );
}

#[test]
fn path_of_discovery_triggers_along_with_other_explore_abilities() {
    cr!("701.44a", "603.3b", "117.3b");
    ruling!(
        "Path of Discovery",
        "Path of Discovery's triggered ability triggers along with any other abilities that say that the creature explores when it enters the battlefield, including abilities that come from the creature itself or from multiples of Path of Discovery. You may take actions between each resolving ability's exploration."
    );
    supported("Merfolk Branchwalker");
    // Two Path of Discovery and Merfolk Branchwalker ("When this creature enters, it
    // explores."): three triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Path of Discovery");
    t.battlefield(P0, "Path of Discovery");
    stack_library(&mut t, P0, &["Hill Giant", "Craw Wurm", "Grizzly Bears"]);
    let bw = enter(&mut t, P0, "Merfolk Branchwalker");
    assert_eq!(t.stack_len(), 3);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.counters(bw, counters::PLUS1), 1);
    assert_eq!(t.stack_len(), 2);
    // Players may act between them: here the Branchwalker is returned to its owner's
    // hand; the remaining explores still happen.
    move_to(&mut t, bw, Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(explored(&t).len(), 3);
}

#[test]
fn amalias_explore_and_power_check_are_one_ability() {
    cr!("701.44a", "608.2", "117.3");
    ruling!(
        "Amalia Benavides Aguirre",
        "The instruction for Amalia Benavides Aguirre to explore and the effect that occurs if Amalia Benavides Aguirre's power is exactly 20 are all part of the same ability. Players do not get to chance to respond to the ability after knowing the result of the explore."
    );
    supported("Amalia Benavides Aguirre");
    // Amalia (2/2) with 17 +1/+1 counters: 19/19. P0 gains life; Amalia explores a nonland
    // card and becomes 20/20; all other creatures are destroyed in the same resolution.
    let mut t = TestGame::new(2);
    let amalia = t.battlefield(P0, "Amalia Benavides Aguirre");
    t.g.add_counters(Entity::Object(amalia), counters::PLUS1, 17, None);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.library_top(P0, "Craw Wurm");
    t.g.gain_life(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(count_asked(&t, from, is_priority), 0);
    assert_eq!(t.pt(amalia), (20, 20));
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(amalia));
}
