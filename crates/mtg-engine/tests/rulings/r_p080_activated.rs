//! Rulings batch P080 — activated abilities that answer attackers: Maze of Ith and Ith on
//! untapped attackers, Desert's timing, removal from combat (Labyrinth of Skophos, Spires
//! of Orazca, Hollowhenge Spirit), Metzali's random choice, costs paid on activation
//! (Soul Snare, Gossamer Chains), payments on resolution (Heroism), partial target
//! legality (Spurnmage Advocate) and "attacking creatures get -1/-1" (Shipwreck Singer).

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::damage_on;
use crate::r_s09_common::declare_in_next_combat;
use crate::r_s10_common::attacking;
use crate::r_s21_common::go_to;
use crate::r_s27_common::can_activate_containing;
use mtg_engine::ability::AbilityKind;
use mtg_engine::combat::remove_from_combat;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn maze_of_ith_and_ith_on_an_untapped_attacker_still_prevent_its_damage() {
    cr!("615.1", "506.4b", "702.20b");
    ruling!(
        "Maze of Ith",
        "Maze of Ith can target an untapped attacking creature. It will still prevent the combat damage it would deal and be dealt."
    );
    ruling!(
        "Ith, High Arcanist",
        "Ith's ability can target an untapped attacking creature. It will still prevent any combat damage dealt to and dealt by that creature this turn."
    );
    for name in ["Maze of Ith", "Ith, High Arcanist"] {
        supported(name);
        // P0's Serra Angel (vigilance: untapped) attacks and P1's Craw Wurm blocks it.
        let mut t = TestGame::new(2);
        let source = t.battlefield(P1, name);
        let angel = t.battlefield(P0, "Serra Angel");
        let wurm = t.battlefield(P1, "Craw Wurm");
        to_blockers(&mut t, &[(angel, Entity::Player(P1))], &[(wurm, angel)]);
        assert!(!t.obj_now(angel).tapped);
        t.answer_targets(P1, &[obj(angel)]);
        activate_containing(&mut t, P1, source, "Untap target").expect("activated");
        t.resolve_all();
        go_to(&mut t, Step::EndOfCombat);
        assert!(t.on_battlefield(angel), "{name}");
        assert_eq!((damage_on(&t, angel), damage_on(&t, wurm)), (0, 0), "{name}");
    }
}

#[test]
fn maze_of_ith_has_no_mana_ability() {
    cr!("605.1a");
    ruling!(
        "Maze of Ith",
        "Maze of Ith doesn't have a mana ability. It doesn't tap for colorless mana."
    );
    let mut t = TestGame::new(2);
    let maze = t.battlefield(P0, "Maze of Ith");
    let abilities = t.obj_now(maze).chars.abilities.clone();
    assert_eq!(
        abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
            .count(),
        1
    );
    assert!(abilities.iter().all(|a| !a.is_mana_ability()));
    // With nothing attacking, it can't be activated at all.
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, maze));
}

#[test]
fn desert_hits_any_players_attackers_but_only_in_the_end_of_combat_step() {
    cr!("602.5b", "511.1", "506.2");
    ruling!(
        "Desert",
        "The ability can be used on any player's attacking creatures. This includes your own and creatures in an attack you are not involved in (for multiplayer games)."
    );
    ruling!(
        "Desert",
        "The end of combat step happens after combat damage is dealt. You can't use Desert to destroy an attacking creature before it has a chance to deal combat damage."
    );
    supported("Desert");
    // P0's own Savannah Lions attack P1.
    let mut t = TestGame::new(2);
    let desert = t.battlefield(P0, "Desert");
    let lions = t.battlefield(P0, "Savannah Lions");
    to_blockers(&mut t, &[(lions, Entity::Player(P1))], &[]);
    assert!(!can_activate_containing(&mut t, P0, desert, "damage"));
    go_to(&mut t, Step::CombatDamage);
    assert!(!can_activate_containing(&mut t, P0, desert, "damage"));
    assert_eq!(t.life(P1), 18, "the Lions dealt their damage");
    go_to(&mut t, Step::EndOfCombat);
    t.answer_targets(P0, &[obj(lions)]);
    activate_containing(&mut t, P0, desert, "damage").expect("activated");
    t.resolve_all();
    assert!(!t.on_battlefield(lions));
    // Three players: P1 attacks P2; P0's Desert hits the attacker.
    let mut t = TestGame::new(3);
    let desert = t.battlefield(P0, "Desert");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(lions, Entity::Player(P2))], &[]);
    go_to(&mut t, Step::EndOfCombat);
    t.answer_targets(P0, &[obj(lions)]);
    activate_containing(&mut t, P0, desert, "damage").expect("activated");
    t.resolve_all();
    assert!(!t.on_battlefield(lions));
    assert_eq!(t.life(P2), 18);
}

#[test]
fn labyrinth_of_skophos_creatures_stay_in_combat_until_removed() {
    cr!("506.4", "511.3");
    ruling!(
        "Labyrinth of Skophos",
        "Once creatures attack or block, they remain attacking or blocking creatures through the end of combat step unless removed from combat"
    );
    supported("Labyrinth of Skophos");
    let mut t = TestGame::new(2);
    let lab = t.battlefield(P1, "Labyrinth of Skophos");
    t.lands(P1, "Wastes", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Omens");
    to_blockers(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[(wall, bears)],
    );
    t.answer_targets(P1, &[obj(giant)]);
    activate_containing(&mut t, P1, lab, "Remove target").expect("activated");
    t.resolve_all();
    assert!(!attacking(&t, giant));
    go_to(&mut t, Step::EndOfCombat);
    assert!(attacking(&t, bears));
    assert!(crate::r_s10_common::blocking(&t, wall));
    assert!(!attacking(&t, giant));
    assert_eq!(t.life(P1), 20, "the removed Giant dealt no damage");
}

#[test]
fn spires_of_orazca_removed_attacker_still_attacked_for_raid() {
    cr!("506.4", "508.1k", "207.2c");
    ruling!(
        "Thaumatic Compass // Spires of Orazca",
        "Removing a creature from combat doesn't change the fact that it attacked, even though it's no longer an attacking creature. Notably, raid abilities will still be satisfied if every attacking creature is removed from combat."
    );
    supported("Thaumatic Compass // Spires of Orazca");
    supported("Wingmate Roc");
    let mut t = TestGame::new(2);
    let spires = back_face(&mut t, P1, "Thaumatic Compass // Spires of Orazca");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_targets(P1, &[obj(bears)]);
    activate_containing(&mut t, P1, spires, "remove it from combat").expect("activated");
    t.resolve_all();
    assert!(!attacking(&t, bears));
    assert!(!t.obj_now(bears).tapped);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 20);
    cast_card(&mut t, P0, "Wingmate Roc", &[]).unwrap();
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1, "raid satisfied");
}

#[test]
fn hollowhenge_spirit_removed_creature_can_attack_in_a_later_combat() {
    cr!("506.4", "500.8");
    ruling!(
        "Hollowhenge Spirit",
        "If there are multiple combat phases in a turn, a creature that’s been removed from combat can still attack or block in future combat phases that turn."
    );
    supported("Hollowhenge Spirit");
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_targets(P1, &[obj(bears)]);
    cast_card(&mut t, P1, "Hollowhenge Spirit", &[]).unwrap();
    t.resolve_all();
    assert!(!attacking(&t, bears));
    t.advance_to(P0, Step::PostcombatMain);
    cast_card(&mut t, P0, "Relentless Assault", &[]).unwrap();
    t.resolve_all();
    let attacks = declare_in_next_combat(&mut t, P0, &[(bears, Entity::Player(P1))]);
    assert_eq!(attacks, vec![(bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

/// P1's Metzali (with mana for its last ability) and P0's attackers: Grizzly Bears
/// (blocked by P1's Craw Wurm) and a Hill Giant.
fn metzali_setup(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let metzali = back_face(t, P1, "Path of Mettle // Metzali, Tower of Triumph");
    t.lands(P1, "Plains", 1);
    t.lands(P1, "Wastes", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    (metzali, bears, giant)
}

#[test]
fn metzali_after_combat_damage_chooses_among_survivors() {
    cr!("508.1k", "608.2c");
    ruling!(
        "Path of Mettle // Metzali, Tower of Triumph",
        "Metzali's last ability can be activated after combat damage has been dealt. You'll choose a creature at random that attacked this turn and has survived combat damage."
    );
    ruling!(
        "Path of Mettle // Metzali, Tower of Triumph",
        "Players can't take actions between choosing the creature at random and destroying it."
    );
    let mut t = TestGame::new(2);
    let (metzali, bears, giant) = metzali_setup(&mut t);
    let wurm = t.battlefield(P1, "Craw Wurm");
    to_blockers(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[(wurm, bears)],
    );
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears));
    activate_containing(&mut t, P1, metzali, "at random").expect("activated");
    let from = t.asked().len();
    t.g.resolve_top();
    // No one got priority (or any other decision) between the choice and the destruction.
    assert_eq!(t.asked().len(), from);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn metzali_can_be_activated_before_blockers() {
    cr!("508.1k", "602.2");
    ruling!(
        "Path of Mettle // Metzali, Tower of Triumph",
        "Metzali's last ability can be activated before blockers are chosen."
    );
    let mut t = TestGame::new(2);
    let (metzali, bears, giant) = metzali_setup(&mut t);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    activate_containing(&mut t, P1, metzali, "at random").expect("activated");
    t.resolve_all();
    assert_eq!(
        [bears, giant]
            .iter()
            .filter(|c| t.on_battlefield(**c))
            .count(),
        1
    );
}

#[test]
fn soul_snare_is_sacrificed_as_the_ability_is_activated() {
    cr!("602.2b", "601.2h", "113.7a");
    ruling!(
        "Soul Snare",
        "Sacrificing Soul Snare is part of the cost to activate the ability. Once you activate the ability, no player can do anything to Soul Snare to stop you from activating it."
    );
    supported("Soul Snare");
    let mut t = TestGame::new(2);
    let snare = t.battlefield(P1, "Soul Snare");
    t.lands(P1, "Plains", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_targets(P1, &[obj(bears)]);
    activate_containing(&mut t, P1, snare, "Exile target").expect("activated");
    assert!(t.in_graveyard(P1, "Soul Snare"), "sacrificed while activating");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn gossamer_chains_returns_to_hand_as_its_cost_once() {
    cr!("602.2b", "118.3");
    ruling!(
        "Gossamer Chains",
        "The return of this card to your hand is part of the activation cost and is paid on activation. You do not have a choice to pay this cost zero times or more than one time in order to multiply the effect."
    );
    supported("Gossamer Chains");
    let mut t = TestGame::new(2);
    let chains = t.battlefield(P1, "Gossamer Chains");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    t.answer_targets(P1, &[obj(bears)]);
    activate_containing(&mut t, P1, chains, "Prevent").expect("activated");
    assert!(t.in_hand(P1, "Gossamer Chains"));
    assert_eq!(t.stack_len(), 1, "a single activation");
    assert!(!crate::r_s02_common::can_activate(&mut t, P1, chains));
    t.resolve_all();
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn heroism_payment_is_made_on_resolution_and_other_prevention_still_applies() {
    cr!("608.2c", "615.1", "118.12");
    ruling!(
        "Heroism",
        "The cost to avoid the penalty must be paid when the ability resolves. You can’t wait until later to do so."
    );
    ruling!(
        "Heroism",
        "Paying the Heroism cost to allow a creature to deal damage does not prevent other effects from preventing it from dealing damage. Paying the cost only prevents the Heroism effect."
    );
    supported("Heroism");
    supported("Holy Day");
    // (P0 pays, P1 also casts Holy Day) -> P1's life after combat. P0's red Hill Giant
    // attacks.
    for (pay, holy_day, life) in [(false, false, 20), (true, false, 17), (true, true, 20)] {
        let mut t = TestGame::new(2);
        let heroism = t.battlefield(P1, "Heroism");
        let lions = t.battlefield(P1, "Savannah Lions");
        let giant = t.battlefield(P0, "Hill Giant");
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 2);
        attack_with(&mut t, &[(giant, Entity::Player(P1))]);
        t.answer_choose(P1, &[obj(lions)]);
        activate_containing(&mut t, P1, heroism, "Sacrifice").expect("activated");
        t.answer_yes(P0, pay);
        let from = t.asked().len();
        t.resolve_all();
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
        if holy_day {
            cast_card(&mut t, P1, "Holy Day", &[]).unwrap();
            t.resolve_all();
        }
        let asks = t.asked().len();
        go_to(&mut t, Step::EndOfCombat);
        assert_eq!(t.life(P1), life, "pay {pay} holy day {holy_day}");
        // Not asked again later.
        assert!(!t.asked()[asks..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    }
}

#[test]
fn spurnmage_advocates_both_destroy_with_the_same_card_targets() {
    cr!("608.2b", "115.1");
    ruling!(
        "Spurnmage Advocate",
        "The ability is not countered if both cards are missing from the graveyard on resolution and the creature target is there. This makes it possible to have two Spurnmage Advocates both target the same two cards and end up destroying two creatures."
    );
    supported("Spurnmage Advocate");
    let mut t = TestGame::new(2);
    let a1 = t.battlefield(P1, "Spurnmage Advocate");
    let a2 = t.battlefield(P1, "Spurnmage Advocate");
    let c1 = t.graveyard(P0, "Memnite");
    let c2 = t.graveyard(P0, "Shock");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    for (adv, target) in [(a1, bears), (a2, giant)] {
        t.answer_targets(P1, &[obj(c1), obj(c2)]);
        t.answer_targets(P1, &[obj(target)]);
        activate_containing(&mut t, P1, adv, "Return two").expect("activated");
    }
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant));
    assert!(t.in_hand(P0, "Memnite") && t.in_hand(P0, "Shock"));
}

#[test]
fn shipwreck_singer_affects_only_creatures_already_attacking() {
    cr!("611.2c");
    ruling!(
        "Shipwreck Singer",
        "The last ability affects creatures that are already attacking when it resolves. Activating it before combat won't affect any creatures that attack later that turn."
    );
    supported("Shipwreck Singer");
    for before_combat in [true, false] {
        let mut t = TestGame::new(2);
        let singer = t.battlefield(P1, "Shipwreck Singer");
        t.lands(P1, "Swamp", 1);
        t.lands(P1, "Wastes", 1);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if before_combat {
            activate_containing(&mut t, P1, singer, "Attacking creatures").expect("activated");
            t.resolve_all();
        }
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        if !before_combat {
            activate_containing(&mut t, P1, singer, "Attacking creatures").expect("activated");
            t.resolve_all();
        }
        assert_eq!(
            t.pt(bears),
            if before_combat { (2, 2) } else { (1, 1) },
            "before combat: {before_combat}"
        );
    }
    // Removing from combat doesn't undo it (the set of affected creatures is locked in).
    let mut t = TestGame::new(2);
    let singer = t.battlefield(P1, "Shipwreck Singer");
    t.lands(P1, "Swamp", 1);
    t.lands(P1, "Wastes", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    activate_containing(&mut t, P1, singer, "Attacking creatures").expect("activated");
    t.resolve_all();
    remove_from_combat(&mut t.g, bears);
    assert_eq!(t.pt(bears), (1, 1));
    assert_eq!(t.zone(bears), Zone::Battlefield);
}
