//! CR 701.53: incubate.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::Layout;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn incubators(t: &TestGame) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Incubator"))
        .map(|o| o.id)
        .collect()
}

#[test]
fn incubate_creates_an_incubator_token_that_enters_with_counters() {
    cr!("701.53a", "701.53b");
    supported("Eyes of Gitaxias");
    // "Incubate 3. Draw a card."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Eyes of Gitaxias");
    t.cast(P0, spell).go();
    t.resolve_all();
    let inc = incubators(&t);
    assert_eq!(inc.len(), 1);
    let inc = inc[0];
    let o = t.obj(inc);
    // The front face: a colorless Incubator artifact, not a creature.
    assert_eq!(o.chars.card_types, CardTypeSet::single(CardType::Artifact));
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert_eq!(o.card.as_ref().map(|c| c.layout), Some(Layout::DoubleFacedToken));
    assert_eq!(t.counters(inc, "+1/+1"), 3);
    // The counters were put on it as it entered: they're part of its zone-change event,
    // which comes before the counters event and the token-created event.
    let idx = |f: &dyn Fn(&Event) -> bool| t.turn_events.iter().position(|e| f(e));
    let entered = idx(&|e| matches!(e, Event::ZoneChange { new, .. } if *new == inc)).unwrap();
    let counters = idx(
        &|e| matches!(e, Event::CountersAdded { target, .. } if *target == Entity::Object(inc)),
    )
    .unwrap();
    let created = idx(&|e| matches!(e, Event::TokenCreated { obj, .. } if *obj == inc)).unwrap();
    assert!(counters < created && entered < created);
    // "{2}: Transform this token." Its back face is a 0/0 Phyrexian artifact creature.
    t.lands(P0, "Island", 2);
    t.activate(P0, inc, 0, &[]).unwrap();
    t.resolve_all();
    let o = t.obj_now(inc);
    assert_eq!(o.chars.name.as_str(), "Phyrexian Token");
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Phyrexian"));
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert_eq!(t.pt(inc), (3, 3));
}

#[test]
fn incubate_zero_creates_an_incubator_without_counters() {
    cr!("701.53a");
    ruling!(
        "Sunfall",
        "you'll incubate 0, creating an Incubator token but putting no +1/+1 counters on it"
    );
    let mut t = TestGame::new(2);
    run(
        &mut t,
        P0,
        None,
        ka(KeywordAction::Incubate, Sel::None, 0),
        &[],
    );
    let inc = incubators(&t);
    assert_eq!(inc.len(), 1);
    assert_eq!(t.counters(inc[0], "+1/+1"), 0);
    // Transformed, it's a 0/0 creature and dies.
    t.lands(P0, "Island", 2);
    t.activate(P0, inc[0], 0, &[]).unwrap();
    t.resolve_all();
    assert!(incubators(&t).is_empty());
    assert!(t.g.battlefield.iter().all(|o| !t.obj(*o).is_token()));
}
