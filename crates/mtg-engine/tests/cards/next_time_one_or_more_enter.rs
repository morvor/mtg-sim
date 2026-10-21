//! "The next time one or more [permanents] enter this turn, [they enter as copies of the
//! chosen creature / each enters with two additional +1/+1 counters on it]." (Mystic
//! Reflection, Storyweave; CR 614.1c, 707.9): used up by the next entry of one or more
//! matching permanents, all of those entering at the same time included.

use crate::basic_effects_common::{assert_supported, oracle_card};
use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts the cards `ids` (owned by `p`) onto the battlefield at the same time.
fn enter_together(t: &mut TestGame, p: PlayerId, ids: &[ObjectId]) -> Vec<ObjectId> {
    let moves = ids
        .iter()
        .map(|id| MoveEv {
            obj: *id,
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    let out = t.g.move_objects(moves).into_iter().flatten().collect();
    t.settle();
    out
}

fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// P0 casts Mystic Reflection ("Choose target nonlegendary creature. The next time one or
/// more creatures or planeswalkers enter this turn, they enter as copies of the chosen
/// creature.") targeting `target`.
fn mystic_reflection(t: &mut TestGame, target: ObjectId) {
    mana(t, P0, ManaType::U, 2);
    let mr = t.hand(P0, "Mystic Reflection");
    t.cast(P0, mr).target(target).go();
    t.resolve_all();
}

#[test]
fn mystic_reflection_copies_every_creature_entering_the_next_time() {
    cr!("614.1c", "707.9", "614.12");
    assert_supported("Mystic Reflection");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    mystic_reflection(&mut t, bears);
    // Two creature cards entering at the same time both enter as copies of the Bears.
    let a = t.graveyard(P0, "Hill Giant");
    let b = t.graveyard(P0, "Savannah Lions");
    let entered = enter_together(&mut t, P0, &[a, b]);
    assert_eq!(entered.len(), 2);
    for id in &entered {
        assert_eq!(t.obj(*id).chars.name, "Grizzly Bears");
        assert_eq!(t.pt(*id), (2, 2));
        assert_eq!(t.obj(*id).controller, P0);
    }
    // That was the next time: another creature entering later is itself.
    let giant = t.enter(P0, "Hill Giant");
    assert_eq!(t.obj_now(giant).chars.name, "Hill Giant");
}

#[test]
fn mystic_reflection_copies_tokens_created_together() {
    cr!("614.1c", "111.1");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    mystic_reflection(&mut t, giant);
    // Raise the Alarm: "Create two 1/1 white Soldier creature tokens."
    assert_supported("Raise the Alarm");
    mana(&mut t, P0, ManaType::W, 2);
    let make = t.hand(P0, "Raise the Alarm");
    t.cast(P0, make).go();
    t.resolve_all();
    let tokens: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|id| t.obj(*id).is_token())
        .collect();
    assert_eq!(tokens.len(), 2);
    for id in tokens {
        assert_eq!(t.obj(id).chars.name, "Hill Giant");
        assert_eq!(t.pt(id), (3, 3));
    }
}

#[test]
fn mystic_reflection_uses_the_last_known_copiable_values_of_a_creature_that_left() {
    cr!("707.2", "608.2h");
    ruling!(
        "Mystic Reflection",
        "use the copiable values of the target creature as it last existed on the battlefield"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    mystic_reflection(&mut t, giant);
    t.g.destroy(giant, None);
    t.settle();
    assert!(!t.on_battlefield(giant));
    let lions = t.enter(P0, "Savannah Lions");
    assert_eq!(t.obj_now(lions).chars.name, "Hill Giant");
    assert_eq!(t.pt(lions), (3, 3));
}

#[test]
fn mystic_reflection_doesnt_wait_past_this_turn_or_apply_to_noncreatures() {
    cr!("614.1c", "514.2");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    mystic_reflection(&mut t, bears);
    // A noncreature permanent entering isn't "creatures or planeswalkers".
    let ring = t.enter(P0, "Sol Ring");
    assert_eq!(t.obj_now(ring).chars.name, "Sol Ring");
    // The effect ends with the turn.
    t.advance_to(P1, Step::Upkeep);
    let lions = t.enter(P1, "Savannah Lions");
    assert_eq!(t.obj_now(lions).chars.name, "Savannah Lions");
}

#[test]
fn a_clone_entering_after_mystic_reflection_copies_the_target_either_way() {
    cr!("616.1", "707.9");
    ruling!(
        "Mystic Reflection",
        "No matter which order these effects are applied in, the creature will be a copy of the target creature"
    );
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        mystic_reflection(&mut t, bears);
        // Clone's own copy effect would copy Hill Giant.
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(giant)]);
        let clone = t.enter(P0, "Clone");
        assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears", "order {pick}");
    }
}

#[test]
fn storyweave_puts_counters_on_each_enchantment_creature_entering_the_next_time() {
    cr!("614.1c", "122.6", "700.2");
    assert_supported("Storyweave");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let saga = t.battlefield(P0, "History of Benalia");
    let lore = t.counters(saga, "lore");
    // Mode 2: "Put two lore counters on target Saga you control. The next time one or more
    // enchantment creatures you control enter this turn, each enters with two additional
    // +1/+1 counters on it."
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let sw = t.hand(P0, "Storyweave");
    t.cast(P0, sw).modes(&[1]).target(saga).go();
    t.resolve();
    assert_eq!(t.counters(saga, "lore"), lore + 2);
    t.resolve_all();
    let nymph = || {
        let mut d = oracle_card("Dream Nymph", "Enchantment Creature — Nymph", "");
        d.faces[0].chars.power = Some(1);
        d.faces[0].chars.toughness = Some(1);
        d
    };
    // Two enchantment creatures and a plain creature enter together; and an opponent's
    // enchantment creature at the same time isn't one P0 controls.
    let a = t.custom(P0, nymph(), Zone::Graveyard(P0));
    let b = t.custom(P0, nymph(), Zone::Graveyard(P0));
    let c = t.graveyard(P0, "Grizzly Bears");
    let entered = enter_together(&mut t, P0, &[a, b, c]);
    let theirs = t.custom(P1, nymph(), Zone::Graveyard(P1));
    let theirs = enter_together(&mut t, P1, &[theirs]);
    assert_eq!(entered.len(), 3);
    assert_eq!(t.counters(entered[0], "+1/+1"), 2);
    assert_eq!(t.counters(entered[1], "+1/+1"), 2);
    assert_eq!(t.counters(entered[2], "+1/+1"), 0);
    assert_eq!(t.counters(theirs[0], "+1/+1"), 0);
    // Used up: a later enchantment creature gets none.
    let d = t.custom(P0, nymph(), Zone::Graveyard(P0));
    let later = enter_together(&mut t, P0, &[d]);
    assert_eq!(t.counters(later[0], "+1/+1"), 0);
}

#[test]
fn storyweave_first_mode_puts_two_counters_on_a_creature() {
    cr!("700.2");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let sw = t.hand(P0, "Storyweave");
    t.cast(P0, sw).modes(&[0]).target(bears).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
}
