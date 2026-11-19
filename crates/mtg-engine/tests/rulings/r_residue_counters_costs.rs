//! Item `rulings-residue-counters` — costs and counters: cost reductions for each counter
//! applied after additional costs and increases (CR 601.2f: Herald of War, Mutated
//! Cultist), "the first spell ... each turn" counted from before the source existed
//! (Zimone), "the amount of mana spent to cast it" (CR 601.2h; Dyadrine: copies, free
//! casts, and mana paid for Mana Leak), and casting a card "without paying its mana cost"
//! during the resolution of an ability (CR 118.9, 608.2g; Malcolm).

use crate::r_residue_counters_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts `name` in `p`'s hand and tries to cast it with exactly the given lands (and the
/// given targets, one per slot), kicked or not.
fn try_cast(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    lands: &[&str],
    targets: &[Entity],
    kicked: bool,
) -> bool {
    for l in lands {
        t.lands(p, l, 1);
    }
    let card = t.hand(p, name);
    let mut b = t.cast(p, card).kicked(kicked);
    for e in targets {
        b = b.target(*e);
    }
    b.try_go().is_ok()
}

// --- Herald of War -----------------------------------------------------------------------

#[test]
fn herald_of_war_reduces_an_angel_human_spell_only_once_per_counter() {
    cr!("601.2f", "702.73a");
    ruling!(
        "Herald of War",
        "A creature spell that's both an Angel and a Human will cost {1} less to cast for each +1/+1 counter on Herald of War."
    );
    supported("Herald of War");
    supported("Avian Changeling");
    // Avian Changeling ({2}{W}) is every creature type: with one counter it costs {1}{W},
    // not {W}.
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Herald of War", 1);
    t.set_step(P0, Step::PrecombatMain);
    assert!(!try_cast(&mut t, P0, "Avian Changeling", &["Plains"], &[], false));
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Herald of War", 1);
    t.set_step(P0, Step::PrecombatMain);
    assert!(try_cast(
        &mut t,
        P0,
        "Avian Changeling",
        &["Plains", "Wastes"],
        &[],
        false
    ));
}

#[test]
fn herald_of_war_reduction_applies_after_kicker() {
    cr!("601.2f", "702.33d");
    ruling!(
        "Herald of War",
        "If there are additional costs to cast a creature spell, such as a kicker cost, apply those increases before applying cost reductions."
    );
    supported("Sergeant-at-Arms");
    // Sergeant-at-Arms ({2}{W}, kicker {2}{W}) kicked with three counters: {4}{W}{W} less
    // {3} is {1}{W}{W} (reducing first would leave {2}{W}{W}).
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Herald of War", 3);
    t.set_step(P0, Step::PrecombatMain);
    assert!(try_cast(
        &mut t,
        P0,
        "Sergeant-at-Arms",
        &["Plains", "Plains", "Wastes"],
        &[],
        true
    ));
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Soldier"), 2, "kicked");
}

// --- Mutated Cultist ---------------------------------------------------------------------

/// P0 casts Mutated Cultist with its cast trigger targeting `target`; returns the Cultist
/// spell (the trigger is on top of it).
fn cultist(t: &mut TestGame, target: Entity) -> ObjectId {
    t.set_step(P0, Step::PrecombatMain);
    lands_for_cost(t, P0, "Mutated Cultist");
    let c = t.hand(P0, "Mutated Cultist");
    t.answer_targets(P0, &[target]);
    let id = t.cast(P0, c).go();
    t.settle();
    id
}

#[test]
fn mutated_cultist_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "113.7a");
    ruling!(
        "Mutated Cultist",
        "If Mutated Cultist is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    supported("Mutated Cultist");
    supported("Counterspell");
    let mut t = TestGame::new(2);
    let bears = with_counters(&mut t, P1, "Grizzly Bears", 2);
    let spell = cultist(&mut t, obj(bears));
    assert_eq!(t.stack_len(), 2, "the trigger is on top of the Cultist");
    cast_new(&mut t, P1, "Counterspell", &[obj(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mutated Cultist"));
    assert_eq!(plus1(&t, bears), 0);
}

#[test]
fn mutated_cultist_reduction_applies_after_increases_and_leaves_mana_value_alone() {
    cr!("601.2f", "202.3");
    ruling!(
        "Mutated Cultist",
        "start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions"
    );
    supported("Thalia, Guardian of Thraben");
    // Two counters removed; Thalia adds {1} to Divination: {2}{U} + {1} - {2} = {1}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let bears = with_counters(&mut t, P1, "Grizzly Bears", 2);
    cultist(&mut t, obj(bears));
    t.resolve_all();
    assert!(!try_cast(&mut t, P0, "Divination", &["Island"], &[], false));
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let bears = with_counters(&mut t, P1, "Grizzly Bears", 2);
    cultist(&mut t, obj(bears));
    t.resolve_all();
    assert!(try_cast(
        &mut t,
        P0,
        "Divination",
        &["Island", "Wastes"],
        &[],
        false
    ));
    let spell = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(spell).chars.mana_value(), 3);
}

// --- Zimone, Infinite Analyst ------------------------------------------------------------

#[test]
fn zimone_misses_the_turn_if_the_first_x_spell_came_before_her() {
    cr!("601.2f", "603.2");
    ruling!(
        "Zimone, Infinite Analyst",
        "If you cast your first spell with {X} in its mana cost during a turn before Zimone is on the battlefield"
    );
    supported("Zimone, Infinite Analyst");
    supported("Blaze");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(1).target(Entity::Player(P1)).go();
    t.resolve_all();
    let z = with_counters(&mut t, P0, "Zimone, Infinite Analyst", 2);
    // The second {X} spell: no reduction (X=1 needs {1}{R}) and no trigger.
    t.lands(P0, "Mountain", 1);
    let blaze = t.hand(P0, "Blaze");
    assert!(t
        .cast(P0, blaze)
        .x(1)
        .target(Entity::Player(P1))
        .try_go()
        .is_err());
    t.lands(P0, "Mountain", 1);
    t.cast(P0, blaze).x(1).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "first spell"), 0);
    t.resolve_all();
    assert_eq!(plus1(&t, z), 2);
}

// --- Dyadrine, Synthesis Amalgam ---------------------------------------------------------

#[test]
fn dyadrine_copy_has_no_mana_spent_to_cast_it() {
    cr!("707.10", "601.2h");
    ruling!(
        "Dyadrine, Synthesis Amalgam",
        "If a copy of Dyadrine is created on the stack, no mana was spent to cast the copy."
    );
    supported("Double Major");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 2);
    let d = t.hand(P0, "Dyadrine, Synthesis Amalgam");
    let spell = t.cast(P0, d).x(2).go();
    cast_new(&mut t, P0, "Double Major", &[obj(spell)]);
    t.resolve_all();
    let all = t.named_on_battlefield("Dyadrine, Synthesis Amalgam");
    assert_eq!(all.len(), 2);
    let mut got: Vec<u32> = all.iter().map(|id| plus1(&t, *id)).collect();
    got.sort();
    assert_eq!(got, vec![0, 4]);
}

#[test]
fn dyadrine_cast_without_paying_its_mana_cost_enters_without_counters() {
    cr!("107.3b", "601.2h");
    ruling!(
        "Dyadrine, Synthesis Amalgam",
        "If an effect allows you to cast Dyadrine without paying its mana cost, X will be 0."
    );
    supported("Omniscience");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Omniscience");
    let d = t.hand(P0, "Dyadrine, Synthesis Amalgam");
    // X = 3 is announced, but the only legal value is 0.
    let spell = t.cast(P0, d).method(CastMethod::Free).x(3).go();
    assert_eq!(crate::r_s25_common::x_of(&t, spell).unwrap_or(0), 0);
    t.resolve_all();
    let all = t.named_on_battlefield("Dyadrine, Synthesis Amalgam");
    assert_eq!(all.len(), 1);
    assert_eq!(plus1(&t, all[0]), 0);
    assert_eq!(t.pt(all[0]), (0, 1));
}

#[test]
fn dyadrine_ignores_mana_paid_for_mana_leak() {
    cr!("601.2h", "118.12");
    ruling!(
        "Dyadrine, Synthesis Amalgam",
        "that mana wasn’t spent to cast Dyadrine and will not be taken into account"
    );
    supported("Mana Leak");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 1);
    let d = t.hand(P0, "Dyadrine, Synthesis Amalgam");
    let spell = t.cast(P0, d).x(1).go();
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    cast_new(&mut t, P1, "Mana Leak", &[obj(spell)]);
    t.resolve_all();
    let all = t.named_on_battlefield("Dyadrine, Synthesis Amalgam");
    assert_eq!(all.len(), 1, "Mana Leak was paid for");
    assert_eq!(plus1(&t, all[0]), 3);
}

// --- Malcolm, Alluring Scoundrel ---------------------------------------------------------

/// P0's Malcolm with three chorus counters deals combat damage to P1, drawing `drawn`
/// (put on top of P0's library) and discarding it; P0 casts it. `prep` queues further
/// answers before combat damage.
fn malcolm_casts(t: &mut TestGame, drawn: &str, prep: impl FnOnce(&mut TestGame)) -> usize {
    let m = t.battlefield(P0, "Malcolm, Alluring Scoundrel");
    put(t, m, "chorus", 3);
    let card = t.library_top(P0, drawn);
    attack_with(t, &[(m, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    t.answer_choose_moved(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    prep(t);
    let from = t.asked().len();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    t.resolve_all();
    from
}

#[test]
fn malcolm_casts_the_card_during_resolution_ignoring_its_type_timing() {
    cr!("608.2g", "117.1a");
    ruling!(
        "Malcolm, Alluring Scoundrel",
        "You can't wait to cast the spell later in the turn. Timing permissions based on the card's type are ignored."
    );
    // A sorcery, cast in the combat damage step.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    malcolm_casts(&mut t, "Divination", |_| {});
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), hand + 2, "Divination resolved");
}

#[test]
fn malcolm_free_cast_pays_additional_costs_but_no_alternative_cost() {
    cr!("118.9a", "601.2b", "601.2f");
    ruling!(
        "Malcolm, Alluring Scoundrel",
        "If you cast a card \"without paying its mana cost\", you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, you must pay those to cast the card."
    );
    supported("Bone Splinters");
    supported("Burst Lightning");
    supported("Fireblast");
    // An optional additional cost (kicker {4}) may be paid: 4 damage.
    let mut t = TestGame::new(2);
    malcolm_casts(&mut t, "Burst Lightning", |t| {
        t.lands(P0, "Wastes", 4);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert_eq!(t.life(P1), 20 - 2 - 4);
    // A mandatory one must be: Bone Splinters sacrifices a creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    malcolm_casts(&mut t, "Bone Splinters", |t| {
        t.answer_targets(P0, &[obj(giant)]);
        t.answer_choose(P0, &[obj(bears)]);
    });
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // No alternative cost is offered: Fireblast's (sacrifice two Mountains) isn't chosen.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let from = malcolm_casts(&mut t, "Fireblast", |t| {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    });
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseCastingMethod { .. })));
    assert_eq!(t.named_on_battlefield("Mountain").len(), 2);
    assert_eq!(t.life(P1), 20 - 2 - 4);
}
