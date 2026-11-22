//! Rulings batch S14 — replicate (CR 702.56): "When you cast this spell, if a replicate
//! cost was paid for it, copy it for each time its replicate cost was paid."; and retrace
//! (CR 702.81): "You may cast this card from your graveyard by discarding a land card as
//! an additional cost to cast it."

use crate::r_s01_common::*;
use crate::r_s04_common::graveyard_names;
use crate::r_s11_common::spells_copied;
use crate::r_s14_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The copies of spells on the stack.
fn copies_on_stack(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|s| t.g.obj(*s).kind == ObjKind::SpellCopy)
        .collect()
}

/// P0 casts Reiterating Bolt ("Replicate—Pay {E}{E}{E}. ... deals 3 damage to target
/// creature or planeswalker.") at Hill Giant `a`, paying replicate once, and the copy
/// targets Hill Giant `b`. Returns (the original spell, the copy) once the copy is on the
/// stack.
fn bolt_and_copy(t: &mut TestGame, a: ObjectId, b: ObjectId) -> (ObjectId, ObjectId) {
    t.g.players[0]
        .counters
        .insert(types::counters::ENERGY.into(), 3);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    let spell = cast_from_hand(t, P0, "Reiterating Bolt", &[Entity::Object(a)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.settle();
    // The replicate trigger resolves: one copy.
    t.resolve();
    let copies = copies_on_stack(t);
    assert_eq!(copies.len(), 1);
    (spell, copies[0])
}

#[test]
fn a_replicate_copy_is_countered_separately_from_the_original() {
    cr!("702.56a", "707.10", "701.6a");
    ruling!(
        "Reiterating Bolt",
        "A copy of a spell can be countered like any other spell, but it must be countered individually. Countering a spell with replicate won't affect the copies."
    );
    supported("Reiterating Bolt");
    // Countering the original: the copy still resolves.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    let (spell, copy) = bolt_and_copy(&mut t, a, b);
    cast_from_hand(&mut t, P1, "Counterspell", &[Entity::Object(spell)]);
    t.resolve();
    assert!(t.g.stack.contains(&copy));
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(t.in_graveyard(P0, "Reiterating Bolt"));
    // Countering the copy: the original still resolves.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    let (spell, copy) = bolt_and_copy(&mut t, a, b);
    cast_from_hand(&mut t, P1, "Counterspell", &[Entity::Object(copy)]);
    t.resolve();
    assert!(t.g.stack.contains(&spell));
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b));
}

#[test]
fn replicate_copies_even_after_the_original_was_countered() {
    cr!("702.56a", "113.7a", "608.2h");
    ruling!(
        "Shattering Spree",
        "As the replicate triggered ability resolves, you'll copy Shattering Spree for each time you paid its replicate cost, even if the original spell is no longer on the stack at that time (perhaps because it was countered)."
    );
    supported("Shattering Spree");
    // Shattering Spree ({R}: destroy target artifact; replicate {R}) with its replicate
    // cost paid twice, targeting one of three Ornithopters. P1 counters it while the
    // replicate trigger is on the stack: the trigger still makes two copies, which
    // destroy the other two Ornithopters.
    let mut t = TestGame::new(2);
    let birds: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Ornithopter")).collect();
    t.lands(P0, "Mountain", 2);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    let spell = cast_from_hand(&mut t, P0, "Shattering Spree", &[Entity::Object(birds[0])]);
    t.settle();
    assert_eq!(triggers_on_stack_now(&t), 1);
    cast_from_hand(&mut t, P1, "Counterspell", &[Entity::Object(spell)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Shattering Spree"));
    for b in &birds[1..] {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Object(*b)]);
    }
    t.resolve();
    assert_eq!(copies_on_stack(&t).len(), 2);
    assert_eq!(spells_copied(&t), 2);
    t.resolve_all();
    assert!(t.on_battlefield(birds[0]));
    assert!(!t.on_battlefield(birds[1]));
    assert!(!t.on_battlefield(birds[2]));
}

const RETRACE: CastMethod = CastMethod::Keyword(KeywordKind::Retrace);

/// P0's answer to a priority decision while Decaying Time Loop is in P0's graveyard and
/// the stack is empty: cast it again with retrace.
fn retrace_time_loop(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::Priority { actions } = d else {
        return None;
    };
    actions
        .iter()
        .find(|a| {
            matches!(a, Action::Cast { card, method }
                if *method == RETRACE
                    && g.obj(*card).chars.name == "Decaying Time Loop"
                    && g.obj(*card).zone == Zone::Graveyard(P0))
        })
        .map(|a| Answer::Action(a.clone()))
}

#[test]
fn the_active_player_can_retrace_a_card_again_before_anyone_else_acts() {
    cr!("702.81a", "117.3b", "601.2a");
    ruling!(
        "Decaying Time Loop",
        "If the active player casts a spell that has retrace, that player may cast that card again after it resolves, before another player can remove the card from the graveyard. The active player has priority after the spell resolves, so they can immediately cast a new spell. Since casting a card with retrace from the graveyard moves that card onto the stack, no one else would have the chance to affect it while it's still in the graveyard."
    );
    supported("Decaying Time Loop");
    // Decaying Time Loop ({3}{R} instant: "Discard all the cards in your hand, then draw
    // that many cards.") is in P0's graveyard; P0 holds two Forests, with a Forest on top
    // of the library. P0 casts it with retrace (discarding a Forest), discards the other
    // and draws the third, and casts it again as soon as it has resolved: P1 never has
    // priority while it's in P0's graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    t.graveyard(P0, "Decaying Time Loop");
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    t.library_top(P0, "Forest");
    let in_graveyard = |g: &mtg_engine::game::Game| {
        g.player(P0)
            .graveyard
            .iter()
            .any(|c| g.obj(*c).chars.name == "Decaying Time Loop")
    };
    let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let p1_saw = watch(&mut t, P1, is_priority, in_graveyard);
    crate::r_s03_common::respond(&mut t, P0, retrace_time_loop);
    // The first cast, from P0's first priority in the main phase; then the game runs
    // until the second one has resolved too and the card is back in the graveyard.
    let ok = t.g.run_until(2000, |g| {
        g.history.spells_cast.len() == 2 && g.stack.is_empty()
    });
    assert!(ok, "Decaying Time Loop was cast twice");
    assert!(t.in_graveyard(P0, "Decaying Time Loop"));
    // P1 had priority only while it was on the stack.
    let seen = p1_saw.lock().unwrap().clone();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|x| !x), "{seen:?}");
}

#[test]
fn a_retrace_spell_goes_back_to_the_graveyard_when_countered_or_resolved() {
    cr!("702.81a", "608.2n", "701.6a");
    ruling!(
        "Decaying Time Loop",
        "When a retrace spell you cast from your graveyard resolves, fails to resolve, or is countered, it's put back into your graveyard. You may use the retrace ability to cast it again."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    let fl = t.graveyard(P0, "Decaying Time Loop");
    let f1 = t.hand(P0, "Forest");
    let f2 = t.hand(P0, "Forest");
    // Cast with retrace (discarding a Forest) and countered: back in the graveyard.
    t.answer_choose(P0, &[Entity::Object(f1)]);
    let spell = t.cast(P0, fl).method(RETRACE).go();
    cast_from_hand(&mut t, P1, "Counterspell", &[Entity::Object(spell)]);
    t.resolve_all();
    assert!(graveyard_names(&t, P0).contains(&"Decaying Time Loop".to_string()));
    // Cast with retrace again (discarding the other Forest), and it resolves: back in
    // the graveyard again.
    let fl = t.g.current(fl);
    t.answer_choose(P0, &[Entity::Object(f2)]);
    t.cast(P0, fl).method(RETRACE).go();
    t.resolve_all();
    assert_eq!(t.zone(fl), Zone::Graveyard(P0));
    assert!(t.in_graveyard(P0, "Decaying Time Loop"));
}
