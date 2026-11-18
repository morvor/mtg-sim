//! Rulings batch P135 — Auras that destroy or change lands: Kudzu and Steam Vines move
//! without targeting (CR 115.1, 303.4), their triggers after the Aura leaves (CR 608.2h),
//! Uncontrolled Infestation's "becomes tapped" (CR 603.2e), Pooling Venom's activated
//! ability (CR 602.2), and Imprisoned in the Moon turning a permanent into a land (CR
//! 303.4d, 704.5m, 704.5n, 606.3).

use crate::r_p108_common::put_counters;
use crate::r_p127_common::{tap, untap};
use crate::r_p135_common::*;
use crate::r_s02_common::can_activate;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn kudzu_and_steam_vines_move_to_a_land_that_cant_be_targeted() {
    cr!("115.1", "303.4", "701.3a");
    ruling!("Kudzu", "Because the ability isn't targeted, the controller of the destroyed land may attach it to a land that can't be the target of abilities.");
    ruling!(
        "Steam Vines",
        "Moving this card to another land does not target the land."
    );
    for name in ["Kudzu", "Steam Vines"] {
        supported(name);
        let mut t = TestGame::new(2);
        let first = t.battlefield(P1, "Forest");
        let next = t.battlefield(P1, "Island");
        spoil(&mut t, next, Spoil::Hexproof);
        let aura = attach_new(&mut t, P0, name, first);
        t.answer_yes(P1, true);
        t.answer_choose(P1, &[ob(next)]);
        tap(&mut t, first);
        t.resolve_all();
        assert!(!t.on_battlefield(first), "{name}");
        assert_eq!(attached_to(&t, aura), Some(ob(next)), "{name}");
    }
}

#[test]
fn kudzu_goes_to_the_graveyard_when_destroyed_or_its_land_is_destroyed_otherwise() {
    cr!("303.4d", "704.5m");
    ruling!("Kudzu", "If Kudzu is destroyed directly, or the land is destroyed by a spell or ability, then Kudzu goes to the graveyard like any Aura would.");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.battlefield(P1, "Island");
    let k = attach_new(&mut t, P0, "Kudzu", land);
    destroy(&mut t, k);
    assert!(t.in_graveyard(P0, "Kudzu"));
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.battlefield(P1, "Island");
    attach_new(&mut t, P0, "Kudzu", land);
    destroy(&mut t, land);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Kudzu"));
}

#[test]
fn steam_vines_trigger_after_it_left_still_destroys_and_damages() {
    cr!("608.2h", "113.7a", "603.10");
    ruling!("Steam Vines", "If this card leaves the battlefield after it triggers but before it resolves, then the trigger will deal damage and destroy the land, but it will not bring this card back to the battlefield.");
    supported("Steam Vines");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    let other = t.battlefield(P1, "Island");
    let sv = attach_new(&mut t, P0, "Steam Vines", land);
    t.answer_choose(P1, &[ob(other)]);
    tap(&mut t, land);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, sv);
    t.resolve_all();
    assert!(!t.on_battlefield(land));
    assert_eq!(t.life(P1), 19);
    assert!(t.in_graveyard(P0, "Steam Vines"));
    assert!(t.g.permanents().all(|o| o.attached_to != Some(ob(other))));
}

#[test]
fn uncontrolled_infestation_needs_the_land_to_become_tapped_later() {
    cr!("603.2e", "603.2");
    ruling!("Uncontrolled Infestation", "It only triggers if the land becomes tapped after this Aura is on it. Attaching it to a tapped land will not destroy the land until the next time the land is untapped and then tapped again.");
    supported("Uncontrolled Infestation");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Mutavault");
    tap(&mut t, land);
    attach_new(&mut t, P0, "Uncontrolled Infestation", land);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(land));
    untap(&mut t, land);
    assert!(t.on_battlefield(land));
    tap(&mut t, land);
    t.resolve_all();
    assert!(!t.on_battlefield(land));
}

#[test]
fn pooling_venom_ability_is_its_controllers_only() {
    cr!("602.2");
    ruling!(
        "Pooling Venom",
        "Only Pooling Venom’s controller can activate the activated ability."
    );
    supported("Pooling Venom");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.lands(P1, "Swamp", 4);
    t.lands(P0, "Swamp", 4);
    let pv = attach_new(&mut t, P0, "Pooling Venom", land);
    assert!(!can_activate(&mut t, P1, pv));
    assert!(can_activate(&mut t, P0, pv));
    t.activate(P0, pv, 0, &[]).expect("P0's");
    t.resolve_all();
    assert!(!t.on_battlefield(land));
}

#[test]
fn imprisoned_in_the_moon_makes_other_attachments_illegal_and_keeps_counters() {
    cr!("303.4d", "704.5m", "704.5n", "122.1");
    ruling!("Imprisoned in the Moon", "Becoming a land may cause other Auras to become illegally attached. These are put into their owner's graveyard, and any Equipment attached to the land become unattached and remain on the battlefield. Counters on the land remain on it even if they don't do anything.");
    supported("Imprisoned in the Moon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P1, "Holy Strength", bears);
    let eq = attach_new(&mut t, P1, "Bonesplitter", bears);
    put_counters(&mut t, bears, counters::PLUS1, 1);
    cast_modes(&mut t, P0, "Imprisoned in the Moon", &[], &[ob(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Holy Strength"));
    assert!(t.on_battlefield(eq));
    assert_eq!(attached_to(&t, eq), None);
    assert!(t.obj_now(bears).is(CardType::Land));
    assert!(!t.obj_now(bears).is(CardType::Creature));
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn imprisoned_planeswalker_tapped_for_mana_can_still_activate_loyalty_abilities() {
    cr!("606.3", "302.6");
    ruling!("Imprisoned in the Moon", "If you remove Imprisoned in the Moon from a planeswalker after tapping it for mana, you can still activate a loyalty ability of that planeswalker even though it's tapped.");
    let mut t = TestGame::new(2);
    let gid = t.battlefield(P0, "Gideon, Ally of Zendikar");
    let aura = attach_new(&mut t, P1, "Imprisoned in the Moon", gid);
    assert!(t.obj_now(gid).is(CardType::Land));
    tap(&mut t, gid);
    destroy(&mut t, aura);
    assert!(t.obj_now(gid).tapped);
    assert!(t.obj_now(gid).is(CardType::Planeswalker));
    assert!(can_activate(&mut t, P0, gid));
    t.activate(P0, gid, 1, &[])
        .expect("a loyalty ability while tapped");
    t.resolve_all();
    assert_eq!(
        crate::r_s25_common::creature_tokens(&t, P0),
        1,
        "the 0 ability made a Knight"
    );
}
