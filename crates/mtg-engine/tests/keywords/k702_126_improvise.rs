//! CR 702.126 Improvise.

use crate::common_k702_125_139::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

fn tapped(t: &TestGame, ids: &[ObjectId]) -> usize {
    ids.iter().filter(|id| t.obj_now(**id).tapped).count()
}

fn improvise_prompts(t: &TestGame) -> Vec<u32> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { prompt, max, .. } if prompt.contains("improvise") => {
                Some(max)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn artifacts_can_be_tapped_to_pay_generic_mana() {
    cr!("702.126", "702.126a");
    ruling!(
        "Whir of Invention",
        "Improvise doesn't change a spell's mana cost or mana value."
    );
    assert_supported_card("Reverse Engineer");
    let mut t = TestGame::new(2);
    // Reverse Engineer: {3}{U}{U} sorcery, improvise, "Draw three cards."
    t.lands(P0, "Island", 2);
    let arts = battlefield_n(&mut t, P0, "Ornithopter", 3);
    let re = t.hand(P0, "Reverse Engineer");
    assert!(castable(&mut t, P0, re, NORMAL));
    t.answer_choose(
        P0,
        &arts.iter().map(|a| Entity::Object(*a)).collect::<Vec<_>>(),
    );
    let spell = t.cast(P0, re).go();
    assert_eq!(tapped(&t, &arts), 3);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 5);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn improvise_cant_pay_colored_mana() {
    cr!("702.126a");
    ruling!(
        "Whir of Invention",
        "Improvise can't pay for {W}, {U}, {B}, {R}, {G}, or {C} mana symbols in a spell's total cost."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    battlefield_n(&mut t, P0, "Ornithopter", 6);
    let re = t.hand(P0, "Reverse Engineer");
    assert!(!castable(&mut t, P0, re, NORMAL));
    t.lands(P0, "Island", 1);
    assert!(castable(&mut t, P0, re, NORMAL));
}

#[test]
fn tapped_artifacts_and_other_players_artifacts_cant_improvise() {
    cr!("702.126a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let mine = battlefield_n(&mut t, P0, "Ornithopter", 3);
    t.g.tap(mine[0]);
    battlefield_n(&mut t, P1, "Ornithopter", 3);
    let re = t.hand(P0, "Reverse Engineer");
    assert!(!castable(&mut t, P0, re, NORMAL));
    t.g.untap(mine[0]);
    assert!(castable(&mut t, P0, re, NORMAL));
}

#[test]
fn improvise_applies_after_the_total_cost_is_determined() {
    cr!("702.126b");
    ruling!(
        "Whir of Invention",
        "When using improvise to cast a spell with {X} in its mana cost, first choose the value for X."
    );
    assert_supported_card("Whir of Invention");
    let mut t = TestGame::new(2);
    // Whir of Invention {X}{U}{U}{U} with X=3: total {3}{U}{U}{U}; two artifacts pay {2}.
    t.lands(P0, "Island", 4);
    let arts = battlefield_n(&mut t, P0, "Ornithopter", 2);
    t.library_top(P0, "Memnite");
    let whir = t.hand(P0, "Whir of Invention");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_choose(
        P0,
        &arts.iter().map(|a| Entity::Object(*a)).collect::<Vec<_>>(),
    );
    t.cast(P0, whir).go();
    assert_eq!(tapped(&t, &arts), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
    // Only the generic part of the total cost can be paid: at most three artifacts.
    assert_eq!(improvise_prompts(&t), vec![2]);
}

#[test]
fn improvise_can_pay_a_cost_increase() {
    cr!("702.126b");
    ruling!(
        "Whir of Invention",
        "When calculating a spell's total cost, include any alternative costs, additional costs, or anything else that increases or reduces the cost to cast the spell. Improvise applies after the total cost is calculated."
    );
    let mut t = TestGame::new(2);
    // Thalia, Guardian of Thraben: noncreature spells cost {1} more.
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 2);
    let arts = battlefield_n(&mut t, P0, "Ornithopter", 3);
    let re = t.hand(P0, "Reverse Engineer");
    assert!(!castable(&mut t, P0, re, NORMAL));
    let more = t.battlefield(P0, "Ornithopter");
    assert!(castable(&mut t, P0, re, NORMAL));
    t.cast(P0, re).go();
    assert_eq!(tapped(&t, &[arts[0], arts[1], arts[2], more]), 4);
}

#[test]
fn multiple_instances_of_improvise_are_redundant() {
    cr!("702.126c");
    let mut def = custom_card(
        "Twice-Improvised Insight",
        "Sorcery",
        None,
        "Improvise\nImprovise\nDraw three cards.",
    );
    def.faces[0].chars.mana_cost = mtg_engine::mana::ManaCost::parse("{3}{U}{U}");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let arts = battlefield_n(&mut t, P0, "Ornithopter", 5);
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, s).go();
    // Offered once, for the three generic mana only.
    assert_eq!(improvise_prompts(&t), vec![3]);
    assert_eq!(tapped(&t, &arts), 3);
}
