//! Timing restrictions: "Activate only during your turn, before attackers are declared."
//! and "Cast this spell only during the declare attackers step and only if you've been
//! attacked this step." (CR 506.8, 601.3, 602.5b).

use mtg_engine::decision::{Action, Answer};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    let ok =
        t.g.legal_actions(p)
            .iter()
            .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card));
    t.g.turn.priority = saved;
    ok
}

fn activatable(t: &mut TestGame, p: PlayerId, src: ObjectId) -> bool {
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    let ok =
        t.g.legal_actions(p)
            .iter()
            .any(|a| matches!(a, Action::Activate { source, .. } if *source == src));
    t.g.turn.priority = saved;
    ok
}

/// Declares attackers for the active player and advances to the declare attackers step.
fn attack_with(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let ap = t.g.turn.active;
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    t.advance_to(ap, Step::DeclareAttackers);
}

#[test]
fn activate_only_during_your_turn_before_attackers_are_declared() {
    cr!("506.8a", "506.8g", "602.5b");
    compiles("Apprentice Sorcerer");
    compiles("Talas Researcher");
    compiles("King's Assassin");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Apprentice Sorcerer");
    // Your precombat main phase and beginning of combat: allowed.
    assert!(activatable(&mut t, P0, sorcerer));
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(activatable(&mut t, P0, sorcerer));
    // Once the declare attackers step has begun (even with no attackers), it's too late.
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(!activatable(&mut t, P0, sorcerer));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!activatable(&mut t, P0, sorcerer));
    // Not during an opponent's turn, even before their attackers are declared.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!activatable(&mut t, P0, sorcerer));
    // The ability itself works as printed.
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P0, "Apprentice Sorcerer");
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn cast_only_if_youve_been_attacked_this_step() {
    cr!("601.3", "508.1");
    compiles("Remove");
    compiles("Defiant Stand");
    compiles("Command of Unsummoning");
    let mut t = TestGame::new(2);
    let remove = t.hand(P1, "Remove");
    t.lands(P1, "Island", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Not before combat.
    assert!(!castable(&mut t, P1, remove));
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(castable(&mut t, P1, remove));
    t.cast(P1, remove).target(bears).go();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn not_attacked_means_not_castable() {
    cr!("601.3");
    // In a three-player game, a creature attacking another player doesn't count.
    let mut t = TestGame::new(3);
    let remove = t.hand(P1, "Remove");
    t.lands(P1, "Island", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P2))]);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(!castable(&mut t, P1, remove));
    let remove2 = t.hand(P2, "Remove");
    t.lands(P2, "Island", 1);
    assert!(castable(&mut t, P2, remove2));
    // After the declare attackers step, no longer.
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(!castable(&mut t, P2, remove2));
}

#[test]
fn cast_only_during_the_declare_blockers_step() {
    cr!("601.3");
    compiles("Dazzling Beauty");
    let mut t = TestGame::new(2);
    let beauty = t.hand(P1, "Dazzling Beauty");
    t.lands(P1, "Plains", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!castable(&mut t, P1, beauty));
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(castable(&mut t, P1, beauty));
    t.advance_to(P0, Step::CombatDamage);
    assert!(!castable(&mut t, P1, beauty));
}

#[test]
fn cast_only_during_your_end_step_or_an_opponents_upkeep() {
    cr!("601.3");
    compiles("Necrologia");
    compiles("Festival");
    let mut t = TestGame::new(2);
    let necro = t.hand(P0, "Necrologia");
    t.lands(P0, "Swamp", 5);
    let festival = t.hand(P0, "Festival");
    t.lands(P0, "Plains", 1);
    assert!(!castable(&mut t, P0, necro) && !castable(&mut t, P0, festival));
    t.set_step(P0, Step::End);
    assert!(castable(&mut t, P0, necro));
    assert!(!castable(&mut t, P0, festival));
    t.set_step(P1, Step::End);
    assert!(!castable(&mut t, P0, necro));
    t.set_step(P1, Step::Upkeep);
    assert!(castable(&mut t, P0, festival));
    t.set_step(P0, Step::Upkeep);
    assert!(!castable(&mut t, P0, festival));
    // Festival: creatures can't attack this turn.
    t.set_step(P1, Step::Upkeep);
    t.cast(P0, festival).go();
    t.resolve();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!t.g.can_attack(bears));
}
