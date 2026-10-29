//! Rulings batch P156 — "whenever you draw your second card each turn" and "as long as
//! you've drawn two or more cards this turn": every card drawn this turn counts (whoever
//! controlled the permanent then), only actual draws count, the trigger happens once each
//! turn and only if the permanent is there for the second draw, and a multiple draw
//! finishes before the trigger's target is chosen.

use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::put_counters;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn draw(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
}

/// A card put into `p`'s hand from the library without being drawn.
fn put_in_hand(t: &mut TestGame, p: PlayerId) {
    let c = t.g.player(p).library.last().copied().unwrap();
    move_to(t, c, Zone::Hand(p));
}

const TRIGGER: &str = "second card";

#[test]
fn second_draw_trigger_needs_only_your_control_at_the_second_draw() {
    cr!("603.2", "121.2");
    ruling!(
        "Erudite Wizard",
        "Erudite Wizard doesn't need to have been under your control when the first card is drawn for its ability to trigger. As long as you control it when you draw your second card in a turn, that ability will trigger."
    );
    ruling!(
        "Mischievous Mystic",
        "Mischievous Mystic doesn't need to have been under your control when the first card is drawn for its ability to trigger. As long as you control it when you draw your second card in a turn, that ability will trigger."
    );
    for name in ["Erudite Wizard", "Mischievous Mystic"] {
        supported(name);
        // P1 controls it as P0 draws their first card; P0 takes it, then draws again.
        let mut t = TestGame::new(2);
        let perm = t.battlefield(P1, name);
        draw(&mut t, P0, 1);
        crate::r_p147_common::take_control(&mut t, P0, perm);
        assert_eq!(t.obj_now(perm).controller, P0);
        draw(&mut t, P0, 1);
        assert_eq!(triggers_on_stack(&t, TRIGGER), 1, "{name}");
        t.resolve_all();
        assert!(
            t.counters(perm, "+1/+1") == 1 || with_subtype(&t, P0, "Faerie").len() == 1,
            "{name}: no effect"
        );
    }
}

#[test]
fn only_cards_drawn_count_and_draws_before_it_entered_count() {
    cr!("121.1", "121.2", "611.3a");
    ruling!(
        "Messenger Hawk",
        "If a spell or ability causes you to put cards into your hand without specifically using the word \"draw,\" Messenger Hawk's ability doesn't count them."
    );
    ruling!(
        "Spinehorn Minotaur",
        "If a spell or ability causes you to put cards into your hand without specifically using the word “draw,” Spinehorn Minotaur’s effect doesn’t count them."
    );
    ruling!(
        "Messenger Hawk",
        "Messenger Hawk's last ability applies even if you drew two or more cards only before Messenger Hawk entered."
    );
    ruling!(
        "Spinehorn Minotaur",
        "Spinehorn Minotaur’s effect applies even if you drew two or more cards only before Spinehorn Minotaur entered the battlefield."
    );
    supported("Messenger Hawk");
    supported("Spinehorn Minotaur");
    // One draw plus one card put into hand: not two draws.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Messenger Hawk");
    let mino = t.battlefield(P0, "Spinehorn Minotaur");
    draw(&mut t, P0, 1);
    put_in_hand(&mut t, P0);
    t.g.recompute();
    assert_eq!(t.pt(hawk), (1, 2));
    assert!(!t.obj_now(mino).has_keyword(KeywordKind::DoubleStrike));
    draw(&mut t, P0, 1);
    t.g.recompute();
    assert_eq!(t.pt(hawk), (3, 2));
    assert!(t.obj_now(mino).has_keyword(KeywordKind::DoubleStrike));
    // Two draws before they entered.
    let mut t = TestGame::new(2);
    draw(&mut t, P0, 2);
    let hawk = t.battlefield(P0, "Messenger Hawk");
    let mino = t.battlefield(P0, "Spinehorn Minotaur");
    t.g.recompute();
    assert_eq!(t.pt(hawk), (3, 2));
    assert!(t.obj_now(mino).has_keyword(KeywordKind::DoubleStrike));
}

#[test]
fn spinehorn_minotaur_gaining_double_strike_after_combat_damage_deals_no_more() {
    cr!("510.4", "702.4b");
    ruling!(
        "Spinehorn Minotaur",
        "If Spinehorn Minotaur gains double strike only after combat damage has already been dealt (most likely because another creature’s ability caused you to draw a card when it dealt combat damage), Spinehorn Minotaur will deal only normal combat damage."
    );
    supported("Spinehorn Minotaur");
    let mut t = TestGame::new(2);
    let mino = t.battlefield(P0, "Spinehorn Minotaur");
    draw(&mut t, P0, 1);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(mino, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(t.life(P1), 18);
    // The second draw after combat damage: double strike, but no more damage this combat.
    draw(&mut t, P0, 1);
    t.g.recompute();
    assert!(t.obj_now(mino).has_keyword(KeywordKind::DoubleStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    // Positive control: with two cards drawn before combat, it deals damage twice.
    let mut t = TestGame::new(2);
    let mino = t.battlefield(P0, "Spinehorn Minotaur");
    draw(&mut t, P0, 2);
    t.attack(&[(mino, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn a_multiple_draw_finishes_before_the_second_draw_trigger_targets() {
    cr!("603.3b", "603.3d", "121.2");
    ruling!(
        "Irencrag Pyromancer",
        "If an effect instructs you to draw multiple cards, the ability triggers after you draw whichever is the second one for the turn. You choose a target (if any) for the ability after you've drawn and looked at all of the cards and finished resolving the spell or ability that caused you to draw them."
    );
    ruling!(
        "Mad Ratter",
        "If an effect instructs you to draw multiple cards, the ability triggers after you draw whichever is the second one for the turn. You choose a target (if any) for the ability after you've drawn and looked at all of the cards and finished resolving the spell or ability that caused you to draw them."
    );
    supported("Irencrag Pyromancer");
    supported("Mad Ratter");
    supported("Divination");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Irencrag Pyromancer");
    t.battlefield(P0, "Mad Ratter");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| (g.player(P0).hand.len(), g.stack.len()),
    );
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // Divination: "Draw two cards." The first two cards drawn this turn.
    cast_new(&mut t, P0, "Divination", &[]);
    t.resolve();
    // Both cards drawn, Divination gone, then the triggers are put on the stack.
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(triggers_on_stack(&t, TRIGGER), 2);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.first(), Some(&(hand + 2, 1)), "{seen:?}");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(with_subtype(&t, P0, "Rat").len(), 2);
    // Drawing three more doesn't trigger them again.
    draw(&mut t, P0, 3);
    assert_eq!(triggers_on_stack(&t, TRIGGER), 0);
}

#[test]
fn second_draw_trigger_once_per_turn_and_only_if_there_for_the_second_card() {
    cr!("603.2", "121.2");
    ruling!(
        "Alandra, Sky Dreamer",
        "The first triggered ability can trigger only once each turn. It doesn't matter whether Alandra, Sky Dreamer was on the battlefield when the first card was drawn. If it's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn. It won't trigger when the third or fourth card is drawn."
    );
    ruling!(
        "Knowledge Seeker",
        "The first triggered ability can trigger only once each turn. It doesn't matter whether Knowledge Seeker was on the battlefield when the first card was drawn. If it's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn. It won't trigger when the third or fourth card is drawn."
    );
    ruling!(
        "Foggy Swamp Spirit Keeper",
        "The triggered ability can trigger only once each turn. It doesn't matter whether Foggy Swamp Spirit Keeper was on the battlefield when the first card was drawn. If it's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn. It won't trigger when the third or fourth card is drawn."
    );
    ruling!(
        "Lakeshore Apothecary",
        "The triggered ability can trigger only once each turn. It doesn't matter whether Lakeshore Apothecary was on the battlefield when the first card was drawn. If it's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn. It won't trigger when the third or fourth card is drawn."
    );
    ruling!(
        "Alandra, Sky Dreamer",
        "The same is true for the last ability and the fifth card drawn."
    );
    for name in [
        "Alandra, Sky Dreamer",
        "Knowledge Seeker",
        "Foggy Swamp Spirit Keeper",
        "Lakeshore Apothecary",
    ] {
        supported(name);
        // Not there for the first card; there for the second; not again for the 3rd/4th.
        let mut t = TestGame::new(2);
        draw(&mut t, P0, 1);
        t.battlefield(P0, name);
        draw(&mut t, P0, 1);
        assert_eq!(triggers_on_stack(&t, TRIGGER), 1, "{name}");
        t.resolve_all();
        draw(&mut t, P0, 1);
        draw(&mut t, P0, 1);
        assert_eq!(triggers_on_stack(&t, TRIGGER), 0, "{name}: 3rd/4th");
        // Not there for the second card: not at all that turn.
        let mut t = TestGame::new(2);
        draw(&mut t, P0, 2);
        t.battlefield(P0, name);
        draw(&mut t, P0, 1);
        draw(&mut t, P0, 1);
        assert_eq!(triggers_on_stack(&t, TRIGGER), 0, "{name}: entered after");
    }
    // Alandra's fifth-card ability: there for the fifth card (not the first four), once.
    let mut t = TestGame::new(2);
    draw(&mut t, P0, 4);
    t.battlefield(P0, "Alandra, Sky Dreamer");
    draw(&mut t, P0, 1);
    assert_eq!(triggers_on_stack(&t, "fifth card"), 1);
    t.resolve_all();
    draw(&mut t, P0, 1);
    assert_eq!(triggers_on_stack(&t, "fifth card"), 0);
    // Not there for the fifth: never that turn.
    let mut t = TestGame::new(2);
    draw(&mut t, P0, 5);
    t.battlefield(P0, "Alandra, Sky Dreamer");
    draw(&mut t, P0, 1);
    assert_eq!(triggers_on_stack(&t, "fifth card"), 0);
}

#[test]
fn alandras_bonus_is_locked_in_as_it_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Alandra, Sky Dreamer",
        "The value of X is locked in as the last ability resolves. The bonus it grants won't change after that point, even if the number of cards in your hand does."
    );
    supported("Alandra, Sky Dreamer");
    let mut t = TestGame::new(2);
    let alandra = t.battlefield(P0, "Alandra, Sky Dreamer");
    draw(&mut t, P0, 5);
    t.resolve_all();
    let hand = t.hand_size(P0) as i32;
    assert_eq!(hand, 5);
    assert_eq!(t.pt(alandra), (2 + hand, 4 + hand));
    let c = t.g.player(P0).hand[0];
    move_to(&mut t, c, Zone::Graveyard(P0));
    t.g.recompute();
    assert_eq!(t.pt(alandra), (2 + hand, 4 + hand));
}

#[test]
fn lady_octopus_casts_the_spell_as_the_ability_resolves_or_not_at_all() {
    cr!("608.2g", "601.2");
    ruling!(
        "Lady Octopus, Inspired Inventor",
        "The spell is cast as Lady Octopus's last ability resolves. You can't wait and cast it later."
    );
    ruling!(
        "Lady Octopus, Inspired Inventor",
        "You don't have to cast a spell as that ability resolves. If you choose not to for some reason, don't let anyone judge your genius plans."
    );
    supported("Lady Octopus, Inspired Inventor");
    // Mind Stone: a mana value 2 artifact.
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        let octo = t.battlefield(P0, "Lady Octopus, Inspired Inventor");
        put_counters(&mut t, octo, "ingenuity", 2);
        let stone = t.hand(P0, "Mind Stone");
        t.answer_yes(P0, cast);
        t.answer_choose(P0, &[Entity::Object(stone)]);
        let uid = crate::r_s27_common::ability_containing(&mut t, octo, "cast");
        t.g.turn.priority = Some(P0);
        assert!(t.g.activate_ability(P0, octo, uid).is_ok());
        t.g.flush_events();
        t.resolve();
        if cast {
            // Cast during the resolution: on the stack now, without mana paid.
            assert_eq!(t.zone(stone), Zone::Stack);
            t.resolve_all();
            assert!(t.on_battlefield(stone));
        } else {
            assert_eq!(t.zone(stone), Zone::Hand(P0));
            assert_eq!(t.stack_len(), 0);
            // It can't be cast for free later: with no mana, casting fails.
            t.g.turn.priority = Some(P0);
            assert!(t.g.cast_spell(P0, stone, mtg_engine::object::CastMethod::Normal).is_err());
        }
    }
}
