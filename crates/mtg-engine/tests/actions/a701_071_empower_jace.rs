//! CR 701.71: empower Jace.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn jaces(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype("Jace"))
        .map(|o| o.id)
        .collect()
}

#[test]
fn empower_jace_creates_a_jace_token_if_needed_and_adds_loyalty() {
    cr!("701.71a");
    supported("Protege's Awakening");
    // "Empower Jace 6. Draw a card."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let spell = t.hand(P0, "Protege's Awakening");
    t.cast(P0, spell).go();
    t.resolve_all();
    let j = jaces(&t, P0);
    assert_eq!(j.len(), 1);
    let jace = j[0];
    let o = t.obj(jace);
    assert!(o.is(CardType::Planeswalker));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
    assert_eq!(t.counters(jace, "loyalty"), 6);
    let loyalty: Vec<i32> = o
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Activated(act) if act.is_loyalty => {
                act.cost.parts.iter().find_map(|p| match p {
                    CostPart::Loyalty(n) => Some(*n),
                    _ => None,
                })
            }
            _ => None,
        })
        .collect();
    assert_eq!(loyalty, vec![-1, -3]);
    // "[−3]: Draw a card."
    let hand = t.hand_size(P0);
    t.activate(P0, jace, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(jace, "loyalty"), 3);
    // With a Jace token, no new token: that one gets the loyalty counters.
    run(
        &mut t,
        P0,
        None,
        ka(KeywordAction::EmpowerJace, Sel::None, 2),
        &[],
    );
    assert_eq!(jaces(&t, P0), vec![jace]);
    assert_eq!(t.counters(jace, "loyalty"), 5);
    // Another player's Jace token isn't yours: you create your own.
    run(
        &mut t,
        P1,
        None,
        ka(KeywordAction::EmpowerJace, Sel::None, 1),
        &[],
    );
    assert_eq!(jaces(&t, P1).len(), 1);
    assert_eq!(t.counters(jaces(&t, P1)[0], "loyalty"), 1);
    assert_eq!(t.counters(jace, "loyalty"), 5);
}
