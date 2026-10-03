//! CR 702.56 Replicate.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_027_037::{optional_costs_offered, untapped_lands};
use crate::common_k702_038_051::{spell_copies_on_stack, with_cost};
use crate::common_k702_052_066::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

const REPLICATE: &str = "Replicate";

/// Declares that the next replicate cost `p` is offered is paid `n` times.
fn replicate(t: &mut TestGame, p: PlayerId, n: i64) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Number(n));
}

#[test]
fn replicate_copies_the_spell_for_each_time_its_cost_was_paid() {
    cr!("702.56", "702.56a");
    assert_supported("Pyromatics");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let pyro = t.hand(P0, "Pyromatics");
    replicate(&mut t, P0, 2);
    t.cast(P0, pyro).target(P1).go();
    // The replicate cost was paid twice as an additional cost: {1}{R} + 2 × {1}{R}.
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(optional_costs_offered(&t, P0), vec!["replicate#1".to_string()]);
    t.settle();
    assert_eq!(stack_triggers(&t, REPLICATE).len(), 1);
    t.resolve();
    assert_eq!(spell_copies_on_stack(&t, "Pyromatics"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The copies weren't cast.
    assert_eq!(t.g.history.spells_cast.len(), 1);
}

#[test]
fn replicate_doesnt_trigger_if_its_cost_wasnt_paid() {
    cr!("702.56a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let pyro = t.hand(P0, "Pyromatics");
    replicate(&mut t, P0, 0);
    t.cast(P0, pyro).target(P1).go();
    t.settle();
    assert!(stack_triggers(&t, REPLICATE).is_empty());
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn copies_from_replicate_may_have_new_targets() {
    cr!("702.56a");
    assert_supported("Gigadrowse");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let drowse = t.hand(P0, "Gigadrowse");
    replicate(&mut t, P0, 2);
    t.cast(P0, drowse).target(a).go();
    // Each copy: choose new targets.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(c)]);
    t.resolve_all();
    for x in [a, b, c] {
        assert!(t.obj_now(x).tapped);
    }
}

#[test]
fn a_replicate_cost_can_be_other_than_mana() {
    cr!("702.56a");
    assert_supported("Reiterating Bolt");
    let mut t = TestGame::new(2);
    t.g.players[0]
        .counters
        .insert(types::counters::ENERGY.into(), 7);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Reiterating Bolt");
    replicate(&mut t, P0, 2);
    t.cast(P0, bolt).target(a).go();
    assert_eq!(t.g.player(P0).counter(types::counters::ENERGY), 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn the_copies_are_made_even_if_the_spell_was_countered() {
    cr!("702.56a");
    ruling!(
        "Changing Loyalty",
        "even if the original spell is no longer on the stack at that time (perhaps because it was countered)"
    );
    ruling!(
        "Changing Loyalty",
        "The copies that replicate creates are created on the stack, so they're not \"cast.\""
    );
    assert_supported("Changing Loyalty");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    let loyalty = t.hand(P0, "Changing Loyalty");
    replicate(&mut t, P0, 1);
    let spell = t.cast(P0, loyalty).target(a).go();
    t.settle();
    assert_eq!(stack_triggers(&t, REPLICATE).len(), 1);
    // In response to the trigger, the Aura spell is countered.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Changing Loyalty"));
    // The copy is still made (a copy of a permanent spell becomes a token).
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve_all();
    let auras: Vec<_> = t.named_on_battlefield("Changing Loyalty");
    assert_eq!(auras.len(), 1);
    assert!(t.obj_now(auras[0]).is_token());
    assert_eq!(t.obj_now(auras[0]).attached_to, Some(Entity::Object(b)));
    // Only the original was cast.
    assert_eq!(
        t.g.history
            .spells_cast
            .iter()
            .filter(|(p, _)| *p == P0)
            .count(),
        1
    );
}

#[test]
fn countering_the_replicate_trigger_stops_the_copies() {
    cr!("702.56a");
    ruling!(
        "Changing Loyalty",
        "If the ability countered, no copies will be put onto the stack."
    );
    assert_supported("Stifle");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let pyro = t.hand(P0, "Pyromatics");
    replicate(&mut t, P0, 1);
    t.cast(P0, pyro).target(P1).go();
    t.settle();
    let (trigger, _) = stack_triggers(&t, REPLICATE)[0];
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(trigger).go();
    t.resolve_all();
    assert_eq!(spell_copies_on_stack(&t, "Pyromatics"), 0);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn each_instance_of_replicate_is_paid_and_triggers_separately() {
    cr!("702.56b");
    let mut t = TestGame::new(2);
    let def = with_cost(
        custom_card(
            "Twin Echo Bolt",
            "Sorcery",
            None,
            "Replicate {1}\nReplicate {2}\n~ deals 1 damage to any target.",
        ),
        "{R}",
    );
    let spell = t.custom(P0, def, object::Zone::Hand(P0));
    t.lands(P0, "Mountain", 1 + 1 + 2 * 2);
    // The first instance paid once, the second twice.
    replicate(&mut t, P0, 1);
    replicate(&mut t, P0, 2);
    t.cast(P0, spell).target(P1).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(
        optional_costs_offered(&t, P0),
        vec!["replicate#1".to_string(), "replicate#2".to_string()]
    );
    t.settle();
    let triggers = stack_triggers(&t, REPLICATE);
    assert_eq!(triggers.len(), 2);
    // Each copies the spell for its own payments: once for one, twice for the other.
    t.resolve();
    let first = spell_copies_on_stack(&t, "Twin Echo Bolt");
    for _ in 0..first {
        t.resolve();
    }
    assert_eq!(stack_triggers(&t, REPLICATE).len(), 1);
    t.resolve();
    let second = spell_copies_on_stack(&t, "Twin Echo Bolt");
    let mut both = vec![first, second];
    both.sort();
    assert_eq!(both, vec![1, 2]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn a_spell_given_replicate_has_a_separate_replicate_ability() {
    cr!("702.56b");
    ruling!(
        "Djinn Illuminatus",
        "the spell will have two replicate abilities: one with a replicate cost as printed on it, and one with a replicate cost equal to its mana cost"
    );
    assert_supported("Djinn Illuminatus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Djinn Illuminatus");
    t.lands(P0, "Mountain", 2 + 2 + 2);
    let pyro = t.hand(P0, "Pyromatics");
    // Its own replicate {1}{R} once, and the granted one ({1}{R}, its mana cost) once.
    replicate(&mut t, P0, 1);
    replicate(&mut t, P0, 1);
    t.cast(P0, pyro).target(P1).go();
    assert_eq!(optional_costs_offered(&t, P0).len(), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.settle();
    assert_eq!(stack_triggers(&t, REPLICATE).len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_replicate_cost_equal_to_the_mana_cost_uses_the_value_of_x() {
    cr!("702.56a");
    ruling!(
        "Djinn Illuminatus",
        "the value of X in the spell's replicate cost will be the same as the value of X you chose for its mana cost"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Djinn Illuminatus");
    // Blaze ({X}{R}) with X = 2, replicated once: {2}{R} + {2}{R}.
    t.lands(P0, "Mountain", 6);
    let blaze = t.hand(P0, "Blaze");
    replicate(&mut t, P0, 1);
    t.cast(P0, blaze).x(2).target(P1).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn hatchery_slivers_replicate_copies_become_tokens() {
    cr!("702.56a", "702.56b");
    assert_supported("Hatchery Sliver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hatchery Sliver");
    t.lands(P0, "Forest", 6);
    let sliver = t.hand(P0, "Hatchery Sliver");
    // Two replicate abilities: its own {1}{G} and the granted one (its mana cost).
    replicate(&mut t, P0, 1);
    replicate(&mut t, P0, 1);
    t.cast(P0, sliver).go();
    t.resolve_all();
    let slivers = t.named_on_battlefield("Hatchery Sliver");
    assert_eq!(slivers.len(), 4);
    assert_eq!(
        slivers.iter().filter(|s| t.obj_now(**s).is_token()).count(),
        2
    );
}
