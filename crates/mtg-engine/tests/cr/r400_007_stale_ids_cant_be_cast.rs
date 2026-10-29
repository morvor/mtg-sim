//! CR 400.7: a card that changed zones is a new object; the old object can't be cast.

use mtg_engine::events::MoveCause;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_card_that_left_your_hand_cant_be_cast_by_its_old_id() {
    cr!("400.7");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let old = t.hand(P0, "Lightning Bolt");
    t.g.move_object(old, Zone::Exile, MoveCause::Effect, None);
    t.g.flush_events();
    let new = t.g.current(old);
    assert_ne!(new, old);
    assert_eq!(t.zone(new), Zone::Exile);
    let r = t.g.cast_spell(P0, old, CastMethod::Normal);
    assert!(r.is_err(), "{r:?}");
    assert_eq!(t.life(P1), 20);
    assert!(t.g.stack.is_empty());
    assert_eq!(t.zone(new), Zone::Exile, "the card stays in exile");
    // Positive control: a card in hand by its current id can be cast.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(t.g.cast_spell(P0, bolt, CastMethod::Normal).is_ok());
}
