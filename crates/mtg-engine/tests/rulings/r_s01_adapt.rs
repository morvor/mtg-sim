//! Rulings batch S01 — adapt (CR 701.46): "If this permanent has no +1/+1 counters on it,
//! put N +1/+1 counters on it."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn adapt_can_be_activated_with_a_counter_but_adds_none() {
    cr!("701.46a");
    ruling!(
        "Benthic Biomancer",
        "You can always activate an ability that will cause a creature to adapt. As that ability resolves, if the creature has a +1/+1 counter on it for any reason, you simply won't put any +1/+1 counters on it."
    );
    supported("Benthic Biomancer");
    let mut t = TestGame::new(2);
    // "{1}{U}: Adapt 1. Whenever one or more +1/+1 counters are put on this creature, draw
    // a card, then discard a card."
    let bio = t.battlefield(P0, "Benthic Biomancer");
    t.g.add_counters(Entity::Object(bio), "+1/+1", 1, None);
    t.resolve_all();
    let (hand, gy) = (t.hand_size(P0), t.graveyard_size(P0));
    t.lands(P0, "Island", 2);
    // The ability can be activated even though the creature already has a counter.
    t.activate(P0, bio, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(bio, "+1/+1"), 1);
    assert_eq!(t.pt(bio), (2, 2));
    // No counters were put on it, so its loot ability didn't trigger.
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.graveyard_size(P0), gy);
}

#[test]
fn counters_put_on_as_a_permanent_enters_trigger_counter_abilities() {
    cr!("122.6");
    ruling!(
        "Simic Ascendancy",
        "An ability that triggers when counters are put on a permanent will trigger if that permanent somehow enters the battlefield with those counters."
    );
    supported("Simic Ascendancy");
    supported("Spike Feeder");
    let mut t = TestGame::new(2);
    // "Whenever one or more +1/+1 counters are put on a creature you control, put that
    // many growth counters on this enchantment."
    let asc = t.battlefield(P0, "Simic Ascendancy");
    // "This creature enters with two +1/+1 counters on it."
    let feeder = t.enter(P0, "Spike Feeder");
    assert_eq!(t.counters(feeder, "+1/+1"), 2);
    t.resolve_all();
    assert_eq!(t.counters(asc, "growth"), 2);
}
