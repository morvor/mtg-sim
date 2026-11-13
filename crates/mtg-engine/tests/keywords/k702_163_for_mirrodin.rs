//! CR 702.163 For Mirrodin!

use crate::common_k702_153_167::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::{CardType, Color};
use mtg_engine::*;

/// The Rebel tokens `p` controls.
fn rebels(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.is_token() && o.controller == p && o.chars.has_subtype("Rebel"))
        .map(|o| o.id)
        .collect()
}

#[test]
fn for_mirrodin_creates_a_rebel_and_attaches_the_equipment_to_it() {
    cr!("702.163", "702.163a");
    assert_supported("Mirran Bardiche");
    // Mirran Bardiche: equipped creature gets +2/+1 and has vigilance.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let card = t.hand(P0, "Mirran Bardiche");
    t.cast(P0, card).go();
    t.resolve();
    assert_eq!(triggers_named(&t, "For Mirrodin!").len(), 1);
    t.resolve();
    let r = rebels(&t, P0);
    assert_eq!(r.len(), 1);
    let rebel = r[0];
    let o = t.g.obj(rebel);
    assert!(o.is(CardType::Creature));
    assert_eq!(o.chars.colors.iter().collect::<Vec<_>>(), vec![Color::Red]);
    let bardiche = named(&t, P0, "Mirran Bardiche")[0];
    assert_eq!(t.g.obj(bardiche).attached_to, Some(Entity::Object(rebel)));
    assert_eq!(t.pt(rebel), (4, 3));
    assert!(t.g.obj(rebel).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn the_rebel_enters_as_a_two_two_before_the_equipment_is_attached() {
    cr!("702.163a");
    ruling!(
        "Mirran Bardiche",
        "The Rebel enters the battlefield as a 2/2 creature, then the Equipment becomes attached to it."
    );
    // Elemental Bond: "Whenever a creature you control with power 3 or greater enters,
    // draw a card." The Rebel is 2/2 as it enters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Bond");
    t.enter(P0, "Mirran Bardiche");
    t.resolve_all();
    let rebel = rebels(&t, P0)[0];
    assert_eq!(t.pt(rebel), (4, 3));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn with_two_rebels_the_equipment_is_attached_to_only_one() {
    cr!("702.163a");
    ruling!(
        "Mirran Bardiche",
        "the Equipment becomes attached to only one of them"
    );
    assert_supported("Mondrak, Glory Dominus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mondrak, Glory Dominus");
    t.enter(P0, "Mirran Bardiche");
    t.resolve_all();
    let r = rebels(&t, P0);
    assert_eq!(r.len(), 2);
    let bardiche = named(&t, P0, "Mirran Bardiche")[0];
    let attached = t.g.obj(bardiche).attached_to;
    assert!(r.iter().any(|x| attached == Some(Entity::Object(*x))));
}

#[test]
fn the_rebel_is_created_even_if_the_equipment_left_the_battlefield() {
    cr!("702.163a");
    ruling!(
        "Mirran Bardiche",
        "If the Rebel is destroyed, the Equipment stays on the battlefield."
    );
    let mut t = TestGame::new(2);
    let eq = t.enter(P0, "Mirran Bardiche");
    t.settle();
    t.g.move_object(
        eq,
        Zone::Hand(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    t.resolve_all();
    let r = rebels(&t, P0);
    assert_eq!(r.len(), 1);
    assert_eq!(t.pt(r[0]), (2, 2));
    // With the Equipment on the battlefield, destroying the Rebel leaves the Equipment.
    let mut t = TestGame::new(2);
    t.enter(P0, "Mirran Bardiche");
    t.resolve_all();
    let rebel = rebels(&t, P0)[0];
    t.g.destroy(rebel, None);
    t.settle();
    assert!(!t.on_battlefield(rebel));
    let bardiche = named(&t, P0, "Mirran Bardiche");
    assert_eq!(bardiche.len(), 1);
    assert_eq!(t.g.obj(bardiche[0]).attached_to, None);
}
