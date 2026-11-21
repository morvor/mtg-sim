//! Rulings batch P051 — "draw matters" and "draw hate": one trigger per card drawn
//! (CR 121.2, 603.2c), "your second card each turn" (CR 121.2), "can't draw more than one
//! card each turn" applies to individual draws, including draws a replacement effect
//! produces and optional draws (CR 121.2b), a replaced draw isn't a draw (CR 121.6,
//! 614.6), "players can't draw cards" (CR 121.3), and draw-replacement doubling (CR 121.6,
//! 614.1a).

use crate::r_p046_common::fill_pool;
use crate::r_p051_common::*;
use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BEARS: &str = "Grizzly Bears";

/// `p` is instructed to draw `n` cards (one instruction); returns the cards drawn.
fn draw(t: &mut TestGame, p: PlayerId, n: u32) -> usize {
    let hand = t.hand_size(p);
    t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
    t.hand_size(p) - hand
}

#[test]
fn leovold_lets_an_opponent_draw_one_card_each_turn() {
    cr!("121.2b");
    ruling!(
        "Leovold, Emissary of Trest",
        "Your opponents can draw a maximum of one card each on each player’s turn. Subsequent card draws during that turn are ignored."
    );
    ruling!(
        "Leovold, Emissary of Trest",
        "If an opponent hasn’t drawn any cards in a turn and a spell or ability instructs that player to draw multiple cards, that player will just draw one card. However, if the draws are optional, the player can’t choose to draw, even if they could draw one card this way."
    );
    supported("Leovold, Emissary of Trest");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Leovold, Emissary of Trest");
    assert_eq!(draw(&mut t, P1, 3), 1);
    assert_eq!(draw(&mut t, P1, 1), 0);
    // Each opponent separately; Leovold's controller isn't affected.
    assert_eq!(draw(&mut t, P2, 2), 1);
    assert_eq!(draw(&mut t, P0, 2), 2);
    // On the next turn (P1's), another card.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(draw(&mut t, P1, 2), 1);
    // An optional draw of two cards: Consecrated Sphinx ("Whenever an opponent draws a
    // card, you may draw two cards.") under P1's control; P1 hasn't drawn this turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leovold, Emissary of Trest");
    t.battlefield(P1, "Consecrated Sphinx");
    let hand = t.hand_size(P1);
    t.answer_yes(P1, true);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand, "P1 can't choose to draw two cards");
    assert_eq!(drawn_this_turn(&t, P1), 0);
}

#[test]
fn leovold_also_limits_draws_a_replacement_effect_produces() {
    cr!("121.2b", "121.6", "614.1a");
    ruling!(
        "Leovold, Emissary of Trest",
        "Your opponents can draw a maximum of one card each on each player’s turn. Subsequent card draws during that turn are ignored."
    );
    supported("Leovold, Emissary of Trest");
    // Thought Reflection ("If you would draw a card, draw two cards instead.") under the
    // opponent's control: the second of the two draws is ignored.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leovold, Emissary of Trest");
    t.battlefield(P1, "Thought Reflection");
    assert_eq!(draw(&mut t, P1, 1), 1);
    assert_eq!(draw(&mut t, P1, 1), 0);
}

#[test]
fn leovold_sees_cards_drawn_before_it_entered() {
    cr!("121.2b");
    ruling!(
        "Leovold, Emissary of Trest",
        "Leovold will “see” cards drawn by opponents earlier in the turn before it entered the battlefield, although Leovold can’t affect cards drawn before it entered the battlefield. For example, if an opponent draws two cards, then Leovold enters the battlefield, that opponent can’t draw more cards that turn, but the two drawn cards are unaffected."
    );
    supported("Leovold, Emissary of Trest");
    let mut t = TestGame::new(2);
    assert_eq!(draw(&mut t, P1, 2), 2);
    let hand = t.hand_size(P1);
    t.battlefield(P0, "Leovold, Emissary of Trest");
    assert_eq!(t.hand_size(P1), hand);
    assert_eq!(draw(&mut t, P1, 1), 0);
}

#[test]
fn leovold_disallowed_draws_cant_be_replaced_but_a_replaced_draw_doesnt_count() {
    cr!("121.2b", "121.6", "702.52a", "614.6");
    ruling!(
        "Leovold, Emissary of Trest",
        "Replacement effects (such as that of dredge or Laboratory Maniac’s ability) can’t be used to replace draws that Leovold disallows. However, if an opponent’s first draw is replaced (by a dredge ability, for example), that draw didn’t happen and Leovold won’t stop the next draw."
    );
    supported("Leovold, Emissary of Trest");
    supported("Golgari Brownscale");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leovold, Emissary of Trest");
    // Dredge 2.
    t.graveyard(P1, "Golgari Brownscale");
    // The first draw is replaced by dredge: the Brownscale returns to P1's hand.
    t.answer_yes(P1, true);
    let library = t.library_size(P1);
    draw(&mut t, P1, 1);
    assert!(t.in_hand(P1, "Golgari Brownscale"));
    assert_eq!(t.library_size(P1), library - 2);
    assert_eq!(drawn_this_turn(&t, P1), 0);
    t.clear_answers();
    // The next draw happens.
    assert_eq!(draw(&mut t, P1, 1), 1);
    // A disallowed draw can't be replaced by dredge.
    let brownscale = t.g.find_in_zone(Zone::Hand(P1), "Golgari Brownscale")[0];
    move_to(&mut t, brownscale, Zone::Graveyard(P1));
    t.answer_yes(P1, true);
    let library = t.library_size(P1);
    assert_eq!(draw(&mut t, P1, 1), 0);
    assert!(t.in_graveyard(P1, "Golgari Brownscale"));
    assert_eq!(t.library_size(P1), library);
}

#[test]
fn leovold_triggers_once_for_each_of_you_and_your_permanents_targeted() {
    cr!("603.2c", "115.1");
    ruling!(
        "Leovold, Emissary of Trest",
        "If you and a permanent you control each become the target of the same spell or ability an opponent controls, Leovold’s ability will trigger twice. The same is true if two permanents you control become the target of the same spell or ability an opponent controls."
    );
    supported("Leovold, Emissary of Trest");
    supported("Lorehold Command");
    supported("Seeds of Strength");
    // You and Leovold: Lorehold Command's third mode ("Lorehold Command deals 3 damage to
    // any target. Target player gains 3 life.").
    let mut t = TestGame::new(2);
    let leo = t.battlefield(P0, "Leovold, Emissary of Trest");
    t.set_step(P1, Step::PrecombatMain);
    fill_pool(&mut t, P1, 10);
    let lc = t.hand(P1, "Lorehold Command");
    t.answer_targets(P1, &[obj(leo)]);
    t.answer_targets(P1, &[pl(P0)]);
    t.cast(P1, lc).modes(&[0, 2]).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you may draw a card"), 2);
    // Two permanents: Seeds of Strength ("Target creature gets +1/+1 until end of turn"
    // three times) on Leovold, P0's Bears and P1's Bears.
    let mut t = TestGame::new(2);
    let leo = t.battlefield(P0, "Leovold, Emissary of Trest");
    let bears = t.battlefield(P0, BEARS);
    let theirs = t.battlefield(P1, BEARS);
    t.set_step(P1, Step::PrecombatMain);
    fill_pool(&mut t, P1, 10);
    let seeds = t.hand(P1, "Seeds of Strength");
    t.answer_targets(P1, &[obj(leo)]);
    t.answer_targets(P1, &[obj(bears)]);
    t.answer_targets(P1, &[obj(theirs)]);
    t.cast(P1, seeds).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you may draw a card"), 2);
}

#[test]
fn draw_triggers_trigger_once_for_each_card_drawn() {
    cr!("121.2", "603.2c");
    ruling!(
        "Underworld Dreams",
        "If multiple cards are drawn at once, this card triggers once for each card drawn."
    );
    supported("Underworld Dreams");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Dreams");
    draw(&mut t, P1, 3);
    assert_eq!(triggers_on_stack(&t, "1 damage"), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);

    ruling!(
        "Niv-Mizzet, the Firemind",
        "If a spell or ability causes you to draw multiple cards, Niv-Mizzet's ability triggers that many times."
    );
    supported("Niv-Mizzet, the Firemind");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Niv-Mizzet, the Firemind");
    for _ in 0..2 {
        t.answer_targets(P0, &[pl(P1)]);
    }
    draw(&mut t, P0, 2);
    assert_eq!(triggers_on_stack(&t, "1 damage"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn underworld_dreams_and_lifeblood_affect_all_opponents() {
    cr!("806.1");
    ruling!(
        "Underworld Dreams",
        "In a multi-player game, it affects all opponents."
    );
    ruling!(
        "Lifeblood",
        "In a multi-player game, it affects all opponents."
    );
    supported("Underworld Dreams");
    supported("Lifeblood");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Underworld Dreams");
    t.battlefield(P0, "Lifeblood");
    let m1 = t.battlefield(P1, "Mountain");
    let m2 = t.battlefield(P2, "Mountain");
    let mine = t.battlefield(P0, "Mountain");
    draw(&mut t, P1, 1);
    draw(&mut t, P2, 1);
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(lives(&t, 3), vec![20, 19, 19]);
    for m in [m1, m2, mine] {
        t.g.tap(m);
    }
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn thought_reflection_doubles_each_draw() {
    cr!("121.2", "121.6", "614.1a");
    ruling!(
        "Thought Reflection",
        "If a spell or ability causes you to draw multiple cards, Thought Reflection's effect doubles each card draw. For example, if you cast Harmonize (“Draw three cards”), you'll draw six cards."
    );
    supported("Thought Reflection");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thought Reflection");
    let hand = t.hand_size(P0);
    cast_pool(&mut t, P0, "Harmonize", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 6);
    assert_eq!(drawn_this_turn(&t, P0), 6);
}

#[test]
fn second_card_each_turn_triggers_once_wherever_the_permanent_was_for_the_first() {
    cr!("121.2", "603.2");
    ruling!(
        "Raven Eagle",
        "The second triggered ability can trigger only once each turn. It doesn't matter whether Raven Eagle was on the battlefield when the first card was drawn. If it's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn. It won't trigger when the third or fourth card is drawn."
    );
    ruling!(
        "Zimone and Dina",
        "Zimone and Dina doesn’t need to have been under your control when the first card is drawn for its ability to trigger. As long as you control it when you draw your second card in a turn, that ability will trigger."
    );
    supported("Raven Eagle");
    supported("Zimone and Dina");
    for name in ["Raven Eagle", "Zimone and Dina"] {
        // Arrives after the first card: triggers on the second only.
        let mut t = TestGame::new(2);
        draw(&mut t, P0, 1);
        t.battlefield(P0, name);
        t.answer_targets(P0, &[pl(P1)]);
        draw(&mut t, P0, 3);
        t.resolve_all();
        let drain = if name == "Raven Eagle" { 1 } else { 2 };
        assert_eq!(lives(&t, 2), vec![20 + drain, 20 - drain], "{name}");
        draw(&mut t, P0, 1);
        t.resolve_all();
        assert_eq!(lives(&t, 2), vec![20 + drain, 20 - drain], "{name}");
        // Arrives after the second card: nothing this turn.
        let mut t = TestGame::new(2);
        draw(&mut t, P0, 2);
        t.battlefield(P0, name);
        draw(&mut t, P0, 2);
        t.resolve_all();
        assert_eq!(lives(&t, 2), vec![20, 20], "{name}");
    }
}

#[test]
fn eyekite_counts_cards_drawn_before_it_entered() {
    cr!("611.3a", "121.2");
    ruling!(
        "Eyekite",
        "Eyekite's effect applies even if you drew two or more cards only before Eyekite entered the battlefield."
    );
    supported("Eyekite");
    let mut t = TestGame::new(2);
    draw(&mut t, P0, 2);
    let e = t.battlefield(P0, "Eyekite");
    assert_eq!(t.pt(e), (3, 2));
}

#[test]
fn gnarled_sage_gets_its_bonus_once_and_vigilance_doesnt_untap_it() {
    cr!("611.3a", "702.20b");
    ruling!(
        "Gnarled Sage",
        "Gnarled Sage gets +0/+2 just once, no matter how many cards you've drawn past the second."
    );
    ruling!(
        "Gnarled Sage",
        "If Gnarled Sage gains vigilance after it has attacked, it won't become untapped."
    );
    supported("Gnarled Sage");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Gnarled Sage");
    draw(&mut t, P0, 5);
    assert_eq!(t.pt(s), (4, 6));
    // Attacks before drawing: tapped; the draws give it vigilance afterwards.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Gnarled Sage");
    attack_with(&mut t, &[(s, pl(P1))]);
    assert!(t.obj_now(s).tapped);
    draw(&mut t, P0, 2);
    assert!(t
        .obj_now(s)
        .has_keyword(mtg_engine::keywords::KeywordKind::Vigilance));
    assert!(t.obj_now(s).tapped);
}

#[test]
fn burlfist_oak_ignores_cards_put_into_hand_without_drawing() {
    cr!("121.5", "121.2");
    ruling!(
        "Burlfist Oak",
        "If a spell or ability causes a player to put a card into their hand without specifically using the word “draw,” it's not a card drawn."
    );
    supported("Burlfist Oak");
    supported("Ancestral Memories");
    let mut t = TestGame::new(2);
    let oak = t.battlefield(P0, "Burlfist Oak");
    cast_pool(&mut t, P0, "Ancestral Memories", &[]);
    t.resolve_all();
    assert_eq!(t.pt(oak), (2, 3));
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.pt(oak), (4, 5));
}

#[test]
fn nadir_kraken_pays_for_a_tentacle_after_leaving() {
    cr!("113.7a", "603.5");
    ruling!(
        "Nadir Kraken",
        "If Nadir Kraken leaves the battlefield after its ability has triggered, you can still pay {1} and get a Tentacle, even though you won't put a +1/+1 counter on Nadir Kraken."
    );
    supported("Nadir Kraken");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Nadir Kraken");
    draw(&mut t, P0, 1);
    assert_eq!(triggers_on_stack(&t, "pay {1}"), 1);
    move_to(&mut t, k, Zone::Graveyard(P0));
    fill_pool(&mut t, P0, 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Tentacle").len(), 1);
    assert_eq!(t.counters(t.g.current(k), "+1/+1"), 0);
}

#[test]
fn dream_trawler_triggers_resolve_before_blockers() {
    cr!("508.1m", "509.1");
    ruling!(
        "Dream Trawler",
        "Dream Trawler's second triggered ability resolves before blockers are declared. This normally causes its first triggered ability to trigger, which also resolves before blockers are declared."
    );
    supported("Dream Trawler");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dream Trawler");
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(d, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.pt(d), (4, 5));
}

#[test]
fn watcher_in_the_water_untaps_itself_by_removing_a_stun_counter() {
    cr!("122.1d", "115.1");
    ruling!(
        "The Watcher in the Water",
        "For the last ability, the target Kraken can (and probably will) be The Watcher in the Water. If it is, and if it has at least one stun counter on it, you'll remove a stun counter from it instead of untapping it."
    );
    supported("The Watcher in the Water");
    let mut t = TestGame::new(2);
    let w = t.enter(P0, "The Watcher in the Water");
    t.settle();
    assert!(t.obj_now(w).tapped);
    assert_eq!(t.counters(w, "stun"), 9);
    // A Tentacle dies: untap the Watcher, a stun counter on P1's Bears.
    let bears = t.battlefield(P1, BEARS);
    t.set_step(P1, Step::PrecombatMain);
    draw(&mut t, P0, 1);
    t.resolve_all();
    let tentacle = with_subtype(&t, P0, "Tentacle")[0];
    t.answer_targets(P0, &[obj(w)]);
    t.answer_targets(P0, &[obj(bears)]);
    move_to(&mut t, tentacle, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.obj_now(w).tapped);
    assert_eq!(t.counters(w, "stun"), 8);
    assert_eq!(t.counters(bears, "stun"), 1);
}

#[test]
fn omen_machine_puts_a_land_onto_the_battlefield_or_must_cast_the_spell() {
    cr!("101.2", "504.1", "608.2g");
    ruling!(
        "Omen Machine",
        "Putting the land onto the battlefield or casting the card is not optional."
    );
    ruling!(
        "Omen Machine",
        "If the player is unable to put the land onto the battlefield (this is rare) or cast the spell (perhaps because there are no legal targets available) as the triggered ability of Omen Machine resolves, the card simply remains in exile."
    );
    supported("Omen Machine");
    // A land: onto the battlefield. A spell: cast. A spell with no legal target: stays
    // exiled.
    for (top, expect) in [
        ("Forest", Zone::Battlefield),
        ("Grizzly Bears", Zone::Battlefield),
        ("Lightning Bolt", Zone::Graveyard(P1)),
        ("Giant Growth", Zone::Exile),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Omen Machine");
        let c = t.library_top(P1, top);
        // Decline every optional choice: casting isn't one.
        for _ in 0..3 {
            t.answer_yes(P1, false);
        }
        t.set_step(P0, Step::End);
        t.advance_to(P1, Step::Draw);
        t.resolve_all();
        let now = t.g.current(c);
        assert_eq!(t.g.obj(now).zone, expect, "{top}");
        // Players can't draw cards: no card drawn this draw step.
        assert_eq!(drawn_this_turn(&t, P1), 0, "{top}");
    }
}
