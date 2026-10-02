//! Rulings batch P211 — entwine (CR 702.42): an entwined spell has all its modes, performed
//! in printed order as one resolution; each mode chooses its own targets; paying the
//! entwine cost doesn't change the spell's color.

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s07_common::chosen_modes;
use crate::r_s23_common::color_word_idx;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn twisted_reflections_modes_may_target_the_same_creature_or_two() {
    cr!("702.42a", "700.2", "115.3");
    ruling!(
        "Twisted Reflection",
        "If Twisted Reflection is entwined, its two modes may each target the same creature, or they may target two different creatures."
    );
    supported("Twisted Reflection");
    // "Choose one — • Target creature gets -6/-0 until end of turn. • Switch target
    // creature's power and toughness until end of turn. Entwine {B}"
    // The same Hill Giant (3/3): -6/-0 makes it -3/3, then switching makes it 3/-3.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let c = in_hand_with_mana(&mut t, P0, "Twisted Reflection");
    t.lands(P0, "Swamp", 1);
    t.cast(P0, c)
        .kicked(true)
        .target(giant)
        .target(giant)
        .go();
    let spell = t.g.current(c);
    let slots: Vec<Vec<Entity>> = t
        .g
        .obj(spell)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|m| m.targets.clone())
        .collect();
    assert_eq!(
        slots,
        vec![vec![Entity::Object(giant)], vec![Entity::Object(giant)]]
    );
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Two different creatures: Hill Giant gets -6/-0, Wall of Wood (0/3) switches to 3/0.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    let c = in_hand_with_mana(&mut t, P0, "Twisted Reflection");
    t.lands(P0, "Swamp", 1);
    t.cast(P0, c).kicked(true).target(giant).target(wall).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (-3, 3));
    assert!(t.in_graveyard(P1, "Wall of Wood"));
    assert!(!t.on_battlefield(wall));
}

#[test]
fn twisted_reflection_is_blue_even_when_entwined() {
    cr!("702.42a", "105.2", "202.2");
    ruling!(
        "Twisted Reflection",
        "Twisted Reflection is always a blue spell. It’s not also black if you paid its entwine cost."
    );
    supported("Twisted Reflection");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let c = in_hand_with_mana(&mut t, P0, "Twisted Reflection");
    t.lands(P0, "Swamp", 1);
    t.cast(P0, c)
        .kicked(true)
        .target(giant)
        .target(giant)
        .go();
    let spell = t.g.current(c);
    assert_eq!(chosen_modes(&t, spell), vec![0, 1]);
    t.g.recompute();
    let colors = t.g.obj(spell).chars.colors;
    assert!(colors.contains(Color::Blue));
    assert!(!colors.contains(Color::Black));
    assert_eq!(colors.count(), 1);
}

#[test]
fn kayas_guile_without_entwine_needs_two_different_modes() {
    cr!("700.2", "700.2d", "702.42a");
    ruling!(
        "Kaya's Guile",
        "If Kaya’s Guile isn’t entwined, you must choose two different modes."
    );
    supported("Kaya's Guile");
    // "Choose two — • Each opponent sacrifices a creature of their choice. • Exile all
    // opponents' graveyards. • Create a 1/1 white and black Spirit creature token with
    // flying. • You gain 4 life. Entwine {3}"
    let mut t = TestGame::new(2);
    let c = in_hand_with_mana(&mut t, P0, "Kaya's Guile");
    let from = t.asked().len();
    // Choosing "You gain 4 life" twice isn't allowed: the answer is rejected and two
    // different modes are chosen instead.
    t.cast(P0, c).kicked(false).modes(&[3, 3]).go();
    let spell = t.g.current(c);
    let asked: Vec<(u32, u32, bool)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes {
                min,
                max,
                allow_repeat,
                ..
            } => Some((*min, *max, *allow_repeat)),
            _ => None,
        })
        .collect();
    assert_eq!(asked, vec![(2, 2, false)]);
    let modes = chosen_modes(&t, spell);
    assert_eq!(modes.len(), 2);
    assert_ne!(modes[0], modes[1]);
    t.resolve_all();
    assert!(t.life(P0) <= 24);
    // Two different modes are fine: a Spirit and 4 life.
    let mut t = TestGame::new(2);
    let c = in_hand_with_mana(&mut t, P0, "Kaya's Guile");
    t.cast(P0, c).kicked(false).modes(&[2, 3]).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 1);
}

#[test]
fn spectral_shifts_modes_may_target_the_same_permanent() {
    cr!("702.42a", "115.3", "612.2");
    ruling!(
        "Spectral Shift",
        "Because of updated targeting rulings, it’s possible to target the same spell or permanent with both abilities when cast with Entwine."
    );
    supported("Spectral Shift");
    supported("Mountain Yeti");
    // Mountain Yeti: "Mountainwalk. Protection from white." Both modes target it:
    // Mountain becomes Island, then white becomes black.
    let mut t = TestGame::new(2);
    let yeti = t.battlefield(P1, "Mountain Yeti");
    let c = in_hand_with_mana(&mut t, P0, "Spectral Shift");
    t.lands(P0, "Wastes", 2);
    t.cast(P0, c).kicked(true).target(yeti).target(yeti).go();
    // Land type words: Plains, Island, Swamp, Mountain, Forest.
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(color_word_idx(Color::White, None)),
    );
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(color_word_idx(Color::Black, Some(Color::White))),
    );
    t.resolve_all();
    let kws: Vec<String> = t
        .obj_now(yeti)
        .chars
        .keywords()
        .map(|k| format!("{:?}", k))
        .collect();
    let text = kws.join(" ");
    assert!(
        t.obj_now(yeti)
            .chars
            .keywords()
            .any(|k| k.kind == KeywordKind::Protection && format!("{:?}", k.filter).contains("Black")),
        "{text}"
    );
    assert!(text.contains("Island"), "{text}");
    assert!(!text.contains("Mountain"), "{text}");
}

#[test]
fn mirage_mockerys_tokens_enter_one_after_the_other() {
    cr!("702.42a", "700.2", "608.2c", "614.12");
    ruling!(
        "Mirage Mockery",
        "If you cast the spell by paying its entwine cost, the tokens don't enter the battlefield at the same time. The token that's a copy of the artifact creature enters the battlefield first, followed by the token that's a copy of the nonartifact creature. Then any abilities that triggered due to either of them entering the battlefield are put on the stack."
    );
    supported("Mirage Mockery");
    supported("Luxknight Breacher");
    supported("Soul Warden");
    // Luxknight Breacher: "This creature enters with a +1/+1 counter on it for each other
    // creature and/or artifact you control." Its copy counts the Ornithopter copy, which
    // entered first: Ornithopter, Breacher, Soul Warden and the Ornithopter token.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let breacher = t.battlefield(P0, "Luxknight Breacher");
    t.battlefield(P0, "Soul Warden");
    let c = in_hand_with_mana(&mut t, P0, "Mirage Mockery");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.cast(P0, c)
        .kicked(true)
        .target(thopter)
        .target(breacher)
        .go();
    t.g.resolve_top();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    let copy = toks
        .iter()
        .copied()
        .find(|id| t.obj_now(*id).chars.name == "Luxknight Breacher")
        .unwrap();
    assert_eq!(t.counters(copy, counters::PLUS1), 4);
    // Soul Warden's triggers for both go on the stack only then.
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "gain 1 life"), 2);
}

#[test]
fn unbounded_potential_entwined_proliferates_the_counters_it_put() {
    cr!("702.42a", "701.34a", "700.2");
    ruling!(
        "Unbounded Potential",
        "If you cast this spell by paying its entwine cost, you will be able to proliferate the counters that you placed with the first mode."
    );
    supported("Unbounded Potential");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = in_hand_with_mana(&mut t, P0, "Unbounded Potential");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    t.cast(P0, c)
        .kicked(true)
        .targets(&[Entity::Object(bears)])
        .go();
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn promise_of_power_entwined_counts_the_drawn_cards() {
    cr!("702.42a", "700.2", "608.2c");
    ruling!(
        "Promise of Power",
        "If you pay the entwine cost, you draw five cards, then lose five life, then put the Demon token onto the battlefield. The five cards you draw count toward the Demon’s power and toughness."
    );
    supported("Promise of Power");
    let mut t = TestGame::new(2);
    let c = in_hand_with_mana(&mut t, P0, "Promise of Power");
    t.lands(P0, "Wastes", 4);
    assert_eq!(t.hand_size(P0), 1);
    t.cast(P0, c).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 5);
    assert_eq!(t.life(P0), 15);
    let demons = with_subtype(&t, P0, "Demon");
    assert_eq!(demons.len(), 1);
    assert_eq!(t.pt(demons[0]), (5, 5));
}
