//! A target whose requirement names another target ("target Equipment attached to that
//! creature", "target permanents that player controls") when that other target has become
//! illegal but is still there (CR 608.2b): the requirement is still checked against it, so
//! the dependent target stays legal and is affected (register
//! `dependent-target-checked-against-an-illegal-target`, provisional; work item
//! ruling-conflicts-4, case 1). Fiery Annihilation's ruling that its Equipment "won't be
//! exiled" whenever the creature is an illegal target goes the other way and isn't cited
//! (logged as inconsistent in docs/RULING_DISCREPANCIES.md, for the user to decide).

use crate::r_p007_common::{pl, player_hexproof};
use crate::r_p047_common::obj;
use crate::r_s01_common::{give_mana_for, supported};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn fiery_annihilation_exiles_equipment_still_attached_to_an_illegal_creature() {
    cr!("608.2b", "702.16b");
    interp!("dependent-target-checked-against-an-illegal-target");
    supported("Fiery Annihilation");
    supported("Gods Willing");
    // "Fiery Annihilation deals 5 damage to target creature. Exile up to one target
    // Equipment attached to that creature. ..." P1's Grizzly Bears wear a Bonesplitter
    // (colorless). In response, Gods Willing gives the Bears protection from red: the Bears
    // are an illegal target but still on the battlefield, and the Bonesplitter is still
    // attached to them, so it's still a legal target.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sword = t.battlefield(P1, "Bonesplitter");
    t.g.attach(sword, obj(bears));
    let fa = t.hand(P0, "Fiery Annihilation");
    give_mana_for(&mut t, P0, "Fiery Annihilation");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(sword)]);
    t.cast(P0, fa).go();
    let gw = t.hand(P1, "Gods Willing");
    give_mana_for(&mut t, P1, "Gods Willing");
    choose_color(&mut t, P1, Color::Red);
    t.cast(P1, gw).target(bears).go();
    t.resolve_all();
    // The illegal creature isn't affected: no damage, and it stays on the battlefield.
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    // The Equipment was checked against it, as it is, and exiled.
    assert_eq!(t.zone(sword), Zone::Exile);
    // Control: protection from blue leaves both targets legal; both are exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sword = t.battlefield(P1, "Bonesplitter");
    t.g.attach(sword, obj(bears));
    let fa = t.hand(P0, "Fiery Annihilation");
    give_mana_for(&mut t, P0, "Fiery Annihilation");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(sword)]);
    t.cast(P0, fa).go();
    let gw = t.hand(P1, "Gods Willing");
    give_mana_for(&mut t, P1, "Gods Willing");
    choose_color(&mut t, P1, Color::Blue);
    t.cast(P1, gw).target(bears).go();
    t.resolve_all();
    assert_eq!(t.zone(sword), Zone::Exile);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn yosei_taps_the_permanents_of_a_player_who_became_an_illegal_target() {
    cr!("608.2b", "702.11c");
    ruling!(
        "Yosei, the Morning Star",
        "If the player becomes an illegal target, the target permanents are tapped, but the player doesn't skip their untap step."
    );
    interp!("dependent-target-checked-against-an-illegal-target");
    supported("Yosei, the Morning Star");
    // "When Yosei dies, target player skips their next untap step. Tap up to five target
    // permanents that player controls." `hexproof`: P1 gains hexproof (Leyline of Sanctity)
    // with the trigger on the stack.
    for hexproof in [true, false] {
        let mut t = TestGame::new(2);
        let yosei = t.battlefield(P0, "Yosei, the Morning Star");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_targets(P0, &[obj(bears), obj(giant)]);
        t.g.destroy(yosei, None);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if hexproof {
            player_hexproof(&mut t, P1);
        }
        t.resolve_all();
        // The permanents "that player controls" are checked against P1, illegal or not, and
        // tapped either way.
        assert!(t.obj_now(bears).tapped, "hexproof {hexproof}");
        assert!(t.obj_now(giant).tapped, "hexproof {hexproof}");
        // P1 skips the untap step only if P1 was a legal target.
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(t.obj_now(bears).tapped, !hexproof, "hexproof {hexproof}");
        assert_eq!(t.obj_now(giant).tapped, !hexproof, "hexproof {hexproof}");
    }
}
