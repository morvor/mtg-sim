//! Rulings batch P033 — generic cost reductions (CR 601.2f, 118.7a): "Instant and sorcery
//! spells you cast cost {1} less to cast" reduces only the generic mana of the total cost,
//! never its colored requirements, and leaves the spell's mana cost and mana value alone
//! (CR 202.3).

use crate::r_p033_common::*;
use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts `name` from hand (paying from a full pool), asserts it cost `cost` and that its
/// mana value on the stack is its printed one, and lets it resolve.
fn costs(t: &mut TestGame, p: PlayerId, name: &str, cost: &str, what: &str) {
    let card = t.hand(p, name);
    let paid = cast_normal(t, p, card);
    assert_spent(&paid.spent, cost, &format!("{what}: {name}"));
    assert_eq!(paid.mv, printed_mv(name), "{what}: {name}'s mana value");
    t.resolve_all();
}

/// The reducers that always apply to the spells of the given kind: each reduces a spell's
/// generic mana and can't reduce a spell with only colored mana in its cost.
#[test]
fn static_generic_reducers_leave_colored_mana_and_mana_values_alone() {
    cr!("601.2f", "118.7a", "202.3");
    ruling!(
        "Marauding Raptor",
        "Effects that reduce the generic mana cost of a spell (such as that of Marauding Raptor) can't reduce that spell's colored mana requirements."
    );
    ruling!(
        "Heartless Summoning",
        "Heartless Summoning’s first ability can only reduce the generic mana portion of a creature spell’s cost."
    );
    ruling!(
        "Starnheim Aspirant",
        "The cost reduction applies only to generic mana in the costs of Angel spells you cast. It can't reduce requirements of specific colors of mana."
    );
    ruling!(
        "Herald of the Pantheon",
        "Herald of the Pantheon’s first ability can’t reduce the colored mana requirement of an enchantment spell."
    );
    ruling!(
        "Arcane Melee",
        "Arcane Melee can’t reduce the colored mana requirement of an instant or sorcery spell."
    );
    ruling!(
        "Archmage of Runes",
        "Archmage of Runes's first ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that spell."
    );
    ruling!(
        "Archmage of Runes",
        "Archmage of Runes's first ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay to cast instant and sorcery spells."
    );
    ruling!(
        "Urza, Lord Protector",
        "Urza, Lord Protector's first ability can't reduce the colored mana costs of spells you cast."
    );
    ruling!(
        "Goblin Electromancer",
        "Goblin Electromancer's effect reduces only generic mana in the spell's total cost."
    );
    ruling!(
        "Mocking Sprite",
        "Mocking Sprite's last ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that cost."
    );
    ruling!(
        "Mocking Sprite",
        "Mocking Sprite's last ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay."
    );
    // (reducer, a spell with generic mana and what it costs, a spell with only colored
    // mana and what it costs)
    let cases: &[(&str, (&str, &str), (&str, &str))] = &[
        // "Creature spells you cast cost {1} less to cast."
        ("Marauding Raptor", ("Grizzly Bears", "{G}"), ("Kird Ape", "{R}")),
        // "Creature spells you cast cost {2} less to cast."
        ("Heartless Summoning", ("Craw Wurm", "{2}{G}{G}"), ("Kird Ape", "{R}")),
        // "Angel spells you cast cost {2} less to cast." Youthful Valkyrie ({1}{W}) has
        // only {1} to reduce.
        ("Starnheim Aspirant", ("Serra Angel", "{1}{W}{W}"), ("Youthful Valkyrie", "{W}")),
        // "Enchantment spells you cast cost {1} less to cast."
        ("Herald of the Pantheon", ("Glorious Anthem", "{W}{W}"), ("Seal of Fire", "{R}")),
        // "Instant and sorcery spells cost {2} less to cast."
        ("Arcane Melee", ("Divination", "{U}"), ("Opt", "{U}")),
        // "Instant and sorcery spells you cast cost {1} less to cast."
        ("Archmage of Runes", ("Divination", "{1}{U}"), ("Opt", "{U}")),
        ("Goblin Electromancer", ("Divination", "{1}{U}"), ("Opt", "{U}")),
        ("Mocking Sprite", ("Divination", "{1}{U}"), ("Opt", "{U}")),
        // "Artifact, instant, and sorcery spells you cast cost {1} less to cast."
        ("Urza, Lord Protector", ("Juggernaut", "{3}"), ("Opt", "{U}")),
    ];
    for (reducer, (a, a_cost), (b, b_cost)) in cases {
        supported(reducer);
        let mut t = main_phase(2);
        t.battlefield(P0, reducer);
        costs(&mut t, P0, a, a_cost, reducer);
        costs(&mut t, P0, b, b_cost, reducer);
        // Without it, the first spell costs its mana cost (a positive control).
        let mut t = main_phase(2);
        let card = t.hand(P0, a);
        let full = cast_normal(&mut t, P0, card);
        assert_eq!(full.spent.len() as i64, printed_mv(a), "{a} alone");
    }
}

#[test]
fn arcane_melee_reduces_one_generic_mana_to_nothing() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Arcane Melee",
        "The cost to cast an instant or sorcery spell with one generic mana in its mana cost will be reduced by {1}. For example, an instant spell with a mana cost of {1}{G} would cost {G} to cast."
    );
    // Naturalize ({1}{G} instant): {G}.
    let mut t = main_phase(2);
    t.battlefield(P0, "Arcane Melee");
    let stone = t.battlefield(P1, "Mind Stone");
    t.answer_targets(P0, &[obj(stone)]);
    let card = t.hand(P0, "Naturalize");
    let paid = cast_normal(&mut t, P0, card);
    assert_spent(&paid.spent, "{G}", "Naturalize");
    t.resolve_all();
    assert!(!t.on_battlefield(stone));
}

#[test]
fn two_goblin_electromancers_reduce_by_two() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Goblin Electromancer",
        "Two Goblin Electromancers will make instant and sorcery spells you cast cost {2} less to cast, and so on."
    );
    // Hieroglyphic Illumination ({3}{U} instant): {1}{U} with two, {U} with three; a
    // sorcery too (Divination {2}{U}: {U} with two).
    for (n, cost) in [(1, "{2}{U}"), (2, "{1}{U}"), (3, "{U}")] {
        let mut t = main_phase(2);
        for _ in 0..n {
            t.battlefield(P0, "Goblin Electromancer");
        }
        costs(&mut t, P0, "Hieroglyphic Illumination", cost, &format!("{n} Electromancers"));
    }
    let mut t = main_phase(2);
    t.battlefield(P0, "Goblin Electromancer");
    t.battlefield(P0, "Goblin Electromancer");
    costs(&mut t, P0, "Divination", "{U}", "two Electromancers");
}

#[test]
fn planar_gate_and_mana_matrix_are_cumulative_and_only_for_their_controller() {
    cr!("601.2f", "118.7a", "109.5");
    ruling!(
        "Planar Gate",
        "Multiple Gates are cumulative. Two will reduce the cost by up to {4}, and so on."
    );
    ruling!("Planar Gate", "Only works for its controller, not all players.");
    ruling!(
        "Planar Gate",
        "Works on all spells with the type Creature, even if the spell has another type like Artifact or Enchantment."
    );
    ruling!(
        "Mana Matrix",
        "Multiple Matrices are cumulative. Two will reduce the cost by up to {4}, and so on."
    );
    ruling!("Mana Matrix", "Only affects its controller, not all players.");
    ruling!(
        "Mana Matrix",
        "Only works on instants and enchantments. It does not work on sorceries."
    );
    supported("Planar Gate");
    supported("Mana Matrix");
    // Planar Gate: "Creature spells you cast cost {2} less to cast."
    let mut t = main_phase(2);
    t.battlefield(P0, "Planar Gate");
    t.battlefield(P0, "Planar Gate");
    // Craw Wurm ({4}{G}{G}): {4} less; Grizzly Bears ({1}{G}): "up to" — only its {1}.
    costs(&mut t, P0, "Craw Wurm", "{G}{G}", "two Gates");
    costs(&mut t, P0, "Grizzly Bears", "{G}", "two Gates");
    // An artifact creature and an enchantment creature are creature spells.
    let mut t = main_phase(2);
    t.battlefield(P0, "Planar Gate");
    costs(&mut t, P0, "Juggernaut", "{2}", "Gate");
    costs(&mut t, P0, "Gravebreaker Lamia", "{2}{B}", "Gate");
    // Not a creature: full cost.
    costs(&mut t, P0, "Mind Stone", "{2}", "Gate");
    // P1's creature spells cost what they cost.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    costs(&mut t, P1, "Craw Wurm", "{4}{G}{G}", "P0's Gate, P1's spell");

    // Mana Matrix: "Instant and enchantment spells you cast cost {2} less to cast."
    let mut t = main_phase(2);
    t.battlefield(P0, "Mana Matrix");
    t.battlefield(P0, "Mana Matrix");
    let target = Entity::Player(P0);
    // Opportunity ({4}{U}{U} instant): {U}{U}; Inspiration ({3}{U}): up to {4} — {U}.
    t.answer_targets(P0, &[target]);
    costs(&mut t, P0, "Opportunity", "{U}{U}", "two Matrices");
    t.answer_targets(P0, &[target]);
    costs(&mut t, P0, "Inspiration", "{U}", "two Matrices");
    // An enchantment: Glorious Anthem ({1}{W}{W}).
    costs(&mut t, P0, "Glorious Anthem", "{W}{W}", "two Matrices");
    // Not a sorcery: Divination ({2}{U}) costs {2}{U}.
    costs(&mut t, P0, "Divination", "{2}{U}", "two Matrices");
    // Nor P1's instants.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.answer_targets(P1, &[Entity::Player(P1)]);
    costs(&mut t, P1, "Inspiration", "{3}{U}", "P0's Matrices, P1's spell");
}

#[test]
fn cloud_key_reduces_only_generic_mana_of_the_chosen_type() {
    cr!("601.2f", "118.7a", "607.2d");
    ruling!(
        "Cloud Key",
        "The cost reduction applies only to generic mana in the cost of spells of the chosen type you cast."
    );
    supported("Cloud Key");
    // "As this artifact enters, choose artifact, creature, enchantment, instant, or
    // sorcery." Instant (index 3).
    let mut t = main_phase(2);
    t.answer(P0, DecisionKind::Option, mtg_engine::decision::Answer::Index(3));
    t.enter(P0, "Cloud Key");
    t.resolve_all();
    costs(&mut t, P0, "Hieroglyphic Illumination", "{2}{U}", "Cloud Key");
    costs(&mut t, P0, "Opt", "{U}", "Cloud Key");
    // Not a sorcery.
    costs(&mut t, P0, "Divination", "{2}{U}", "Cloud Key");
}

#[test]
fn stone_calendar_reduces_what_you_pay_not_the_mana_cost() {
    cr!("601.2f", "118.7a", "202.3");
    ruling!(
        "Stone Calendar",
        "Does not change the mana cost of the spell, it just reduces what you pay for it."
    );
    supported("Stone Calendar");
    // "Spells you cast cost {1} less to cast." Divination ({2}{U}): {1}{U}, mana value 3;
    // Opt ({U}): {U}. (Its 2004 ruling that the reduction may be declined, from the
    // printed "up to 1 less", isn't followed: the Oracle text has no "up to"; see the
    // report of rulings-P033.)
    let mut t = main_phase(2);
    t.battlefield(P0, "Stone Calendar");
    costs(&mut t, P0, "Divination", "{1}{U}", "Stone Calendar");
    costs(&mut t, P0, "Opt", "{U}", "Stone Calendar");
}
