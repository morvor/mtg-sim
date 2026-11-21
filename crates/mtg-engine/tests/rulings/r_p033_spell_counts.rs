//! Rulings batch P033 — reductions for "the second spell you cast each turn", "the first
//! creature spell you cast each turn", "the next spell you cast this turn" and "during
//! turns other than yours" (CR 601.2f, 118.7a): which spells count (the whole turn, the
//! reducer itself, spells that didn't resolve), and a reducer doesn't reduce its own cost
//! while it's being cast.

use crate::r_p033_common::*;
use crate::r_s01_common::supported;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts `name` from hand (from a full pool) and returns what it cost.
fn cast(t: &mut TestGame, p: PlayerId, name: &str) -> Paid {
    let card = t.hand(p, name);
    cast_normal(t, p, card)
}

/// Asserts that `name` cost `cost` and has its printed mana value, and resolves the stack.
fn costs(t: &mut TestGame, name: &str, cost: &str, what: &str) {
    let paid = cast(t, P0, name);
    assert_spent(&paid.spent, cost, what);
    assert_eq!(paid.mv, printed_mv(name), "{what}: mana value");
    t.resolve_all();
}

#[test]
fn second_spell_reducers_count_spells_cast_before_they_entered_but_not_themselves() {
    cr!("601.2f", "118.7a", "202.3", "113.6");
    ruling!(
        "Highspire Bell-Ringer",
        "Spells that were cast before Highspire Bell-Ringer entered the battlefield count. If Highspire Bell-Ringer was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    ruling!(
        "Raging Battle Mouse",
        "Spells that were cast before Raging Battle Mouse entered the battlefield count. If Raging Battle Mouse was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    ruling!(
        "Highspire Bell-Ringer",
        "Highspire Bell-Ringer can’t reduce its own cost, even if it’s the second spell you cast in a turn."
    );
    ruling!(
        "Raging Battle Mouse",
        "Raging Battle Mouse can't reduce its own cost, even if it's the second spell you cast in a turn."
    );
    ruling!(
        "Highspire Bell-Ringer",
        "Highspire Bell-Ringer’s cost-reduction ability can only reduce the generic mana component of the cost you pay for a spell."
    );
    ruling!(
        "Raging Battle Mouse",
        "Raging Battle Mouse's cost-reduction ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that cost."
    );
    ruling!(
        "Highspire Bell-Ringer",
        "Highspire Bell-Ringer’s cost-reduction ability doesn’t change the mana cost or mana value of any spell. It changes only the total cost you pay."
    );
    ruling!(
        "Raging Battle Mouse",
        "Raging Battle Mouse's cost-reduction ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay."
    );
    // "The second spell you cast each turn costs {1} less to cast."
    for (reducer, own) in [
        ("Highspire Bell-Ringer", "{2}{U}"),
        ("Raging Battle Mouse", "{1}{R}"),
    ] {
        supported(reducer);
        // Cast first: full cost; the next spell is the second: Divination ({2}{U}) costs
        // {1}{U}, mana value 3; the third costs {2}{U}.
        let mut t = main_phase(2);
        costs(&mut t, reducer, own, reducer);
        costs(&mut t, "Divination", "{1}{U}", reducer);
        costs(&mut t, "Divination", "{2}{U}", reducer);
        // Cast second, it costs its full cost; another one on the battlefield reduces it.
        let mut t = main_phase(2);
        costs(&mut t, "Opt", "{U}", reducer);
        costs(&mut t, reducer, own, reducer);
        let mut t = main_phase(2);
        t.battlefield(P0, reducer);
        costs(&mut t, "Opt", "{U}", reducer);
        let cheaper = if reducer == "Raging Battle Mouse" {
            "{R}"
        } else {
            "{1}{U}"
        };
        costs(&mut t, reducer, cheaper, reducer);
        // The second spell with only colored mana: Opt still costs {U}.
        let mut t = main_phase(2);
        t.battlefield(P0, reducer);
        costs(&mut t, "Opt", "{U}", reducer);
        costs(&mut t, "Opt", "{U}", reducer);
    }
}

#[test]
fn uthros_psionicist_counts_every_spell_cast_this_turn() {
    cr!("601.2f", "118.7a", "701.6a");
    ruling!(
        "Uthros Psionicist",
        "Uthros Psionicist’s ability will count any spells you’ve cast this turn, which may include Uthros Psionicist itself. It doesn’t matter if the other spells resolved, didn’t resolve, were countered, or are still on the stack. If Uthros Psionicist was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    supported("Uthros Psionicist");
    // "The second spell you cast each turn costs {2} less to cast."
    // Cast first, it resolved: Divination ({2}{U}) is the second spell: {U}.
    let mut t = main_phase(2);
    costs(&mut t, "Uthros Psionicist", "{2}{U}", "Psionicist first");
    costs(&mut t, "Divination", "{U}", "after the Psionicist");
    // The first spell was countered: Divination is still the second.
    let mut t = main_phase(2);
    t.battlefield(P0, "Uthros Psionicist");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let bolt = cast(&mut t, P0, "Lightning Bolt");
    let cs = t.hand(P1, "Counterspell");
    add(&mut t, P1, ManaType::U, 2);
    t.cast(P1, cs).target(obj(bolt.spell)).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    costs(&mut t, "Divination", "{U}", "after a countered spell");
    // The first spell is still on the stack: Hieroglyphic Illumination ({3}{U}) costs
    // {1}{U}.
    let mut t = main_phase(2);
    t.battlefield(P0, "Uthros Psionicist");
    cast(&mut t, P0, "Opt");
    assert_eq!(t.stack_len(), 1);
    let paid = cast(&mut t, P0, "Hieroglyphic Illumination");
    assert_spent(&paid.spent, "{1}{U}", "with the first spell on the stack");
}

#[test]
fn radagast_looks_at_the_whole_turn_and_reduces_only_generic_mana() {
    cr!("601.2f", "118.7a", "202.3");
    ruling!(
        "Radagast of Rhosgobel",
        "Radagast of Rhosgobel's ability will look at the entire turn, even if Radagast wasn't on the battlefield for some of it. Notably, if you cast Radagast of Rhosgobel in a turn, then no other creature spell you cast that turn can be your first."
    );
    ruling!(
        "Radagast of Rhosgobel",
        "Radagast's ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic component of that cost."
    );
    ruling!(
        "Radagast of Rhosgobel",
        "Radagast of Rhosgobel's ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you actually pay."
    );
    supported("Radagast of Rhosgobel");
    // "The first creature spell you cast each turn costs {2} less to cast ..."
    // Radagast cast this turn was the first creature spell: Craw Wurm costs {4}{G}{G}.
    let mut t = main_phase(2);
    costs(&mut t, "Radagast of Rhosgobel", "{2}{G}{G}", "Radagast");
    costs(&mut t, "Craw Wurm", "{4}{G}{G}", "after Radagast");
    // A creature spell cast before Radagast was on the battlefield was the first.
    let mut t = main_phase(2);
    costs(&mut t, "Grizzly Bears", "{1}{G}", "before Radagast");
    t.battlefield(P0, "Radagast of Rhosgobel");
    costs(
        &mut t,
        "Craw Wurm",
        "{4}{G}{G}",
        "the second creature spell",
    );
    // A noncreature spell doesn't count: the first creature spell costs {2} less, mana
    // value unchanged.
    let mut t = main_phase(2);
    t.battlefield(P0, "Radagast of Rhosgobel");
    costs(&mut t, "Divination", "{2}{U}", "a sorcery");
    costs(&mut t, "Craw Wurm", "{2}{G}{G}", "the first creature spell");
    // Only generic mana: Grizzly Bears ({1}{G}) costs {G}; Kird Ape ({R}) costs {R}.
    let mut t = main_phase(2);
    t.battlefield(P0, "Radagast of Rhosgobel");
    costs(&mut t, "Grizzly Bears", "{G}", "Bears first");
    let mut t = main_phase(2);
    t.battlefield(P0, "Radagast of Rhosgobel");
    costs(&mut t, "Kird Ape", "{R}", "Kird Ape first");
}

#[test]
fn hardened_berserker_cant_reduce_colored_mana() {
    cr!("601.2f", "118.7a", "603.2");
    ruling!(
        "Hardened Berserker",
        "Hardened Berserker can’t reduce the colored mana requirement of a spell you cast."
    );
    // "Whenever this creature attacks, the next spell you cast this turn costs {1} less to
    // cast."
    let attacked = || {
        let mut t = main_phase(2);
        let b = t.battlefield(P0, "Hardened Berserker");
        t.attack(&[(b, Entity::Player(P1))], &[]);
        t.resolve_all();
        t
    };
    let mut t = attacked();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    costs(&mut t, "Lightning Bolt", "{R}", "Lightning Bolt");
    let mut t = attacked();
    costs(
        &mut t,
        "Hieroglyphic Illumination",
        "{2}{U}",
        "Hieroglyphic Illumination",
    );
    // Without the attack: {3}{U}.
    let mut t = main_phase(2);
    t.battlefield(P0, "Hardened Berserker");
    costs(&mut t, "Hieroglyphic Illumination", "{3}{U}", "no attack");
}

#[test]
fn geyser_drake_reduces_spells_on_other_turns_without_changing_mana_values() {
    cr!("601.2f", "202.3");
    ruling!(
        "Geyser Drake",
        "Geyser Drake’s last ability doesn’t change the mana cost or mana value of any spell. It changes only the total cost you pay to cast spells."
    );
    supported("Geyser Drake");
    // "During turns other than yours, spells you cast cost {1} less to cast."
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Geyser Drake");
    let card = t.hand(P0, "Hieroglyphic Illumination");
    t.g.turn.priority = Some(P0);
    let paid = cast_normal(&mut t, P0, card);
    assert_spent(&paid.spent, "{2}{U}", "P1's turn");
    assert_eq!(paid.mv, 4);
    t.resolve_all();
    let mut t = main_phase(2);
    t.battlefield(P0, "Geyser Drake");
    costs(&mut t, "Hieroglyphic Illumination", "{3}{U}", "P0's turn");
}
