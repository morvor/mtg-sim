//! CR 506: combat phase general rules — steps, attacking/defending players, what can
//! attack/block and be attacked, entering attacking/blocking, "alone", "had to attack".

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn attack_options(t: &TestGame, p: PlayerId) -> Vec<(ObjectId, Vec<Entity>)> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::DeclareAttackers { options } if q == p => Some(options),
            _ => None,
        })
        .expect("no attack declaration was asked")
}

fn block_options(t: &TestGame, p: PlayerId) -> Vec<(ObjectId, Vec<ObjectId>)> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::DeclareBlockers { options } if q == p => Some(options),
            _ => None,
        })
        .expect("no block declaration was asked")
}

#[test]
fn combat_phase_steps_in_order() {
    cr!("506.1");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::PostcombatMain);
    assert_eq!(
        steps_this_turn(&t),
        vec![
            Step::BeginningOfCombat,
            Step::DeclareAttackers,
            Step::DeclareBlockers,
            Step::CombatDamage,
            Step::EndOfCombat,
            Step::PostcombatMain
        ]
    );
    assert_eq!(t.life(P1), 18);
}

#[test]
fn no_attackers_skips_declare_blockers_and_combat_damage() {
    cr!("506.1");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    // P0 declares no attackers (the scripted default).
    go_to(&mut t, Step::PostcombatMain);
    assert_eq!(
        steps_this_turn(&t),
        vec![
            Step::BeginningOfCombat,
            Step::DeclareAttackers,
            Step::EndOfCombat,
            Step::PostcombatMain
        ]
    );
}

#[test]
fn first_strike_adds_a_combat_damage_step() {
    cr!("506.1");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    declare(&mut t, &[(knight, Entity::Player(P1))]);
    go_to(&mut t, Step::PostcombatMain);
    assert_eq!(
        steps_this_turn(&t),
        vec![
            Step::BeginningOfCombat,
            Step::DeclareAttackers,
            Step::DeclareBlockers,
            Step::FirstStrikeDamage,
            Step::CombatDamage,
            Step::EndOfCombat,
            Step::PostcombatMain
        ]
    );
}

#[test]
fn active_player_attacks_nonactive_player_and_their_permanents() {
    cr!("506.2", "508.1b");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    // A Siege battle P0 controls that P1 protects can be attacked by P0 (CR 310.8).
    let battle = t.battlefield(P0, "Invasion of Azgol");
    t.g.objects[battle.0 as usize].choices.player = Some(P1);
    go_to(&mut t, Step::DeclareAttackers);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attacking_player, Some(P0));
    assert_eq!(c.defending_players, vec![P1]);
    let opts = attack_options(&t, P0);
    // Only the active player's creatures may attack.
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0].0, mine);
    // The defending player, planeswalkers they control, and battles they protect.
    let targets = &opts[0].1;
    assert!(targets.contains(&Entity::Player(P1)));
    assert!(targets.contains(&Entity::Object(jace)));
    assert!(targets.contains(&Entity::Object(battle)));
    assert!(!targets.contains(&Entity::Object(theirs)));
    assert!(!targets.contains(&Entity::Player(P0)));
}

#[test]
fn multiplayer_active_player_chooses_the_defending_player() {
    cr!("506.2a", "507.1");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: false,
            ..Default::default()
        },
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    // As the beginning of combat step begins, P0 chooses P2.
    t.answer_choose(P0, &[Entity::Player(P2)]);
    go_to(&mut t, Step::BeginningOfCombat);
    assert_eq!(t.g.combat.as_ref().unwrap().defending_players, vec![P2]);
    // The choice is a turn-based action made before anyone gets priority (CR 507.1).
    let asked = t.asked();
    let choose = asked
        .iter()
        .position(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. }))
        .unwrap();
    assert!(!asked[choose..]
        .iter()
        .skip(1)
        .any(|(p, d)| *p != P0 && matches!(d, Decision::Priority { .. })));
    go_to(&mut t, Step::DeclareAttackers);
    let opts = attack_options(&t, P0);
    assert_eq!(opts[0].0, bears);
    assert_eq!(opts[0].1, vec![Entity::Player(P2)]);
}

#[test]
fn multiplayer_attack_multiple_players_makes_all_opponents_defending() {
    cr!("506.2a");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grizzly Bears");
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(t.g.combat.as_ref().unwrap().defending_players, vec![P1, P2]);
    // No choice of defending player was asked.
    assert_eq!(
        count_asked(&t, P0, |d| matches!(d, Decision::ChooseEntities { .. })),
        0
    );
}

#[test]
fn shared_team_turns_active_team_attacks_nonactive_team_defends() {
    cr!("506.2b");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: mtg_engine::game::Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    let a0 = t.battlefield(P0, "Grizzly Bears");
    let a1 = t.battlefield(P1, "Hill Giant");
    let b3 = t.battlefield(P3, "Craw Wurm");
    // The active team's creatures (P0's and P1's) attack as a group.
    declare(
        &mut t,
        &[(a0, Entity::Player(P2)), (a1, Entity::Player(P2))],
    );
    // P3's creature can block a creature attacking its teammate P2 (CR 805.10d).
    block(&mut t, P2, &[(b3, a1)]);
    go_to(&mut t, Step::DeclareBlockers);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attacking_players, vec![P0, P1]);
    assert_eq!(c.defending_players, vec![P2, P3]);
    assert!(t.g.is_attacking(a0) && t.g.is_attacking(a1));
    assert!(t.g.is_blocking(b3));
    go_to(&mut t, Step::EndOfCombat);
    // The defending team's shared life total started at 30 (CR 810.4).
    assert_eq!(t.life(P2), 28);
    assert!(!t.on_battlefield(a1));
}

#[test]
fn only_creatures_attack_and_block_and_only_players_planeswalkers_battles_are_attacked() {
    cr!("506.3");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Sol Ring");
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Sol Ring");
    // Trying to attack a creature is illegal; the declaration is replaced by a legal one.
    declare(&mut t, &[(bears, Entity::Object(their_bears))]);
    go_to(&mut t, Step::DeclareAttackers);
    let opts = attack_options(&t, P0);
    assert_eq!(opts.len(), 1, "only the creature can attack");
    assert!(opts[0].1.iter().all(|e| matches!(e, Entity::Player(_))));
    assert!(!t.g.is_attacking(bears));
    // Next turn: only the creature can block.
    t.clear_answers();
    t.advance_to(P1, Step::BeginningOfCombat);
    let attacker = their_bears;
    declare(&mut t, &[(attacker, Entity::Player(P0))]);
    go_to(&mut t, Step::DeclareBlockers);
    let bopts = block_options(&t, P0);
    assert_eq!(bopts.len(), 1);
    assert_eq!(bopts[0].0, bears);
}

#[test]
fn noncreature_put_onto_battlefield_attacking_or_blocking_isnt_attacking_or_blocking() {
    cr!("506.3a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    let rock = enter_with(
        &mut t,
        P0,
        custom_with("Rock", "Artifact", None, vec![]),
        Some(Entity::Player(P1)),
        None,
    );
    assert!(t.on_battlefield(rock));
    assert!(!t.g.is_attacking(rock));
    // A noncreature put onto the battlefield blocking isn't blocking either.
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let _ = bears;
    t.g.combat.as_mut().unwrap().attackers.clear();
    mtg_engine::combat::put_onto_battlefield_attacking(&mut t.g, attacker, Entity::Player(P1));
    let wall = enter_with(
        &mut t,
        P1,
        custom_with("Stone Wall", "Artifact", None, vec![]),
        None,
        Some(attacker),
    );
    assert!(t.on_battlefield(wall));
    assert!(!t.g.is_blocking(wall));
    assert!(!is_blocked(&t, attacker));
}

#[test]
fn creature_entering_attacking_under_a_nonattacking_players_control_isnt_attacking() {
    cr!("506.3b");
    let mut t = TestGame::new(2);
    to_combat(&mut t, P0);
    let c = enter_with(
        &mut t,
        P1,
        vanilla("Turncoat", 2, 2),
        Some(Entity::Player(P0)),
        None,
    );
    assert!(t.on_battlefield(c));
    assert_eq!(t.obj_now(c).controller, P1);
    assert!(!t.g.is_attacking(c));
    // Under the attacking player's control it would be attacking.
    let d = enter_with(
        &mut t,
        P0,
        vanilla("Recruit", 2, 2),
        Some(Entity::Player(P1)),
        None,
    );
    assert!(t.g.is_attacking(d));
}

#[test]
fn creature_entering_attacking_an_invalid_target_isnt_attacking() {
    cr!("506.3c", "508.4a");
    let mut t = TestGame::new(3);
    let jace = t.battlefield(P1, "Jace Beleren");
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    // A player no longer in the game.
    t.g.player_loses(P2);
    assert!(!t.g.player(P2).in_game());
    let a = enter_with(
        &mut t,
        P0,
        vanilla("A", 1, 1),
        Some(Entity::Player(P2)),
        None,
    );
    assert!(t.on_battlefield(a) && !t.g.is_attacking(a));
    // A permanent that isn't a planeswalker or battle.
    let b = enter_with(
        &mut t,
        P0,
        vanilla("B", 1, 1),
        Some(Entity::Object(their_bears)),
        None,
    );
    assert!(t.on_battlefield(b) && !t.g.is_attacking(b));
    // A planeswalker that's no longer on the battlefield.
    t.g.destroy(jace, None);
    let c = enter_with(
        &mut t,
        P0,
        vanilla("C", 1, 1),
        Some(Entity::Object(jace)),
        None,
    );
    assert!(t.on_battlefield(c) && !t.g.is_attacking(c));
    // A valid target works.
    let d = enter_with(
        &mut t,
        P0,
        vanilla("D", 1, 1),
        Some(Entity::Player(P1)),
        None,
    );
    assert!(t.g.is_attacking(d));
}

#[test]
fn planeswalker_no_longer_controlled_by_a_defending_player_cant_be_attacked_by_entering_creature() {
    cr!("508.4a");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    to_combat(&mut t, P0);
    // P0 gains control of Jace: it's no longer controlled by a defending player.
    t.g.objects[jace.0 as usize].base_controller = P0;
    t.g.recompute();
    let a = enter_with(
        &mut t,
        P0,
        vanilla("A", 1, 1),
        Some(Entity::Object(jace)),
        None,
    );
    assert!(t.on_battlefield(a) && !t.g.is_attacking(a));
    // A battle that's no longer protected by a defending player.
    let battle = t.battlefield(P0, "Invasion of Azgol");
    set_protector(&mut t, battle, P1);
    let b = enter_with(
        &mut t,
        P0,
        vanilla("B", 1, 1),
        Some(Entity::Object(battle)),
        None,
    );
    assert!(t.g.is_attacking(b));
    set_protector(&mut t, battle, P0);
    let c = enter_with(
        &mut t,
        P0,
        vanilla("C", 1, 1),
        Some(Entity::Object(battle)),
        None,
    );
    assert!(t.on_battlefield(c) && !t.g.is_attacking(c));
}

#[test]
fn creature_entering_attacking_after_blockers_are_declared_is_unblocked() {
    cr!("506.3d", "508.4d");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Craw Wurm");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    block(&mut t, P1, &[(wall, bears)]);
    go_to(&mut t, Step::DeclareBlockers);
    let giant = enter_with(
        &mut t,
        P0,
        vanilla("Late Giant", 3, 3),
        Some(Entity::Player(P1)),
        None,
    );
    assert!(t.g.is_attacking(giant));
    let unblocked = Filter::Unblocked;
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.matches(giant, &unblocked, &ctx));
    assert!(!is_blocked(&t, giant));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn creature_entering_attacking_before_blockers_is_neither_blocked_nor_unblocked() {
    cr!("508.4d", "509.1h");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    let giant = enter_with(
        &mut t,
        P0,
        vanilla("Early Giant", 3, 3),
        Some(Entity::Player(P1)),
        None,
    );
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.is_attacking(giant));
    assert!(!t.g.matches(giant, &Filter::Unblocked, &ctx));
    assert!(!t.g.matches(giant, &Filter::Blocked, &ctx));
    // It can still be blocked when blockers are declared.
    block(&mut t, P1, &[(wurm, giant)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(is_blocked(&t, giant));
    assert!(t.g.matches(bears, &Filter::Unblocked, &ctx));
}

#[test]
fn creature_entering_blocking_a_creature_not_attacking_its_controller_isnt_blocking() {
    cr!("506.3e", "509.4a");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareBlockers);
    // P2 isn't being attacked by the bears.
    let c = enter_with(&mut t, P2, vanilla("Bystander", 2, 2), None, Some(bears));
    assert!(t.on_battlefield(c));
    assert!(!t.g.is_blocking(c));
    assert!(!is_blocked(&t, bears));
    // P1's creature entering blocking it is blocking.
    let d = enter_with(&mut t, P1, vanilla("Defender", 2, 2), None, Some(bears));
    assert!(t.g.is_blocking(d));
    assert!(is_blocked(&t, bears));
}

#[test]
fn creature_that_is_also_a_battle_never_enters_attacking_or_blocking() {
    cr!("506.3f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.g.combat.as_mut().unwrap().attackers.clear();
    mtg_engine::combat::put_onto_battlefield_attacking(&mut t.g, bears, Entity::Player(P0));
    assert!(t.g.is_attacking(bears));
    let odd = || custom_with("Odd Siege", "Battle Creature — Siege", Some((3, 3)), vec![]);
    let blocker = enter_with(&mut t, P0, odd(), None, Some(bears));
    assert!(t.on_battlefield(blocker) && !t.g.is_blocking(blocker));
    // Now on P0's turn: entering attacking.
    let mut t = TestGame::new(2);
    to_combat(&mut t, P0);
    let attacker = enter_with(&mut t, P0, odd(), Some(Entity::Player(P1)), None);
    assert!(t.on_battlefield(attacker) && !t.g.is_attacking(attacker));
}

#[test]
fn effect_cant_make_a_battle_a_blocking_creature() {
    cr!("506.3g");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let odd = bf(
        &mut t,
        P0,
        custom_with("Odd Siege", "Battle Creature — Siege", Some((3, 3)), vec![]),
    );
    let normal = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::DeclareBlockers);
    let c = t.g.combat.as_mut().unwrap();
    c.attackers.clear();
    c.blockers_declared = true;
    mtg_engine::combat::put_onto_battlefield_attacking(&mut t.g, bears, Entity::Player(P0));
    // "Have it block an attacking creature" does nothing for the battle...
    assert!(!mtg_engine::combat::block_by_effect(&mut t.g, odd, bears));
    assert!(!t.g.is_blocking(odd));
    assert!(!is_blocked(&t, bears));
    // ...but works for an ordinary creature.
    assert!(mtg_engine::combat::block_by_effect(&mut t.g, normal, bears));
    assert!(t.g.is_blocking(normal));

    // "It's attacking" does nothing for a battle either.
    let mut t = TestGame::new(2);
    let odd = bf(
        &mut t,
        P0,
        custom_with("Odd Siege", "Battle Creature — Siege", Some((3, 3)), vec![]),
    );
    let normal = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    assert!(!mtg_engine::combat::make_attacking(
        &mut t.g,
        odd,
        Entity::Player(P1)
    ));
    assert!(!t.g.is_attacking(odd));
    assert!(mtg_engine::combat::make_attacking(
        &mut t.g,
        normal,
        Entity::Player(P1)
    ));
}

#[test]
fn attacks_alone_and_attacking_alone() {
    cr!("506.5");
    // Rogue Kavu: "Whenever this creature attacks alone, it gets +2/+0 until end of turn."
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Rogue Kavu");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(kavu, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.pt(kavu), (3, 1));
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.matches(kavu, &Filter::AttackingAlone, &ctx));
    assert!(!t.g.matches(bears, &Filter::AttackingAlone, &ctx));

    // With another attacker, it doesn't attack alone.
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Rogue Kavu");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(
        &mut t,
        &[(kavu, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.pt(kavu), (1, 1));
    assert!(!t.g.matches(kavu, &Filter::AttackingAlone, &ctx));
    // Once the other attacker is removed from combat, it's attacking alone — but it still
    // didn't attack alone.
    mtg_engine::combat::remove_from_combat(&mut t.g, bears);
    assert!(t.g.matches(kavu, &Filter::AttackingAlone, &ctx));
    assert!(!t.g.combat.as_ref().unwrap().attacked_alone(kavu));
}

#[test]
fn blocks_alone_and_blocking_alone() {
    cr!("506.5");
    // Craven Hulk: "This creature can't block alone."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let hulk = t.battlefield(P1, "Craven Hulk");
    let other = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    // Blocking alone is illegal; the engine substitutes a legal (empty) block.
    block(&mut t, P1, &[(hulk, a)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(!t.g.is_blocking(hulk));
    // Blocking together with another creature is fine.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let hulk = t.battlefield(P1, "Craven Hulk");
    let other2 = t.battlefield(P1, "Grizzly Bears");
    let _ = other;
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    block(&mut t, P1, &[(hulk, a), (other2, b)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.g.is_blocking(hulk) && t.g.is_blocking(other2));
    let ctx = mtg_engine::eval::Ctx::new(None, P1);
    assert!(!t.g.matches(hulk, &Filter::BlockingAlone, &ctx));
    // After the other blocker leaves combat, the Hulk is blocking alone.
    mtg_engine::combat::remove_from_combat(&mut t.g, other2);
    assert!(t.g.matches(hulk, &Filter::BlockingAlone, &ctx));
}

#[test]
fn attacks_a_player_alone() {
    cr!("506.6");
    // Yuriko: "Whenever a creature you control attacks a player alone, it gains double
    // strike until end of turn."
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Yuriko, Blade of the Mighty");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Craw Wurm");
    let jace = t.battlefield(P1, "Jace Beleren");
    // a attacks P1 alone (c attacks P1's planeswalker, not P1); b attacks P2 alone.
    declare(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P2)),
            (c, Entity::Object(jace)),
        ],
    );
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    let ds = |t: &TestGame, id| {
        t.obj_now(id)
            .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike)
    };
    assert!(ds(&t, a));
    assert!(ds(&t, b));
    assert!(!ds(&t, c));
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.matches(a, &Filter::AttackingPlayerAlone, &ctx));
    assert!(!t.g.matches(c, &Filter::AttackingPlayerAlone, &ctx));

    // Two creatures attacking the same player: neither attacks that player alone.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Yuriko, Blade of the Mighty");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert!(!ds(&t, a) && !ds(&t, b));
    assert!(!t.g.matches(a, &Filter::AttackingPlayerAlone, &ctx));
    // After b leaves combat, a is attacking that player alone.
    mtg_engine::combat::remove_from_combat(&mut t.g, b);
    assert!(t.g.matches(a, &Filter::AttackingPlayerAlone, &ctx));
}

#[test]
fn had_to_attack() {
    cr!("506.7");
    let mut t = TestGame::new(2);
    let berserker = bf(
        &mut t,
        P0,
        custom_card(
            "Eager Berserker",
            "Creature — Human Berserker",
            Some((2, 2)),
            "This creature attacks each combat if able.",
        ),
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A creature with no requirement doesn't "have to attack" even if it attacks.
    declare(
        &mut t,
        &[(berserker, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.had_to_attack(berserker));
    assert!(!t.g.had_to_attack(bears));
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.matches(berserker, &Filter::HadToAttack, &ctx));
    assert!(!t.g.matches(bears, &Filter::HadToAttack, &ctx));

    // A lone creature with no requirements didn't have to attack even though attacking
    // with it was the only way to attack.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.is_attacking(bears));
    assert!(!t.g.had_to_attack(bears));
}
