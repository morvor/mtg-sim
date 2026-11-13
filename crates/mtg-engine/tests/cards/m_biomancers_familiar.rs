//! Biomancer's Familiar (hand-written, `src/cards/biomancers_familiar.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_next_adapt_ignores_counters() {
    cr!("701.46a");
    ruling!("Biomancer's Familiar", "It just lets the creature adapt despite already having +1/+1 counters");
    let mut t = TestGame::new(2);
    let fam = t.battlefield(P0, "Biomancer's Familiar");
    // Growth-Chamber Guardian: "{2}{G}: Adapt 2."
    let g = t.battlefield(P0, "Growth-Chamber Guardian");
    t.g.add_counters(Entity::Object(g), "+1/+1", 1, None);
    t.activate(P0, fam, 0, &[Entity::Object(g)]).unwrap();
    t.resolve_all();
    t.lands(P0, "Forest", 3);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(g, "+1/+1"), 3);
    // Only the next time.
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(g, "+1/+1"), 3);
}
