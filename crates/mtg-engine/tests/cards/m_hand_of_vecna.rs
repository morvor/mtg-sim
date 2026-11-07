//! Hand of Vecna (hand-written, `src/cards/hand_of_vecna.rs`).

use mtg_engine::card::{card, CardDef};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::object::Zone;
use mtg_engine::*;

fn vecna(t: &mut TestGame) -> ObjectId {
    let mut chars = card("Grizzly Bears").faces[0].chars.clone();
    chars.name = "Vecna".into();
    t.custom(P0, CardDef::custom(chars), Zone::Battlefield)
}

#[test]
fn equipped_creature_gets_the_bonus() {
    cr!("603.2");
    ruling!("Hand of Vecna", "uses the number of cards in your hand as it resolves");
    let mut t = TestGame::new(2);
    let hand = t.battlefield(P0, "Hand of Vecna");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(hand, Entity::Object(bear));
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(bear), (5, 5));
}

#[test]
fn may_choose_a_creature_named_vecna_instead() {
    cr!("603.2");
    ruling!("Hand of Vecna", "you choose which creature to apply the bonus to");
    let mut t = TestGame::new(2);
    let hand = t.battlefield(P0, "Hand of Vecna");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(hand, Entity::Object(bear));
    let v = vecna(&mut t);
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.answer_choose(P0, &[Entity::Object(v)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(v), (4, 4));
    assert_eq!(t.pt(bear), (2, 2));
}

#[test]
fn unattached_it_can_still_pump_vecna() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hand of Vecna");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let v = vecna(&mut t);
    t.hand(P0, "Island");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(v), (3, 3));
    assert_eq!(t.pt(bear), (2, 2));
}
