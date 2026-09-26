//! CR 702.92 Living weapon.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::{destroy, stack_triggers};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::GameObject;
use mtg_engine::testing::*;
use mtg_engine::types::{CardType, Color};
use mtg_engine::*;

/// The Germ tokens `p` controls.
fn germs(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.is_token() && o.controller == p && o.chars.has_subtype("Germ"))
        .map(|o| o.id)
        .collect()
}

#[test]
fn living_weapon_creates_a_germ_and_attaches_the_equipment_to_it() {
    cr!("702.92", "702.92a");
    assert_supported("Batterskull");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 5);
    let skull = t.hand(P0, "Batterskull");
    t.cast(P0, skull).go();
    t.resolve();
    assert_eq!(stack_triggers(&t, "Living Weapon").len(), 1);
    t.resolve();
    let germ = germs(&t, P0);
    assert_eq!(germ.len(), 1);
    let germ = germ[0];
    let o = t.g.obj(germ);
    assert!(o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Phyrexian"));
    assert_eq!(o.chars.colors.iter().collect::<Vec<_>>(), vec![Color::Black]);
    let skull = t.named_on_battlefield("Batterskull")[0];
    assert_eq!(t.g.obj(skull).attached_to, Some(Entity::Object(germ)));
    // The 0/0 Germ survives thanks to the Equipment: 4/4 with vigilance and lifelink.
    assert_eq!(t.pt(germ), (4, 4));
    assert!(t.g.obj(germ).has_keyword(KeywordKind::Vigilance));
    assert!(t.g.obj(germ).has_keyword(KeywordKind::Lifelink));
}

#[test]
fn the_germ_dies_once_the_equipment_moves_to_another_creature() {
    cr!("702.92a", "704.5f");
    ruling!(
        "Batterskull",
        "Once the Germ token is no longer equipped, it will be put into your graveyard and subsequently cease to exist"
    );
    let mut t = TestGame::new(2);
    t.enter(P0, "Batterskull");
    t.resolve_all();
    let germ = germs(&t, P0)[0];
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 5);
    let skull = t.named_on_battlefield("Batterskull")[0];
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_named(&mut t, P0, skull, "Equip", 0).expect("equip");
    t.resolve_all();
    assert!(!t.g.is_live(germ));
    assert!(germs(&t, P0).is_empty());
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn the_equipment_stays_when_the_germ_is_destroyed() {
    cr!("702.92a");
    ruling!(
        "Batterskull",
        "If the Germ token is destroyed, the Equipment remains on the battlefield as with any other Equipment."
    );
    let mut t = TestGame::new(2);
    let skull = t.enter(P0, "Batterskull");
    t.resolve_all();
    let germ = germs(&t, P0)[0];
    destroy(&mut t, germ);
    t.settle();
    assert!(germs(&t, P0).is_empty());
    assert!(t.on_battlefield(skull));
    assert_eq!(t.g.obj(skull).attached_to, None);
}

#[test]
fn the_germ_is_still_created_if_the_equipment_left_the_battlefield() {
    cr!("702.92a");
    let mut t = TestGame::new(2);
    let skull = t.enter(P0, "Batterskull");
    t.settle();
    assert_eq!(stack_triggers(&t, "Living Weapon").len(), 1);
    // In response, the Equipment returns to its owner's hand.
    t.lands(P0, "Wastes", 3);
    activate_named(&mut t, P0, skull, "{3}: Return ~ to its owner's hand.", 0)
        .expect("bounce");
    t.resolve();
    assert!(t.in_hand(P0, "Batterskull"));
    // The Germ is created, unequipped, and dies as a 0/0.
    t.resolve_all();
    assert!(germs(&t, P0).is_empty());
    let created: Vec<&GameObject> = t
        .g
        .objects
        .iter()
        .filter(|o| o.is_token() && o.chars.has_subtype("Germ"))
        .collect();
    assert!(!created.is_empty());
    assert!(created.iter().all(|o| !t.g.is_live(o.id)));
    assert!(!t.g.player(P0).graveyard.iter().any(|c| t.g.obj(*c).is_token()));
}
