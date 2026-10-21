//! Rulings batch P056 — Tahngarth, First Mate: "Whenever an opponent attacks with one or
//! more creatures, if ~ is tapped, you may have that opponent gain control of ~ until end of
//! combat. If you do, choose a player or planeswalker that opponent is attacking. ~ is
//! attacking that player or planeswalker." It becomes an attacking creature without being
//! declared (CR 506.3d, 508.4): attack restrictions and "whenever ... attacks" don't apply,
//! it stays tapped, and commander damage follows the commander (CR 903.10a).

use crate::r_p056_common::*;
use crate::r_s09_common::{attacks_now, declare};
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::combat::{block_declaration_legal, block_options};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p`'s tapped Tahngarth.
fn tapped_tahngarth(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let tahn = t.battlefield(p, "Tahngarth, First Mate");
    tap(t, tahn);
    tahn
}

/// `attacker`'s controller declares `decl`; Tahngarth's controller answers yes and
/// chooses `choice` (when given), and the trigger resolves.
fn join(t: &mut TestGame, owner: PlayerId, decl: &[(ObjectId, Entity)], choice: Option<Entity>) {
    let ap = t.obj_now(decl[0].0).controller;
    t.answer_yes(owner, true);
    if let Some(c) = choice {
        t.answer_choose(owner, &[c]);
    }
    declare(t, ap, decl);
    t.resolve_all();
}

#[test]
fn whenever_attacks_abilities_dont_trigger_for_tahngarth() {
    cr!("508.4", "508.3a");
    ruling!(
        "Tahngarth, First Mate",
        "This means that abilities that trigger whenever a creature attacks won't trigger when it changes control and becomes attacking."
    );
    let mut t = TestGame::new(2);
    let tahn = tapped_tahngarth(&mut t, P0);
    // Fervent Charge: "Whenever a creature you control attacks, it gets +2/+2 until end of
    // turn."
    t.battlefield(P1, "Fervent Charge");
    let bears = t.battlefield(P1, "Grizzly Bears");
    join(
        &mut t,
        P0,
        &[(bears, Entity::Player(P0))],
        Some(Entity::Player(P0)),
    );
    assert!(t.g.is_attacking(tahn));
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(tahn), (5, 5));
}

#[test]
fn cant_attack_effects_dont_stop_it_becoming_attacking() {
    cr!("508.4", "702.3b");
    ruling!(
        "Tahngarth, First Mate",
        "They won't stop Tahngarth from becoming an attacking creature through its triggered ability."
    );
    let mut t = TestGame::new(2);
    let tahn = tapped_tahngarth(&mut t, P0);
    modify_until_eot(
        &mut t,
        tahn,
        vec![Modification::AddKeyword(KeywordKind::Defender.into())],
    );
    let bears = t.battlefield(P1, "Grizzly Bears");
    join(
        &mut t,
        P0,
        &[(bears, Entity::Player(P0))],
        Some(Entity::Player(P0)),
    );
    assert!(t.g.is_attacking(tahn));
    assert_eq!(t.obj_now(tahn).controller, P1);
}

#[test]
fn it_stays_tapped_and_may_attack_several_times_between_your_turns() {
    cr!("508.4", "506.4");
    ruling!(
        "Tahngarth, First Mate",
        "Giving control of Tahngarth to an opponent doesn't cause it to become untapped. This means that Tahngarth may end up attacking several times between your turns."
    );
    let mut t = TestGame::new(3);
    let tahn = tapped_tahngarth(&mut t, P0);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    join(
        &mut t,
        P0,
        &[(b1, Entity::Player(P0))],
        Some(Entity::Player(P0)),
    );
    assert!(t.g.is_attacking(tahn) && tapped(&t, tahn));
    next_step(&mut t, Step::EndOfCombat);
    next_step(&mut t, Step::PostcombatMain);
    assert_eq!(t.obj_now(tahn).controller, P0);
    assert!(tapped(&t, tahn));
    t.g.combat = None;
    join(
        &mut t,
        P0,
        &[(b2, Entity::Player(P0))],
        Some(Entity::Player(P0)),
    );
    assert!(t.g.is_attacking(tahn));
    assert_eq!(t.obj_now(tahn).controller, P2);
}

#[test]
fn with_menace_it_cant_be_blocked_at_all() {
    cr!("702.111b", "509.1b");
    ruling!(
        "Tahngarth, First Mate",
        "If Tahngarth gains menace, it can't be blocked at all."
    );
    let mut t = TestGame::new(2);
    let tahn = t.battlefield(P0, "Tahngarth, First Mate");
    modify_until_eot(
        &mut t,
        tahn,
        vec![Modification::AddKeyword(KeywordKind::Menace.into())],
    );
    let w1 = t.battlefield(P1, "Grizzly Bears");
    let w2 = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, P0, &[(tahn, Entity::Player(P1))]);
    t.g.recompute();
    let options = block_options(&t.g, &[P1]);
    assert!(!block_declaration_legal(&t.g, &options, &[(w1, tahn)]));
    assert!(!block_declaration_legal(
        &t.g,
        &options,
        &[(w1, tahn), (w2, tahn)]
    ));
    assert!(block_declaration_legal(&t.g, &options, &[]));
}

#[test]
fn only_players_and_planeswalkers_still_attacked_can_be_chosen() {
    cr!("506.4", "508.4");
    ruling!(
        "Tahngarth, First Mate",
        "that player or planeswalker can't be chosen for Tahngarth to attack. If there's no player or planeswalker to choose this way but you still choose to give the opponent control of Tahngarth, Tahngarth isn't attacking."
    );
    let mut t = TestGame::new(2);
    let tahn = tapped_tahngarth(&mut t, P0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P0)]);
    declare(&mut t, P1, &[(bears, Entity::Player(P0))]);
    t.settle();
    // The attacker leaves before the trigger resolves.
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.obj_now(tahn).controller, P1);
    assert!(!t.g.is_attacking(tahn));
    assert!(attacks_now(&t).is_empty());
}

#[test]
fn it_cant_attack_the_controller_of_an_attacked_planeswalker() {
    cr!("508.4");
    ruling!(
        "Tahngarth, First Mate",
        "If an opponent is attacking a planeswalker but not that planeswalker's controller, Tahngarth can't attack that player."
    );
    let mut t = TestGame::new(3);
    let tahn = tapped_tahngarth(&mut t, P0);
    let jace = t.battlefield(P2, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P0 tries to choose P2: only Jace can be chosen.
    join(&mut t, P0, &[(bears, obj(jace))], Some(Entity::Player(P2)));
    let attacks = attacks_now(&t);
    assert!(attacks.contains(&(tahn, obj(jace))), "{attacks:?}");
    assert!(!attacks.contains(&(tahn, Entity::Player(P2))));
}

#[test]
fn no_one_acts_between_the_choice_and_it_attacking() {
    cr!("608.2", "508.4");
    ruling!(
        "Tahngarth, First Mate",
        "Players can't take any actions between the time you choose to give control of Tahngarth to the attacking player and the time it becomes an attacking creature."
    );
    let mut t = TestGame::new(2);
    let tahn = tapped_tahngarth(&mut t, P0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P0)]);
    declare(&mut t, P1, &[(bears, Entity::Player(P0))]);
    t.settle();
    let from = t.asked().len();
    t.g.resolve_top();
    // One resolution: control changed and it's attacking, with no priority in between.
    assert_eq!(t.obj_now(tahn).controller, P1);
    assert!(t.g.is_attacking(tahn));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Priority { .. })));
}

#[test]
fn commander_damage_counts_whoever_controls_it() {
    cr!("903.10a", "704.6c");
    ruling!(
        "Tahngarth, First Mate",
        "Commander damage is tracked for each commander. It doesn't matter who controls the commander as it deals damage."
    );
    let mut t = TestGame::with_config(
        3,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::Commander,
            ..Default::default()
        },
    );
    t.g.players[2].life = 100;
    let tahn = t.battlefield(P0, "Tahngarth, First Mate");
    t.g.objects[tahn.0 as usize].is_commander = true;
    t.g.players[0]
        .commander_names
        .push("Tahngarth, First Mate".into());
    // Under P0's control: 10 damage to P2.
    modify_until_eot(
        &mut t,
        tahn,
        vec![Modification::ModifyPT(Value::Const(5), Value::Const(0))],
    );
    declare(&mut t, P0, &[(tahn, Entity::Player(P2))]);
    next_step(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P2), 90);
    // Under P1's control: another 11.
    t.set_step(P1, Step::PrecombatMain);
    t.g.combat = None;
    // (The +5/+0 is still there: no cleanup step was performed.)
    modify_until_eot(
        &mut t,
        tahn,
        vec![Modification::ModifyPT(Value::Const(1), Value::Const(0))],
    );
    assert_eq!(t.pt(tahn).0, 11);
    let bears = t.battlefield(P1, "Grizzly Bears");
    join(
        &mut t,
        P0,
        &[(bears, Entity::Player(P2))],
        Some(Entity::Player(P2)),
    );
    assert_eq!(t.obj_now(tahn).controller, P1);
    next_step(&mut t, Step::EndOfCombat);
    assert!(t.has_lost(P2), "{}", t.dump_log());
}
