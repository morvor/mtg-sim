//! CR 506.4: removal from combat.

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::eval::Ctx;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 attacks P1 with Grizzly Bears, which P1 blocks with Hill Giant; returns (bears, giant)
/// at the declare blockers step.
fn bears_blocked_by_giant(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    declare(t, &[(bears, Entity::Player(P1))]);
    block(t, P1, &[(giant, bears)]);
    go_to(t, Step::DeclareBlockers);
    assert!(t.g.is_attacking(bears) && t.g.is_blocking(giant));
    (bears, giant)
}

#[test]
fn leaving_the_battlefield_removes_from_combat() {
    cr!("506.4");
    let mut t = TestGame::new(2);
    let (bears, giant) = bears_blocked_by_giant(&mut t);
    t.g.destroy(giant, None);
    t.g.recompute();
    assert!(!t.g.is_blocking(giant));
    assert!(t.g.blockers().is_empty());
    t.g.destroy(bears, None);
    assert!(!t.g.is_attacking(bears));
    assert!(t.g.attackers().is_empty());
}

#[test]
fn control_change_removes_from_combat() {
    cr!("506.4");
    let mut t = TestGame::new(2);
    let (bears, giant) = bears_blocked_by_giant(&mut t);
    apply(
        &mut t,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &[bears],
    );
    assert_eq!(t.obj_now(bears).controller, P1);
    assert!(!t.g.is_attacking(bears));
    let ctx = Ctx::new(None, P0);
    // It stops being a blocked creature too.
    assert!(!t.g.matches(bears, &Filter::Blocked, &ctx));
    assert!(!t.g.matches(bears, &Filter::Attacking, &ctx));
    // The giant is still a blocking creature, blocking nothing.
    assert!(t.g.is_blocking(giant));
    assert!(t.g.combat.as_ref().unwrap().blocking(giant).is_empty());
}

#[test]
fn phasing_out_removes_from_combat() {
    cr!("506.4");
    let mut t = TestGame::new(2);
    let (bears, giant) = bears_blocked_by_giant(&mut t);
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![giant]);
    assert!(!t.g.is_blocking(giant));
    // The attacker remains blocked (CR 509.1h).
    assert!(is_blocked(&t, bears));
}

#[test]
fn an_effect_can_remove_a_creature_from_combat() {
    cr!("506.4");
    let mut t = TestGame::new(2);
    let (bears, giant) = bears_blocked_by_giant(&mut t);
    apply(
        &mut t,
        P0,
        Effect::RemoveFromCombat {
            what: Sel::Target(0),
        },
        &[bears],
    );
    assert!(!t.g.is_attacking(bears));
    assert!(t.on_battlefield(bears));
    go_to(&mut t, Step::EndOfCombat);
    // Neither dealt nor received combat damage.
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn regenerating_removes_from_combat() {
    cr!("506.4");
    let mut t = TestGame::new(2);
    let (bears, _giant) = bears_blocked_by_giant(&mut t);
    apply(
        &mut t,
        P0,
        Effect::Regenerate {
            what: Sel::Target(0),
        },
        &[bears],
    );
    t.g.destroy(bears, None);
    t.g.recompute();
    assert!(t.on_battlefield(bears), "regenerated");
    assert!(!t.g.is_attacking(bears));
}

#[test]
fn stopping_being_a_creature_or_becoming_a_battle_removes_from_combat() {
    cr!("506.4");
    let mut t = TestGame::new(2);
    let (bears, giant) = bears_blocked_by_giant(&mut t);
    change_types(&mut t, giant, &[], &[CardType::Creature]);
    assert!(!t.g.is_blocking(giant));
    change_types(&mut t, bears, &[CardType::Battle], &[]);
    assert!(!t.g.is_attacking(bears));
}

#[test]
fn attacked_planeswalker_or_battle_that_stops_being_one_is_removed_from_combat() {
    cr!("506.4", "506.4c");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let jace = t.battlefield(P1, "Jace Beleren");
    let battle = t.battlefield(P0, "Invasion of Azgol");
    set_protector(&mut t, battle, P1);
    declare(
        &mut t,
        &[(a, Entity::Object(jace)), (b, Entity::Object(battle))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attack_target(&t, a), Some(Entity::Object(jace)));
    change_types(
        &mut t,
        jace,
        &[CardType::Artifact],
        &[CardType::Planeswalker],
    );
    // Jace is no longer attacked; the creature keeps attacking, attacking nothing.
    assert!(t.g.is_attacking(a));
    assert_eq!(attack_target(&t, a), None);
    change_types(&mut t, battle, &[CardType::Artifact], &[CardType::Battle]);
    assert!(t.g.is_attacking(b));
    assert_eq!(attack_target(&t, b), None);
}

#[test]
fn battle_whose_protector_changes_is_removed_from_combat() {
    cr!("506.4");
    let mut t = TestGame::with_config(3, GameConfig::default());
    let a = t.battlefield(P0, "Grizzly Bears");
    let battle = t.battlefield(P0, "Invasion of Azgol");
    set_protector(&mut t, battle, P1);
    declare(&mut t, &[(a, Entity::Object(battle))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attack_target(&t, a), Some(Entity::Object(battle)));
    set_protector(&mut t, battle, P2);
    assert_eq!(attack_target(&t, a), None);
    assert!(t.g.is_attacking(a));
}

#[test]
fn restrictions_applied_after_declaration_dont_remove_from_combat() {
    cr!("506.4a");
    let mut t = TestGame::new(2);
    let (bears, giant) = bears_blocked_by_giant(&mut t);
    apply(
        &mut t,
        P1,
        Effect::AddRestriction {
            restriction: Restriction::CantAttack(Filter::In(Box::new(Sel::Target(0)))),
            duration: Duration::EndOfTurn,
        },
        &[bears],
    );
    apply(
        &mut t,
        P0,
        Effect::AddRestriction {
            restriction: Restriction::CantBlock(Filter::In(Box::new(Sel::Target(0)))),
            duration: Duration::EndOfTurn,
        },
        &[giant],
    );
    // The restrictions really apply...
    assert!(!t.g.can_attack(bears));
    assert!(!t.g.can_block_at_all(giant));
    // ...but don't remove anything from combat.
    assert!(t.g.is_attacking(bears) && t.g.is_blocking(giant));
    go_to(&mut t, Step::EndOfCombat);
    assert!(
        !t.on_battlefield(bears),
        "the giant dealt its combat damage"
    );
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn tapping_or_untapping_doesnt_remove_from_combat_or_prevent_damage() {
    cr!("506.4b");
    let mut t = TestGame::new(2);
    let (bears, giant) = bears_blocked_by_giant(&mut t);
    assert!(t.obj_now(bears).tapped);
    t.g.untap(bears);
    t.g.tap(giant);
    t.g.flush_events();
    assert!(t.g.is_attacking(bears) && t.g.is_blocking(giant));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.obj_now(giant).damage, 2, "untapped attacker dealt damage");
    assert!(!t.on_battlefield(bears), "tapped blocker dealt damage");
}

#[test]
fn creature_attacking_a_removed_planeswalker_can_be_blocked_and_deals_no_damage() {
    cr!("506.4c");
    // Unblocked: no combat damage.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let jace = t.battlefield(P1, "Jace Beleren");
    declare(&mut t, &[(a, Entity::Object(jace))]);
    go_to(&mut t, Step::DeclareAttackers);
    apply(
        &mut t,
        P1,
        Effect::RemoveFromCombat {
            what: Sel::Target(0),
        },
        &[jace],
    );
    assert!(t.g.is_attacking(a));
    assert_eq!(attack_target(&t, a), None);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(jace, "loyalty"), 3);

    // It may still be blocked.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Object(jace))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.g.destroy(jace, None);
    t.g.recompute();
    assert!(t.g.is_attacking(a));
    block(&mut t, P1, &[(bears, a)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.g.is_blocking(bears));
    assert!(is_blocked(&t, a));
}

/// A permanent that's both a creature and a planeswalker, controlled by P1.
fn creature_walker(t: &mut TestGame) -> ObjectId {
    let def = with_counters_base(
        custom_with(
            "Walking Titan",
            "Legendary Planeswalker Creature — Titan",
            Some((4, 4)),
            vec![],
        ),
        Some(5),
        None,
    );
    bf(t, P1, def)
}

/// P0 attacks the creature-planeswalker with Grizzly Bears and P1 with Hill Giant; the
/// creature-planeswalker blocks the giant.
fn walker_setup(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let w = creature_walker(t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    declare(
        t,
        &[(bears, Entity::Object(w)), (giant, Entity::Player(P1))],
    );
    block(t, P1, &[(w, giant)]);
    go_to(t, Step::DeclareBlockers);
    assert!(t.g.is_blocking(w));
    assert_eq!(attack_target(t, bears), Some(Entity::Object(w)));
    (w, bears, giant)
}

#[test]
fn blocking_planeswalker_creature_stays_in_combat_as_whatever_it_still_is() {
    cr!("506.4d");
    // Stops being a creature: no longer blocking, still attacked.
    let mut t = TestGame::new(2);
    let (w, bears, giant) = walker_setup(&mut t);
    change_types(&mut t, w, &[], &[CardType::Creature]);
    assert!(!t.g.is_blocking(w));
    assert_eq!(attack_target(&t, bears), Some(Entity::Object(w)));
    assert!(is_blocked(&t, giant));

    // Stops being a planeswalker: no longer attacked, still blocking.
    let mut t = TestGame::new(2);
    let (w, bears, _giant) = walker_setup(&mut t);
    change_types(&mut t, w, &[], &[CardType::Planeswalker]);
    assert!(t.g.is_blocking(w));
    assert_eq!(attack_target(&t, bears), None);

    // Stops being both: removed from combat.
    let mut t = TestGame::new(2);
    let (w, bears, _giant) = walker_setup(&mut t);
    change_types(
        &mut t,
        w,
        &[CardType::Artifact],
        &[CardType::Creature, CardType::Planeswalker],
    );
    assert!(!t.g.is_blocking(w));
    assert_eq!(attack_target(&t, bears), None);
}

/// A planeswalker battle protected by `protector` and controlled by `controller`.
fn walker_battle(t: &mut TestGame, controller: PlayerId, protector: PlayerId) -> ObjectId {
    let def = with_counters_base(
        custom_with("Walking Siege", "Planeswalker Battle — Test", None, vec![]),
        Some(5),
        Some(5),
    );
    let id = bf(t, controller, def);
    set_protector(t, id, protector);
    id
}

#[test]
fn attacked_planeswalker_battle_removal_rules() {
    cr!("506.4e");
    // Controlled by its protector (P1): stops being a battle but still a planeswalker →
    // still attacked.
    let mut t = TestGame::new(2);
    let wb = walker_battle(&mut t, P1, P1);
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Object(wb))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attack_target(&t, a), Some(Entity::Object(wb)));
    change_types(&mut t, wb, &[], &[CardType::Battle]);
    assert_eq!(attack_target(&t, a), Some(Entity::Object(wb)));

    // Not controlled by its protector (P0 controls it, P1 protects it): removed.
    let mut t = TestGame::new(2);
    let wb = walker_battle(&mut t, P0, P1);
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Object(wb))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attack_target(&t, a), Some(Entity::Object(wb)));
    change_types(&mut t, wb, &[], &[CardType::Battle]);
    assert_eq!(attack_target(&t, a), None);

    // Stops being a planeswalker but is still a battle: still attacked.
    let mut t = TestGame::new(2);
    let wb = walker_battle(&mut t, P0, P1);
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Object(wb))]);
    go_to(&mut t, Step::DeclareAttackers);
    change_types(&mut t, wb, &[], &[CardType::Planeswalker]);
    assert_eq!(attack_target(&t, a), Some(Entity::Object(wb)));

    // Stops being both: removed.
    let mut t = TestGame::new(2);
    let wb = walker_battle(&mut t, P1, P1);
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(a, Entity::Object(wb))]);
    go_to(&mut t, Step::DeclareAttackers);
    change_types(
        &mut t,
        wb,
        &[CardType::Artifact],
        &[CardType::Planeswalker, CardType::Battle],
    );
    assert_eq!(attack_target(&t, a), None);
    assert!(t.g.is_attacking(a));
}
