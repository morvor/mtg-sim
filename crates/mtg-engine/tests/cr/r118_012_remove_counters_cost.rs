//! CR 118.12, 118.3: "You may remove N counters from it. If you do, ..." — removing the
//! counters is a cost paid on resolution, and a cost can't be paid in part: with fewer
//! than N such counters the player can't choose to remove them, and "if you do" is false.
//! "You may remove all counters from it" can be chosen whatever their number.

use crate::r506_common::custom_card;
use mtg_engine::card::card;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn too_few_counters_to_remove_means_you_dont() {
    cr!("118.12", "118.3");
    let c = card("Nazar, the Velvet Fang");
    assert!(c.is_fully_supported(), "{:?}", c.unsupported_text());
    // "Whenever Nazar attacks, you may remove three feeding counters from it. If you do,
    // you draw three cards and you lose 3 life."
    for (n, drew) in [(2u32, false), (3, true)] {
        let mut t = TestGame::new(2);
        let nazar = t.battlefield(P0, "Nazar, the Velvet Fang");
        t.g.add_counters(Entity::Object(nazar), "feeding", n, None);
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.attack(&[(nazar, Entity::Player(P1))], &[]);
        assert_eq!(t.hand_size(P0), hand + if drew { 3 } else { 0 }, "{n}");
        assert_eq!(t.life(P0), if drew { 17 } else { 20 }, "{n}");
        assert_eq!(
            t.counters(nazar, "feeding"),
            if drew { 0 } else { n },
            "{n}"
        );
    }
}

#[test]
fn removing_all_counters_can_be_chosen_whatever_their_number() {
    cr!("118.12");
    let def = custom_card(
        "Counter Scrubber",
        "Creature — Human",
        Some((2, 2)),
        "Whenever this creature attacks, you may remove all counters from up to one target permanent or opponent. If you do, draw a card.",
    );
    // Two +1/+1 counters and a charge counter: all three are removed, and P0 draws.
    let mut t = TestGame::new(2);
    let scrubber = t.custom(P0, def, Zone::Battlefield);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bear), "+1/+1", 2, None);
    t.g.add_counters(Entity::Object(bear), "charge", 1, None);
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.answer_yes(P0, true);
    t.attack(&[(scrubber, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(bear, "+1/+1"), 0);
    assert_eq!(t.counters(bear, "charge"), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
}
