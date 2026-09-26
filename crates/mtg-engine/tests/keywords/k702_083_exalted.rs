//! CR 702.83 Exalted.

use crate::common_k702_011_017::{assert_supported, attack_with};
use crate::common_k702_018_026::{declare_blocks, triggers_on_stack};
use crate::common_k702_052_066::run_effect;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn each_exalted_ability_pumps_a_creature_that_attacks_alone() {
    cr!("702.83", "702.83a");
    ruling!(
        "Rafiq of the Many",
        "Ultimately, the attacking creature will wind up with +1/+1 for each of your exalted abilities."
    );
    assert_supported("Akrasan Squire");
    assert_supported("Noble Hierarch");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akrasan Squire");
    t.battlefield(P0, "Noble Hierarch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.settle();
    // The abilities trigger as attackers are declared and resolve before blockers.
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(triggers_on_stack(&t, "Exalted"), 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn exalted_bonuses_last_until_end_of_turn() {
    cr!("702.83a");
    ruling!(
        "Akrasan Squire",
        "Exalted bonuses last until end of turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akrasan Squire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::End);
    assert_eq!(t.pt(bears), (3, 3));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn the_attacking_creatures_own_exalted_triggers() {
    cr!("702.83a");
    ruling!(
        "Noble Hierarch",
        "each exalted ability on each permanent you control (including, perhaps, the attacking creature itself) will trigger"
    );
    let mut t = TestGame::new(2);
    let squire = t.battlefield(P0, "Akrasan Squire");
    t.battlefield(P0, "Akrasan Squire");
    attack_with(&mut t, &[(squire, Entity::Player(P1))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Exalted"), 2);
    t.resolve_all();
    assert_eq!(t.pt(squire), (3, 3));
}

#[test]
fn exalted_doesnt_trigger_when_several_creatures_attack() {
    cr!("702.83a", "702.83b");
    ruling!(
        "Noble Hierarch",
        "You must attack with exactly one creature for exalted abilities to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akrasan Squire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (elves, Entity::Player(P1))],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Exalted"), 0);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn a_creature_left_alone_after_others_are_removed_didnt_attack_alone() {
    cr!("702.83b");
    ruling!(
        "Rafiq of the Many",
        "If you attack with multiple creatures, but then all but one are removed from combat, your exalted abilities won't trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akrasan Squire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (elves, Entity::Player(P1))],
    );
    run_effect(
        &mut t,
        None,
        P0,
        Effect::RemoveFromCombat {
            what: Sel::Target(0),
        },
        &[Entity::Object(elves)],
    );
    t.settle();
    assert!(!t.g.is_attacking(elves));
    assert_eq!(triggers_on_stack(&t, "Exalted"), 0);
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn creatures_put_onto_the_battlefield_attacking_are_ignored() {
    cr!("702.83b");
    ruling!(
        "Rafiq of the Many",
        "Since those creatures were never declared as attackers, they're ignored by exalted abilities."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akrasan Squire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.hand(P0, "Llanowar Elves");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Exalted"), 1);
    // In response, another creature is put onto the battlefield attacking: the exalted
    // ability still resolves.
    let mut to = Destination::battlefield();
    to.attacking = true;
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to,
        },
        &[Entity::Object(elves)],
    );
    assert!(t.g.is_attacking(t.g.current(elves)));
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(elves), (1, 1));
}

#[test]
fn an_opponents_exalted_doesnt_trigger() {
    cr!("702.83a");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Akrasan Squire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Exalted"), 0);
}

#[test]
fn exalted_triggers_again_for_a_creature_attacking_alone_in_an_additional_combat() {
    cr!("702.83a", "702.83b");
    ruling!(
        "Rafiq of the Many",
        "If a creature attacks alone during the second combat phase, all your exalted abilities will trigger again."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akrasan Squire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.pt(bears), (3, 3));
    // After this combat phase, there's an additional combat phase.
    run_effect(
        &mut t,
        None,
        P0,
        Effect::ExtraCombat { after_this: true },
        &[],
    );
    // Untap it and attack alone again.
    let now = t.g.current(bears);
    t.g.objects[now.0 as usize].tapped = false;
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Exalted"), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn in_two_headed_giant_only_the_attacking_creatures_controllers_exalted_triggers() {
    cr!("702.83a", "702.83b");
    ruling!(
        "Akrasan Squire",
        "If you control that attacking creature, your exalted abilities will trigger but your teammate’s exalted abilities won’t."
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.battlefield(P0, "Akrasan Squire");
    t.battlefield(P1, "Akrasan Squire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P2))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Exalted"), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
}
