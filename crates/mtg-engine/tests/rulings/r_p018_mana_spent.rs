//! Rulings batch P018 — "if {C} was spent to cast this spell" (CR 601.2h, 106.1): only
//! the mana spent on the spell's total cost counts, not mana paid for another ability's
//! cost; targets are chosen before the cost is paid (CR 601.2c, 601.2h); what's checked
//! as the spell resolves (CR 608.2b); "you" in a granted ability (CR 109.5); "can't play
//! lands" and "can't cast" don't stop putting cards onto the battlefield (CR 305.2,
//! 101.2), nor counter spells already cast; counters a creature enters with (CR 614.1c).

use crate::r_p018_common::*;
use crate::r_s01_common::{asked_since, supported, watch};
use crate::r_s04_common::{add_mana, crew};
use crate::r_s06_common::give_control;
use crate::r_s25_common::targets_of;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts the card `name` (put into their hand) paying with exactly `mana` (put into
/// their mana pool), with the given targets; returns the spell.
fn cast_paying(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    mana: &[(ManaType, u32)],
    targets: &[Entity],
) -> ObjectId {
    for (ty, n) in mana {
        add_mana(t, p, *ty, *n);
    }
    let card = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    let spell = t.cast(p, card).go();
    assert_eq!(
        t.g.players[p.idx()].mana_pool.total(),
        0,
        "all of the mana was spent"
    );
    spell
}

#[test]
fn squealing_devil_needs_black_spent_on_its_mana_cost_not_on_x() {
    cr!("601.2h");
    ruling!(
        "Squealing Devil",
        "For this creature to not be sacrificed, {B} must have been spent on the {1}{R} mana cost. It doesn't matter if {B} is spent on {X}."
    );
    supported("Squealing Devil");
    for (generic, kept) in [(ManaType::G, false), (ManaType::B, true)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        cast_paying(
            &mut t,
            P0,
            "Squealing Devil",
            &[(ManaType::R, 1), (generic, 1)],
            &[],
        );
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.resolve();
        // {X} = 1, paid with {B}.
        add_mana(&mut t, P0, ManaType::B, 1);
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::X, Answer::Number(1));
        t.answer(P0, DecisionKind::Number, Answer::Number(1));
        t.resolve_all();
        assert_eq!(t.g.players[P0.idx()].mana_pool.total(), 0);
        assert_eq!(t.pt(bears), (3, 2));
        assert_eq!(
            !t.named_on_battlefield("Squealing Devil").is_empty(),
            kept,
            "paid {{1}} with {generic:?}"
        );
    }
}

#[test]
fn squealing_devil_chooses_x_as_its_ability_resolves_and_it_can_be_zero() {
    cr!("608.2", "107.3");
    ruling!(
        "Squealing Devil",
        "You choose the value of X as Squealing Devil's ability resolves. X can be 0."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_paying(
        &mut t,
        P0,
        "Squealing Devil",
        &[(ManaType::R, 1), (ManaType::B, 1)],
        &[],
    );
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    t.settle();
    // Both enters triggers are on the stack and no X has been chosen.
    assert_eq!(t.stack_len(), 2);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.answer(P0, DecisionKind::Number, Answer::Number(0));
    t.resolve_all();
    let xs: Vec<i64> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseX { min, .. } => Some(min as i64),
            Decision::ChooseNumber { min, .. } => Some(min as i64),
            _ => None,
        })
        .collect();
    assert_eq!(xs, vec![0], "X is chosen on resolution, and 0 is allowed");
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.named_on_battlefield("Squealing Devil").is_empty());
}

#[test]
fn flash_conscriptions_lifegain_goes_to_the_creatures_controller() {
    cr!("109.5", "510.2");
    ruling!(
        "Flash Conscription",
        "If white mana was paid, the player who controls the creature when it deals combat damage will be the one who gains life."
    );
    supported("Flash Conscription");
    // P0 conscripts their own Bears paying {W}; P1 then gains control of the Bears and
    // blocks P0's Hill Giant with them: P1 gains the life.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_paying(
        &mut t,
        P0,
        "Flash Conscription",
        &[(ManaType::R, 1), (ManaType::W, 1), (ManaType::C, 4)],
        &[Entity::Object(bears)],
    );
    t.resolve_all();
    give_control(&mut t, bears, P1);
    assert_eq!(t.obj_now(bears).controller, P1);
    t.attack(&[(giant, Entity::Player(P1))], &[(bears, giant)]);
    assert_eq!(t.life(P1), 22);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn moonhold_doesnt_stop_putting_cards_onto_the_battlefield() {
    cr!("305.4");
    ruling!(
        "Moonhold",
        "Moonhold doesn't stop effects that \"put a card onto the battlefield.\""
    );
    supported("Moonhold");
    supported("Growth Spiral");
    supported("Through the Breach");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_paying(
        &mut t,
        P0,
        "Moonhold",
        &[(ManaType::R, 1), (ManaType::W, 1), (ManaType::C, 1)],
        &[Entity::Player(P1)],
    );
    t.resolve_all();
    let forest = t.hand(P1, "Forest");
    assert!(t.play_land(P1, forest).is_err());
    let bears = t.hand(P1, "Grizzly Bears");
    add_mana(&mut t, P1, ManaType::G, 2);
    assert!(t.cast(P1, bears).try_go().is_err());
    empty_pool(&mut t, P1);
    // Growth Spiral: "You may put a land card from your hand onto the battlefield."
    t.answer_choose(P1, &[Entity::Object(forest)]);
    t.answer_yes(P1, true);
    cast_paying(
        &mut t,
        P1,
        "Growth Spiral",
        &[(ManaType::G, 1), (ManaType::U, 1)],
        &[],
    );
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    // Through the Breach: "You may put a creature card from your hand onto the
    // battlefield."
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.answer_yes(P1, true);
    cast_paying(
        &mut t,
        P1,
        "Through the Breach",
        &[(ManaType::R, 1), (ManaType::C, 4)],
        &[],
    );
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn moonhold_doesnt_counter_a_creature_spell_already_cast() {
    cr!("601.2i");
    ruling!(
        "Moonhold",
        "Moonhold is not a counterspell. It will have no effect on creature spells the targeted player cast before Moonhold resolved."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = cast_paying(&mut t, P1, "Grizzly Bears", &[(ManaType::G, 2)], &[]);
    cast_paying(
        &mut t,
        P0,
        "Moonhold",
        &[(ManaType::W, 1), (ManaType::C, 2)],
        &[Entity::Player(P1)],
    );
    t.resolve();
    assert_eq!(t.g.stack, vec![bears]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // But P1 can't cast another one.
    let other = t.hand(P1, "Grizzly Bears");
    add_mana(&mut t, P1, ManaType::G, 2);
    assert!(t.cast(P1, other).try_go().is_err());
}

#[test]
fn mythos_of_nethroi_checks_as_it_resolves_whether_it_destroys() {
    cr!("608.2b", "608.2h", "601.2h");
    ruling!(
        "Mythos of Nethroi",
        "Mythos of Nethroi can target any nonland permanent. Determine as the spell resolves whether the permanent is destroyed, either because it’s a creature at that time or because {G}{W} was spent to cast the spell."
    );
    supported("Mythos of Nethroi");
    supported("Smuggler's Copter");
    let mana_b = [(ManaType::B, 1), (ManaType::C, 2)];
    let mana_gwb = [(ManaType::B, 1), (ManaType::G, 1), (ManaType::W, 1)];
    // An uncrewed Vehicle, without {G}{W}: not destroyed.
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P1, "Smuggler's Copter");
    t.battlefield(P1, "Grizzly Bears");
    cast_paying(
        &mut t,
        P0,
        "Mythos of Nethroi",
        &mana_b,
        &[Entity::Object(copter)],
    );
    t.resolve_all();
    assert!(t.on_battlefield(copter));
    // Crewed in response, it's a creature as Mythos resolves: destroyed.
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P1, "Smuggler's Copter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_paying(
        &mut t,
        P0,
        "Mythos of Nethroi",
        &mana_b,
        &[Entity::Object(copter)],
    );
    assert!(crew(&mut t, P1, copter, &[bears]));
    t.resolve_all();
    assert!(!t.on_battlefield(copter));
    // {G}{W} spent: a noncreature permanent is destroyed (an enchantment too).
    for name in ["Smuggler's Copter", "Glorious Anthem"] {
        let mut t = TestGame::new(2);
        let target = t.battlefield(P1, name);
        cast_paying(
            &mut t,
            P0,
            "Mythos of Nethroi",
            &mana_gwb,
            &[Entity::Object(target)],
        );
        t.resolve_all();
        assert!(!t.on_battlefield(target), "{name}");
    }
}

#[test]
fn vigor_mortis_counter_is_in_addition_to_other_counters() {
    cr!("614.1c", "122.6");
    ruling!(
        "Vigor Mortis",
        "The +1/+1 counter from this ability is in addition to any other counters the creature enters with."
    );
    supported("Vigor Mortis");
    supported("Arcbound Worker");
    for (generic, n) in [(ManaType::G, 2), (ManaType::C, 1)] {
        let mut t = TestGame::new(2);
        let worker = t.graveyard(P0, "Arcbound Worker");
        cast_paying(
            &mut t,
            P0,
            "Vigor Mortis",
            &[(ManaType::B, 2), (generic, 1), (ManaType::C, 1)],
            &[Entity::Object(worker)],
        );
        t.resolve_all();
        assert!(t.on_battlefield(worker));
        assert_eq!(t.counters(worker, counters::PLUS1), n, "{generic:?}");
    }
}

#[test]
fn targets_are_chosen_before_the_spell_is_paid_for() {
    cr!("601.2c", "601.2h");
    ruling!(
        "Cankerous Thirst",
        "When you cast the spell, you choose its targets before you pay for it."
    );
    ruling!(
        "Moonhold",
        "When you cast the spell, you choose its targets before you pay for it."
    );
    supported("Cankerous Thirst");
    // The pool is still full when the targets are chosen; Cankerous Thirst's {G} target
    // is chosen even though only {B} is then spent (so only -3/-3 happens).
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    let pools = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.players[0].mana_pool.total(),
    );
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let spell = cast_paying(
        &mut t,
        P0,
        "Cankerous Thirst",
        &[(ManaType::B, 1), (ManaType::C, 3)],
        &[Entity::Object(a), Entity::Object(b)],
    );
    assert_eq!(
        targets_of(&t, spell),
        vec![Entity::Object(a), Entity::Object(b)]
    );
    assert_eq!(*pools.lock().unwrap(), vec![4, 4]);
    t.resolve_all();
    assert!(!t.on_battlefield(a), "the Giant got -3/-3");
    assert_eq!(t.pt(b), (2, 2), "no {{G}} was spent");
    // Moonhold too.
    let mut t = TestGame::new(2);
    let pools = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.players[0].mana_pool.total(),
    );
    cast_paying(
        &mut t,
        P0,
        "Moonhold",
        &[(ManaType::R, 1), (ManaType::C, 2)],
        &[Entity::Player(P1)],
    );
    assert_eq!(*pools.lock().unwrap(), vec![3]);
}
