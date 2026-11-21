//! CR 702.167c: abilities of crafted permanents that refer to "the exiled cards used to
//! craft" them, on real cards.

use crate::common_k702_140_152::{ability_uid, activate_uid};
use crate::common_k702_153_167::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Activates the craft ability of `src` exiling `mats`, and resolves it (and whatever
/// triggers); returns the permanent it became.
fn craft(t: &mut TestGame, p: PlayerId, src: ObjectId, mats: &[ObjectId]) -> ObjectId {
    let uid = ability_uid(t, src, "Craft");
    let es: Vec<Entity> = mats.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(p, &es);
    activate_uid(t, p, src, uid).expect("craft");
    t.resolve_all();
    t.g.current(src)
}

/// Moves a card out of exile to its owner's graveyard.
fn out_of_exile(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let owner = t.obj(id).owner;
    t.g.move_object(
        id,
        Zone::Graveyard(owner),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    t.g.recompute();
}

#[test]
fn mastercraft_raptors_power_is_the_total_power_of_the_cards_used_to_craft_it() {
    cr!("702.167c");
    assert_supported("Saheeli's Lattice // Mastercraft Raptor");
    // Saheeli's Lattice: "Craft with one or more Dinosaurs {4}{R}"; Mastercraft Raptor is
    // a */4 whose power is the total power of the exiled cards used to craft it.
    let mut t = TestGame::new(2);
    let lattice = t.battlefield(P0, "Saheeli's Lattice");
    let dreadmaw = t.battlefield(P0, "Colossal Dreadmaw");
    let raptors = t.graveyard(P0, "Ranging Raptors");
    // A Dinosaur card in exile that wasn't used to craft it doesn't count.
    t.exile(P0, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 5);
    let raptor = craft(&mut t, P0, lattice, &[dreadmaw, raptors]);
    assert_eq!(t.obj(raptor).chars.name, "Mastercraft Raptor");
    // 6 (Colossal Dreadmaw) + 2 (Ranging Raptors).
    assert_eq!(t.pt(raptor), (8, 4));
    // Once one of them leaves exile, it isn't a card used to craft it any more.
    out_of_exile(&mut t, dreadmaw);
    assert_eq!(t.pt(raptor), (2, 4));
}

#[test]
fn an_exiled_cards_characteristic_defining_ability_applies_in_exile() {
    cr!("702.167c", "604.3");
    ruling!(
        "Saheeli's Lattice // Mastercraft Raptor",
        "If any of the exiled cards has a characteristic-defining ability that defines its power, that ability will apply."
    );
    // A Dinosaur whose power is defined by the number of cards in its owner's hand.
    let def = custom_card(
        "Hoarding Saurian",
        "Creature — Dinosaur",
        Some((0, 4)),
        "~'s power is equal to the number of cards in your hand.",
    );
    let mut t = TestGame::new(2);
    let lattice = t.battlefield(P0, "Saheeli's Lattice");
    let saurian = t.custom(P0, def, Zone::Graveyard(P0));
    t.lands(P0, "Mountain", 5);
    let raptor = craft(&mut t, P0, lattice, &[saurian]);
    assert_eq!(t.obj(raptor).chars.name, "Mastercraft Raptor");
    assert_eq!(t.pt(raptor).0, 0);
    // Its power changes with the number of cards in P0's hand.
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.g.recompute();
    assert_eq!(t.pt(raptor).0, 2);
}

#[test]
fn jadeheart_attendant_gains_life_equal_to_the_mana_value_of_the_card_used_to_craft_it() {
    cr!("702.167c");
    assert_supported("Jade Seedstones // Jadeheart Attendant");
    // Jade Seedstones: "Craft with creature {5}{G}{G}"; Jadeheart Attendant: "When this
    // creature enters, you gain life equal to the mana value of the exiled card used to
    // craft it."
    let mut t = TestGame::new(2);
    let seedstones = t.battlefield(P0, "Jade Seedstones");
    let giant = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Forest", 7);
    let attendant = craft(&mut t, P0, seedstones, &[giant]);
    assert_eq!(t.obj(attendant).chars.name, "Jadeheart Attendant");
    assert_eq!(t.life(P0), 24);
}

#[test]
fn jadeheart_attendant_counts_every_card_used_to_craft_it() {
    cr!("702.167c", "707.2");
    ruling!(
        "Jade Seedstones // Jadeheart Attendant",
        "The enters-the-battlefield ability will count all the exiled cards, and you'll gain life equal to their total mana values."
    );
    // Jade Seedstones becomes a copy of Sunbird Standard ("Craft with one or more {5}"),
    // crafts with two cards, and returns as Jadeheart Attendant.
    let mut t = TestGame::new(2);
    let seedstones = t.battlefield(P0, "Jade Seedstones");
    let sunbird = t.battlefield(P0, "Sunbird Standard");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![
        vec![Entity::Object(seedstones)],
        vec![Entity::Object(sunbird)],
    ];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert_eq!(t.obj(seedstones).chars.name, "Sunbird Standard");
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    let attendant = craft(&mut t, P0, seedstones, &[giant, bears]);
    assert_eq!(t.obj(attendant).chars.name, "Jadeheart Attendant");
    // 4 + 2.
    assert_eq!(t.life(P0), 26);
}
