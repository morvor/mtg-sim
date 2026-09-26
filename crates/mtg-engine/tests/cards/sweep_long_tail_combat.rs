//! "You may have ~ assign its combat damage as though it weren't blocked." (Thorn
//! Elemental, Proud Wildbonder) and "remove [creature] from combat" (Gustcloak Runner,
//! Hollowhenge Spirit).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Thorn Elemental attacks P1 and Grizzly Bears blocks it; P0 answers `unblocked`.
fn thorn(unblocked: bool) -> TestGame {
    let mut t = TestGame::new(2);
    let thorn = t.battlefield(P0, "Thorn Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, unblocked);
    t.attack(&[(thorn, Entity::Player(P1))], &[(bears, thorn)]);
    t
}

#[test]
fn thorn_elemental_may_assign_all_damage_to_the_player() {
    cr!("510.1c");
    ruling!(
        "Thorn Elemental",
        "you choose whether you want to assign all damage to blocking creatures, or if you want to assign all of it to the player"
    );
    assert_supported("Thorn Elemental");
    let t = thorn(true);
    assert_eq!(t.life(P1), 13, "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Or all of it to the blocker.
    let t = thorn(false);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
}

#[test]
fn proud_wildbonder_grants_it_to_tramplers() {
    cr!("510.1c", "613.1f");
    assert_supported("Proud Wildbonder");
    let mut t = TestGame::new(2);
    let wb = t.battlefield(P0, "Proud Wildbonder");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.answer_yes(P0, true);
    t.attack(&[(wb, Entity::Player(P1))], &[(wall, wb)]);
    // A 4/3 trampler blocked by a 0/8 wall would deal nothing to P1 with trample alone.
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    assert_eq!(t.obj_now(wall).damage, 0);
}

#[test]
fn gustcloak_runner_untaps_and_leaves_combat_when_blocked() {
    cr!("506.4");
    assert_supported("Gustcloak Runner");
    let mut t = TestGame::new(2);
    let runner = t.battlefield(P0, "Gustcloak Runner");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_yes(P0, true);
    t.attack(&[(runner, Entity::Player(P1))], &[(ogre, runner)]);
    // Removed from combat: the blocker deals no damage to it, and it's untapped.
    assert!(t.on_battlefield(runner), "{}", t.dump_log());
    assert!(!t.obj_now(runner).tapped);
    assert_eq!(t.obj_now(runner).damage, 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn hollowhenge_spirit_removes_an_attacker_from_combat() {
    cr!("506.4");
    assert_supported("Hollowhenge Spirit");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    assert!(t.g.is_attacking(bears), "{}", t.dump_log());
    let spirit = t.hand(P0, "Hollowhenge Spirit");
    t.cast(P0, spirit).go();
    t.resolve();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(!t.g.is_attacking(bears));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20, "{}", t.dump_log());
}
