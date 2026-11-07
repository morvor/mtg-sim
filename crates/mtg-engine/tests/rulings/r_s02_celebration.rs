//! Rulings batch S02 — celebration (an ability word, CR 207.2c): abilities that care
//! whether "two or more nonland permanents entered the battlefield under your control this
//! turn".

use crate::r_s01_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// `who` gains control of `id` until end of turn (as a resolving effect would).
fn gain_control(t: &mut TestGame, who: PlayerId, id: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, who);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
}

/// `p` creates a Treasure token (as a resolving effect would).
fn create_treasure(t: &mut TestGame, p: PlayerId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec: mtg_engine::tokens::predefined("Treasure").expect("Treasure"),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
}

#[test]
fn a_celebration_trigger_checks_what_entered_this_turn() {
    cr!("207.2c", "603.4");
    ruling!(
        "Pests of Honor",
        "Some celebration abilities trigger at specific parts of the turn and check whether two or more nonland permanents entered the battlefield under your control already in that turn."
    );
    supported("Pests of Honor");
    // "Celebration — At the beginning of combat on your turn, if two or more nonland
    // permanents entered the battlefield under your control this turn, put a +1/+1 counter
    // on this creature."
    let mut t = TestGame::new(2);
    let pests = t.battlefield(P0, "Pests of Honor");
    t.enter(P0, "Grizzly Bears");
    // A land, and a permanent entering under an opponent's control, don't count.
    t.enter(P0, "Forest");
    t.enter(P1, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(pests, counters::PLUS1), 0);
    // Next turn, two nonland permanents (a creature and a token) enter first.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.enter(P0, "Ornithopter");
    create_treasure(&mut t, P0);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(pests, counters::PLUS1), 1);
}

#[test]
fn a_celebration_static_ability_applies_as_long_as_two_nonland_permanents_entered() {
    cr!("207.2c", "611.3a");
    ruling!(
        "Armory Mice",
        "Others are static abilities that give creatures abilities or power and toughness increases as long as two or more nonland permanents entered the battlefield under your control that turn."
    );
    supported("Armory Mice");
    let mut t = TestGame::new(2);
    // "Celebration — This creature gets +0/+2 as long as two or more nonland permanents
    // entered the battlefield under your control this turn."
    let mice = t.battlefield(P0, "Armory Mice");
    assert_eq!(t.pt(mice), (3, 1));
    t.enter(P0, "Ornithopter");
    assert_eq!(t.pt(mice), (3, 1));
    t.enter(P0, "Memnite");
    assert_eq!(t.pt(mice), (3, 3));
    // It lasts only for that turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(mice), (3, 1));
}

#[test]
fn the_permanents_that_entered_needn_t_still_be_there_or_under_your_control() {
    cr!("207.2c", "611.3a");
    ruling!(
        "Lady of Laughter",
        "The permanents that entered the battlefield don't need to remain on the battlefield or under your control. Celebration abilities are checking for past events, not the current game state."
    );
    supported("Lady of Laughter");
    let mut t = TestGame::new(2);
    // "Celebration — At the beginning of your end step, if two or more nonland permanents
    // entered the battlefield under your control this turn, draw a card."
    t.battlefield(P0, "Lady of Laughter");
    let mice = t.battlefield(P0, "Armory Mice");
    let bears = t.enter(P0, "Grizzly Bears");
    let thopter = t.enter(P0, "Ornithopter");
    // One is destroyed, the other comes under P1's control.
    t.g.destroy(thopter, None);
    gain_control(&mut t, P1, bears);
    t.settle();
    assert!(!t.on_battlefield(thopter));
    assert_eq!(t.obj(bears).controller, P1);
    assert_eq!(t.pt(mice), (3, 3));
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn celebration_abilities_dont_get_stronger_with_more_permanents() {
    cr!("207.2c", "603.4");
    ruling!(
        "Pests of Honor",
        "Celebration abilities only care if two or more nonland permanents entered the battlefield under your control in a turn. They won't get more powerful if more than two permanents entered the battlefield under your control in a turn."
    );
    let mut t = TestGame::new(2);
    let pests = t.battlefield(P0, "Pests of Honor");
    let mice = t.battlefield(P0, "Armory Mice");
    for _ in 0..4 {
        t.enter(P0, "Memnite");
    }
    assert_eq!(t.pt(mice), (3, 3));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(pests, counters::PLUS1), 1);
    assert_eq!(t.pt(pests), (3, 3));
}

#[test]
fn an_artifact_that_entered_this_turn_enables_a_static_ability() {
    cr!("611.3a");
    supported("Sentinel Sarah Lyons");
    let mut t = TestGame::new(2);
    // "As long as an artifact entered the battlefield under your control this turn,
    // creatures you control get +2/+2."
    let sarah = t.battlefield(P0, "Sentinel Sarah Lyons");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    // An artifact entering under an opponent's control doesn't count.
    t.enter(P1, "Memnite");
    assert_eq!(t.pt(bears), (2, 2));
    let memnite = t.enter(P0, "Memnite");
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(sarah), (6, 6));
    // It still counts after it leaves.
    t.g.destroy(memnite, None);
    t.settle();
    assert_eq!(t.pt(bears), (4, 4));
}

/// P0 manifests the card `name` from the top of their library: a face-down creature
/// enters the battlefield under P0's control.
fn manifest(t: &mut TestGame, name: &str) -> ObjectId {
    use mtg_engine::kwa::manifest::{put_face_down, MANIFESTED};
    let card = t.library_top(P0, name);
    let id = put_face_down(&mut t.g, card, P0, MANIFESTED, None).expect("manifested");
    t.settle();
    assert!(t.obj(id).face_down);
    id
}

#[test]
fn a_permanent_counts_as_what_it_was_when_it_entered() {
    cr!("603.4", "701.40a", "708.8", "708.2a");
    ruling!(
        "Tunnel Tipster",
        "Tunnel Tipster's first ability will trigger as long as a face-down creature entered the battlefield under your control this turn, even if that creature has turned face up or left the battlefield since. A creature that enters the battlefield face up and turns face down later in the turn won't cause Tunnel Tipster's first ability to trigger."
    );
    supported("Tunnel Tipster");
    // "At the beginning of your end step, if a face-down creature entered the battlefield
    // under your control this turn, put a +1/+1 counter on this creature."
    let end_step_counters = |t: &mut TestGame, tipster: ObjectId| {
        t.advance_to(P0, Step::End);
        t.resolve_all();
        t.counters(tipster, counters::PLUS1)
    };
    // A manifested Grizzly Bears card is turned face up before the end step.
    let mut t = TestGame::new(2);
    let tipster = t.battlefield(P0, "Tunnel Tipster");
    let bears = manifest(&mut t, "Grizzly Bears");
    assert!(mtg_engine::facedown::turn_face_up(&mut t.g, bears, false));
    t.settle();
    assert!(!t.obj(bears).face_down);
    assert_eq!(t.obj(bears).chars.name, "Grizzly Bears");
    assert_eq!(end_step_counters(&mut t, tipster), 1);
    // A manifested card that has left the battlefield.
    let mut t = TestGame::new(2);
    let tipster = t.battlefield(P0, "Tunnel Tipster");
    let bears = manifest(&mut t, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(end_step_counters(&mut t, tipster), 1);
    // Grizzly Bears enter face up and are turned face down later: no counter.
    let mut t = TestGame::new(2);
    let tipster = t.battlefield(P0, "Tunnel Tipster");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, bears));
    t.settle();
    assert!(t.obj_now(bears).face_down);
    assert_eq!(end_step_counters(&mut t, tipster), 0);
}
