//! Rulings batch P056 — Dazzling Theater ("Creature spells you cast have convoke.")
//! sacrificed while casting a creature spell: mana abilities are activated before the
//! total cost is paid (CR 601.2g–h), and convoke applies as the cost is paid (CR 702.51a),
//! so a spell that lost convoke that way can't be paid for with it.

use crate::r_s01_common::{custom_card, supported};
use mtg_engine::object::Zone;
use mtg_engine::rooms;
use mtg_engine::testing::*;
use mtg_engine::*;

const THEATER: &str = "Dazzling Theater // Prop Room";

/// P0 controls an unlocked Dazzling Theater, an artifact with "Sacrifice an enchantment:
/// Add {C}." and a green creature, and no lands. Returns (Theater, the creature).
fn setup(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported(THEATER);
    let room = t.battlefield(P0, THEATER);
    assert!(rooms::unlock(&mut t.g, room, 0, P0));
    t.g.recompute();
    let altar = custom_card(
        "Enchantment Altar",
        "Artifact",
        "{1}",
        None,
        "Sacrifice an enchantment: Add {C}.",
    );
    t.custom(P0, altar, Zone::Battlefield);
    let bears = t.battlefield(P0, "Grizzly Bears");
    (room, bears)
}

#[test]
fn sacrificing_dazzling_theater_for_mana_removes_convoke_before_payment() {
    cr!("601.2g", "601.2h", "702.51a");
    ruling!(
        "Dazzling Theater // Prop Room",
        "If you sacrifice Dazzling Theater while casting a creature spell (to activate a mana ability, for example), the spell won't have convoke when you pay its costs unless it has convoke some other way."
    );
    let mut t = TestGame::new(2);
    let (room, bears) = setup(&mut t);
    // Grizzly Bears {1}{G}: the only way would be tapping the Bears for {G} with convoke
    // and sacrificing Theater for {1}, but then the spell has no convoke when it's paid.
    let spell = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, spell).try_go().is_err());
    assert_eq!(t.obj_now(spell).zone, Zone::Hand(P0));
    assert_eq!(t.obj_now(room).zone, Zone::Battlefield);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn a_spell_with_convoke_of_its_own_keeps_it_when_theater_is_sacrificed() {
    cr!("601.2g", "601.2h", "702.51a");
    ruling!(
        "Dazzling Theater // Prop Room",
        "the spell won't have convoke when you pay its costs unless it has convoke some other way."
    );
    let mut t = TestGame::new(2);
    let (room, bears) = setup(&mut t);
    let def = custom_card(
        "Convoking Bear",
        "Creature — Bear",
        "{1}{G}",
        Some((2, 2)),
        "Convoke",
    );
    let spell = t.custom(P0, def, Zone::Hand(P0));
    assert!(t.cast(P0, spell).try_go().is_ok());
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.obj_now(room).zone, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Convoking Bear").len(), 1);
}
