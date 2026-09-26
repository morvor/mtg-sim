//! CR 732: taking shortcuts, and loops.

use crate::r506_common::custom_card;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::shortcuts::propose;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// The uid of the first activated ability of `src`.
fn ability_uid(t: &TestGame, src: ObjectId) -> u64 {
    t.g.obj(src)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.uid)
        .expect("an activated ability")
}

/// P0's artifact "{0}: You gain 1 life."
fn life_pump(t: &mut TestGame, p: PlayerId) -> (ObjectId, Action) {
    let c = custom_card("Life Pump", "Artifact", None, "{0}: You gain 1 life.");
    let id = t.custom(p, c, Zone::Battlefield);
    let a = Action::Activate {
        source: id,
        ability: ability_uid(t, id),
    };
    (id, a)
}

#[test]
fn a_loop_repeated_by_a_shortcut() {
    cr!("732.1", "732.1b", "732.2", "732.2a", "732.2c");
    let mut t = TestGame::new(2);
    let (_, pump) = life_pump(&mut t, P0);
    // "I activate the Pump and we both pass, five times."
    let taken = propose(
        &mut t.g,
        P0,
        &[(P0, pump.clone()), (P0, Action::Pass), (P1, Action::Pass)],
        5,
    )
    .unwrap();
    assert_eq!(taken, 15);
    // P1 accepted; the game advanced to the ending point with every choice taken.
    assert_eq!(t.life(P0), 25);
    assert!(t.g.stack.is_empty());
    assert_eq!(t.g.turn.stage, Stage::Priority);
    assert_eq!(t.g.turn.priority, Some(P0));
    assert_eq!(t.g.turn.step, Step::PrecombatMain);
}

#[test]
fn a_shortcut_must_be_a_legal_sequence_of_choices() {
    cr!("732.2a");
    let mut t = TestGame::new(2);
    let (_, pump) = life_pump(&mut t, P0);
    // Only the player with priority may propose one.
    assert!(propose(&mut t.g, P1, &[(P1, Action::Pass)], 1).is_err());
    // Each choice must be one the player who has priority at that point may make: P1
    // doesn't get priority first.
    assert!(propose(&mut t.g, P0, &[(P1, Action::Pass)], 1).is_err());
    // P1 can't activate P0's artifact.
    assert!(propose(
        &mut t.g,
        P0,
        &[(P0, Action::Pass), (P1, pump.clone())],
        1
    )
    .is_err());
    // Nothing happened.
    assert_eq!(t.life(P0), 20);
    assert!(t.g.stack.is_empty());
    assert_eq!(t.g.turn.priority, Some(P0));
}

#[test]
fn another_player_may_shorten_the_shortcut() {
    cr!("732.2b", "732.2c");
    let mut t = TestGame::new(2);
    let (_, pump) = life_pump(&mut t, P0);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    // P1 names the ninth choice (their third pass) as a place they'll do something else:
    // the shortcut ends after eight choices.
    t.answer(P1, DecisionKind::Number, Answer::Number(8));
    let taken = propose(
        &mut t.g,
        P0,
        &[(P0, pump.clone()), (P0, Action::Pass), (P1, Action::Pass)],
        5,
    )
    .unwrap();
    assert_eq!(taken, 8);
    // Two activations resolved; the third is on the stack and P1 has priority.
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.g.stack.len(), 1);
    assert_eq!(t.g.turn.priority, Some(P1));
    // P1 must make a different choice than proposed: passing isn't offered.
    t.answer(
        P1,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: bolt,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let asked = t.asked().len();
    t.g.advance();
    let offered = t.asked()[asked..]
        .iter()
        .find_map(|(p, d)| match d {
            Decision::Priority { actions } if *p == P1 => Some(actions.clone()),
            _ => None,
        })
        .unwrap();
    assert!(!offered.contains(&Action::Pass));
    assert_eq!(t.g.stack.len(), 2);
}

/// A game with "{0}: Tap target creature." for P0 and "{0}: Untap target creature." for
/// P1, and P1's Grizzly Bears. Agents: P0 taps the Bears whenever it can while they're
/// untapped; P1 untaps them whenever they're tapped.
struct Looper {
    rod: ObjectId,
    uid: u64,
    bears: ObjectId,
    tap: bool,
}

impl Agent for Looper {
    fn name(&self) -> &str {
        "looper"
    }
    fn decide(&mut self, g: &Game, _p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::Priority { actions } => {
                let act = Action::Activate {
                    source: self.rod,
                    ability: self.uid,
                };
                let wants = g.stack.is_empty() && g.obj(self.bears).tapped != self.tap;
                if wants && actions.contains(&act) {
                    Answer::Action(act)
                } else {
                    Answer::Action(Action::Pass)
                }
            }
            Decision::ChooseTargets { .. } => Answer::Entities(vec![Entity::Object(self.bears)]),
            Decision::DeclareAttackers { .. } => Answer::Attackers(vec![]),
            Decision::DeclareBlockers { .. } => Answer::Blockers(vec![]),
            _ => Answer::Default,
        }
    }
}

#[test]
fn a_fragmented_loop_is_broken_by_the_active_player() {
    cr!("732.3");
    let mut t = TestGame::new(2);
    let tapper = custom_card("Tapping Rod", "Artifact", None, "{0}: Tap target creature.");
    let untapper = custom_card("Untapping Rod", "Artifact", None, "{0}: Untap target creature.");
    let tap_rod = t.custom(P0, tapper, Zone::Battlefield);
    let untap_rod = t.custom(P1, untapper, Zone::Battlefield);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let (u0, u1) = (ability_uid(&t, tap_rod), ability_uid(&t, untap_rod));
    t.g.set_agent(
        P0,
        Box::new(Looper {
            rod: tap_rod,
            uid: u0,
            bears,
            tap: true,
        }),
    );
    t.g.set_agent(
        P1,
        Box::new(Looper {
            rod: untap_rod,
            uid: u1,
            bears,
            tap: false,
        }),
    );
    // Each player's independent actions bring back the same game state; after it
    // repeats, P0 (the active player) must make a different choice, and the game goes on.
    let ok = t
        .g
        .run_until(3000, |g| g.turn.step != Step::PrecombatMain || g.is_over());
    assert!(ok);
    assert_eq!(t.g.result, None);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    assert!(t.g.actions_taken < 200);
}

/// "Whenever a creature dies, return that card to the battlefield under its owner's
/// control[ unless you pay {1}]." plus a 0/0 creature: a loop.
fn loop_game(t: &mut TestGame, text: &str) {
    let e = custom_card("Endless Return", "Enchantment", None, text);
    t.custom(P0, e, Zone::Battlefield);
    let z = custom_card("Hollow Husk", "Creature — Construct", Some((0, 0)), "");
    let husk = t.custom(P0, z, Zone::Graveyard(P0));
    t.g.move_object(
        husk,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
}

#[test]
fn a_loop_of_only_mandatory_actions_is_a_draw() {
    cr!("732.4");
    let mut t = TestGame::new(2);
    loop_game(
        &mut t,
        "Whenever a creature dies, return that card to the battlefield under its owner's control.",
    );
    t.g.run_until(2000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn no_player_is_forced_to_end_a_loop() {
    cr!("732.5");
    let mut t = TestGame::new(2);
    loop_game(
        &mut t,
        "Whenever a creature dies, return that card to the battlefield under its owner's control.",
    );
    // P1 could end the loop by destroying the enchantment, but can't be made to.
    t.lands(P1, "Forest", 2);
    t.hand(P1, "Naturalize");
    t.g.run_until(4000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
    assert!(t.g.actions_taken < 500, "recognized quickly");
}

#[test]
fn a_loop_with_a_declined_unless_payment_continues_as_though_mandatory() {
    cr!("732.6");
    let mut t = TestGame::new(2);
    loop_game(
        &mut t,
        "Whenever a creature dies, return that card to the battlefield under its owner's control unless you pay {1}.",
    );
    // P0 could pay {1} each time, but declines.
    t.lands(P0, "Wastes", 3);
    t.g.run_until(4000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
    assert!(t.g.actions_taken < 500, "recognized quickly");
}

#[test]
fn a_fragmented_loop_is_broken_by_the_first_player_in_turn_order_involved_in_it() {
    cr!("732.3", "732.5");
    let mut t = TestGame::new(3);
    let tapper = custom_card("Tapping Rod", "Artifact", None, "{0}: Tap target creature.");
    let untapper = custom_card("Untapping Rod", "Artifact", None, "{0}: Untap target creature.");
    let tap_rod = t.custom(P1, tapper, Zone::Battlefield);
    let untap_rod = t.custom(P2, untapper, Zone::Battlefield);
    let bears = t.battlefield(P2, "Grizzly Bears");
    let (u1, u2) = (ability_uid(&t, tap_rod), ability_uid(&t, untap_rod));
    t.g.set_agent(
        P1,
        Box::new(Looper {
            rod: tap_rod,
            uid: u1,
            bears,
            tap: true,
        }),
    );
    t.g.set_agent(
        P2,
        Box::new(Looper {
            rod: untap_rod,
            uid: u2,
            bears,
            tap: false,
        }),
    );
    // The active player acted earlier this step (played a land), then only passes while
    // P1 and P2 loop. P0 could cast Lightning Bolt, but isn't forced to: P1, the first
    // player in turn order involved in the loop, must make a different choice.
    let mountain = t.hand(P0, "Mountain");
    t.hand(P0, "Lightning Bolt");
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::PlayLand { card: mountain }),
    );
    let ok = t
        .g
        .run_until(3000, |g| g.turn.step != Step::PrecombatMain || g.is_over());
    assert!(ok);
    assert_eq!(t.g.result, None);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    assert!(t.on_battlefield(mountain) || t.named_on_battlefield("Mountain").len() == 1);
    assert!(t.in_hand(P0, "Lightning Bolt"));
    // P1 stopped tapping the Bears.
    assert!(!t.g.obj(bears).tapped);
    assert!(t.g.actions_taken < 300);
}
