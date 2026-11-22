//! Rulings batch P038 — combat: tokens put onto the battlefield attacking (CR 508.4),
//! attacking alone (CR 506.5), attack and block restrictions checked only as attackers
//! and blockers are declared (CR 506.4, 509.1h), "must be blocked" (CR 509.1c), and
//! beginning-of-combat intervening "if" clauses (CR 603.4).

use crate::r_p038_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported, triggers_on_stack};
use crate::r_s02_common::create_token;
use mtg_engine::decision::Answer;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn attack_target(t: &TestGame, id: ObjectId) -> Option<Entity> {
    t.g.combat.as_ref().and_then(|c| c.attack_target(id))
}

#[test]
fn tokens_entering_attacking_may_attack_something_else() {
    cr!("508.4", "508.4a");
    ruling!(
        "Kavaron Harrier",
        "You choose the player, planeswalker, or battle the token is attacking. It doesn’t have to be the same player, planeswalker, or battle that Kavaron Harrier or any other attacking creatures are attacking."
    );
    ruling!(
        "Ghired, Conclave Exile",
        "You choose which player or planeswalker the created token is attacking. It doesn't have to be attacking the same player or planeswalker that Ghired is attacking."
    );
    // Kavaron Harrier attacks P1; the Robot attacks P1's planeswalker.
    supported("Kavaron Harrier");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let harrier = t.battlefield(P0, "Kavaron Harrier");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(jace)]);
    attack_with(&mut t, &[(harrier, pl(P1))]);
    t.resolve_all();
    let robot = tokens_of(&t, P0, "Robot");
    assert_eq!(robot.len(), 1);
    assert_eq!(attack_target(&t, robot[0]), Some(obj(jace)));
    assert_eq!(attack_target(&t, harrier), Some(pl(P1)));
    assert!(tapped(&t, robot[0]));

    // Ghired attacks P1; the populated Rhino attacks the planeswalker.
    supported("Ghired, Conclave Exile");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    t.answer_choose(P0, &[]);
    let ghired = enter(&mut t, P0, "Ghired, Conclave Exile");
    t.resolve_all();
    t.g.objects[ghired.0 as usize].summoning_sick = false;
    t.clear_answers();
    let rhino = tokens_of(&t, P0, "Rhino")[0];
    t.answer_choose(P0, &[obj(rhino)]); // the token to populate
    t.answer_choose(P0, &[obj(jace)]);
    attack_with(&mut t, &[(ghired, pl(P1))]);
    t.resolve_all();
    let rhinos = tokens_of(&t, P0, "Rhino");
    assert_eq!(rhinos.len(), 2);
    let attacking: Vec<_> = rhinos.iter().filter(|r| t.g.is_attacking(**r)).collect();
    assert_eq!(attacking.len(), 1);
    assert_eq!(attack_target(&t, *attacking[0]), Some(obj(jace)));
    assert_eq!(attack_target(&t, ghired), Some(pl(P1)));
}

#[test]
fn crash_the_party_tokens_do_not_enter_attacking() {
    cr!("508.4", "506.4");
    ruling!(
        "Crash the Party",
        "The tokens created by Crash the Party don't enter the battlefield attacking."
    );
    let mut t = TestGame::new(2);
    let b = bears(&mut t, P0, 2);
    attack_with(&mut t, &[(b[0], pl(P1)), (b[1], pl(P1))]);
    cast_resolve(&mut t, P0, "Crash the Party", &[]);
    let rhinos = tokens_of(&t, P0, "Rhino");
    assert_eq!(rhinos.len(), 2);
    for r in rhinos {
        assert!(tapped(&t, r));
        assert!(!t.g.is_attacking(r));
    }
}

fn two_headed_giant() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

#[test]
fn attacking_alone_means_the_only_creature_declared() {
    cr!("506.5", "506.4", "805.10");
    ruling!(
        "Angelic Exaltation",
        "A creature attacks alone if it’s the only creature declared as an attacker during the declare attackers step (including creatures controlled by your teammates, if applicable). For example, Angelic Exaltation’s ability won’t trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    ruling!(
        "Sigil of Valor",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step (including creatures controlled by your teammates, if applicable). For example, Sigil of Valor's ability won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    supported("Angelic Exaltation");
    supported("Sigil of Valor");
    // `who` attacks with these creatures; returns how many "attacks alone" triggers.
    fn alone(exaltation: bool, two: bool, teammate: bool) -> usize {
        let mut t = if teammate {
            two_headed_giant()
        } else {
            TestGame::new(2)
        };
        let b = bears(&mut t, P0, 2);
        if exaltation {
            t.battlefield(P0, "Angelic Exaltation");
        } else {
            let sigil = t.battlefield(P0, "Sigil of Valor");
            assert!(t.g.attach(sigil, obj(b[0])));
        }
        let defender = if teammate { pl(P2) } else { pl(P1) };
        let mut attackers = vec![(b[0], defender)];
        if two {
            attackers.push((b[1], defender));
        }
        if teammate {
            let mate = t.battlefield(P1, "Grizzly Bears");
            attackers.push((mate, defender));
        }
        attack_with(&mut t, &attackers);
        let n = triggers_on_stack(&t, "attacks alone");
        if two {
            // One is removed from combat: still no trigger later.
            kill(&mut t, b[1]);
            assert_eq!(triggers_on_stack(&t, "attacks alone"), n);
        }
        n
    }
    for exaltation in [true, false] {
        assert_eq!(alone(exaltation, false, false), 1, "{exaltation}");
        assert_eq!(alone(exaltation, true, false), 0, "{exaltation}");
        assert_eq!(alone(exaltation, false, true), 0, "{exaltation}: teammate");
    }
}

#[test]
fn attack_and_block_restrictions_are_checked_only_when_declared() {
    cr!("506.4", "508.1c", "509.1b");
    ruling!(
        "Chief Warg's Company",
        "Once Chief Warg's Company has attacked, it will remain an attacking creature even if you no longer control two or more other Wolves."
    );
    ruling!(
        "Oketra the True",
        "Once Oketra has attacked or blocked, it will remain in combat even if the number of other creatures you control becomes two or fewer."
    );
    ruling!(
        "Lydia Frye",
        "Once Lydia Frye has been blocked, increasing the blocking creature’s power to 3 or greater won’t cause Lydia Frye to become unblocked."
    );
    // Chief Warg's Company with two Wolves; both die after attackers are declared.
    supported("Chief Warg's Company");
    let mut t = TestGame::new(2);
    let company = t.battlefield(P0, "Chief Warg's Company");
    let w1 = create_token(&mut t, P0, "Wolf");
    let w2 = create_token(&mut t, P0, "Wolf");
    attack_with(&mut t, &[(company, pl(P1))]);
    assert!(t.g.is_attacking(company));
    kill(&mut t, w1);
    kill(&mut t, w2);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 15);

    // Oketra attacks with three other creatures, which then die; and blocks likewise.
    supported("Oketra the True");
    let mut t = TestGame::new(2);
    let oketra = t.battlefield(P0, "Oketra the True");
    let others = bears(&mut t, P0, 3);
    attack_with(&mut t, &[(oketra, pl(P1))]);
    for o in &others {
        kill(&mut t, *o);
    }
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 14, "double strike");
    let mut t = TestGame::new(2);
    let oketra = t.battlefield(P1, "Oketra the True");
    let others = bears(&mut t, P1, 3);
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(giant, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(oketra, giant)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert!(t.g.combat.as_ref().unwrap().is_blocked(giant));
    for o in &others {
        kill(&mut t, *o);
    }
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P0, "Hill Giant"));

    // Lydia Frye is blocked by a 2/2 that then gets +3/+3.
    supported("Lydia Frye");
    let mut t = TestGame::new(2);
    let lydia = t.battlefield(P0, "Lydia Frye");
    let bear = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(lydia, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bear, lydia)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.set_step(P0, Step::DeclareBlockers);
    t.g.turn.priority = Some(P1);
    t.lands(P1, "Forest", 1);
    let growth = t.hand(P1, "Giant Growth");
    t.answer_targets(P1, &[obj(bear)]);
    t.cast(P1, growth).go();
    t.resolve_all();
    assert_eq!(t.pt(bear).0, 5);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(lydia));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Lydia Frye"));
}

#[test]
fn king_haralds_revenge_must_be_blocked_by_at_least_one_creature() {
    cr!("509.1c");
    ruling!(
        "King Harald's Revenge",
        "If the creature attacks, the defending player must assign at least one blocker to it if that player controls any creatures that could block it. Other creatures may also block it, block other creatures, or not block at all."
    );
    supported("King Harald's Revenge");
    // Returns whether the blocks P1 tries are kept as declared, and whether the pumped
    // creature ends up blocked.
    fn run(blocks: fn(ObjectId, ObjectId, ObjectId, ObjectId) -> Vec<(ObjectId, ObjectId)>) -> (bool, bool) {
        let mut t = TestGame::new(2);
        let target = t.battlefield(P0, "Grizzly Bears");
        let other = t.battlefield(P0, "Hill Giant");
        let b1 = t.battlefield(P1, "Grizzly Bears");
        let b2 = t.battlefield(P1, "Grizzly Bears");
        cast_resolve(&mut t, P0, "King Harald's Revenge", &[obj(target)]);
        attack_with(&mut t, &[(target, pl(P1)), (other, pl(P1))]);
        let wanted = blocks(target, other, b1, b2);
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(wanted.clone()));
        t.advance_to(P0, Step::DeclareBlockers);
        let c = t.g.combat.as_ref().unwrap();
        let kept = wanted
            .iter()
            .all(|(b, a)| c.blocking(*b).contains(a))
            && c.blockers.len() == wanted.len();
        (kept, c.is_blocked(target))
    }
    // One blocks it, the other blocks another attacker: legal.
    assert_eq!(run(|tg, o, b1, b2| vec![(b1, tg), (b2, o)]), (true, true));
    // One blocks it, the other doesn't block: legal.
    assert_eq!(run(|tg, _, b1, _| vec![(b1, tg)]), (true, true));
    // Both block it: legal.
    assert_eq!(run(|tg, _, b1, b2| vec![(b1, tg), (b2, tg)]), (true, true));
    // Nobody blocks it although they could: not allowed, it ends up blocked.
    assert_eq!(run(|_, o, b1, _| vec![(b1, o)]).1, true);
    assert_eq!(run(|_, _, _, _| vec![]).1, true);
}

#[test]
fn leonin_vanguard_checks_three_creatures_as_combat_begins_and_resolves() {
    cr!("603.4", "507.1");
    ruling!(
        "Leonin Vanguard",
        "If you don't control three or more creatures as Leonin Vanguard's ability resolves, the ability does nothing. However, if you do control three or more creatures as it resolves, the +1/+1 bonus doesn't stop applying if you no longer control three creatures later in the turn."
    );
    ruling!(
        "Leonin Vanguard",
        "If you don't control three or more creatures as your combat phase begins, Leonin Vanguard's ability won't trigger at all. Causing a permanent to become a creature during the beginning of combat step won't cause Leonin Vanguard's ability to trigger."
    );
    supported("Leonin Vanguard");
    // Two creatures as combat begins: no trigger; a third arriving in the step changes
    // nothing.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Leonin Vanguard");
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    add(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(v), (1, 1));

    // Three as it triggers, two as it resolves: nothing.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Leonin Vanguard");
    let b = bears(&mut t, P0, 2);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, b[0]);
    t.resolve_all();
    assert_eq!(t.pt(v), (1, 1));
    assert_eq!(t.life(P0), 20);

    // Three as it resolves: the bonus stays after one dies.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Leonin Vanguard");
    let b = bears(&mut t, P0, 2);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(v), (2, 2));
    assert_eq!(t.life(P0), 21);
    kill(&mut t, b[0]);
    assert_eq!(t.pt(v), (2, 2));
}
