//! Rulings batch P140 — removing things from the stack and from combat, and ending the
//! turn: Reconnaissance, Sundial of the Infinite, Obeka, Brute Chronologist and Nivmagus
//! Elemental.

use crate::r_p140_common::*;
use crate::r_s09_common::declare;
use mtg_engine::decision::Answer;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn reconnaissance_doesnt_undo_attack_triggers() {
    cr!("506.4", "603.2");
    ruling!(
        "Reconnaissance",
        "Does not undo any effects which triggered on declaration of attackers or blockers."
    );
    supported("Reconnaissance");
    supported("Roxanne, Starfall Savant");
    let mut t = TestGame::new(2);
    let recon = t.battlefield(P0, "Reconnaissance");
    let rox = t.battlefield(P0, "Roxanne, Starfall Savant");
    declare(&mut t, P0, &[(rox, Entity::Player(P1))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Meteorite"), 1, "the attack trigger");
    activate_lots(&mut t, P0, recon, 0, &[obj(rox)]);
    t.resolve();
    assert!(!t.obj(rox).tapped, "untapped");
    assert!(crate::r_s09_common::attacks_now(&t).is_empty(), "removed from combat");
    t.resolve_all();
    assert_eq!(controlled_named(&t, P0, "Meteorite").len(), 1, "the trigger still resolves");
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn sundial_ends_the_turn_in_order_exiling_even_uncounterable_spells() {
    cr!("724.1", "724.1b", "724.1c", "724.1d", "724.1e");
    ruling!(
        "Sundial of the Infinite",
        "Ending the turn this way means the following things happen in order: 1) All spells and abilities on the stack are exiled."
    );
    ruling!(
        "Sundial of the Infinite",
        "Though spells and abilities that are exiled won't get a chance to resolve, they don't count as being \"countered.\""
    );
    supported("Sundial of the Infinite");
    let mut t = TestGame::new(2);
    let dial = t.battlefield(P0, "Sundial of the Infinite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    crate::r_s03_common::to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    // P1 casts Abrupt Decay (can't be countered) at the Sundial; P0 ends the turn.
    let decay = cast_lots(&mut t, P1, "Abrupt Decay", &[obj(dial)]);
    for _ in 0..9 {
        t.hand(P0, "Island");
    }
    activate_lots(&mut t, P0, dial, 0, &[]);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(zone_of(&t, decay), Zone::Exile, "exiled, not resolved");
    assert!(t.on_battlefield(dial));
    assert!(t.stack.is_empty());
    assert!(!anything_countered(&t));
    assert!(
        t.g.combat.as_ref().is_none_or(|c| c.attackers.is_empty()),
        "no attackers remain"
    );
    // No combat damage: the turn skips straight to the cleanup step, where P0 discards
    // down to seven.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj(t.g.current(bears)).damage, 0);
    assert!(t.on_battlefield(t.g.current(bears)));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn ending_the_turn_early_delays_next_end_step_triggers_to_the_next_turn() {
    cr!("724.1e", "603.7c");
    ruling!(
        "Sundial of the Infinite",
        "If Sundial of the Infinite's ability is activated before the end step, any \"at the beginning of the end step\"-triggered abilities won't get the chance to trigger that turn because the end step is skipped."
    );
    supported("Chrome Dome");
    let mut t = TestGame::new(2);
    let dial = t.battlefield(P0, "Sundial of the Infinite");
    let dome = t.battlefield(P0, "Chrome Dome");
    let thopter = t.battlefield(P0, "Ornithopter");
    activate_lots(&mut t, P0, dome, 0, &[obj(thopter)]);
    t.resolve_all();
    let copies = controlled_named(&t, P0, "Ornithopter");
    assert_eq!(copies.len(), 2);
    let token = *copies.iter().find(|c| t.obj(**c).is_token()).unwrap();
    activate_lots(&mut t, P0, dial, 0, &[]);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(token), "no end step this turn");
    // The next end step is P1's: the token is sacrificed then.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(token), "sacrificed at the next end step");
}

#[test]
fn obeka_ending_the_turn_after_next_end_step_triggers_doesnt_retrigger_them() {
    cr!("724.1b", "603.7c");
    ruling!(
        "Obeka, Brute Chronologist",
        "If the turn ends during the end step after \"At the beginning of the next end step\" abilities have triggered but before they've resolved, those abilities won't trigger again next turn."
    );
    supported("Obeka, Brute Chronologist");
    let mut t = TestGame::new(2);
    let obeka = t.battlefield(P0, "Obeka, Brute Chronologist");
    let dome = t.battlefield(P0, "Chrome Dome");
    let thopter = t.battlefield(P0, "Ornithopter");
    activate_lots(&mut t, P0, dome, 0, &[obj(thopter)]);
    t.resolve_all();
    let token = *controlled_named(&t, P0, "Ornithopter")
        .iter()
        .find(|c| t.obj(**c).is_token())
        .unwrap();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the delayed sacrifice trigger");
    t.answer_yes(P0, true);
    activate_lots(&mut t, P0, obeka, 0, &[]);
    t.resolve();
    assert!(t.stack.is_empty(), "the turn ended: the trigger was exiled");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    t.advance_to(P0, Step::Upkeep);
    assert!(t.on_battlefield(token), "never sacrificed");
}

#[test]
fn sundial_earliest_in_your_upkeep_after_upkeep_triggers() {
    cr!("502.4", "503.1", "724.1b");
    ruling!(
        "Sundial of the Infinite",
        "The earliest that you can activate Sundial of the Infinite's ability is during your upkeep step, after abilities that trigger \"at the beginning of your upkeep\" have been put onto the stack but before they resolve."
    );
    let mut t = TestGame::new(2);
    let dial = t.battlefield(P0, "Sundial of the Infinite");
    t.battlefield(P0, "Phyrexian Arena");
    crate::r_p141_common::into_upkeep(&mut t, P0);
    // The first time P0 gets priority in its turn, the Arena's trigger is on the stack.
    assert_eq!(t.g.turn.step, Step::Upkeep);
    assert_eq!(t.stack_len(), 1);
    lots(&mut t, P0);
    assert!(can_activate(&mut t, P0, dial));
    activate_lots(&mut t, P0, dial, 0, &[]);
    t.resolve();
    assert!(t.stack.is_empty());
    assert_eq!(t.life(P0), 20, "the Arena's trigger never resolved");
}

#[test]
fn nivmagus_exiles_an_uncounterable_spell_without_countering_it() {
    cr!("601.2h", "118.3");
    ruling!(
        "Nivmagus Elemental",
        "Exiling a spell is not the same as countering it, although the spell won’t resolve in both cases. You may exile a spell that can’t be countered."
    );
    supported("Nivmagus Elemental");
    let mut t = TestGame::new(2);
    let niv = t.battlefield(P0, "Nivmagus Elemental");
    let stone = t.battlefield(P1, "Millstone");
    let decay = cast_lots(&mut t, P0, "Abrupt Decay", &[obj(stone)]);
    t.answer_choose(P0, &[obj(decay)]);
    activate_lots(&mut t, P0, niv, 0, &[]);
    assert_eq!(zone_of(&t, decay), Zone::Exile, "exiled as a cost");
    t.resolve_all();
    assert_eq!(t.counters(niv, counters::PLUS1), 2);
    assert!(t.on_battlefield(stone), "Abrupt Decay didn't resolve");
    assert!(!anything_countered(&t));
}

#[test]
fn nivmagus_can_exile_a_copy_you_control_but_not_cards() {
    cr!("707.10", "118.3");
    ruling!(
        "Nivmagus Elemental",
        "You can exile a copy of an instant or sorcery spell to activate Nivmagus Elemental’s ability if you control the copy."
    );
    ruling!(
        "Nivmagus Elemental",
        "You exile an instant or sorcery spell you control from the stack, not an instant or sorcery card in any other zone."
    );
    // Cards in hand and graveyard don't count.
    let mut t = TestGame::new(2);
    let niv = t.battlefield(P0, "Nivmagus Elemental");
    t.hand(P0, "Lightning Bolt");
    t.graveyard(P0, "Lightning Bolt");
    lots(&mut t, P0);
    assert!(!can_activate(&mut t, P0, niv));
    // A copy of Lightning Bolt that P0 controls.
    let bolt = cast_lots(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.answer_yes(P0, false);
    cast_lots(&mut t, P0, "Twincast", &[obj(bolt)]);
    t.resolve();
    let copy = *t
        .stack
        .iter()
        .find(|s| t.obj(**s).kind == ObjKind::SpellCopy)
        .expect("a copy");
    assert_eq!(t.obj(copy).controller, P0);
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![obj(copy)]));
    activate_lots(&mut t, P0, niv, 0, &[]);
    assert!(!t.stack.contains(&copy), "the copy was exiled");
    t.resolve_all();
    assert_eq!(t.counters(niv, counters::PLUS1), 2);
    assert_eq!(t.life(P1), 17, "only the original Bolt resolved");
}
