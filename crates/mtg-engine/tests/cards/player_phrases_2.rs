//! Player phrases (`oracle/patterns/player_phrases_2.rs`): "each player who controls a
//! white creature", "each of its controller's opponents", "each of your opponents",
//! "another target player", "target opponent whose turn it is".

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn disorder_damages_each_player_who_controls_a_white_creature_once() {
    cr!("120.3");
    ruling!("Disorder", "Players take 0 or 2 damage");
    compiles("Disorder");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let lions = t.battlefield(P1, "Savannah Lions");
    t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Disorder");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(lions));
    assert_eq!(t.life(P1), 18, "two white creatures, 2 damage");
    assert_eq!(t.life(P0), 20, "no white creature");
    assert_eq!(t.g.obj(bears).damage, 0);
}

#[test]
fn bounty_board_each_of_its_controllers_opponents_draws_and_gains_life() {
    cr!("102.3", "603.10a");
    compiles("Bounty Board");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Bounty Board");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "bounty", 1, None);
    let hands = [t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)];
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert_eq!(
        [t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)],
        [hands[0] + 1, hands[1], hands[2] + 1]
    );
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (22, 20, 22));
}

#[test]
fn aurelia_damages_each_of_your_opponents() {
    cr!("508.1");
    compiles("Aurelia, the Law Above");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Aurelia, the Law Above");
    let attackers: Vec<(ObjectId, Entity)> = (0..5)
        .map(|_| (t.battlefield(P0, "Grizzly Bears"), Entity::Player(P1)))
        .collect();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(attackers));
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (23, 17, 17));
}

#[test]
fn cybernetica_datasmith_another_target_player_gets_the_robot() {
    cr!("115.1");
    ruling!(
        "Cybernetica Datasmith",
        "Both targets must be chosen when putting Cybernetica Datasmith's activated ability on the stack"
    );
    compiles("Cybernetica Datasmith");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ds = t.battlefield(P0, "Cybernetica Datasmith");
    t.lands(P0, "Island", 1);
    let hand = t.hand_size(P0);
    t.activate(P0, ds, 0, &[Entity::Player(P0), Entity::Player(P1)])
        .unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    let robot: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).chars.has_subtype("Robot"))
        .collect();
    assert_eq!(robot.len(), 1);
    assert_eq!(t.g.obj(robot[0]).controller, P1);
    // The second target can't be the first one.
    let second = t.asked().into_iter().filter_map(|(_, d)| match d {
        Decision::ChooseTargets { candidates, .. } => Some(candidates),
        _ => None,
    });
    assert!(second.last().is_some_and(|c| !c.contains(&Entity::Player(P0))));
}

#[test]
fn the_beamtown_bullies_only_the_opponent_whose_turn_it_is() {
    cr!("115.1", "701.15a");
    compiles("The Beamtown Bullies");
    let mut t = TestGame::new(3);
    let bullies = t.battlefield(P0, "The Beamtown Bullies");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    t.activate(P0, bullies, 0, &[Entity::Player(P1), Entity::Object(bears)])
        .unwrap();
    t.resolve_all();
    let new = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(new.len(), 1);
    assert_eq!(t.g.obj(new[0]).controller, P1);
    let players = t.asked().into_iter().find_map(|(_, d)| match d {
        Decision::ChooseTargets { candidates, .. }
            if candidates.iter().all(|e| e.player().is_some()) =>
        {
            Some(candidates)
        }
        _ => None,
    });
    assert_eq!(players, Some(vec![Entity::Player(P1)]), "P1's turn");
}
