//! The worked examples of CR 800 (multiplayer games: leaving the game), played out as
//! tests. Alex is P0, Bianca is P1; a third player (P2) keeps the game going after one of
//! them leaves (CR 800.4).

use super::r600_examples_common::supported;
use super::r800_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const ALEX: PlayerId = P0;
const BIANCA: PlayerId = P1;

fn ffa3() -> TestGame {
    TestGame::with_config(3, GameConfig::free_for_all())
}

/// Whether the object left the game (it's in no zone, and nothing follows it).
fn left_game(t: &TestGame, id: ObjectId) -> bool {
    let now = t.g.current(id);
    t.g.obj(now).zone == Zone::Nowhere && t.g.obj(now).next.is_none()
}

/// Alex casts Mind Control on Bianca's Assault Griffin.
fn mind_control_on_griffin() -> (TestGame, ObjectId, ObjectId) {
    supported("Mind Control");
    supported("Assault Griffin");
    let mut t = ffa3();
    let griffin = t.battlefield(BIANCA, "Assault Griffin");
    t.set_step(ALEX, Step::PrecombatMain);
    t.lands(ALEX, "Island", 5);
    let mc = t.hand(ALEX, "Mind Control");
    t.cast(ALEX, mc).target(griffin).go();
    t.resolve();
    let mc = t.g.current(mc);
    assert!(t.on_battlefield(mc));
    assert_eq!(t.obj_now(griffin).controller, ALEX);
    (t, mc, griffin)
}

#[test]
fn mind_control_alex_leaves_and_the_griffin_reverts_to_bianca() {
    cr!("800.4a");
    cr_example!("800.4a", "Alex casts Mind Control");
    let (mut t, mc, griffin) = mind_control_on_griffin();
    concede(&mut t, ALEX);
    t.settle();
    assert!(left_game(&t, mc), "Mind Control leaves the game with its owner");
    assert!(on_bf(&t, griffin));
    assert_eq!(t.obj_now(griffin).controller, BIANCA);
}

#[test]
fn mind_control_bianca_leaves_and_mind_control_goes_to_alexs_graveyard() {
    cr!("800.4a", "704.5m");
    cr_example!("800.4a", "Alex casts Mind Control");
    let (mut t, mc, griffin) = mind_control_on_griffin();
    concede(&mut t, BIANCA);
    assert!(left_game(&t, griffin), "Assault Griffin leaves the game with its owner");
    t.settle();
    assert_eq!(t.zone(mc), Zone::Graveyard(ALEX));
}

#[test]
fn act_of_treason_alex_leaves_and_the_bears_revert_to_bianca() {
    cr!("800.4a");
    cr_example!("800.4a", "Alex casts Act of Treason");
    supported("Act of Treason");
    supported("Runeclaw Bear");
    let mut t = ffa3();
    let bears = t.battlefield(BIANCA, "Runeclaw Bear");
    t.set_step(ALEX, Step::PrecombatMain);
    t.lands(ALEX, "Mountain", 3);
    let treason = t.hand(ALEX, "Act of Treason");
    t.cast(ALEX, treason).target(bears).go();
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, ALEX);
    concede(&mut t, ALEX);
    t.settle();
    assert!(on_bf(&t, bears), "the bears aren't exiled");
    assert_eq!(t.obj_now(bears).controller, BIANCA);
    // The change-of-control effect ended: it doesn't come back.
    to_turn_of(&mut t, BIANCA);
    assert_eq!(t.obj_now(bears).controller, BIANCA);
}

/// Alex casts Bribery targeting Bianca and puts her Serra Angel onto the battlefield.
fn bribery_for_serra_angel() -> (TestGame, ObjectId) {
    supported("Bribery");
    supported("Serra Angel");
    let mut t = ffa3();
    let angel = t.library_top(BIANCA, "Serra Angel");
    t.set_step(ALEX, Step::PrecombatMain);
    t.lands(ALEX, "Island", 5);
    let bribery = t.hand(ALEX, "Bribery");
    t.answer(ALEX, DecisionKind::Entities, Answer::Entities(vec![Entity::Object(angel)]));
    t.cast(ALEX, bribery).target(Entity::Player(BIANCA)).go();
    t.resolve();
    let angel = t.g.current(angel);
    assert!(t.on_battlefield(angel), "{}", t.dump_log());
    assert_eq!(t.obj_now(angel).controller, ALEX);
    assert_eq!(t.obj_now(angel).owner, BIANCA);
    (t, angel)
}

#[test]
fn bribery_bianca_leaves_and_serra_angel_leaves_the_game() {
    cr!("800.4a");
    cr_example!("800.4a", "Alex casts Bribery");
    let (mut t, angel) = bribery_for_serra_angel();
    concede(&mut t, BIANCA);
    t.settle();
    assert!(left_game(&t, angel));
    assert!(!t.in_exile("Serra Angel"));
}

#[test]
fn bribery_alex_leaves_and_serra_angel_is_exiled() {
    cr!("800.4a");
    cr_example!("800.4a", "Alex casts Bribery");
    let (mut t, angel) = bribery_for_serra_angel();
    concede(&mut t, ALEX);
    t.settle();
    assert_eq!(t.zone(angel), Zone::Exile);
}

#[test]
fn genesis_chamber_tokens_that_entered_under_alexs_control_leave_with_him() {
    cr!("800.4a", "111.2");
    cr_example!("800.4a", "Alex controls Genesis Chamber");
    supported("Genesis Chamber");
    let mut t = ffa3();
    t.battlefield(ALEX, "Genesis Chamber");
    // A nontoken creature enters under each of Alex's and Bianca's control.
    t.enter(ALEX, "Grizzly Bears");
    t.resolve_all();
    t.enter(BIANCA, "Grizzly Bears");
    t.resolve_all();
    let myr = t.named_on_battlefield("Myr Token");
    assert_eq!(myr.len(), 2, "{}", t.dump_log());
    let alexs: Vec<ObjectId> = myr
        .iter()
        .copied()
        .filter(|m| t.obj_now(*m).controller == ALEX)
        .collect();
    let biancas: Vec<ObjectId> = myr
        .iter()
        .copied()
        .filter(|m| t.obj_now(*m).controller == BIANCA)
        .collect();
    assert_eq!((alexs.len(), biancas.len()), (1, 1));
    concede(&mut t, ALEX);
    t.settle();
    assert!(left_game(&t, alexs[0]), "Alex's Myr token leaves the game");
    assert!(on_bf(&t, biancas[0]), "Bianca's Myr token remains");
    assert_eq!(t.obj_now(biancas[0]).controller, BIANCA);
}

#[test]
fn astral_slides_delayed_trigger_isnt_put_on_the_stack_after_bianca_leaves() {
    cr!("800.4d", "603.7");
    cr_example!("800.4d", "Astral Slide");
    supported("Astral Slide");
    supported("Hypnotic Specter");
    let mut t = ffa3();
    t.battlefield(BIANCA, "Astral Slide");
    let specter = t.battlefield(ALEX, "Hypnotic Specter");
    // During Alex's turn, Bianca cycles a card and exiles Hypnotic Specter.
    t.set_step(ALEX, Step::PrecombatMain);
    t.lands(BIANCA, "Swamp", 1);
    let moor = t.hand(BIANCA, "Barren Moor");
    t.activate(BIANCA, moor, 1, &[]).unwrap();
    t.answer_targets(BIANCA, &[Entity::Object(specter)]);
    t.answer_yes(BIANCA, true);
    t.resolve_all();
    assert_eq!(t.zone(specter), Zone::Exile, "{}", t.dump_log());
    let exiled = t.g.current(specter);
    // Astral Slide created the delayed triggered ability that would return it.
    assert!(t.g.delayed_triggers.iter().any(|d| d.controller == BIANCA));
    // Bianca leaves before the end of the turn.
    concede(&mut t, BIANCA);
    to_step(&mut t, ALEX, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Hypnotic Specter").is_empty());
    // It never returns.
    to_turn_of(&mut t, P2);
    t.resolve_all();
    assert!(t.named_on_battlefield("Hypnotic Specter").is_empty());
    assert_eq!(t.zone(exiled), Zone::Exile);
}
