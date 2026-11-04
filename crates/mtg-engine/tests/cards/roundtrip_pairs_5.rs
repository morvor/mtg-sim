//! roundtrip-pairs-5 (docs/agents/items/roundtrip-pairs-5.md): untriaged round-trip
//! mismatches (the 3+-card `gap:` clusters, two-card clusters and the single-card tail).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn the_deck_of_many_things_creature_owner_loses_when_it_dies() {
    cr!("603.7a", "104.3e", "108.3");
    // "20 | Put a creature card from any graveyard onto the battlefield under your control.
    // When that creature dies, its owner loses the game." "Its owner" is the owner of the
    // creature that died (the delayed ability has no targets): P1's Craw Wurm, so P1 loses.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let deck = t.battlefield(P0, "The Deck of Many Things");
    t.lands(P0, "Plains", 2);
    let wurm = t.graveyard(P1, "Craw Wurm");
    t.g.dice.loaded.push_back(20);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.activate(P0, deck, 0, &[]).unwrap();
    t.resolve_all();
    let wurms = t.named_on_battlefield("Craw Wurm");
    assert_eq!(wurms.len(), 1);
    t.g.destroy(wurms[0], None);
    t.settle();
    t.resolve_all();
    assert!(t.has_lost(P1), "the Wurm's owner loses");
    assert!(!t.has_lost(P0));
}
