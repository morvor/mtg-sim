//! Rulings batch P171 — copying spells and loyalty abilities: Jaya, Fiery Negotiator
//! (her emblem copies red instants and sorceries; her −2 and −1 abilities) and Chandra's
//! Regulator (copies loyalty abilities of Chandras).

use crate::r_p171_common::*;
use crate::r_s02_common::can_play_land;
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use crate::r_s11_common::triggered_from;
use crate::r_s21_common::castable;
use crate::r_s22_common::choose_named_when_offered;
use crate::r_s25_common::{
    abilities_from, change_copy_targets, keep_copy_targets, spell_copies, x_of,
};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// ---------------------------------------------------------------------------------
// Jaya's emblem: "Whenever you cast a red instant or sorcery spell, copy it twice. You
// may choose new targets for the copies."
// ---------------------------------------------------------------------------------

/// P0 gets Jaya, Fiery Negotiator's emblem (activating her −8).
fn jaya_emblem(t: &mut TestGame) {
    supported("Jaya, Fiery Negotiator");
    let jaya = t.battlefield(P0, "Jaya, Fiery Negotiator");
    t.g.add_counters(obj(jaya), counters::LOYALTY, 4, None);
    activate_containing(t, P0, jaya, "emblem").expect("−8");
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn jaya_emblem_copies_even_if_the_spell_was_countered_and_copies_resolve_first() {
    cr!("707.10", "603.3", "405.5");
    ruling!(
        "Jaya, Fiery Negotiator",
        "Copies are created even if the spell that caused the ability to trigger has been countered by the time that ability resolves. The copies resolve before the original spell."
    );
    // Not countered: the copies are put on the stack above the original.
    let mut t = TestGame::new(2);
    jaya_emblem(&mut t);
    let bolt = cast_targeting(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    assert_eq!(t.stack[0], bolt);
    assert_eq!(spell_copies(&t).len(), 2);
    t.resolve();
    t.resolve();
    // Both copies resolved; the original is still on the stack.
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.stack.as_slice(), &[bolt]);
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
    // Countered before the trigger resolves: the copies are still created.
    let mut t = TestGame::new(2);
    jaya_emblem(&mut t);
    let bolt = cast_targeting(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    t.answer_targets(P1, &[obj(bolt)]);
    cast_card(&mut t, P1, "Cancel");
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn jaya_emblem_copies_arent_cast() {
    cr!("707.10", "603.2");
    ruling!(
        "Jaya, Fiery Negotiator",
        "The copies are created on the stack, so they're not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    let mut t = TestGame::new(2);
    jaya_emblem(&mut t);
    let snipe = t.battlefield(P0, "Guttersnipe");
    cast_targeting(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(triggered_from(&t, snipe), 1);
    assert_eq!(t.life(P1), 20 - 9 - 2);
}

#[test]
fn jaya_emblem_copies_may_get_new_targets() {
    cr!("707.10c", "115.7");
    ruling!(
        "Jaya, Fiery Negotiator",
        "The copies will have the same targets as the spell it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    let mut t = TestGame::new(2);
    jaya_emblem(&mut t);
    let wall = t.battlefield(P1, "Indomitable Ancients");
    cast_targeting(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    // The first copy keeps P1; the second changes to the Ancients.
    keep_copy_targets(&mut t, P0);
    change_copy_targets(&mut t, P0, &[Some(obj(wall))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(damage_on(&t, wall), 3);
}

#[test]
fn jaya_emblem_copies_spells_without_targets() {
    cr!("707.10");
    ruling!(
        "Jaya, Fiery Negotiator",
        "The triggered ability of Jaya's emblem will copy any red instant or sorcery spell you cast, not just one with targets."
    );
    supported("Pyroclasm");
    let mut t = TestGame::new(2);
    jaya_emblem(&mut t);
    let wall = t.battlefield(P1, "Indomitable Ancients");
    // Pyroclasm: "Pyroclasm deals 2 damage to each creature."
    cast_card(&mut t, P0, "Pyroclasm");
    t.resolve_all();
    assert_eq!(damage_on(&t, wall), 6);
}

#[test]
fn jaya_emblem_copies_of_a_kicked_spell_are_kicked() {
    cr!("707.10", "702.33d");
    ruling!(
        "Jaya, Fiery Negotiator",
        "You can't choose to pay any additional costs for the copies. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copies too. Notably, if the spell you copy was kicked, the copies will also be kicked."
    );
    supported("Burst Lightning");
    // Burst Lightning: "Kicker {4}. Burst Lightning deals 2 damage to any target. If this
    // spell was kicked, it deals 4 damage instead."
    for kicked in [true, false] {
        let mut t = TestGame::new(2);
        jaya_emblem(&mut t);
        t.lands(P0, "Mountain", 1);
        if kicked {
            t.lands(P0, "Wastes", 4);
        }
        let card = t.hand(P0, "Burst Lightning");
        t.cast(P0, card).kicked(kicked).target(P1).go();
        // No additional cost is offered for the copies.
        let from = t.asked().len();
        keep_copy_targets(&mut t, P0);
        keep_copy_targets(&mut t, P0);
        t.resolve_all();
        assert!(!t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::OptionalCost { .. })));
        assert_eq!(t.life(P1), if kicked { 8 } else { 14 });
    }
}

// ---------------------------------------------------------------------------------
// Jaya's −2 and −1 abilities.
// ---------------------------------------------------------------------------------

#[test]
fn jaya_minus_two_triggers_each_attack_even_after_jaya_leaves() {
    cr!("603.7a", "603.7b", "113.7a");
    ruling!(
        "Jaya, Fiery Negotiator",
        "Activating Jaya's third loyalty ability creates a delayed triggered ability that will trigger each time you declare attackers this turn"
    );
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    let jaya = t.battlefield(P0, "Jaya, Fiery Negotiator");
    let wall = t.battlefield(P1, "Indomitable Ancients");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(wall)]);
    activate_containing(&mut t, P0, jaya, "Whenever you attack").expect("−2");
    t.resolve_all();
    // Jaya leaves the battlefield.
    move_to(&mut t, jaya, Zone::Graveyard(P0));
    let attack = vec![(bears, Entity::Player(P1)), (giant, Entity::Player(P1))];
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(attack.clone()),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    t.resolve_all();
    assert_eq!(damage_on(&t, wall), 2, "{}", t.dump_log());
    // The damage is dealt by Jaya (as she last existed).
    assert!(t.g.turn_events.iter().any(|e| matches!(
        e,
        Event::Damage { source, target, amount: 2, .. } if *source == jaya && *target == obj(wall)
    )));
    // A second combat this turn: it triggers again.
    t.advance_to(P0, Step::PostcombatMain);
    cast_card(&mut t, P0, "Relentless Assault");
    t.resolve_all();
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(attack));
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    t.resolve_all();
    assert_eq!(damage_on(&t, wall), 4);
}

#[test]
fn jaya_minus_one_card_is_played_following_normal_timing_rules() {
    cr!("305.2", "305.1", "601.3");
    ruling!(
        "Jaya, Fiery Negotiator",
        "You must pay all costs and follow all normal timing rules to play a card chosen for Jaya's second loyalty ability."
    );
    // −1: "Exile the top two cards of your library. Choose one of them. You may play that
    // card this turn."
    let setup = |t: &mut TestGame| -> ObjectId {
        let jaya = t.battlefield(P0, "Jaya, Fiery Negotiator");
        let forest = t.library_top(P0, "Forest");
        t.library_top(P0, "Grizzly Bears");
        choose_named_when_offered(t, P0, "Forest");
        activate_containing(t, P0, jaya, "Exile the top two").expect("−1");
        t.resolve_all();
        assert!(t.in_exile("Forest") && t.in_exile("Grizzly Bears"));
        t.g.current(forest)
    };
    // A land drop available, in a main phase with an empty stack: it can be played.
    let mut t = TestGame::new(2);
    let forest = setup(&mut t);
    assert!(can_play_land(&mut t, P0, forest));
    // The card that wasn't chosen can't be played.
    t.lands(P0, "Forest", 2);
    let bears = t.g.find_in_zone(Zone::Exile, "Grizzly Bears")[0];
    assert!(!castable(&mut t, P0, bears));
    // Not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, forest));
    // Not after playing a land this turn.
    let mut t = TestGame::new(2);
    let forest = setup(&mut t);
    let other = t.hand(P0, "Mountain");
    t.play_land(P0, other).expect("play Mountain");
    assert!(!can_play_land(&mut t, P0, forest));
}

// ---------------------------------------------------------------------------------
// Chandra's Regulator: "Whenever you activate a loyalty ability of a Chandra planeswalker,
// you may pay {1}. If you do, copy that ability. You may choose new targets for the copy.
// {1}, {T}, Discard a Mountain card or a red card: Draw a card."
// ---------------------------------------------------------------------------------

#[test]
fn chandras_regulator_a_mountain_card_has_the_subtype_mountain() {
    cr!("205.3i", "105.2c");
    ruling!(
        "Chandra's Regulator",
        "A Mountain card is a land card with the subtype Mountain, not any land card with a mana ability that produces red mana. Land cards are normally colorless, even if they produce red mana."
    );
    supported("Chandra's Regulator");
    // Dragonskull Summit (taps for {R}, no subtype, colorless): can't be discarded.
    let mut t = TestGame::new(2);
    let regulator = t.battlefield(P0, "Chandra's Regulator");
    t.lands(P0, "Wastes", 1);
    t.hand(P0, "Dragonskull Summit");
    assert!(activate_containing(&mut t, P0, regulator, "Draw a card").is_err());
    // Stomping Ground (a nonbasic Mountain Forest): it can.
    let ground = t.hand(P0, "Stomping Ground");
    t.answer_choose(P0, &[obj(ground)]);
    activate_containing(&mut t, P0, regulator, "Draw a card").expect("activate");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Stomping Ground"));
    assert!(t.in_hand(P0, "Dragonskull Summit"));
}

#[test]
fn chandras_regulator_copy_of_a_minus_x_ability_uses_the_same_x() {
    cr!("707.10", "606.4");
    ruling!(
        "Chandra's Regulator",
        "If the loyalty ability has −X in its cost, the copy uses the same value of X."
    );
    supported("Chandra Nalaar");
    // Chandra Nalaar: "−X: Chandra Nalaar deals X damage to target creature."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chandra's Regulator");
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    let wall = t.battlefield(P1, "Indomitable Ancients");
    t.lands(P0, "Wastes", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[obj(wall)]);
    activate_containing(&mut t, P0, chandra, "X damage").expect("−X");
    t.settle();
    t.answer_yes(P0, true);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let abs = abilities_from(&t, chandra);
    assert_eq!(abs.len(), 2);
    assert_eq!(x_of(&t, abs[1]), Some(2));
    t.resolve_all();
    assert_eq!(damage_on(&t, wall), 4);
}

#[test]
fn chandras_regulator_trigger_and_copy_resolve_first_even_if_countered() {
    cr!("603.3", "405.5", "707.10");
    ruling!(
        "Chandra's Regulator",
        "The triggered ability of Chandra’s Regulator and the copy it creates both resolve before the loyalty ability that caused it to trigger. They resolve even if that loyalty ability is countered."
    );
    supported("Stifle");
    // Chandra Nalaar: "+1: Chandra Nalaar deals 1 damage to target player or
    // planeswalker."
    for countered in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Chandra's Regulator");
        let chandra = t.battlefield(P0, "Chandra Nalaar");
        t.lands(P0, "Wastes", 1);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let ability = activate_containing(&mut t, P0, chandra, "deals 1 damage")
            .expect("+1")
            .expect("on the stack");
        t.settle();
        assert_eq!(t.stack[0], ability);
        assert_eq!(t.stack_len(), 2);
        if countered {
            t.answer_targets(P1, &[obj(ability)]);
            cast_card(&mut t, P1, "Stifle");
            t.resolve();
            assert!(!t.stack.contains(&ability));
        }
        t.answer_yes(P0, true);
        keep_copy_targets(&mut t, P0);
        t.resolve();
        // The copy is above the original (if it's still there).
        let abs = abilities_from(&t, chandra);
        assert_eq!(abs.len(), if countered { 1 } else { 2 });
        assert_ne!(*abs.last().unwrap(), ability);
        t.resolve();
        assert_eq!(t.life(P1), 19);
        t.resolve_all();
        assert_eq!(t.life(P1), if countered { 19 } else { 18 });
    }
}

#[test]
fn chandras_regulator_pays_only_once_per_resolution() {
    cr!("118.12", "603.5");
    ruling!(
        "Chandra's Regulator",
        "You can’t pay {1} more than once for each time the triggered ability of Chandra’s Regulator resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chandra's Regulator");
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    let lands = t.lands(P0, "Wastes", 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, chandra, "deals 1 damage").expect("+1");
    t.settle();
    let from = t.asked().len();
    t.answer_yes(P0, true);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    // One payment offered, one {1} paid, one copy — with mana left over.
    let offers = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(offers, 2, "the payment and the new-targets choice");
    assert_eq!(lands.iter().filter(|l| t.obj_now(**l).tapped).count(), 1);
    assert_eq!(abilities_from(&t, chandra).len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}
