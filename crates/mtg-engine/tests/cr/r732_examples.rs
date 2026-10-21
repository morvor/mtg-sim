//! The worked examples of CR 732 (taking shortcuts, and loops), played out as tests.

use crate::r506_common::custom_card;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
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

fn elf_warriors(t: &TestGame) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| t.obj(**id).is_token() && t.obj(**id).chars.has_subtype("Elf"))
        .count()
}

#[test]
fn a_shortcut_creates_many_tokens_with_presence_of_gond_and_intruder_alarm() {
    cr!("732.2", "732.2c", "732.1b");
    cr_example!("732.2a", "Presence of Gond");
    for name in ["Presence of Gond", "Intruder Alarm"] {
        let d = mtg_engine::card::card(name);
        assert!(d.is_fully_supported(), "{name}: {:?}", d.unsupported_text());
    }
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Presence of Gond: enchanted creature has "{T}: Create a 1/1 green Elf Warrior
    // creature token."
    let gond = t.battlefield(P0, "Presence of Gond");
    t.g.attach(gond, Entity::Object(bears));
    // Intruder Alarm (P1): "Whenever a creature enters, untap all creatures."
    t.battlefield(P1, "Intruder Alarm");
    t.g.recompute();
    let tap = Action::Activate {
        source: bears,
        ability: ability_uid(&t, bears),
    };
    // "I'll create [five] tokens": activate, all pass (the ability resolves and Intruder
    // Alarm triggers; its controller puts it on the stack), all pass (it resolves and
    // untaps the creature), repeated — ending just after the last token-creating ability
    // resolves. (Five rather than a million, to keep the test fast.)
    let n = 5;
    let mut seq = Vec::new();
    for i in 0..n {
        seq.extend([(P0, tap.clone()), (P0, Action::Pass), (P1, Action::Pass)]);
        if i + 1 < n {
            seq.extend([(P0, Action::Pass), (P1, Action::Pass)]);
        }
    }
    let taken = propose(&mut t.g, P0, &seq, 1).unwrap();
    assert_eq!(taken, seq.len());
    assert_eq!(elf_warriors(&t), n);
    // Just after the last token-creating ability resolved: the creature is still tapped,
    // and Intruder Alarm's last trigger is put on the stack before P0 (the active player)
    // gets priority.
    assert!(t.g.stack.is_empty());
    assert!(t.obj(bears).tapped);
    t.settle();
    assert_eq!(t.g.stack.len(), 1);
    assert!(t.g.describe(t.g.stack[0]).contains("Intruder Alarm"));
    assert!(t.obj(bears).tapped);
    assert_eq!(t.g.turn.priority, Some(P0));
    assert_eq!(t.g.turn.stage, Stage::Priority);
    t.resolve_all();
    assert!(!t.obj(bears).tapped);
}

#[test]
fn a_go_shortened_to_the_beginning_of_combat_for_into_the_fray() {
    cr!("732.2a", "732.2c");
    cr_example!("732.2b", "Into the Fray");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fray = t.hand(P1, "Into the Fray");
    t.lands(P1, "Mountain", 1);
    // The active player draws a card during her draw step, then says "Go": all players
    // pass at every opportunity for the rest of the turn.
    t.advance_to(P0, Step::Draw);
    let turn = t.g.turn.number;
    let pass_both = [(P0, Action::Pass), (P1, Action::Pass)];
    // Draw step, main phase, beginning of combat, ... end step.
    let seq: Vec<(PlayerId, Action)> = (0..4).flat_map(|_| pass_both.clone()).collect();
    // The nonactive player shortens it: "I'd like to cast a spell during your beginning
    // of combat step" — the place of their pass there (the sixth choice).
    t.answer(P1, DecisionKind::Number, Answer::Number(5));
    let taken = propose(&mut t.g, P0, &seq, 1).unwrap();
    assert_eq!(taken, 5);
    assert_eq!(t.g.turn.number, turn);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.priority, Some(P1));
    // P1 must now do something other than pass: casts Into the Fray ("Target creature
    // attacks this turn if able") on the Bears.
    t.answer(
        P1,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: fray,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P1, &[Entity::Object(bears)]);
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
    assert_eq!(t.g.stack.len(), 1);
    t.resolve_all();
    // P0 declares no attackers, but the Bears must attack.
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

/// An agent for the fragmented flying loop: at priority with an empty stack, activates
/// `rod`'s ability (targeting `creature`) whenever the creature's flying isn't `want`.
struct FlyingLooper {
    rod: ObjectId,
    uid: u64,
    creature: ObjectId,
    want: bool,
}

impl Agent for FlyingLooper {
    fn name(&self) -> &str {
        "flying looper"
    }
    fn decide(&mut self, g: &Game, _p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::Priority { actions } => {
                let act = Action::Activate {
                    source: self.rod,
                    ability: self.uid,
                };
                let flying = g.obj(self.creature).chars.has_keyword(KeywordKind::Flying);
                if g.stack.is_empty() && flying != self.want && actions.contains(&act) {
                    Answer::Action(act)
                } else {
                    Answer::Action(Action::Pass)
                }
            }
            Decision::ChooseTargets { .. } => {
                Answer::Entities(vec![Entity::Object(self.creature)])
            }
            Decision::DeclareAttackers { .. } => Answer::Attackers(vec![]),
            Decision::DeclareBlockers { .. } => Answer::Blockers(vec![]),
            _ => Answer::Default,
        }
    }
}

/// The active player's creature "{0}: This creature gains flying" and the nonactive
/// player's permanent "{0}: Target creature loses flying"; `nap_loops`: whether the
/// nonactive player answers each activation.
fn flying_loop(nap_loops: bool) -> TestGame {
    let mut t = TestGame::new(2);
    let wings = custom_card(
        "Wing Seeker",
        "Creature — Bird",
        Some((1, 1)),
        "{0}: This creature gains flying until end of turn.",
    );
    let clip = custom_card(
        "Wing Clipper",
        "Artifact",
        None,
        "{0}: Target creature loses flying until end of turn.",
    );
    let creature = t.custom(P0, wings, Zone::Battlefield);
    let clipper = t.custom(P1, clip, Zone::Battlefield);
    let (u0, u1) = (ability_uid(&t, creature), ability_uid(&t, clipper));
    t.g.set_agent(
        P0,
        Box::new(FlyingLooper {
            rod: creature,
            uid: u0,
            creature,
            want: true,
        }),
    );
    if nap_loops {
        t.g.set_agent(
            P1,
            Box::new(FlyingLooper {
                rod: clipper,
                uid: u1,
                creature,
                want: false,
            }),
        );
    }
    let ok = t
        .g
        .run_until(3000, |g| g.turn.step != Step::PrecombatMain || g.is_over());
    assert!(ok);
    assert_eq!(t.g.result, None);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    t
}

#[test]
fn a_fragmented_flying_loop_ends_with_the_creature_not_flying() {
    cr!("732.3");
    cr_example!("732.3", "This creature gains flying");
    let t = flying_loop(true);
    let creature = t.g.battlefield[0];
    assert_eq!(t.obj(creature).chars.name, "Wing Seeker");
    // The game returned to an earlier state; the active player had to make a different
    // choice. The nonactive player had the final choice: the creature doesn't fly.
    assert!(!t.obj(creature).chars.has_keyword(KeywordKind::Flying));
    assert!(t.g.actions_taken < 200, "the loop was recognized");
    // Had the nonactive player not answered, the creature would have had flying.
    let t = flying_loop(false);
    assert!(t
        .obj(t.g.battlefield[0])
        .chars
        .has_keyword(KeywordKind::Flying));
}

#[test]
fn seal_of_cleansing_isnt_forced_to_end_a_mandatory_loop_with_an_artifact() {
    cr!("732.4", "104.4b");
    cr_example!("732.5", "Seal of Cleansing");
    let d = mtg_engine::card::card("Seal of Cleansing");
    assert!(d.is_fully_supported(), "{:?}", d.unsupported_text());
    let mut t = TestGame::new(2);
    // P0 controls Seal of Cleansing: "Sacrifice this enchantment: Destroy target
    // artifact or enchantment."
    let seal = t.battlefield(P0, "Seal of Cleansing");
    // A mandatory loop involving an artifact: "Whenever a creature dies, return that card
    // to the battlefield under its owner's control." and a 0/0 creature.
    let a = custom_card(
        "Endless Engine",
        "Artifact",
        None,
        "Whenever a creature dies, return that card to the battlefield under its owner's control.",
    );
    t.custom(P1, a, Zone::Battlefield);
    let z = custom_card("Hollow Husk", "Creature — Construct", Some((0, 0)), "");
    let husk = t.custom(P1, z, Zone::Graveyard(P1));
    t.g.move_object(
        husk,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P1),
    );
    t.g.run_until(4000, |g| g.is_over());
    // The player isn't forced to sacrifice the Seal to destroy the artifact: the loop of
    // mandatory actions goes on and the game is a draw.
    assert_eq!(t.g.result, Some(GameResult::Draw));
    assert!(t.g.is_live(seal) && t.obj(seal).zone == Zone::Battlefield);
    assert!(t.g.actions_taken < 500, "recognized quickly");
}

/// The active player pumps their creature while it's smaller than `target`; the nonactive
/// player answers each activation with an ability that changes nothing ("loses flying" on
/// a creature without flying).
struct Pumper {
    pump: ObjectId,
    pump_uid: u64,
    clip: ObjectId,
    clip_uid: u64,
    creature: ObjectId,
    target: i32,
    answered: Vec<ObjectId>,
}

impl Agent for Pumper {
    fn name(&self) -> &str {
        "pumper"
    }
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::Priority { actions } => {
                let pump = Action::Activate {
                    source: self.pump,
                    ability: self.pump_uid,
                };
                let clip = Action::Activate {
                    source: self.clip,
                    ability: self.clip_uid,
                };
                if p == P0 && g.stack.is_empty() && g.obj(self.creature).power() < self.target {
                    if actions.contains(&pump) {
                        return Answer::Action(pump);
                    }
                }
                if p == P1 && g.stack.len() == 1 {
                    let top = g.stack[0];
                    if g.obj(top).controller == P0
                        && !self.answered.contains(&top)
                        && actions.contains(&clip)
                    {
                        self.answered.push(top);
                        return Answer::Action(clip);
                    }
                }
                Answer::Action(Action::Pass)
            }
            Decision::ChooseTargets { .. } => {
                Answer::Entities(vec![Entity::Object(self.creature)])
            }
            Decision::DeclareAttackers { .. } => Answer::Attackers(vec![]),
            Decision::DeclareBlockers { .. } => Answer::Blockers(vec![]),
            _ => Answer::Default,
        }
    }
}

#[test]
fn a_repetition_that_grows_a_creature_never_returns_to_an_earlier_state() {
    cr!("732.3");
    // Both players act each time around, and the piled-up "+1/+1" effects are identical,
    // but the creature grows: the game never reaches the same state twice, so it isn't a
    // fragmented loop and nobody is made to stop.
    let mut t = TestGame::new(2);
    let grower = custom_card(
        "Swelling Brute",
        "Creature — Beast",
        Some((1, 1)),
        "{0}: This creature gets +1/+1 until end of turn.",
    );
    let clip = custom_card(
        "Wing Clipper",
        "Artifact",
        None,
        "{0}: Target creature loses flying until end of turn.",
    );
    let creature = t.custom(P0, grower, Zone::Battlefield);
    let clipper = t.custom(P1, clip, Zone::Battlefield);
    let (u0, u1) = (ability_uid(&t, creature), ability_uid(&t, clipper));
    let agent = || Pumper {
        pump: creature,
        pump_uid: u0,
        clip: clipper,
        clip_uid: u1,
        creature,
        target: 12,
        answered: Vec::new(),
    };
    t.g.set_agent(P0, Box::new(agent()));
    t.g.set_agent(P1, Box::new(agent()));
    let ok = t
        .g
        .run_until(3000, |g| g.turn.step != Step::PrecombatMain || g.is_over());
    assert!(ok);
    assert_eq!(t.g.result, None);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    assert_eq!(t.obj(creature).power(), 12);
    assert_eq!(t.obj(creature).toughness(), 12);
}
