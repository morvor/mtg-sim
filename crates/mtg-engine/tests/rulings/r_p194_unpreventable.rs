//! Rulings batch P194 — damage that can't be prevented (CR 615.12): regeneration,
//! redirection and replacement effects that don't prevent still apply; prevention shields
//! aren't used up; protection doesn't prevent it; shield counters are removed but don't
//! save a creature from lethal damage.

use crate::r_p194_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s21_common::legal_blocks;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Excruciator (7/7) attacks P1; P1 blocks with `blocker` (if any) and combat ends.
fn excruciator_attacks(t: &mut TestGame, blocker: Option<ObjectId>) -> ObjectId {
    let ex = t.battlefield(P0, "Excruciator");
    attack_with(t, &[(ex, pl(P1))]);
    let blocks: Vec<(ObjectId, ObjectId)> = blocker.map(|b| (b, ex)).into_iter().collect();
    block_and_finish(t, P1, &blocks);
    ex
}

/// `caster` casts Shieldmate's Blessing ("Prevent the next 3 damage that would be dealt
/// to any target this turn") on `target`.
fn shield(t: &mut TestGame, caster: PlayerId, target: Entity) {
    let s = t.hand(caster, "Shieldmate's Blessing");
    mana(t, caster, ManaType::W, 1);
    t.cast(caster, s).target(target).go();
    t.resolve_all();
}

#[test]
fn excruciator_damage_can_be_regenerated_redirected_or_replaced() {
    cr!("615.12", "701.19a", "614.1a");
    ruling!(
        "Excruciator",
        "A creature dealt lethal damage by Excruciator can be regenerated."
    );
    ruling!("Excruciator", "An effect may redirect Excruciator’s damage.");
    ruling!(
        "Excruciator",
        "Replacement effects that don’t use the word “prevent” can replace Excruciator’s damage with something else. See Phytohydra and Szadek, Lord of Secrets, for example."
    );
    supported("Excruciator");
    supported("Drudge Skeletons");
    supported("Palisade Giant");
    supported("Phytohydra");
    // Regeneration.
    let mut t = TestGame::new(2);
    let skel = t.battlefield(P1, "Drudge Skeletons");
    t.set_step(P0, Step::BeginningOfCombat);
    mana(&mut t, P1, ManaType::B, 1);
    t.activate(P1, skel, 0, &[]).unwrap();
    t.resolve_all();
    excruciator_attacks(&mut t, Some(skel));
    assert!(t.on_battlefield(skel));
    assert!(t.obj_now(skel).tapped);
    assert_eq!(t.obj_now(skel).damage, 0);
    // Redirection: Palisade Giant takes the damage dealt to P1.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Palisade Giant");
    excruciator_attacks(&mut t, None);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(giant));
    // Replacement: Phytohydra gets +1/+1 counters instead.
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P1, "Phytohydra");
    excruciator_attacks(&mut t, Some(hydra));
    assert!(t.on_battlefield(hydra));
    assert_eq!(t.counters(hydra, "+1/+1"), 7);
    assert_eq!(t.obj_now(hydra).damage, 0);
}

#[test]
fn excruciator_doesnt_use_up_prevention_shields() {
    cr!("615.12");
    ruling!(
        "Excruciator",
        "Damage prevention shields that would prevent this damage aren’t used up and they stick around for the next time something would deal damage."
    );
    supported("Shieldmate's Blessing");
    let mut t = TestGame::new(2);
    let big = t.battlefield(P1, "Ancient Brontodon");
    shield(&mut t, P1, obj(big));
    excruciator_attacks(&mut t, Some(big));
    assert_eq!(t.obj_now(big).damage, 7);
    // The shield is still there: a Shock is prevented.
    t.set_step(P0, Step::PostcombatMain);
    shock(&mut t, P0, obj(big));
    assert!(t.on_battlefield(big));
    assert_eq!(t.obj_now(big).damage, 7);
}

#[test]
fn excruciator_damages_protection_from_red_but_cant_block_it() {
    cr!("615.12", "702.16e", "702.16f");
    ruling!(
        "Excruciator",
        "Excruciator can deal damage to creatures with protection from red. (It can’t block creatures with protection from red, though.)"
    );
    supported("Kor Firewalker");
    let mut t = TestGame::new(2);
    let fw = t.battlefield(P1, "Kor Firewalker");
    excruciator_attacks(&mut t, Some(fw));
    assert!(!t.on_battlefield(fw));
    // On P1's turn, Excruciator can't block a Firewalker.
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Excruciator");
    let fw = t.battlefield(P1, "Kor Firewalker");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(fw, pl(P0))]);
    assert!(!legal_blocks(&mut t, P0, &[(ex, fw)]));
    assert!(legal_blocks(&mut t, P0, &[]));
}

#[test]
fn call_in_a_professional_removes_a_shield_counter_and_still_deals_damage() {
    cr!("122.1c", "615.12", "704.5g");
    ruling!(
        "Call In a Professional",
        "If Call In a Professional deals damage to a creature with a shield counter on it, that damage will not be prevented, but a shield counter will still be removed. If that damage is enough to be lethal damage, the creature will be destroyed, even if it still has more shield counters on it."
    );
    supported("Call In a Professional");
    for (name, dies) in [("Ancient Brontodon", false), ("Grizzly Bears", true)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, name);
        t.g.add_counters(obj(c), "shield", 2, None);
        let ciap = t.hand(P0, "Call In a Professional");
        mana(&mut t, P0, ManaType::R, 3);
        t.cast(P0, ciap).target(c).go();
        t.resolve_all();
        assert_eq!(t.on_battlefield(c), !dies, "{name}");
        if !dies {
            assert_eq!(t.obj_now(c).damage, 3);
            assert_eq!(t.counters(c, "shield"), 1);
        }
    }
}

#[test]
fn flaring_pain_stops_prevention_but_not_replacement_or_redirection() {
    cr!("615.12", "614.1a");
    ruling!(
        "Flaring Pain",
        "Spells and abilities that replace or redirect damage are not affected by this card."
    );
    ruling!(
        "Flaring Pain",
        "The \"damage can't be prevented\" statement overrides all forms of preventing damage. Damage prevention spells and abilities can still be cast and activated, they just don't do anything."
    );
    supported("Flaring Pain");
    let flaring_pain = |t: &mut TestGame| {
        let fp = t.hand(P0, "Flaring Pain");
        mana(t, P0, ManaType::R, 2);
        t.cast(P0, fp).go();
        t.resolve_all();
    };
    // A prevention spell can be cast, but the Bears still dies to a Shock.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    flaring_pain(&mut t);
    shield(&mut t, P1, obj(bears));
    assert!(t.in_graveyard(P1, "Shieldmate's Blessing"));
    shock(&mut t, P0, obj(bears));
    assert!(!t.on_battlefield(bears));
    // Phytohydra's replacement and Palisade Giant's redirection still apply.
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P1, "Phytohydra");
    flaring_pain(&mut t);
    shock(&mut t, P0, obj(hydra));
    assert_eq!(t.counters(hydra, "+1/+1"), 2);
    assert_eq!(t.obj_now(hydra).damage, 0);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Palisade Giant");
    flaring_pain(&mut t);
    shock(&mut t, P0, pl(P1));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(giant).damage, 2);
}
