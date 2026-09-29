//! Rulings batch S28 — countering (CR 701.6) and what isn't countering: replacement
//! effects never use the stack (CR 614.12), countered delayed triggers don't trigger again
//! (CR 603.7b), returning a spell to hand or exiling it isn't countering it, "can't be
//! countered" spells can still be targeted (CR 101.2), countered Beacons go to the
//! graveyard, a modal spell's mode stays when its target changes (CR 115.7), ending the
//! turn (CR 724.1), and cycling abilities and their triggers are separate (CR 702.29c).

use crate::r_s01_common::supported;
use crate::r_s02_common::target_candidates;
use crate::r_s04_common::{cycle, top_of_stack};
use crate::r_s18_common::unearthed;
use crate::r_s21_common::castable;
use crate::r_s28_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts Pterafractyl ("This creature enters with X +1/+1 counters on it. When this
/// creature enters, you gain 2 life.") with X = 2 and it resolves; its enters trigger is
/// left on the stack.
fn pterafractyl(t: &mut TestGame) -> ObjectId {
    supported("Pterafractyl");
    t.lands(P0, "Wastes", 2);
    crate::r_s25_common::lands_for_cost(t, P0, "Pterafractyl");
    let card = t.hand(P0, "Pterafractyl");
    t.cast(P0, card).x(2).go();
    t.resolve();
    let ptera = t.g.current(card);
    assert!(t.on_battlefield(ptera));
    ptera
}

#[test]
fn a_permanents_enters_with_counters_replacement_cant_be_targeted() {
    cr!("614.1c", "614.12", "115.1");
    ruling!(
        "Voidslime",
        "Abilities that create replacement effects, such as a permanent entering the battlefield tapped or with counters on it, can't be targeted. Abilities that apply \"as [this creature] enters the battlefield\" are also replacement effects and can't be targeted."
    );
    supported("Voidslime");
    let mut t = TestGame::new(2);
    let ptera = pterafractyl(&mut t);
    // It already has its counters; the only ability on the stack is the enters trigger.
    assert_eq!(t.counters(ptera, "+1/+1"), 2);
    assert_eq!(t.stack_len(), 1);
    let trigger = top_of_stack(&t);
    let from = t.asked().len();
    t.answer_targets(P1, &[Entity::Object(trigger)]);
    cast_card(&mut t, P1, "Voidslime");
    let offered = target_candidates(&t, P1, from);
    assert_eq!(offered, vec![vec![Entity::Object(trigger)]]);
    t.resolve_all();
    // Countering the trigger doesn't undo the counters.
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(ptera, "+1/+1"), 2);
    assert_eq!(t.pt(ptera), (3, 2));
}

#[test]
fn a_permanents_enters_with_counters_replacement_cant_be_countered() {
    cr!("614.1c", "614.12", "700.2");
    ruling!(
        "Sublime Epiphany",
        "Abilities that create replacement effects, such as a permanent entering the battlefield tapped or with counters on it, can't be countered. Abilities that apply \"as [this creature] enters the battlefield\" are also replacement effects and can't be countered."
    );
    supported("Sublime Epiphany");
    let mut t = TestGame::new(2);
    let ptera = pterafractyl(&mut t);
    let trigger = top_of_stack(&t);
    // "• Counter target activated or triggered ability."
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P1, &[Entity::Object(trigger)]);
    cast_card(&mut t, P1, "Sublime Epiphany");
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(ptera, "+1/+1"), 2);
}

#[test]
fn a_countered_next_end_step_trigger_doesnt_trigger_again() {
    cr!("603.7b", "701.6a");
    ruling!(
        "Voidslime",
        "If you counter a delayed triggered ability that triggered at the beginning of the \"next\" occurrence of a specified step or phase, that ability won't trigger again the following time that phase or step occurs."
    );
    // Dregscape Zombie's unearth: "... Exile it at the beginning of the next end step ..."
    let mut t = TestGame::new(2);
    let zombie = unearthed(&mut t, P0, "Dregscape Zombie", "{B}");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trigger = top_of_stack(&t);
    t.answer_targets(P1, &[Entity::Object(trigger)]);
    cast_card(&mut t, P1, "Voidslime");
    t.resolve_all();
    assert!(t.on_battlefield(zombie));
    // The next end steps: nothing triggers, and the Zombie stays.
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(zombie));
}

/// P1 casts Abrupt Decay ("This spell can't be countered. Destroy target nonland
/// permanent with mana value 3 or less.") at P0's Grizzly Bears.
fn decay_the_bears(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P1, &[Entity::Object(bears)]);
    let decay = cast_card(t, P1, "Abrupt Decay");
    (bears, decay)
}

#[test]
fn returning_an_uncounterable_spell_to_hand_isnt_countering_it() {
    cr!("701.6a", "400.7");
    ruling!(
        "Unsubstantiate",
        "If a spell is returned to its owner's hand, it's removed from the stack and thus will not resolve. The spell isn't countered; it just no longer exists. This works against a spell that can't be countered."
    );
    supported("Unsubstantiate");
    supported("Abrupt Decay");
    let mut t = TestGame::new(2);
    let (bears, decay) = decay_the_bears(&mut t);
    // Counterspell can target it but doesn't counter it.
    t.answer_targets(P0, &[Entity::Object(decay)]);
    cast_card(&mut t, P0, "Counterspell");
    t.resolve();
    assert_eq!(t.zone(decay), Zone::Stack);
    // Unsubstantiate: "Return target spell or creature to its owner's hand."
    t.answer_targets(P0, &[Entity::Object(decay)]);
    cast_card(&mut t, P0, "Unsubstantiate");
    t.resolve_all();
    assert!(t.in_hand(P1, "Abrupt Decay"));
    assert!(!t.in_graveyard(P1, "Abrupt Decay"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn target_spell_or_creature_includes_creatures_on_the_battlefield() {
    cr!("115.1a");
    // Unsubstantiate: "Return target spell or creature to its owner's hand." — a spell on
    // the stack or a creature permanent.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let bolt = cast_card(&mut t, P1, "Lightning Bolt");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_card(&mut t, P0, "Unsubstantiate");
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&Entity::Object(bolt)));
    assert!(offered[0].contains(&Entity::Object(bears)));
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn venser_returns_a_spell_that_cant_be_countered() {
    cr!("701.6a", "603.3");
    ruling!(
        "Venser, Shaper Savant",
        "If a spell is returned to its owner's hand, it's removed from the stack and thus will not resolve. The spell isn't countered; it just no longer exists. This works even against a spell that can't be countered."
    );
    supported("Venser, Shaper Savant");
    // Flash; "When Venser enters, return target spell or permanent to its owner's hand."
    let mut t = TestGame::new(2);
    let (bears, decay) = decay_the_bears(&mut t);
    cast_card(&mut t, P0, "Venser, Shaper Savant");
    t.answer_targets(P0, &[Entity::Object(decay)]);
    t.resolve_all();
    assert!(t.in_hand(P1, "Abrupt Decay"));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.named_on_battlefield("Venser, Shaper Savant").len(), 1);
}

#[test]
fn a_counterspells_other_effects_happen_to_an_uncounterable_spell() {
    cr!("101.2", "701.6a");
    ruling!(
        "Chimil, the Inner Sun",
        "A spell or ability that counters spells can still target spells that can't be countered. When that spell or ability resolves, the uncounterable spell won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    supported("Chimil, the Inner Sun");
    supported("Exclude");
    // Chimil: "Spells you control can't be countered." Exclude: "Counter target creature
    // spell. Draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chimil, the Inner Sun");
    let bears = cast_card(&mut t, P0, "Grizzly Bears");
    let hand = t.hand_size(P1);
    let exclude = t.hand(P1, "Exclude");
    crate::r_s25_common::lands_for_cost(&mut t, P1, "Exclude");
    assert!(castable(&mut t, P1, exclude));
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.cast(P1, exclude).go();
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Stack);
    assert_eq!(t.hand_size(P1), hand + 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn a_countered_or_fizzled_beacon_goes_to_the_graveyard() {
    cr!("701.6a", "608.2b");
    ruling!(
        "Beacon of Destruction",
        "If a Beacon is countered or doesn’t resolve, it’s put into its owner’s graveyard, not shuffled into the library."
    );
    supported("Beacon of Destruction");
    // "Beacon of Destruction deals 5 damage to any target. Shuffle Beacon of Destruction
    // into its owner's library."
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let beacon = cast_card(&mut t, P0, "Beacon of Destruction");
    t.answer_targets(P1, &[Entity::Object(beacon)]);
    cast_card(&mut t, P1, "Counterspell");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Beacon of Destruction"));
    assert_eq!(t.life(P1), 20);
    // Its only target is gone: it doesn't resolve, and goes to the graveyard too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_card(&mut t, P0, "Beacon of Destruction");
    t.answer_targets(P1, &[Entity::Object(bears)]);
    cast_card(&mut t, P1, "Unsubstantiate");
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Beacon of Destruction"));
    // Resolving, it's shuffled into the library.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_card(&mut t, P0, "Beacon of Destruction");
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert!(!t.in_graveyard(P0, "Beacon of Destruction"));
    assert!(!t
        .g
        .find_in_zone(Zone::Library(P0), "Beacon of Destruction")
        .is_empty());
}

#[test]
fn blue_elemental_blasts_mode_stays_when_its_target_changes() {
    cr!("700.2", "115.7d");
    ruling!(
        "Blue Elemental Blast",
        "The decision to counter a spell or destroy a permanent is a decision made on announcement before the target is selected. If the spell is redirected, this mode can't be changed, so only targets of the selected type are valid."
    );
    supported("Blue Elemental Blast");
    supported("Redirect");
    let mut t = TestGame::new(2);
    let piker = t.battlefield(P1, "Goblin Piker");
    let raging = t.battlefield(P1, "Raging Goblin");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let bolt = cast_card(&mut t, P1, "Lightning Bolt");
    // "• Destroy target red permanent." at Goblin Piker.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[Entity::Object(piker)]);
    let blast = cast_card(&mut t, P0, "Blue Elemental Blast");
    // Redirect: "You may choose new targets for target spell."
    t.answer_targets(P1, &[Entity::Object(blast)]);
    cast_card(&mut t, P1, "Redirect");
    let from = t.asked().len();
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Object(raging)]);
    t.resolve();
    // Only red permanents were offered as new targets, not the red spell.
    // The only other red permanent was offered as a new target, not the red spell.
    let offered = target_candidates(&t, P1, from);
    assert_eq!(offered, vec![vec![Entity::Object(raging)]]);
    assert_eq!(t.zone(bolt), Zone::Stack);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Raging Goblin"));
    assert!(t.on_battlefield(piker));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn spells_exiled_by_ending_the_combat_phase_arent_countered() {
    cr!("724.2", "701.6a");
    ruling!(
        "Mandate of Peace",
        "Though other spells and abilities that are exiled won’t get a chance to resolve, they don’t count as being countered."
    );
    supported("Mandate of Peace");
    // "Cast this spell only during combat. Your opponents can't cast spells this turn. End
    // the combat phase."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    cast_card(&mut t, P1, "Shock");
    cast_card(&mut t, P0, "Mandate of Peace");
    t.resolve();
    // The Shock was exiled, not countered (a countered spell goes to the graveyard).
    assert!(t.g.stack.is_empty());
    assert!(t.in_exile("Shock"));
    assert!(!t.in_graveyard(P1, "Shock"));
    assert!(t.in_exile("Mandate of Peace"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn ending_the_turn_exiles_the_stack_removes_attackers_and_goes_to_cleanup() {
    cr!("724.1b", "724.1d", "514.1");
    ruling!(
        "Discontinuity",
        "Ending the turn this way means the following things happen in order: 1) All spells and abilities on the stack are exiled. This includes spells and abilities that can't be countered. 2) If there are any attacking and blocking creatures, they're removed from combat. 3) State-based actions are checked. No player gets priority, and no triggered abilities are put onto the stack. 4) The current phase and/or step ends. The game skips straight to the cleanup step. 5) The cleanup step happens in its entirety."
    );
    supported("Discontinuity");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // Damage marked on a creature wears off in the cleanup step.
    crate::r_s06_common::damage(&mut t, bears, 2, giant);
    for _ in 0..9 {
        t.hand(P0, "Grizzly Bears");
    }
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    // Abrupt Decay (can't be countered) is exiled too.
    t.answer_targets(P1, &[Entity::Object(bears)]);
    cast_card(&mut t, P1, "Abrupt Decay");
    // During P0's turn it costs {1}{U}.
    t.lands(P0, "Island", 2);
    let disc = t.hand(P0, "Discontinuity");
    t.cast(P0, disc).go();
    t.resolve();
    assert!(t.g.stack.is_empty());
    assert!(t.in_exile("Abrupt Decay"));
    assert!(t.in_exile("Discontinuity"));
    assert!(t.g.combat.as_ref().is_none_or(|c| c.attackers.is_empty()));
    t.advance_to(P1, Step::Upkeep);
    // The cleanup step: P0 discarded to seven cards, damage wore off; no combat damage.
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn a_cycling_ability_and_its_cycle_trigger_are_separate() {
    cr!("702.29a", "702.29c", "603.3d");
    ruling!(
        "Snare Tactician",
        "You can cycle a card even if it has a triggered ability from cycling that won’t have a legal target. This is because the cycling ability and the triggered ability are separate. This also means that if either ability is countered (with Disallow, for example), the other ability will still resolve."
    );
    supported("Snare Tactician");
    supported("Disallow");
    // "Whenever you cycle a card, tap target creature an opponent controls."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Snare Tactician");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Drannith Stinger");
    let hand = t.hand_size(P0);
    // No creature to target: the card can still be cycled.
    cycle(&mut t, P0, card, 0).expect("cycle with no target for the trigger");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Drannith Stinger"));
    // The cycling ability is countered: the trigger still resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Snare Tactician");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Drannith Stinger");
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cycle(&mut t, P0, card, 0).expect("cycle");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let cycling = t.g.stack[0];
    t.answer_targets(P1, &[Entity::Object(cycling)]);
    cast_card(&mut t, P1, "Disallow");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1);
    assert!(t.obj_now(bears).tapped);
}
