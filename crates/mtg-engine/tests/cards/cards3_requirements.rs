//! Combat grammar IV: attack and block requirements, restrictions and evasion scoped to a
//! combat (CR 508.1c-d, 509.1b-c): Noctis, Heir Apparent.

use crate::combat2_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn noctis_warp_strike_returns_attacking_and_unblockable_that_combat() {
    cr!("508.4", "509.1b");
    supported("Noctis, Heir Apparent");
    let mut t = TestGame::new(2);
    let noctis = t.battlefield(P0, "Noctis, Heir Apparent");
    t.lands(P0, "Mountain", 3);
    let blocker = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, noctis, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Noctis, Heir Apparent"));
    // No attackers are declared; Noctis returns attacking as the step begins.
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let n = t.named_on_battlefield("Noctis, Heir Apparent");
    assert_eq!(n.len(), 1);
    assert!(t.g.obj(n[0]).tapped);
    assert!(t.g.is_attacking(n[0]));
    assert!(!legal_blocks(&mut t, &[(blocker, n[0])]));
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn slicer_goes_to_the_opponent_goaded_and_unsacrificeable() {
    cr!("701.15a");
    supported("Slicer, Hired Muscle // Slicer, High-Speed Antagonist");
    let mut t = TestGame::new(2);
    let slicer = t.battlefield(P0, "Slicer, Hired Muscle // Slicer, High-Speed Antagonist");
    t.g.objects[slicer.0 as usize].tapped = true;
    t.set_step(P0, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let o = t.g.obj(slicer);
    assert_eq!(o.controller, P1);
    assert!(!o.tapped);
    assert!(t.g.goaders(slicer).contains(&P0));
    assert!(t.g.cant_be_sacrificed(slicer));
    // Back under P0's control next turn, and sacrificeable again.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.g.obj(slicer).controller, P0);
    assert!(!t.g.cant_be_sacrificed(slicer));
}

#[test]
fn capricopian_the_attacked_player_pays_to_grow_it_and_send_it_elsewhere() {
    cr!("508.7");
    supported("Capricopian");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let goat = t.battlefield(P0, "Capricopian");
    t.g.objects[goat.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Forest", 2);
    declare(&mut t, P0, &[(goat, Entity::Player(P1))]);
    // Only the player it's attacking may activate the ability.
    assert!(t.activate(P0, goat, 0, &[]).is_err());
    t.answer_choose(P1, &[Entity::Player(P2)]);
    t.activate(P1, goat, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(goat, "+1/+1"), 3);
    let target = t
        .g
        .combat
        .as_ref()
        .and_then(|c| c.attack_target(goat));
    assert_eq!(target, Some(Entity::Player(P2)));
    // Now P1 isn't the player it's attacking.
    assert!(t.activate(P1, goat, 0, &[]).is_err());
}

#[test]
fn portal_manipulator_redirects_attackers_to_the_chosen_player() {
    cr!("508.7");
    supported("Portal Manipulator");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let pm = t.hand(P0, "Portal Manipulator");
    t.lands(P0, "Island", 4);
    declare(
        &mut t,
        P1,
        &[(bears, Entity::Player(P0)), (giant, Entity::Player(P0))],
    );
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, pm).go();
    t.resolve_all();
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attack_target(bears), Some(Entity::Player(P2)));
    assert_eq!(c.attack_target(giant), Some(Entity::Player(P0)));
}

#[test]
fn galactus_must_attack_the_opponent_with_the_most_life_unless_the_herald_is_around() {
    cr!("508.1d");
    supported("Galactus, Devourer of Worlds");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let g = t.battlefield(P0, "Galactus, Devourer of Worlds");
    t.g.player_mut(P2).life = 15;
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(g, Entity::Player(P2))]));
    assert!(legal_attack(&mut t, &[(g, Entity::Player(P1))]));
    t.battlefield(P0, "Silver Surfer, Galactus's Herald");
    assert!(legal_attack(&mut t, &[]));
}
