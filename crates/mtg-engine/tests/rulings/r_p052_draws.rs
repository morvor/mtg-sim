//! Rulings batch P052 — "draw matters": cards put into a hand without the word "draw"
//! aren't drawn (CR 121.5), one trigger per card drawn (CR 121.2, 603.2c),
//! replacement effects that modify a draw instruction or the individual draws (CR 121.2a,
//! 616.1), and the triggers of a simultaneous draw (CR 603.3b).

use crate::r_p046_common::*;
use crate::r_p052_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// What a "draw matters" permanent shows: its power/toughness, its +1/+1 counters, and
/// whether it has vigilance.
fn shape(t: &TestGame, id: ObjectId) -> (i32, i32, u32, bool) {
    let (p, x) = t.pt(id);
    (
        p,
        x,
        t.counters(id, "+1/+1"),
        has_kw(t, id, KeywordKind::Vigilance),
    )
}

/// `name` on P0's battlefield; P0 casts Ancestral Memories (put two of the top seven
/// cards into your hand) and then draws two cards. Returns the shape before, after the
/// two cards were put into P0's hand, and after the two draws.
fn put_then_draw(name: &str) -> [(i32, i32, u32, bool); 3] {
    supported(name);
    supported("Ancestral Memories");
    let mut t = TestGame::new(2);
    let id = t.battlefield(P0, name);
    let before = shape(&t, id);
    let memories = t.hand(P0, "Ancestral Memories");
    fill_pool(&mut t, P0, 5);
    let hand = t.hand_size(P0);
    t.cast(P0, memories).go();
    t.resolve_all();
    // Ancestral Memories left the hand; two cards were put into it.
    assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    assert_eq!(t.g.history.cards_drawn.get(&P0).copied().unwrap_or(0), 0);
    let put = shape(&t, id);
    draw(&mut t, P0, 2);
    t.resolve_all();
    [before, put, shape(&t, id)]
}

#[test]
fn cards_put_into_a_hand_without_draw_arent_drawn() {
    cr!("121.5", "121.2", "603.2c");
    ruling!(
        "Gnarled Sage",
        "If a spell or ability causes you to put a card into your hand without specifically using the word “draw,” it's not a card drawn."
    );
    let [b, p, d] = put_then_draw("Gnarled Sage");
    assert_eq!(b, p);
    assert_eq!(d, (b.0, b.1 + 2, 0, true));
    ruling!(
        "Eyekite",
        "If a spell or ability causes you to put cards into your hand without specifically using the word \"draw,\" Eyekite's effect doesn't count them."
    );
    let [b, p, d] = put_then_draw("Eyekite");
    assert_eq!(b, p);
    assert_eq!(d, (b.0 + 2, b.1, 0, false));
    ruling!(
        "Oneirophage",
        "If a spell or ability causes you to put cards into your hand without specifically using the word \"draw,\" Oneirophage's ability won't trigger."
    );
    let [b, p, d] = put_then_draw("Oneirophage");
    assert_eq!(b, p);
    assert_eq!(d, (b.0 + 2, b.1 + 2, 2, false));
    ruling!(
        "Jace's Projection",
        "If a spell or ability causes you to put cards into your hand without specifically using the word “draw,” the triggered ability of Jace’s Projection won’t trigger."
    );
    ruling!(
        "Jace's Projection",
        "If an effect instructs you to draw multiple cards, the triggered ability of Jace’s Projection triggers that many times."
    );
    let [b, p, d] = put_then_draw("Jace's Projection");
    assert_eq!(b, p);
    assert_eq!(d, (b.0 + 2, b.1 + 2, 2, false));
}

#[test]
fn horizon_chimera_gains_life_once_per_card_in_separate_events() {
    cr!("121.2", "603.2c", "119.9");
    ruling!(
        "Horizon Chimera",
        "If you draw multiple cards, Horizon Chimera's ability will trigger that many times. Each of these abilities will cause a separate life-gaining event."
    );
    supported("Horizon Chimera");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Horizon Chimera");
    // Ajani's Pridemate: "Whenever you gain life, put a +1/+1 counter on this creature."
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    t.g.draw_cards(P0, 3);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "gain 1 life"), 3);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.counters(pridemate, "+1/+1"), 3);
}

#[test]
fn putting_a_loyalty_counter_on_jace_activates_nothing() {
    cr!("606.3");
    ruling!(
        "Jace's Projection",
        "Putting a loyalty counter on a Jace planeswalker doesn’t cause any of its activated abilities to become activated."
    );
    supported("Jace's Projection");
    let mut t = TestGame::new(2);
    let projection = t.battlefield(P0, "Jace's Projection");
    let jace = t.battlefield(P0, "Jace Beleren");
    let loyalty = t.counters(jace, "loyalty");
    let hands = (t.hand_size(P0), t.hand_size(P1));
    activate_resolve(&mut t, P0, projection, 0, &[obj(jace)]);
    assert_eq!(t.counters(jace, "loyalty"), loyalty + 1);
    // No loyalty ability happened (Jace Beleren's +2: each player draws a card)...
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), hands);
    // ...and Jace can still activate one this turn.
    t.activate(P0, jace, 0, &[]).expect("Jace's +2");
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (hands.0 + 1, hands.1 + 1));
}

#[test]
fn fractal_anomaly_counts_the_draws_once_as_it_resolves() {
    cr!("608.2h", "121.2");
    ruling!(
        "Fractal Anomaly",
        "The value of X is calculated only once, as Fractal Anomaly resolves."
    );
    supported("Fractal Anomaly");
    let mut t = TestGame::new(2);
    draw(&mut t, P0, 2);
    let anomaly = t.hand(P0, "Fractal Anomaly");
    fill_pool(&mut t, P0, 1);
    t.cast(P0, anomaly).go();
    // A draw in response counts (X is determined on resolution)...
    draw(&mut t, P0, 1);
    t.resolve_all();
    let fractal = tokens(&t, P0)[0];
    assert_eq!(t.counters(fractal, "+1/+1"), 3);
    // ...later draws don't change it.
    draw(&mut t, P0, 2);
    assert_eq!(t.counters(fractal, "+1/+1"), 3);
    assert_eq!(t.pt(fractal), (3, 3));
}

#[test]
fn nadir_kraken_pays_once_per_trigger_after_seeing_every_drawn_card() {
    cr!("603.2c", "603.5", "121.2");
    ruling!(
        "Nadir Kraken",
        "You choose whether to pay {1} after drawing the card and seeing what it is. If you draw multiple cards, you see them all before deciding how many of the resulting triggers you'll pay {1} for."
    );
    ruling!(
        "Nadir Kraken",
        "While resolving Nadir Kraken's ability, you can't pay {1} multiple times to create multiple Tentacles or to give it multiple +1/+1 counters."
    );
    supported("Nadir Kraken");
    let mut t = TestGame::new(2);
    let kraken = t.battlefield(P0, "Nadir Kraken");
    fill_pool(&mut t, P0, 3);
    let hand = t.hand_size(P0);
    // P0's hand size whenever P0 is asked whether to pay.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.player(P0).hand.len(),
    );
    t.answer_yes(P0, true).answer_yes(P0, false).answer_yes(P0, true);
    t.g.draw_cards(P0, 3);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "pay {1}"), 3);
    let pool_before = pool(&t, P0);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![hand + 3; 3]);
    // Two of the three paid: {2} spent, two counters, two Tentacles.
    assert_eq!(pool_before - pool(&t, P0), 2);
    assert_eq!(t.counters(kraken, "+1/+1"), 2);
    assert_eq!(with_subtype(&t, P0, "Tentacle").len(), 2);
}

#[test]
fn dream_trawler_gains_hexproof_while_already_tapped() {
    cr!("602.2", "118.3", "701.26a");
    ruling!(
        "Dream Trawler",
        "You can activate Dream Trawler's last ability even if it's already tapped. It still gains hexproof until end of turn."
    );
    supported("Dream Trawler");
    let mut t = TestGame::new(2);
    let trawler = t.battlefield(P0, "Dream Trawler");
    t.g.objects[trawler.0 as usize].tapped = true;
    t.hand(P0, "Island");
    t.activate(P0, trawler, 0, &[]).expect("discard ability");
    t.resolve_all();
    assert!(has_kw(&t, trawler, KeywordKind::Hexproof));
    assert!(t.obj_now(trawler).tapped);
    assert!(t.in_graveyard(P0, "Island"));
}

#[test]
fn quantum_riddler_modifies_the_whole_draw_instruction_once() {
    cr!("121.2a", "616.1g", "616.1f", "614.5");
    ruling!(
        "Quantum Riddler",
        "If an effect instructs you to draw more than one card, Quantum Riddler’s replacement effect applies to that entire instruction, not the individual card draws represented by the instruction. For example, if you control Quantum Riddler and you cast Decode Transmissions while you have no other cards in your hand, Quantum Riddler’s replacement effect will apply, and you’ll draw three cards instead of two."
    );
    supported("Quantum Riddler");
    supported("Decode Transmissions");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Quantum Riddler");
    let decode = t.hand(P0, "Decode Transmissions");
    fill_pool(&mut t, P0, 3);
    t.cast(P0, decode).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    ruling!(
        "Quantum Riddler",
        "Quantum Riddler’s replacement effect applies to an instruction to draw more than one card before any replacement effects apply to individual cards drawn."
    );
    ruling!(
        "Quantum Riddler",
        "Once a replacement effect has been applied to an event, it can’t be applied again to the newly modified event. For example, once Quantum Riddler’s replacement effect has modified the “draw two cards” instruction of Decode Transmissions, Thought Reflection’s replacement effect can double the resulting card draws without Quantum Riddler’s replacement effect applying again."
    );
    // With one other card in hand and Thought Reflection: three draws, each doubled.
    for other in [0, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Quantum Riddler");
        t.battlefield(P0, "Thought Reflection");
        for _ in 0..other {
            t.hand(P0, "Island");
        }
        let decode = t.hand(P0, "Decode Transmissions");
        fill_pool(&mut t, P0, 3);
        t.cast(P0, decode).go();
        let m = mark(&t);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), other + 6, "{}", t.dump_log());
        assert_eq!(t.g.history.cards_drawn.get(&P0).copied(), Some(6));
        // Nobody was asked to order the two replacement effects.
        assert!(
            who_was_asked(&t, m, |d| matches!(d, Decision::ChooseReplacement { .. }))
                .is_empty()
        );
    }
}

#[test]
fn multiple_thought_reflections_are_cumulative() {
    cr!("616.1f", "121.2");
    ruling!(
        "Thought Reflection",
        "The effects of multiple Thought Reflections are cumulative. For example, if you have two Thought Reflections on the battlefield, you'll draw four times the original number of cards. If you have three, you'll draw eight times the number of cards, and so on."
    );
    supported("Thought Reflection");
    for (n, drawn) in [(1, 4), (2, 8), (3, 16)] {
        let mut t = TestGame::new(2);
        for _ in 0..n {
            t.battlefield(P0, "Thought Reflection");
        }
        draw(&mut t, P0, 2);
        assert_eq!(t.hand_size(P0), drawn, "{n} Thought Reflections");
    }
}

#[test]
fn the_drawing_player_orders_replacement_effects_on_a_draw() {
    cr!("616.1", "616.1e", "121.6");
    ruling!(
        "Thought Reflection",
        "If two or more replacement effects would apply to a card-drawing event, the player who's drawing the card chooses what order to apply them."
    );
    // P1's Thought Reflection doesn't apply to P0's draws; P0's own does. P0 draws with
    // P0's Thought Reflection and P1's Notion Thief ("If an opponent would draw a card
    // except the first one they draw in each of their draw steps, instead you skip that
    // draw and you draw a card."): P0 (the drawing player) chooses the order.
    supported("Notion Thief");
    let mut results = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Thought Reflection");
        t.battlefield(P1, "Notion Thief");
        let m = mark(&t);
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        draw(&mut t, P0, 1);
        assert_eq!(
            who_was_asked(&t, m, |d| matches!(d, Decision::ChooseReplacement { .. })),
            vec![P0]
        );
        results.push((t.hand_size(P0), t.hand_size(P1)));
    }
    results.sort();
    // Notion Thief first: P1 draws one. Thought Reflection first: two draws, each
    // replaced by Notion Thief: P1 draws two.
    assert_eq!(results, vec![(0, 1), (0, 2)]);
}

#[test]
fn sheoldreds_controller_orders_the_triggers_of_a_simultaneous_draw() {
    cr!("603.3b", "121.2");
    ruling!(
        "Sheoldred, the Apocalypse",
        "If you and an opponent draw a card at the same time, you choose the order that the triggered abilities will resolve in."
    );
    supported("Sheoldred, the Apocalypse");
    supported("Vision Skeins");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sheoldred, the Apocalypse");
    let skeins = t.hand(P1, "Vision Skeins");
    fill_pool(&mut t, P1, 2);
    t.cast(P1, skeins).go();
    t.resolve();
    // Each player drew two cards at once: four triggers, all P0's, ordered by P0 alone.
    assert_eq!(t.stack_len(), 4);
    let order: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (24, 16));
}

#[test]
fn niv_mizzet_and_curiosity_loop_until_a_player_loses() {
    cr!("603.2", "704.5a", "121.2");
    ruling!(
        "Niv-Mizzet, the Firemind",
        "If dealing 1 damage to a permanent or player causes you to draw a card, perhaps because Niv-Mizzet is enchanted by Curiosity, you'll continue this loop until the loop is interrupted, perhaps by a player losing the game."
    );
    supported("Niv-Mizzet, the Firemind");
    supported("Curiosity");
    let mut t = TestGame::new(2);
    let niv = t.battlefield(P0, "Niv-Mizzet, the Firemind");
    let curiosity = t.battlefield(P0, "Curiosity");
    t.g.attach(curiosity, obj(niv));
    t.g.players[1].life = 5;
    for _ in 0..10 {
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_yes(P0, true);
    }
    let lib = t.library_size(P0);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert!(t.has_lost(P1), "{}", t.dump_log());
    // One drawn card started it; each of the first four damage events drew another (the
    // fifth made P1 lose, ending the game).
    assert_eq!(lib - t.library_size(P0), 5);
}

#[test]
fn moonring_mirror_exiles_after_the_draw_and_swaps_hands_with_its_cards() {
    cr!("603.2", "406.3");
    ruling!(
        "Moonring Mirror",
        "Note that Moonring Mirror’s first ability is not a replacement effect."
    );
    ruling!(
        "Moonring Mirror",
        "If you choose to use Moonring Mirror’s second ability, you return all the cards you own exiled by both of its abilities, but not any of the cards you just exiled from your hand."
    );
    supported("Moonring Mirror");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moonring Mirror");
    let lib = t.g.player(P0).library.clone();
    let (top, second) = (lib[lib.len() - 1], lib[lib.len() - 2]);
    // The draw happens; then the trigger exiles the next card face down.
    draw(&mut t, P0, 1);
    assert_eq!(t.zone(top), Zone::Hand(P0));
    assert_eq!(t.zone(second), Zone::Library(P0));
    assert_eq!(triggers_on_stack(&t, "exile the top card"), 1);
    t.resolve_all();
    assert_eq!(t.zone(second), Zone::Exile);
    assert!(t.obj_now(second).face_down);
    // Two upkeeps: the first exiles the hand and returns the card the first ability
    // exiled; the second returns cards exiled by both abilities.
    for _ in 0..2 {
        let hand = t.g.player(P0).hand.clone();
        let exiled = t.g.exile.clone();
        assert!(!hand.is_empty() && !exiled.is_empty());
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        for id in &exiled {
            assert_eq!(t.zone(*id), Zone::Hand(P0));
        }
        for id in &hand {
            assert_eq!(t.zone(*id), Zone::Exile);
            assert!(t.obj_now(*id).face_down);
        }
        assert_eq!(t.hand_size(P0), exiled.len());
        // P0's draw step: the drawn card stays; the next one is exiled.
        t.advance_to(P0, Step::PrecombatMain);
        t.resolve_all();
    }
}

#[test]
fn teferis_talent_grants_one_of_the_loyalty_abilities_activatable_each_turn() {
    cr!("606.3");
    ruling!(
        "Teferi's Talent",
        "You can activate one loyalty ability of each planeswalker you control during each of your turns, whether that ability is one it always has or the one it gains because of Teferi's Talent."
    );
    supported("Teferi's Talent");
    for granted_first in [false, true] {
        let mut t = TestGame::new(2);
        let jace = t.battlefield(P0, "Jace Beleren");
        t.g.objects[jace.0 as usize]
            .counters
            .insert("loyalty".into(), 13);
        let talent = t.battlefield(P0, "Teferi's Talent");
        t.g.attach(talent, obj(jace));
        t.g.recompute();
        // Jace's abilities: +2, -1, -10, and the granted -12.
        let (first, second) = if granted_first { (3, 0) } else { (0, 3) };
        t.activate(P0, jace, first, &[pl(P0)]).expect("first loyalty ability");
        t.resolve_all();
        assert!(t.activate(P0, jace, second, &[]).is_err());
        assert_eq!(t.g.command.len(), granted_first as usize, "emblem");
    }
}
