//! Rulings batch S20 — attacking: noncreature permanents that become creatures (manlands,
//! animated artifacts, Monuments) are subject to the "summoning sickness" rule (CR 302.6,
//! 508.1a): what matters is how long their controller has controlled them, not how long
//! they've been creatures.

use crate::r_s01_common::*;
use crate::r_s02_common::can_attack;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s09_common::legal_attack;
use crate::r_s20_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Activates the ability of the manland `land` that makes it a creature (paying from mana
/// added to `p`'s pool), and resolves it.
fn animate(t: &mut TestGame, p: PlayerId, land: ObjectId) {
    add_mana(t, p, ManaType::W, 3);
    add_mana(t, p, ManaType::U, 3);
    add_mana(t, p, ManaType::B, 3);
    add_mana(t, p, ManaType::R, 3);
    add_mana(t, p, ManaType::G, 3);
    add_mana(t, p, ManaType::C, 5);
    activate_containing(t, p, land, "becomes").expect("animate");
    t.resolve_all();
    t.g.players[p.idx()].mana_pool.mana.clear();
    assert!(creature_now(t, land));
}

/// From P0's beginning of combat step, which of `attackers` could attack P1 (declaring
/// all of them is legal only if each could); P0 then attacks with those, and they're
/// returned.
fn attack_all(t: &mut TestGame, attackers: &[ObjectId]) -> Vec<ObjectId> {
    to_beginning_of_combat(t, P0);
    let able: Vec<ObjectId> = attackers
        .iter()
        .copied()
        .filter(|a| legal_attack(t, &[(*a, Entity::Player(P1))]))
        .collect();
    let all: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    assert_eq!(legal_attack(t, &all), able.len() == attackers.len());
    if able.is_empty() {
        return able;
    }
    let decl: Vec<(ObjectId, Entity)> = able.iter().map(|a| (*a, Entity::Player(P1))).collect();
    attack_with(t, &decl);
    assert!(able.iter().all(|a| t.g.is_attacking(*a)));
    able
}

#[test]
fn mutavault_can_attack_and_tap_only_if_controlled_since_the_turn_began() {
    cr!("302.6", "508.1a");
    ruling!(
        "Mutavault",
        "A noncreature permanent that turns into a creature can attack, and its {T} abilities can be activated, only if its controller has continuously controlled that permanent since the beginning of their most recent turn. It doesn't matter how long the permanent has been a creature."
    );
    supported("Mutavault");
    let mut t = TestGame::new(2);
    // One Mutavault has been P0's since before the turn; the other was played this turn.
    let old = t.battlefield(P0, "Mutavault");
    let new = entered_this_turn(&mut t, P0, "Mutavault");
    animate(&mut t, P0, old);
    animate(&mut t, P0, new);
    // The new one can't tap for mana while it's a creature.
    assert!(!tap_for_mana(&mut t, P0, new, "Add {C}"));
    // Only the old one attacks, although it became a creature just now.
    to_beginning_of_combat(&mut t, P0);
    assert!(can_attack(&mut t, old));
    assert!(!can_attack(&mut t, new));
    assert_eq!(attack_all(&mut t, &[old, new]), vec![old]);
}

#[test]
fn a_restless_land_animated_the_turn_it_came_under_your_control_cant_tap_or_attack() {
    cr!("302.6", "508.1a");
    ruling!(
        "Restless Cottage",
        "If this becomes a creature but you haven't controlled it continuously since your most recent turn began, you won't be able to activate its mana ability or attack with it that turn."
    );
    supported("Restless Cottage");
    let mut t = TestGame::new(2);
    let new = entered_this_turn(&mut t, P0, "Restless Cottage");
    // It entered tapped; untapped (as by an effect), it could tap for mana as a land.
    t.g.untap(new);
    animate(&mut t, P0, new);
    assert!(!tap_for_mana(&mut t, P0, new, "Add"));
    to_beginning_of_combat(&mut t, P0);
    assert!(!can_attack(&mut t, new));
    // One controlled since the turn began can do both.
    let old = t.battlefield(P0, "Restless Cottage");
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    animate(&mut t, P0, old);
    assert!(tap_for_mana(&mut t, P0, old, "Add"));
    t.g.untap(old);
    assert_eq!(attack_all(&mut t, &[old, new]), vec![old]);
}

#[test]
fn a_restless_land_animated_by_another_effect_still_has_its_attack_trigger() {
    cr!("508.3a", "613.1d");
    ruling!(
        "Restless Cottage",
        "If this becomes a creature because of an effect other than its own ability, its last ability will still trigger whenever it attacks."
    );
    supported("Restless Cottage");
    supported("Living Plane");
    let mut t = TestGame::new(2);
    // Living Plane: "All lands are 1/1 creatures that are still lands."
    t.battlefield(P0, "Living Plane");
    let cottage = t.battlefield(P0, "Restless Cottage");
    assert!(creature_now(&mut t, cottage));
    assert_eq!(attack_all(&mut t, &[cottage]), vec![cottage]);
    // "Whenever this land attacks, create a Food token and exile up to one target card
    // from a graveyard."
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Food").len(), 1);
}

#[test]
fn a_manland_that_changed_control_this_turn_is_summoning_sick() {
    cr!("302.6", "508.1a");
    ruling!(
        "Creeping Tar Pit",
        "A land that becomes a creature may be affected by \"summoning sickness.\" You can't attack with it or use any of its {T} abilities (including its mana abilities) unless it began your most recent turn on the battlefield under your control. Note that summoning sickness cares about when that permanent came under your control, not when it became a creature."
    );
    supported("Creeping Tar Pit");
    let mut t = TestGame::new(2);
    // P1's Tar Pit has been on the battlefield for a long time; P0 gains control of it
    // this turn.
    let stolen = t.battlefield(P1, "Creeping Tar Pit");
    give_control(&mut t, stolen, P0);
    assert_eq!(t.obj_now(stolen).controller, P0);
    animate(&mut t, P0, stolen);
    assert!(!tap_for_mana(&mut t, P0, stolen, "Add"));
    // P0's own Tar Pit became a creature just now, but began the turn under P0's control.
    let own = t.battlefield(P0, "Creeping Tar Pit");
    animate(&mut t, P0, own);
    assert!(tap_for_mana(&mut t, P0, own, "Add"));
    t.g.untap(own);
    assert_eq!(attack_all(&mut t, &[stolen, own]), vec![own]);
}

#[test]
fn a_manland_is_sick_by_when_it_came_under_your_control_not_when_it_entered() {
    cr!("302.6", "508.1a");
    ruling!(
        "Lumbering Falls",
        "Note that summoning sickness cares about when that permanent came under your control, not when it became a creature nor when it entered the battlefield."
    );
    supported("Lumbering Falls");
    let mut t = TestGame::new(2);
    // It entered long ago under P1's control; P0 gains control of it this turn.
    let falls = t.battlefield(P1, "Lumbering Falls");
    give_control(&mut t, falls, P0);
    animate(&mut t, P0, falls);
    assert!(!tap_for_mana(&mut t, P0, falls, "Add"));
    to_beginning_of_combat(&mut t, P0);
    assert!(!can_attack(&mut t, falls));
    // On P0's next turn, it began the turn under P0's control: it can attack.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    assert_eq!(t.obj_now(falls).controller, P0);
    animate(&mut t, P0, falls);
    assert_eq!(attack_all(&mut t, &[falls]), vec![falls]);
}

#[test]
fn an_ensouled_artifact_can_attack_if_it_was_on_the_battlefield_since_the_turn_began() {
    cr!("302.6", "508.1a");
    ruling!(
        "Ensoul Artifact",
        "The resulting artifact creature will be able to attack on your turn if it's been under your control continuously since the turn began. That is, it doesn't matter how long it's been a creature, just how long it's been on the battlefield."
    );
    supported("Ensoul Artifact");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Mind Stone");
    let new = entered_this_turn(&mut t, P0, "Mind Stone");
    for artifact in [old, new] {
        add_mana(&mut t, P0, ManaType::U, 2);
        let ensoul = t.hand(P0, "Ensoul Artifact");
        t.cast(P0, ensoul).target(artifact).go();
        t.resolve_all();
        assert!(creature_now(&mut t, artifact));
        assert_eq!(t.pt(artifact), (5, 5));
    }
    assert_eq!(attack_all(&mut t, &[old, new]), vec![old]);
}

#[test]
fn a_monument_cant_attack_the_turn_it_enters() {
    cr!("302.6", "508.1a");
    ruling!(
        "Atarka Monument",
        "A Monument can't attack on the turn it enters the battlefield."
    );
    supported("Atarka Monument");
    let mut t = TestGame::new(2);
    let monument = entered_this_turn(&mut t, P0, "Atarka Monument");
    animate(&mut t, P0, monument);
    to_beginning_of_combat(&mut t, P0);
    assert!(!can_attack(&mut t, monument));
    assert!(attack_all(&mut t, &[monument]).is_empty());
    // On P0's next turn it can.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    animate(&mut t, P0, monument);
    assert_eq!(attack_all(&mut t, &[monument]), vec![monument]);
}

#[test]
fn artifacts_made_creatures_by_march_of_the_machines_are_subject_to_summoning_sickness() {
    cr!("302.6", "508.1a");
    ruling!(
        "March of the Machines",
        "A noncreature permanent that turns into a creature is subject to the \"summoning sickness\" rule: It can only attack, and its {T} abilities can only be activated, if its controller has continuously controlled that permanent since the beginning of their most recent turn."
    );
    supported("March of the Machines");
    let mut t = TestGame::new(2);
    // "Each noncreature artifact is an artifact creature with power and toughness each
    // equal to its mana value."
    t.battlefield(P0, "March of the Machines");
    let old = t.battlefield(P0, "Mind Stone");
    let new = entered_this_turn(&mut t, P0, "Mind Stone");
    assert!(creature_now(&mut t, old) && creature_now(&mut t, new));
    assert_eq!(t.pt(new), (2, 2));
    // The new Mind Stone's {T} abilities can't be activated; the old one's can.
    assert!(!tap_for_mana(&mut t, P0, new, "Add {C}"));
    assert!(tap_for_mana(&mut t, P0, old, "Add {C}"));
    t.g.untap(old);
    assert_eq!(attack_all(&mut t, &[old, new]), vec![old]);
}

#[test]
fn lands_made_creatures_by_natural_emergence_are_subject_to_summoning_sickness() {
    cr!("302.6", "508.1a");
    ruling!(
        "Natural Emergence",
        "A noncreature permanent that turns into a creature is subject to the “summoning sickness” rule: It can only attack, and its {T} abilities can only be activated, if its controller has continuously controlled that permanent since the beginning of their most recent turn."
    );
    supported("Natural Emergence");
    let mut t = TestGame::new(2);
    // "Lands you control are 2/2 creatures with first strike. They're still lands."
    t.battlefield(P0, "Natural Emergence");
    let old = t.battlefield(P0, "Forest");
    // A land played this turn is a creature that can't tap for mana.
    let new = t.hand(P0, "Forest");
    t.play_land(P0, new).expect("land play");
    let new = t.g.current(new);
    assert!(creature_now(&mut t, new));
    assert!(!tap_for_mana(&mut t, P0, new, "Add {G}"));
    assert!(tap_for_mana(&mut t, P0, old, "Add {G}"));
    t.g.untap(old);
    assert_eq!(attack_all(&mut t, &[old, new]), vec![old]);
}
