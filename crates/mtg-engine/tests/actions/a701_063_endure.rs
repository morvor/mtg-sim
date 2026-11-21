//! CR 701.63: endure.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::kwa::endure::ENDURED;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn spirits(t: &TestGame) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Spirit"))
        .map(|o| o.id)
        .collect()
}

#[test]
fn endure_puts_counters_on_it_or_creates_a_spirit() {
    cr!("701.63a");
    ruling!(
        "Warden of the Grove",
        "You choose whether to put +1/+1 counters on the creature or create a Spirit token as the ability that includes the endure instruction is resolving."
    );
    supported("Fortress Kin-Guard");
    // "When this creature enters, it endures 1."
    let mut t = TestGame::new(2);
    option(&mut t, P0, 0);
    let guard = t.enter(P0, "Fortress Kin-Guard");
    t.resolve_all();
    assert_eq!(t.counters(guard, "+1/+1"), 1);
    assert!(spirits(&t).is_empty());
    // Or an N/N white Spirit creature token.
    let mut t = TestGame::new(2);
    option(&mut t, P0, 1);
    let guard = t.enter(P0, "Fortress Kin-Guard");
    t.resolve_all();
    assert_eq!(t.counters(guard, "+1/+1"), 0);
    let s = spirits(&t);
    assert_eq!(s.len(), 1);
    assert_eq!(t.pt(s[0]), (1, 1));
    assert_eq!(t.obj(s[0]).chars.colors, ColorSet::single(Color::White));
    assert_eq!(t.obj(s[0]).controller, P0);
    // Bigger N: a 3/3.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    option(&mut t, P0, 1);
    run(
        &mut t,
        P0,
        Some(bears),
        ka(KeywordAction::Endure, Sel::This, 3),
        &[],
    );
    assert_eq!(t.pt(spirits(&t)[0]), (3, 3));
}

#[test]
fn a_permanent_that_left_endures_by_creating_a_spirit() {
    cr!("701.63a");
    ruling!(
        "Warden of the Grove",
        "if the creature is no longer on the battlefield), you’ll just create a Spirit token."
    );
    let mut t = TestGame::new(2);
    let guard = t.enter(P0, "Fortress Kin-Guard");
    t.settle();
    t.g.move_object(guard, Zone::Hand(P0), MoveCause::Effect, None);
    option(&mut t, P0, 0);
    t.resolve_all();
    let s = spirits(&t);
    assert_eq!(s.len(), 1);
    assert_eq!(t.obj(s[0]).controller, P0);
}

#[test]
fn enduring_zero_does_nothing() {
    cr!("701.63b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    option(&mut t, P0, 1);
    run(
        &mut t,
        P0,
        Some(bears),
        ka(KeywordAction::Endure, Sel::This, 0),
        &[],
    );
    assert!(spirits(&t).is_empty());
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert!(custom_events(&t, ENDURED).is_empty());
    // The choice wasn't even asked.
    assert!(t
        .asked()
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseOption { .. })));
}
