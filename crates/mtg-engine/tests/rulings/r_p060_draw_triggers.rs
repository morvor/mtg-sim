//! Rulings batch P060 — abilities that trigger on drawing, or at the beginning of a step
//! to make a player draw: "their second card each turn" (CR 603.2), cards put into a
//! hand without "draw" (CR 121.1), replacement effects on draws (CR 614.11), and
//! "if" conditions checked twice (CR 603.4) with last known information (CR 608.2h).

use crate::r_p060_common::*;
use crate::r_s01_common::watch;
use crate::r_s02_common::can_play_land;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// `p` casts Opt ("Scry 1. Draw a card.") and it resolves: `p` draws one card.
fn opt(t: &mut TestGame, p: PlayerId) {
    t.lands(p, "Island", 1);
    let o = t.hand(p, "Opt");
    t.cast(p, o).go();
    t.resolve_all();
}

/// Treasure tokens `p` controls.
fn treasures(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype("Treasure"))
        .count()
}

#[test]
fn second_card_triggers_count_draws_from_before_you_controlled_it() {
    cr!("603.2", "603.10");
    ruling!(
        "Faerie Mastermind",
        "Faerie Mastermind doesn't need to have been under your control when the first card is drawn for its ability to trigger. As long as you control it when an opponent draws their second card in a turn, that ability will trigger."
    );
    ruling!(
        "Gleaming Splendor",
        "Gleaming Splendor doesn't need to have been under your control when the first card is drawn for its ability to trigger. As long as you control it when an opponent draws their second card in a turn, that ability will trigger. The ability can trigger only once each turn for each opponent."
    );
    supported("Faerie Mastermind");
    supported("Gleaming Splendor");
    // P1 draws a card; then P0 gets Faerie Mastermind; P1's second card triggers it, and
    // P1's third doesn't.
    let mut t = TestGame::new(2);
    opt(&mut t, P1);
    t.battlefield(P0, "Faerie Mastermind");
    let h0 = t.hand_size(P0);
    opt(&mut t, P1);
    assert_eq!(t.hand_size(P0), h0 + 1, "second card: you draw");
    opt(&mut t, P1);
    assert_eq!(t.hand_size(P0), h0 + 1, "third card: no trigger");
    // Gleaming Splendor with two opponents: P1's second card makes a Treasure, the third
    // doesn't; P2's second card makes another.
    let mut t = TestGame::new(3);
    opt(&mut t, P1);
    t.battlefield(P0, "Gleaming Splendor");
    opt(&mut t, P1);
    assert_eq!(treasures(&t, P0), 1);
    opt(&mut t, P1);
    assert_eq!(treasures(&t, P0), 1, "only once each turn for P1");
    opt(&mut t, P2);
    assert_eq!(treasures(&t, P0), 1, "P2's first card");
    opt(&mut t, P2);
    assert_eq!(treasures(&t, P0), 2, "P2's second card");
}

#[test]
fn putting_a_card_into_a_hand_without_drawing_doesnt_trigger_draw_abilities() {
    cr!("121.1", "603.2");
    ruling!(
        "Zurzoth, Chaos Rider",
        "If a spell or ability causes a player to put a card into their hand without specifically using the word \"draw,\" it's not a card drawn."
    );
    ruling!(
        "Nekusar, the Mindrazer",
        "If a spell or ability causes you to put cards into your hand without specifically using the word \"draw,\" it doesn't count as drawing cards and it doesn't cause Nekusar's second ability to trigger."
    );
    supported("Zurzoth, Chaos Rider");
    supported("Nekusar, the Mindrazer");
    let devils = |t: &TestGame| crate::r_s01_common::with_subtype(t, P0, "Devil").len();
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zurzoth, Chaos Rider");
    t.battlefield(P0, "Nekusar, the Mindrazer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Unsummon puts the Bears into P1's hand: no card drawn.
    t.lands(P0, "Island", 1);
    let u = t.hand(P0, "Unsummon");
    t.cast(P0, u).target(bears).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(devils(&t), 1, "only Zurzoth itself");
    assert_eq!(t.life(P1), 20, "no Nekusar damage");
    // Drawing a card does trigger both: the first card P1 draws this turn.
    opt(&mut t, P1);
    assert_eq!(devils(&t), 2, "a Devil token");
    assert_eq!(t.life(P1), 19, "Nekusar's 1 damage");
}

#[test]
fn zurzoth_you_lose_before_the_devils_death_trigger_goes_on_the_stack() {
    cr!("704.3", "704.5a", "704.5g", "603.3", "104.3b");
    ruling!(
        "Zurzoth, Chaos Rider",
        "If your life total is brought to 0 or less at the same time that a Devil token is dealt lethal damage, you lose the game before its ability goes on the stack."
    );
    for life in [2, 20] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Zurzoth, Chaos Rider");
        // P1 draws on P0's turn: P0 gets a Devil.
        opt(&mut t, P1);
        let devil: Vec<ObjectId> = crate::r_s01_common::tokens(&t, P0);
        assert_eq!(devil.len(), 1);
        t.g.players[0].life = life;
        // Earthquake (X = 2): 2 damage to each creature without flying and each player.
        t.lands(P0, "Mountain", 3);
        let eq = t.hand(P0, "Earthquake");
        t.cast(P0, eq).x(2).go();
        t.settle();
        t.g.resolve_top();
        t.settle();
        assert!(!t.on_battlefield(devil[0]), "the Devil died");
        if life == 2 {
            assert!(t.has_lost(P0));
            assert!(t.g.stack.is_empty(), "its ability never went on the stack");
            assert_eq!(t.life(P1), 18);
        } else {
            assert!(!t.has_lost(P0));
            // The Devil's ability ("it deals 1 damage to any target") is on the stack.
            assert_eq!(t.stack_len(), 1);
        }
    }
}

#[test]
fn spiteful_visions_damages_for_any_draw() {
    cr!("603.2", "121.1");
    ruling!(
        "Spiteful Visions",
        "Spiteful Visions's second ability triggers any time any player draws a card, not just when a player draws a card as a result of its first ability."
    );
    supported("Spiteful Visions");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spiteful Visions");
    // P1 draws with Opt in P0's main phase.
    opt(&mut t, P1);
    assert_eq!(t.life(P1), 19);
    opt(&mut t, P0);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn cut_a_deal_draws_one_per_opponent_who_drew() {
    cr!("614.6", "614.11", "608.2c");
    ruling!(
        "Cut a Deal",
        "If an opponent drew more than one card this way due to a replacement effect, you still draw only one card for that player."
    );
    ruling!(
        "Cut a Deal",
        "Although the draws are not optional, if an opponent doesn't draw a card, perhaps because a replacement effect replaced the draw with another effect, that player is not counted when determining how many cards you draw."
    );
    supported("Cut a Deal");
    supported("Notion Thief");
    // P1's Thought Reflection: "If you would draw a card, draw two cards instead." P1
    // draws two cards, and counts once; without it, one each.
    for reflection in [true, false] {
        let mut t = TestGame::new(3);
        if reflection {
            t.battlefield(P1, "Thought Reflection");
        }
        give_mana_for(&mut t, P0, "Cut a Deal");
        let s = t.hand(P0, "Cut a Deal");
        t.cast(P0, s).go();
        let before = libraries(&t);
        t.resolve_all();
        let p1 = if reflection { 2 } else { 1 };
        assert_eq!(drawn_since(&t, &before), vec![2, p1, 1], "{reflection}");
    }
    // P0's Notion Thief: each opponent's draw is replaced by P0 drawing ("that player skips
    // that draw and you draw a card"). No opponent drew, so P0 draws only the two cards
    // Notion Thief gave.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Notion Thief");
    give_mana_for(&mut t, P0, "Cut a Deal");
    let s = t.hand(P0, "Cut a Deal");
    t.cast(P0, s).go();
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![2, 0, 0]);
}

#[test]
fn a_player_whose_draw_was_replaced_by_draws_drew_a_card_this_way() {
    cr!("614.6", "614.11", "608.2c");
    // Kwain, Itinerant Meddler: "{T}: Each player may draw a card, then each player who
    // drew a card this way gains 1 life." P1's Thought Reflection replaces P1's draw with
    // two draws: P1 drew cards this way too.
    supported("Kwain, Itinerant Meddler");
    for reflection in [true, false] {
        let mut t = TestGame::new(2);
        let kwain = t.battlefield(P0, "Kwain, Itinerant Meddler");
        if reflection {
            t.battlefield(P1, "Thought Reflection");
        }
        t.answer_yes(P0, true);
        t.answer_yes(P1, true);
        t.activate(P0, kwain, 0, &[]).unwrap();
        let before = libraries(&t);
        t.resolve_all();
        let p1 = if reflection { 2 } else { 1 };
        assert_eq!(drawn_since(&t, &before), vec![1, p1], "{reflection}");
        assert_eq!((t.life(P0), t.life(P1)), (21, 21), "{reflection}");
    }
    // P0 draws, P1 declines: only P0 gains life (what P0 drew isn't P1's).
    let mut t = TestGame::new(2);
    let kwain = t.battlefield(P0, "Kwain, Itinerant Meddler");
    t.battlefield(P1, "Thought Reflection");
    t.answer_yes(P0, true);
    t.answer_yes(P1, false);
    t.activate(P0, kwain, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 20));
    // A draw replaced by another player's draw isn't a card drawn this way: with P0's
    // Notion Thief, P1's draw is replaced by P0 drawing; P1 didn't draw and gains no life.
    supported("Notion Thief");
    let mut t = TestGame::new(2);
    let kwain = t.battlefield(P0, "Kwain, Itinerant Meddler");
    t.battlefield(P0, "Notion Thief");
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.activate(P0, kwain, 0, &[]).unwrap();
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![2, 0]);
    assert_eq!((t.life(P0), t.life(P1)), (21, 20));
    // With P1's Notion Thief, P0's draw is replaced by P1 drawing: P0 didn't draw and gains
    // no life; P1 draws its own card too, and gains 1.
    let mut t = TestGame::new(2);
    let kwain = t.battlefield(P0, "Kwain, Itinerant Meddler");
    t.battlefield(P1, "Notion Thief");
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.activate(P0, kwain, 0, &[]).unwrap();
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![0, 2]);
    assert_eq!((t.life(P0), t.life(P1)), (20, 21));
}

/// Advances from P1's end step into P0's draw step (the draw-step draw done and triggers
/// on the stack), with P0's hand empty at the start.
fn to_p0_draw_step(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    t.settle();
}

/// P0's draws (true) and discards (false), in order, since event `from`.
fn p0_draws_discards(t: &TestGame, from: usize) -> Vec<bool> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Drew { player, .. } if *player == P0 => Some(true),
            Event::Discarded { player, .. } if *player == P0 => Some(false),
            _ => None,
        })
        .collect()
}

#[test]
fn anvil_of_bogardan_triggers_resolve_separately_and_discard_even_if_the_draw_is_replaced() {
    cr!("603.2", "614.11", "614.10");
    ruling!(
        "Anvil of Bogardan",
        "If there are two Anvils on the battlefield, they trigger and resolve separately. This means that you first draw one card and discard one card, and then do it again. You do not draw two cards and then discard two cards."
    );
    ruling!(
        "Anvil of Bogardan",
        "If the draw part of the effect is skipped due to a replacement effect, the discard part of the effect is not skipped."
    );
    supported("Anvil of Bogardan");
    // Two Anvils.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anvil of Bogardan");
    t.battlefield(P0, "Anvil of Bogardan");
    let from = t.g.turn_events.len();
    to_p0_draw_step(&mut t);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(
        p0_draws_discards(&t, from),
        vec![true, true, false, true, false],
        "draw-step draw, then draw and discard twice"
    );
    assert_eq!(t.hand_size(P0), 1);
    // P1's Notion Thief: P0's additional draw (not the first one in the draw step) is
    // skipped and P1 draws instead; P0 still discards.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anvil of Bogardan");
    t.battlefield(P1, "Notion Thief");
    let from = t.g.turn_events.len();
    to_p0_draw_step(&mut t);
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![0, 1]);
    assert_eq!(p0_draws_discards(&t, from), vec![true, false]);
    assert_eq!((t.hand_size(P0), t.graveyard_size(P0)), (0, 1));
}

#[test]
fn hand_size_effects_apply_in_timestamp_order() {
    cr!("613.11", "402.2");
    ruling!(
        "Anvil of Bogardan",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you put Null Profusion (an enchantment that says your maximum hand size is two) onto the battlefield and then put Anvil of Bogardan onto the battlefield, you'll have no maximum hand size. However, if those permanents entered in the opposite order, your maximum hand size would be two."
    );
    supported("Null Profusion");
    for (first, second, max) in [
        ("Null Profusion", "Anvil of Bogardan", None),
        ("Anvil of Bogardan", "Null Profusion", Some(2)),
    ] {
        let mut t = TestGame::new(2);
        t.enter(P0, first);
        t.enter(P0, second);
        t.g.recompute();
        assert_eq!(t.g.player(P0).max_hand_size, max, "{first} then {second}");
        // P1 has no maximum hand size either way (Anvil: "Players").
        let p1 = t.g.player(P1).max_hand_size;
        assert_eq!(p1, None, "{first} then {second}: P1");
    }
}

#[test]
fn howling_mine_checks_untapped_twice_using_last_known_information() {
    cr!("603.4", "608.2h", "504.1");
    ruling!(
        "Howling Mine",
        "It does not trigger at all if this is tapped at the start of the draw step, and it checks this again on resolution."
    );
    ruling!(
        "Howling Mine",
        "If Howling Mine leaves the battlefield before it resolves, then the last known tap or untap state of the card is used for resolution."
    );
    supported("Howling Mine");
    // (tapped in the upkeep, tapped in response, destroyed in response, extra card)
    for (tap_early, tap_late, destroy_it, extra) in [
        (false, false, false, 1),
        (true, false, false, 0),
        (false, true, false, 0),
        (false, false, true, 1),
        (false, true, true, 0),
    ] {
        let what = format!("early {tap_early}, late {tap_late}, destroyed {destroy_it}");
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Howling Mine");
        let icy = t.battlefield(P1, "Icy Manipulator");
        t.lands(P1, "Wastes", 1);
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        if tap_early {
            t.activate(P1, icy, 0, &[obj(mine)]).unwrap();
            t.resolve_all();
            assert!(t.obj_now(mine).tapped);
        }
        let before = libraries(&t);
        t.advance_to(P0, Step::Draw);
        t.settle();
        assert_eq!(t.stack_len(), usize::from(!tap_early), "{what}: triggered");
        if tap_late {
            t.activate(P1, icy, 0, &[obj(mine)]).unwrap();
            t.resolve();
        }
        if destroy_it {
            destroy(&mut t, mine);
        }
        t.resolve_all();
        assert_eq!(drawn_since(&t, &before), vec![1 + extra, 0], "{what}");
    }
}

#[test]
fn ghirapur_orrery_additional_land_plays_follow_normal_timing_and_add_up() {
    cr!("305.1", "305.2", "305.2a", "505.6b");
    ruling!(
        "Ghirapur Orrery",
        "Ghirapur Orrery's first ability allows a player to play an additional land during their main phase. Doing so follows the normal timing rules for playing lands."
    );
    ruling!(
        "Ghirapur Orrery",
        "Ghirapur Orrery's first ability is cumulative if more than one is on the battlefield."
    );
    supported("Ghirapur Orrery");
    for orreries in [1usize, 2] {
        let mut t = TestGame::new(2);
        for _ in 0..orreries {
            t.battlefield(P1, "Ghirapur Orrery");
        }
        let lands: Vec<ObjectId> = (0..orreries + 2).map(|_| t.hand(P0, "Forest")).collect();
        // Not outside a main phase, even with land plays left.
        t.set_step(P0, Step::Upkeep);
        assert!(!can_play_land(&mut t, P0, lands[0]), "upkeep");
        // Not while the stack isn't empty.
        t.set_step(P0, Step::PrecombatMain);
        t.lands(P0, "Mountain", 1);
        let shock = t.hand(P0, "Shock");
        t.cast(P0, shock).target(pl(P1)).go();
        assert!(!can_play_land(&mut t, P0, lands[0]), "stack not empty");
        t.resolve_all();
        // In the main phase with an empty stack: 1 + orreries lands.
        for (i, land) in lands.iter().enumerate() {
            let ok = can_play_land(&mut t, P0, *land);
            assert_eq!(ok, i <= orreries, "{orreries} Orreries: land {i}");
            if ok {
                t.play_land(P0, *land).unwrap();
            }
        }
        let forests = t
            .g
            .permanents()
            .filter(|o| o.is(CardType::Land) && o.controller == P0 && o.chars.name == "Forest")
            .count();
        assert_eq!(forests, orreries + 1);
        // In the combat phase, a land play left can't be used.
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Ghirapur Orrery");
        let f = t.hand(P0, "Forest");
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(!can_play_land(&mut t, P0, f), "combat");
    }
}

#[test]
fn ghirapur_orrery_checks_the_hand_at_the_start_of_the_upkeep_and_on_resolution() {
    cr!("603.4", "503.1a");
    ruling!(
        "Ghirapur Orrery",
        "No player may take actions in a turn before Ghirapur Orrery's second ability checks to see if it should trigger. If the player whose turn it is has any cards in hand, it won't trigger."
    );
    ruling!(
        "Ghirapur Orrery",
        "If the player has any cards in hand as Ghirapur Orrery's second ability resolves, the ability does nothing. Notably, a second Ghirapur Orrery won't have a player draw another three cards unless the player empties their hand after resolving the first one's trigger."
    );
    // One card in hand as the upkeep begins (P0 could cast it, but has no chance to
    // before the check): no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ghirapur Orrery");
    t.lands(P0, "Mountain", 1);
    t.hand(P0, "Lightning Bolt");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Two Orreries, empty hand: both trigger, but only the first one draws three.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ghirapur Orrery");
    t.battlefield(P1, "Ghirapur Orrery");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    // Emptying the hand in between: the second one draws three more. P0 draws three
    // Lightning Bolts and casts them before the second trigger resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ghirapur Orrery");
    t.battlefield(P1, "Ghirapur Orrery");
    t.lands(P0, "Mountain", 3);
    stack_library(&mut t, P0, &["Lightning Bolt", "Lightning Bolt", "Lightning Bolt"]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
    let bolts: Vec<ObjectId> = t.g.player(P0).hand.clone();
    for b in bolts {
        t.cast(P0, b).target(pl(P1)).go();
        t.resolve();
    }
    assert_eq!(t.life(P1), 11);
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn ghirapur_orrery_in_two_headed_giant_triggers_for_each_player() {
    cr!("603.4", "805.4d");
    ruling!(
        "Ghirapur Orrery",
        "In a Two-Headed Giant game, Ghirapur Orrery's second ability triggers for each player on a team separately. If one player has cards in hand, it can still trigger for the other player."
    );
    // P2 and P3's team turn: P3 holds a card, P2 doesn't; then both empty.
    for (p3_cards, triggers) in [(1usize, 1usize), (0, 2)] {
        let mut t = crate::r_p050_common::two_headed_giant();
        t.battlefield(P0, "Ghirapur Orrery");
        bears_in_hand(&mut t, P3, p3_cards);
        t.advance_to(P2, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), triggers);
        t.resolve_all();
        assert_eq!(t.hand_size(P2), 3);
        assert_eq!(t.hand_size(P3), if p3_cards == 1 { 1 } else { 3 });
    }
}

#[test]
fn fevered_visions_checks_the_hand_right_after_the_draw() {
    cr!("608.2c", "513.1");
    ruling!(
        "Fevered Visions",
        "No player may take any action in between the two steps of Fevered Visions's triggered ability, so if your opponent has four or more cards in hand after drawing a card, Fevered Visions will deal 2 damage to that player."
    );
    supported("Fevered Visions");
    // (player whose end step it is, cards in hand before, damage)
    for (p, cards, damage) in [(P1, 3usize, 2), (P1, 2, 0), (P0, 3, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Fevered Visions");
        bears_in_hand(&mut t, p, cards);
        t.set_step(p, Step::PostcombatMain);
        t.advance_to(p, Step::End);
        t.settle();
        t.resolve_all();
        assert_eq!(t.hand_size(p), cards + 1);
        assert_eq!(t.life(p), 20 - damage, "{p} with {cards}");
    }
}

#[test]
fn shatter_the_sky_draws_then_destroys_all_creatures() {
    cr!("608.2c", "701.8a");
    ruling!(
        "Shatter the Sky",
        "Shatter the Sky destroys all creatures, even if no player draws a card."
    );
    ruling!(
        "Shatter the Sky",
        "No player may take any actions between the time each qualifying player draws a card and the time all creatures are destroyed."
    );
    supported("Shatter the Sky");
    // P1's Serra Angel (power 4) qualifies; P0's Grizzly Bears doesn't. The players pass
    // priority as the game runs; at each priority, the Angels on the battlefield and the
    // cards in P1's hand are recorded.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    give_mana_for(&mut t, P0, "Shatter the Sky");
    let s = t.hand(P0, "Shatter the Sky");
    t.cast(P0, s).go();
    let before = libraries(&t);
    let look = |g: &mtg_engine::game::Game| {
        (
            g.find_in_zone(Zone::Battlefield, "Serra Angel").len(),
            g.player(PlayerId(1)).hand.len(),
        )
    };
    let priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let seen: Vec<_> = [P0, P1].map(|p| watch(&mut t, p, priority, look)).into();
    assert!(t.g.run_until(100, |g| g.stack.is_empty()));
    // One more priority, after it has resolved.
    t.g.advance();
    assert_eq!(drawn_since(&t, &before), vec![0, 1]);
    assert!(!t.on_battlefield(angel));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let all: Vec<(usize, usize)> = seen
        .iter()
        .flat_map(|s| s.lock().unwrap().clone())
        .collect();
    // Players had priority before it resolved (the Angel there, no card drawn) and after
    // (the Angel destroyed, the card drawn), but never with the card drawn and the Angel
    // still on the battlefield.
    assert!(all.contains(&(1, 0)) && all.contains(&(0, 1)), "{all:?}");
    assert!(!all.contains(&(1, 1)), "{all:?}");
    // No creature with power 4 or greater: no one draws; all creatures are destroyed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P0, "Shatter the Sky");
    let s = t.hand(P0, "Shatter the Sky");
    t.cast(P0, s).go();
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![0, 0]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn the_revealed_card_is_the_one_drawn() {
    cr!("701.20a", "121.1", "707.10");
    ruling!(
        "Gandalf, Westward Voyager",
        "Except in some very rare cases, the card each opponent draws will be the card revealed from the top of their library."
    );
    ruling!(
        "Storm Fleet Negotiator",
        "Except in some very unusual cases, the card each player reveals is the one they'll draw."
    );
    supported("Gandalf, Westward Voyager");
    supported("Storm Fleet Negotiator");
    // Gandalf: P1 reveals Grizzly Bears, sharing a type with Craw Wurm, and draws it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gandalf, Westward Voyager");
    let top = t.library_top(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 6);
    let w = t.hand(P0, "Craw Wurm");
    t.cast(P0, w).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 2, "copied");
    assert_eq!(t.zone(top), Zone::Hand(P1));
    // Storm Fleet Negotiator attacks: P0 reveals a nonland card, P1 a land: one Map; each
    // draws the card they revealed.
    let mut t = TestGame::new(2);
    let sfn = t.battlefield(P0, "Storm Fleet Negotiator");
    let top0 = t.library_top(P0, "Hill Giant");
    let top1 = t.library_top(P1, "Forest");
    crate::r_s01_common::attack_with(&mut t, &[(sfn, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(top0), Zone::Hand(P0));
    assert_eq!(t.zone(top1), Zone::Hand(P1));
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Map").len(), 1);
}

#[test]
fn gandalf_copy_keeps_the_division_of_damage_and_counters() {
    cr!("707.10", "707.10c", "601.2d");
    ruling!(
        "Gandalf, Westward Voyager",
        "If the spell has damage divided as it was put onto the stack, the division can't be changed, although the targets receiving that damage still can. The same is true of spells that distribute counters to targets."
    );
    supported("Pyrokinesis");
    // Pyrokinesis (mana value 6): "4 damage divided as you choose among any number of
    // target creatures". P1 reveals an instant: Gandalf copies it, and P0 chooses new
    // targets for the copy; the 3/1 division stays.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gandalf, Westward Voyager");
    t.library_top(P1, "Lightning Bolt");
    let m: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P1, "Darksteel Myr")).collect();
    t.lands(P0, "Mountain", 6);
    let pk = t.hand(P0, "Pyrokinesis");
    t.answer_targets(P0, &[obj(m[0]), obj(m[1])]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 1]));
    // New targets for the copy, one for each target of the original.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(m[2])]);
    t.answer_targets(P0, &[obj(m[3])]);
    t.cast(P0, pk).go();
    let from = t.asked().len();
    t.resolve_all();
    let dmg: Vec<u32> = m.iter().map(|id| t.obj_now(*id).damage).collect();
    assert_eq!(dmg, vec![3, 1, 3, 1]);
    // No new division was asked for the copy.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Divide { .. })));
    // Counters: Blessings of Nature (mana value 5), "Distribute four +1/+1 counters among
    // any number of target creatures." P1 reveals a sorcery: the copy, with new targets,
    // keeps the 3/1 distribution.
    supported("Blessings of Nature");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gandalf, Westward Voyager");
    t.library_top(P1, "Divination");
    let b: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    t.lands(P0, "Forest", 5);
    let bon = t.hand(P0, "Blessings of Nature");
    t.answer_targets(P0, &[obj(b[0]), obj(b[1])]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 1]));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(b[2])]);
    t.answer_targets(P0, &[obj(b[3])]);
    t.cast(P0, bon).go();
    let from = t.asked().len();
    t.resolve_all();
    let counters: Vec<u32> = b.iter().map(|id| t.counters(*id, "+1/+1")).collect();
    assert_eq!(counters, vec![3, 1, 3, 1]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Divide { .. })));
}

#[test]
fn selvalas_enforcer_still_reveals_and_draws_if_it_left() {
    cr!("113.7a", "400.7");
    ruling!(
        "Selvala's Enforcer",
        "If Selvala’s Enforcer leaves the battlefield before its ability resolves, the top card of each library will still be revealed and drawn, although no +1/+1 counters will be placed."
    );
    supported("Selvala's Enforcer");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let top0 = t.library_top(P0, "Hill Giant");
        let top1 = t.library_top(P1, "Grizzly Bears");
        let e = t.enter(P0, "Selvala's Enforcer");
        t.settle();
        if gone {
            destroy(&mut t, e);
        }
        t.resolve_all();
        assert_eq!(t.zone(top0), Zone::Hand(P0), "gone: {gone}");
        assert_eq!(t.zone(top1), Zone::Hand(P1), "gone: {gone}");
        if !gone {
            assert_eq!(t.counters(e, "+1/+1"), 2);
        } else {
            assert!(t.in_graveyard(P0, "Selvala's Enforcer"));
        }
    }
}

#[test]
fn selvala_explorer_returned_isnt_a_mana_ability() {
    cr!("605.1a", "605.3a", "602.2");
    ruling!(
        "Selvala, Explorer Returned",
        "Selvala’s ability is not a mana ability. Players can respond to it, and it can’t be activated in the middle of paying for a cost. A recent update to rule 605.1a has made any abilities that move cards to or from libraries no longer mana abilities."
    );
    supported("Selvala, Explorer Returned");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Selvala, Explorer Returned");
    assert!(t.obj(s).chars.abilities.iter().all(|a| !matches!(
        &a.kind,
        mtg_engine::ability::AbilityKind::Activated(act) if act.is_mana_ability
    )));
    // It can't pay for a spell: with only Selvala for mana, Llanowar Elves ({G}) can't be
    // cast, though Selvala would add {G} for the Hill Giant on top of P0's library.
    t.library_top(P0, "Hill Giant");
    t.library_top(P1, "Forest");
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(t.cast(P0, elves).try_go().is_err());
    // Activated: it uses the stack (no mana or life yet), and can be responded to.
    t.activate(P0, s, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert_eq!(t.life(P0), 20);
    t.g.resolve_top();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1, "{{G}} for the Hill Giant");
    assert_eq!(t.life(P0), 21);
}

#[test]
fn playing_a_land_triggers_but_putting_one_onto_the_battlefield_doesnt() {
    cr!("305.1", "305.4", "603.2");
    ruling!(
        "Horn of Greed",
        "Playing a land will trigger it, but putting a land onto the battlefield as part of an effect will not."
    );
    ruling!(
        "Burgeoning",
        "Playing a land will trigger it, but putting a land onto the battlefield as part of an effect will not."
    );
    supported("Horn of Greed");
    supported("Burgeoning");
    // Horn of Greed: P0 plays a Forest and draws; Rampant Growth's land doesn't draw.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Horn of Greed");
    let f = t.hand(P0, "Forest");
    let before = libraries(&t);
    t.play_land(P0, f).unwrap();
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![1, 0]);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let rg = t.hand(P0, "Rampant Growth");
    let plains = t.library_top(P0, "Plains");
    t.answer_choose(P0, &[obj(plains)]);
    t.cast(P0, rg).go();
    let before = libraries(&t);
    t.resolve_all();
    assert!(t.named_on_battlefield("Plains").len() == 1, "the land entered");
    assert_eq!(drawn_since(&t, &before)[0], 1, "only the searched card left");
    assert_eq!(t.hand_size(P0), 1, "no card drawn");
    // Burgeoning: P1 plays a land on P1's turn: P0 may put a land from hand onto the
    // battlefield. Rampant Growth's land doesn't trigger it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Burgeoning");
    let mine = t.hand(P0, "Island");
    t.set_step(P1, Step::PrecombatMain);
    let f = t.hand(P1, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(mine)]);
    t.play_land(P1, f).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(mine), "Burgeoning put the Island onto the battlefield");
    let mine2 = t.hand(P0, "Island");
    t.lands(P1, "Wastes", 1);
    let plains = t.library_top(P1, "Plains");
    t.answer_choose(P1, &[obj(plains)]);
    // (Had it triggered, P0 would put the second Island onto the battlefield.)
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(mine2)]);
    let rg = t.hand(P1, "Rampant Growth");
    t.cast(P1, rg).go();
    t.settle();
    t.g.resolve_top();
    t.settle();
    assert_eq!(t.named_on_battlefield("Plains").len(), 1, "the land entered");
    assert_eq!(t.stack_len(), 0, "no trigger");
    assert_eq!(t.zone(mine2), Zone::Hand(P0));
}
