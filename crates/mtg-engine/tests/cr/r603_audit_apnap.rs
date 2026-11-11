//! CR 603.3a–b audit: who controls a triggered ability and in which order abilities go on
//! the stack. A triggered ability is controlled by the player who controlled its source
//! when it triggered — not when it's put on the stack, and not whoever controls the
//! source later; abilities go on the stack in APNAP order starting with the active player
//! (in a multiplayer game, in turn order from the active player), and abilities that
//! trigger on another ability triggering go on the stack after all the others.

use crate::r603_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn triggers_go_on_the_stack_in_turn_order_from_the_active_player() {
    cr!("603.3b", "101.4");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P1, "Soul Warden");
    t.battlefield(P2, "Soul Warden");
    t.enter(P2, "Grizzly Bears");
    t.settle();
    // P1 is active: P1's goes on the stack first, then P2's, then P0's (on top).
    assert_eq!(trigger_controllers(&t, "Soul Warden"), vec![P1, P2, P0]);
    // The last one put on the stack resolves first.
    t.resolve();
    assert_eq!(
        (t.life(P0), t.life(P1), t.life(P2)),
        (21, 20, 20),
        "P0's resolved first"
    );
}

#[test]
fn a_stolen_creatures_dies_ability_belongs_to_the_player_who_controlled_it_then() {
    cr!("603.3a", "603.10a");
    supported("Doomed Traveler");
    let mut t = TestGame::new(2);
    let traveler = t.battlefield(P1, "Doomed Traveler");
    t.lands(P0, "Island", 4);
    let cm = t.hand(P0, "Control Magic");
    t.cast(P0, cm).target(traveler).go();
    t.resolve();
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(traveler).go();
    t.resolve();
    assert_eq!(trigger_controllers(&t, "Doomed Traveler"), vec![P0]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Doomed Traveler"), "its owner's graveyard");
    assert_eq!(tokens_named(&t, P0, "Spirit"), 1);
    assert_eq!(tokens_named(&t, P1, "Spirit"), 0);
}

#[test]
fn an_ability_triggered_before_its_source_changed_control_in_the_same_resolution() {
    cr!("603.3a", "608.2c");
    supported("Oreskos Sun Guide");
    let mut t = TestGame::new(2);
    let guide = t.battlefield(P1, "Oreskos Sun Guide");
    t.g.tap(guide);
    t.lands(P0, "Mountain", 3);
    let threaten = t.hand(P0, "Threaten");
    // "Untap target creature and gain control of it": it becomes untapped while P1
    // controls it, so P1 controls the inspired ability, though P0 controls the creature
    // when the ability goes on the stack.
    t.cast(P0, threaten).target(guide).go();
    t.resolve();
    assert_eq!(t.obj_now(guide).controller, P0);
    assert_eq!(trigger_controllers(&t, "Oreskos Sun Guide"), vec![P1]);
    t.resolve_all();
    assert_eq!(t.life(P1), 22);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_triggered_ability_keeps_its_controller_when_its_source_is_stolen_in_response() {
    cr!("603.3a", "113.7a");
    supported("Ray of Command");
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(trigger_controllers(&t, "Blood Artist"), vec![P0]);
    // In response, P1 takes the Blood Artist.
    t.lands(P1, "Island", 4);
    let ray = t.hand(P1, "Ray of Command");
    t.cast(P1, ray).target(artist).go();
    t.resolve();
    assert_eq!(t.obj_now(artist).controller, P1);
    assert_eq!(trigger_controllers(&t, "Blood Artist"), vec![P0]);
    t.resolve();
    assert_eq!(t.life(P0), 21, "P0 still controls the ability: \"you gain 1 life\"");
    assert_eq!(t.life(P1), 19);
}

#[test]
fn abilities_triggering_on_an_ability_triggering_go_above_every_player_s_abilities() {
    cr!("603.3b");
    ruling!(
        "Strict Proctor",
        "Abilities that trigger when something else causes an ability to trigger will always go on the stack after the ability that caused them to trigger."
    );
    supported("Strict Proctor");
    let mut t = TestGame::new(2);
    // The active player controls the Proctor; both players control a Soul Warden.
    t.battlefield(P0, "Strict Proctor");
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P1, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    let names: Vec<(PlayerId, String)> = stack_triggers(&t)
        .into_iter()
        .map(|(_, p, n)| (p, n))
        .collect();
    assert_eq!(
        names,
        vec![
            (P0, "Soul Warden".to_string()),
            (P1, "Soul Warden".to_string()),
            (P0, "Strict Proctor".to_string()),
            (P0, "Strict Proctor".to_string()),
        ],
        "P0's Proctor abilities go on the stack after P1's Warden ability"
    );
    // Nobody can pay {2}: both Warden abilities are countered.
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn abilities_triggering_in_the_untap_step_go_on_the_stack_with_the_upkeep_ones() {
    cr!("603.3", "502.4", "503.1a");
    ruling!(
        "Oreskos Sun Guide",
        "If an inspired ability triggers during your untap step, the ability will be put on the stack at the beginning of your upkeep."
    );
    supported("Phyrexian Arena");
    let mut t = TestGame::new(2);
    let guide = t.battlefield(P0, "Oreskos Sun Guide");
    t.battlefield(P0, "Phyrexian Arena");
    t.g.tap(guide);
    t.advance_to(P1, Step::PrecombatMain);
    // The Guide stays tapped through P1's turn.
    assert!(t.obj_now(guide).tapped);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(guide).tapped);
    t.settle();
    assert_eq!(triggers_of(&t, "Oreskos Sun Guide"), 1);
    assert_eq!(triggers_of(&t, "Phyrexian Arena"), 1);
}
