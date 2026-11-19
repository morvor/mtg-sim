//! Rulings batch P170 — cost changes (CR 601.2f, 118.7, 202.3): the total cost is the mana
//! cost or alternative cost plus increases, then minus reductions; generic reductions
//! can't reduce colored mana; the mana value never changes.

use crate::r_p033_common::{assert_spent, cast_normal, cast_paying, main_phase, Paid};
use crate::r_p170_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

fn flashback() -> CastMethod {
    CastMethod::Keyword(KeywordKind::Flashback)
}

/// `p` casts the card `name` from hand, paying from a full pool; returns what it cost.
fn cast_from_hand(t: &mut TestGame, p: PlayerId, name: &str) -> Paid {
    let card = t.hand(p, name);
    cast_normal(t, p, card)
}

#[test]
fn stormwing_entity_reduction_applies_once_even_if_the_spell_was_countered() {
    cr!("601.2f", "118.7", "202.3");
    ruling!(
        "Stormwing Entity",
        "Stormwing Entity's cost reduction applies even if the instant or sorcery spell you cast was countered or didn't resolve."
    );
    ruling!(
        "Stormwing Entity",
        "Stormwing Entity's cost reduction applies just once, no matter how many instant and sorcery spells you've cast this turn."
    );
    ruling!(
        "Stormwing Entity",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Stormwing Entity). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Stormwing Entity");
    // No instant or sorcery yet: {3}{U}{U}.
    let mut t = main_phase(2);
    let paid = cast_from_hand(&mut t, P0, "Stormwing Entity");
    assert_spent(&paid.spent, "{3}{U}{U}", "nothing cast");
    // An Opt that was countered: {1}{U}, mana value 5.
    let mut t = main_phase(2);
    let opt = cast_from_hand(&mut t, P0, "Opt").spell;
    assert!(t.g.counter(opt, None));
    t.resolve_all();
    let paid = cast_from_hand(&mut t, P0, "Stormwing Entity");
    assert_spent(&paid.spent, "{1}{U}", "after a countered Opt");
    assert_eq!(paid.mv, 5);
    // Three instants: still {1}{U}. With Sphere of Resistance: {2}{U}.
    let mut t = main_phase(2);
    for _ in 0..3 {
        cast_from_hand(&mut t, P0, "Opt");
        t.resolve_all();
    }
    let paid = cast_from_hand(&mut t, P0, "Stormwing Entity");
    assert_spent(&paid.spent, "{1}{U}", "after three Opts");
    let mut t = main_phase(2);
    t.battlefield(P1, "Sphere of Resistance");
    cast_from_hand(&mut t, P0, "Opt");
    t.resolve_all();
    let paid = cast_from_hand(&mut t, P0, "Stormwing Entity");
    assert_spent(&paid.spent, "{2}{U}", "with Sphere of Resistance");
    assert_eq!(paid.mv, 5);
}

#[test]
fn graveyard_count_reductions_apply_after_increases_and_not_to_colored_mana() {
    cr!("601.2f", "118.7", "118.7a", "202.3");
    ruling!(
        "Serpent of the Pass",
        "Serpent of the Pass's second ability cannot reduce the {U}{U} in its cost."
    );
    ruling!(
        "Serpent of the Pass",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Serpent of the Pass's second ability). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast that spell was."
    );
    ruling!(
        "Bedlam Reveler",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying, add any cost increases, then apply any cost reductions (such as that of Bedlam Reveler’s first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Serpent of the Pass");
    supported("Bedlam Reveler");
    // (card, graveyard card, how many, Sphere of Resistance, total cost)
    let cases: &[(&str, &str, usize, bool, &str)] = &[
        // Ten noncreature, nonland cards: {5} is gone, {U}{U} stays.
        ("Serpent of the Pass", "Divination", 10, false, "{U}{U}"),
        // Sphere's {1} is added first: {6}{U}{U} minus 6.
        ("Serpent of the Pass", "Divination", 6, true, "{U}{U}"),
        ("Serpent of the Pass", "Divination", 5, true, "{1}{U}{U}"),
        ("Bedlam Reveler", "Opt", 7, true, "{R}{R}"),
        ("Bedlam Reveler", "Opt", 6, true, "{1}{R}{R}"),
        ("Bedlam Reveler", "Opt", 9, false, "{R}{R}"),
    ];
    for &(card, gy, n, sphere, cost) in cases {
        let mut t = main_phase(2);
        for _ in 0..n {
            t.graveyard(P0, gy);
        }
        if sphere {
            t.battlefield(P1, "Sphere of Resistance");
        }
        let paid = cast_from_hand(&mut t, P0, card);
        let what = format!("{card} with {n} cards, sphere {sphere}");
        assert_spent(&paid.spent, cost, &what);
        assert_eq!(
            paid.mv,
            crate::r_p033_common::printed_mv(card),
            "{what}: mana value"
        );
    }
}

/// The index of the option named `name` that Stenn offers as it enters.
fn stenn_option(name: &str) -> usize {
    let mut t = TestGame::new(2);
    t.enter(P0, "Stenn, Paranoid Partisan");
    t.asked()
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => options
                .iter()
                .position(|o| o.eq_ignore_ascii_case(name)),
            _ => None,
        })
        .expect("Stenn offers the card type")
}

#[test]
fn noncreature_reducers_reduce_the_total_cost_including_flashback_and_kicker() {
    cr!("601.2f", "118.7", "118.7a", "118.9", "202.3", "702.34a");
    ruling!(
        "Lyse Hext",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of Lyse Hext's second ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    ruling!(
        "Stenn, Paranoid Partisan",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of Stenn's second ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    ruling!(
        "Stenn, Paranoid Partisan",
        "The cost reduction applies only to generic mana in the cost of spells you cast of the chosen card type."
    );
    ruling!(
        "Stormcatch Mentor",
        "The cost reduction applies only to generic mana in the total cost of instant and sorcery spells you cast."
    );
    supported("Lyse Hext");
    supported("Stenn, Paranoid Partisan");
    supported("Stormcatch Mentor");
    let instant = stenn_option("instant");
    for reducer in ["Lyse Hext", "Stenn, Paranoid Partisan", "Stormcatch Mentor"] {
        let setup = |t: &mut TestGame| {
            if reducer.starts_with("Stenn") {
                t.answer(P0, DecisionKind::Option, Answer::Index(instant));
                t.enter(P0, reducer);
            } else {
                t.battlefield(P0, reducer);
            }
        };
        // Think Twice's flashback cost {2}{U}: {1}{U}; mana value 2.
        let mut t = main_phase(2);
        setup(&mut t);
        let tt = t.graveyard(P0, "Think Twice");
        let paid = cast_paying(&mut t, P0, tt, flashback()).expect("flashback");
        assert_spent(&paid.spent, "{1}{U}", reducer);
        assert_eq!(paid.mv, 2, "{reducer}");
        // Opt ({U}): only colored mana, nothing to reduce.
        let mut t = main_phase(2);
        setup(&mut t);
        let paid = cast_from_hand(&mut t, P0, "Opt");
        assert_spent(&paid.spent, "{U}", reducer);
        // Hieroglyphic Illumination (an instant, {3}{U}): {2}{U}.
        let mut t = main_phase(2);
        setup(&mut t);
        let paid = cast_from_hand(&mut t, P0, "Hieroglyphic Illumination");
        assert_spent(&paid.spent, "{2}{U}", reducer);
    }
    // Kicker is added before Lyse Hext's reduction: Gift of Growth ({1}{G}, kicker {2})
    // kicked costs {2}{G}; mana value 2.
    let mut t = main_phase(2);
    t.battlefield(P0, "Lyse Hext");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let paid = cast_from_hand(&mut t, P0, "Gift of Growth");
    assert_spent(&paid.spent, "{2}{G}", "kicked Gift of Growth");
    assert_eq!(paid.mv, 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn kasmina_adds_two_once_however_many_of_your_permanents_are_targeted() {
    cr!("601.2f", "118.7", "202.3");
    ruling!(
        "Kasmina, Enigmatic Mentor",
        "Spells that target more than one creature and/or planeswalker you control cost only {2} more to cast."
    );
    ruling!(
        "Kasmina, Enigmatic Mentor",
        "To determine the total cost of an opponent's spell that targets a creature or planeswalker you control, start with the mana cost or alternative cost that player is paying, add any cost increases (such as that of Kasmina's effect), then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Kasmina, Enigmatic Mentor");
    // (P1's reducer, Electrolyze's targets: two of P0's creatures or one, total cost)
    let cases: &[(bool, bool, &str)] = &[
        (false, true, "{3}{U}{R}"),
        (false, false, "{3}{U}{R}"),
        // Stormcatch Mentor's {1} less applies after Kasmina's {2} more.
        (true, true, "{2}{U}{R}"),
    ];
    for &(mentor, two, cost) in cases {
        let mut t = main_phase(2);
        t.battlefield(P0, "Kasmina, Enigmatic Mentor");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        if mentor {
            t.battlefield(P1, "Stormcatch Mentor");
        }
        if two {
            t.answer_targets(P1, &[obj(a), obj(b)]);
            t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
        } else {
            t.answer_targets(P1, &[obj(a)]);
        }
        let card = t.hand(P1, "Electrolyze");
        let paid = cast_paying(&mut t, P1, card, CastMethod::Normal).expect("cast");
        assert_spent(&paid.spent, cost, &format!("mentor {mentor}, two {two}"));
        assert_eq!(paid.mv, 3);
    }
    // P1's spell targeting only P1's own creature: no increase.
    let mut t = main_phase(2);
    t.battlefield(P0, "Kasmina, Enigmatic Mentor");
    let own = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P1, &[obj(own)]);
    let card = t.hand(P1, "Electrolyze");
    let paid = cast_paying(&mut t, P1, card, CastMethod::Normal).expect("cast");
    assert_spent(&paid.spent, "{1}{U}{R}", "targeting P1's creature");
}
