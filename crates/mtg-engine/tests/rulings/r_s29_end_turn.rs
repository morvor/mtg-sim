//! Rulings batch S29 — ending the turn (CR 724.1): spells and abilities exiled from the
//! stack aren't countered (CR 701.6a); "Whenever a spell you've cast is countered"
//! (Multani's Presence, CR 603.10e) and spells that don't resolve for illegal targets
//! (CR 608.2b); "the next end step" after the end step was skipped (CR 603.7b).

use crate::r_s01_common::supported;
use crate::r_s04_common::add_mana;
use crate::r_s08_common::mana_value;
use crate::r_s25_common::cast_new;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts Discontinuity ("During your turn, this spell costs {2}{U}{U} less to cast.
/// End the turn.") on P0's turn, paying {1}{U}, and it resolves.
fn discontinuity(t: &mut TestGame) -> ObjectId {
    supported("Discontinuity");
    add_mana(t, P0, ManaType::U, 1);
    add_mana(t, P0, ManaType::C, 1);
    let d = t.hand(P0, "Discontinuity");
    let spell = t.cast(P0, d).go();
    t.resolve();
    spell
}

#[test]
fn spells_exiled_as_the_turn_ends_arent_countered() {
    cr!("724.1b", "701.6a", "603.10e");
    ruling!(
        "Discontinuity",
        "Though other spells and abilities that are exiled won't get a chance to resolve, they don't count as being countered."
    );
    supported("Multani's Presence");
    // Multani's Presence (P0): "Whenever a spell you've cast is countered, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Multani's Presence");
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    let hand = t.hand_size(P0);
    discontinuity(&mut t);
    assert!(t.g.stack.is_empty());
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Exile);
    // The turn ends (through the cleanup step): Multani's Presence never triggered.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), hand);
    // A spell that is countered does make it trigger.
    t.set_step(P0, Step::PrecombatMain);
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_spell_that_fizzles_isnt_countered() {
    cr!("608.2b", "701.6a");
    ruling!(
        "Multani's Presence",
        "A spell that doesn’t resolve because its targets are illegal isn’t countered, so this ability won’t trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Multani's Presence");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shock = cast_new(&mut t, P0, "Shock", &[Entity::Object(bears)]);
    // The target leaves before Shock resolves.
    crate::r_s02_common::destroy(&mut t, bears);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.zone(shock), mtg_engine::object::Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn discontinuitys_cost_reduction_doesnt_change_its_mana_value() {
    cr!("601.2f", "202.3");
    ruling!(
        "Discontinuity",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Discontinuity). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    let mut t = TestGame::new(2);
    // On P0's turn it costs {1}{U}: two mana is enough.
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    let d = t.hand(P0, "Discontinuity");
    let spell = t.cast(P0, d).go();
    assert_eq!(mana_value(&t, spell), 6);
    // On P1's turn, P0 can't cast it for two mana.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::U, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    let d = t.hand(P0, "Discontinuity");
    assert!(t.cast(P0, d).try_go().is_err());
}

#[test]
fn a_next_end_step_trigger_waits_for_the_next_turns_end_step() {
    cr!("724.1d", "724.1e", "603.7b");
    ruling!(
        "Discontinuity",
        "Any “at the beginning of the next end step” triggered abilities won't get the chance to trigger that turn because the end step is skipped. Those abilities will trigger at the beginning of the end step of the next turn."
    );
    supported("Sneak Attack");
    // Sneak Attack puts Grizzly Bears onto the battlefield: "Sacrifice the creature at the
    // beginning of the next end step." Then P0 ends the turn.
    let mut t = TestGame::new(2);
    let sneak = t.battlefield(P0, "Sneak Attack");
    t.lands(P0, "Mountain", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, sneak, 0, &[]).unwrap();
    t.resolve();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    discontinuity(&mut t);
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.on_battlefield(bears), "the end step was skipped");
    // P1's end step is the next one: it's sacrificed then.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_cast_copy_of_a_card_is_a_spell_youve_cast_a_copied_spell_isnt() {
    cr!("707.12", "707.10", "701.6a");
    supported("Isochron Scepter");
    supported("Twincast");
    // Multani's Presence (P0): "Whenever a spell you've cast is countered, draw a card."
    // P0 casts a copy of the Lightning Bolt imprinted on Isochron Scepter; P1 counters
    // it: it was cast, so P0 draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Multani's Presence");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    add_mana(&mut t, P0, ManaType::C, 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s06_common::activate_containing(&mut t, P0, scepter, "copy").unwrap();
    t.resolve();
    let copy = crate::r_s04_common::top_of_stack(&t);
    assert!(t.g.obj(copy).kind == mtg_engine::object::ObjKind::CardCopy);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(copy)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "the copy was countered");
    assert_eq!(t.hand_size(P0), hand + 1);
    // A copy of a spell put onto the stack by Twincast ("Copy target instant or sorcery
    // spell.") wasn't cast: countering it doesn't make P0 draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Multani's Presence");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(bolt)]);
    t.resolve();
    let copies = crate::r_s25_common::spell_copies(&t);
    assert_eq!(copies.len(), 1);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(copies[0])]);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
