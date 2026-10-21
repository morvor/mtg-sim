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

#[test]
fn gogo_copies_a_creature_and_both_must_attack() {
    cr!("707.2", "508.1d");
    supported("Gogo, Mysterious Mime");
    let mut t = TestGame::new(2);
    let gogo = t.battlefield(P0, "Gogo, Mysterious Mime");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    // A Hill Giant (3/3) named Gogo, and both get +2/+0.
    assert_eq!(t.pt(gogo), (5, 3));
    assert_eq!(t.pt(giant), (5, 3));
    assert!(t.g.obj(gogo).chars.name.starts_with("Gogo"));
    let p1 = Entity::Player(P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(gogo, p1)]));
    assert!(legal_attack(&mut t, &[(gogo, p1), (giant, p1)]));
}

#[test]
fn arcums_whistle_forces_an_unpaid_creature_to_attack() {
    cr!("508.1d");
    supported("Arcum's Whistle");
    let mut t = TestGame::new(2);
    let whistle = t.battlefield(P0, "Arcum's Whistle");
    t.lands(P0, "Island", 3);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::Upkeep);
    // P1 has no mana: the {4} isn't paid.
    t.activate(P0, whistle, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(giant, Entity::Player(P0))]));
}

#[test]
fn arcums_whistle_paid_leaves_the_creature_free() {
    cr!("508.1d");
    let mut t = TestGame::new(2);
    let whistle = t.battlefield(P0, "Arcum's Whistle");
    t.lands(P0, "Island", 3);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Mountain", 4);
    t.set_step(P1, Step::Upkeep);
    t.answer_yes(P1, true);
    t.activate(P0, whistle, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    let tapped = t
        .g
        .permanents()
        .filter(|o| o.controller == P1 && o.tapped)
        .count();
    assert_eq!(tapped, 4);
    to_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[]));
}

#[test]
fn immortal_obligation_creature_attacks_others_while_it_has_a_duty_counter() {
    cr!("701.15b", "508.1c", "509.1b");
    supported("Immortal Obligation");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let card = t.graveyard(P1, "Hill Giant");
    let spell = t.hand(P0, "Immortal Obligation");
    t.lands(P0, "Plains", 2);
    let pw = t.battlefield(P0, "Ajani Goldmane");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(card)).go();
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(t.g.obj(giant).controller, P1);
    assert_eq!(t.counters(giant, "duty"), 1);
    // On P1's turn it must attack, and not P0 or P0's planeswalker.
    t.g.objects[giant.0 as usize].summoning_sick = false;
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(giant, Entity::Player(P0))]));
    assert!(!legal_attack(&mut t, &[(giant, Entity::Object(pw))]));
    assert!(legal_attack(&mut t, &[(giant, Entity::Player(P2))]));
    // It can't block P0's creatures.
    t.g.combat = None;
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(mine, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, &[(giant, mine)]));
    // Without the duty counter, the effects end.
    t.g.objects[giant.0 as usize].counters.remove("duty");
    t.g.dirty = true;
    assert!(legal_blocks(&mut t, &[(giant, mine)]));
}

#[test]
fn the_brothers_war_chapter_two_each_chosen_players_creatures_attack_the_other() {
    cr!("508.1d");
    supported("The Brothers' War");
    let mut card = crate::basic_effects_common::oracle_card(
        "Brothers Test",
        "Sorcery",
        "Choose two target players. Until your next turn, each creature they control attacks the other chosen player each combat if able.",
    );
    card.faces[0].chars.mana_cost = Some(mtg_engine::mana::ManaCost::generic(0));
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    let c = t.custom(P0, card, mtg_engine::object::Zone::Hand(P0));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, c)
        .targets(&[Entity::Player(P1), Entity::Player(P2)])
        .go();
    t.resolve_all();
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P2))]));
    to_combat(&mut t, P2);
    assert!(!legal_attack(&mut t, &[(giant, Entity::Player(P0))]));
    assert!(legal_attack(&mut t, &[(giant, Entity::Player(P1))]));
}

#[test]
fn hollow_warrior_cant_pay_its_attack_cost_with_a_creature_being_declared() {
    cr!("508.1h");
    supported("Hollow Warrior");
    let mut t = TestGame::new(2);
    let hw = t.battlefield(P0, "Hollow Warrior");
    let angel = t.battlefield(P0, "Serra Angel");
    let atk = declare(
        &mut t,
        P0,
        &[(hw, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    // Only the vigilant Angel could be tapped, and it's being declared.
    assert!(!atk.iter().any(|(a, _)| *a == hw));
}

#[test]
fn hollow_warrior_attacks_by_tapping_a_creature_not_attacking() {
    cr!("508.1h");
    let mut t = TestGame::new(2);
    let hw = t.battlefield(P0, "Hollow Warrior");
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let atk = declare(
        &mut t,
        P0,
        &[(hw, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    assert!(atk.iter().any(|(a, _)| *a == hw));
    assert!(t.g.obj(bears).tapped);
    assert!(!t.g.obj(angel).tapped);
}
