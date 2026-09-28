//! Rulings batch S22 — Galvanoth: "At the beginning of your upkeep, you may look at the
//! top card of your library. You may cast it without paying its mana cost if it's an
//! instant or sorcery spell." Casting a spell without paying its mana cost still requires
//! paying its mandatory additional costs (CR 118.9b, 601.2h).

use crate::r_s01_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's upkeep begins with Galvanoth on the battlefield and `top` on top of P0's library;
/// the trigger resolves. Returns the top card.
fn galvanoth_upkeep(t: &mut TestGame, top: &str) -> ObjectId {
    t.battlefield(P0, "Galvanoth");
    let card = t.library_top(P0, top);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    card
}

#[test]
fn galvanoth_the_free_spells_mandatory_additional_costs_must_be_paid() {
    cr!("118.9b", "601.2h", "608.2g");
    ruling!(
        "Galvanoth",
        "If the card has any mandatory additional costs, as Kuldotha Rebirth does, you must pay them in order to cast the spell."
    );
    supported("Galvanoth");
    supported("Kuldotha Rebirth");
    // Kuldotha Rebirth {R}: "As an additional cost to cast this spell, sacrifice an
    // artifact. Create three 1/1 red Goblin creature tokens." With an artifact to
    // sacrifice, it's cast for free and the artifact is sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    galvanoth_upkeep(&mut t, "Kuldotha Rebirth");
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert!(t.in_graveyard(P0, "Kuldotha Rebirth"));
    assert_eq!(with_subtype(&t, P0, "Goblin").len(), 3);
    assert_eq!(tapped_lands(&t, P0), 0);
    // Without an artifact, it can't be cast: it stays on top of the library.
    let mut t = TestGame::new(2);
    let card = galvanoth_upkeep(&mut t, "Kuldotha Rebirth");
    assert_eq!(t.zone(card), Zone::Library(P0));
    assert_eq!(t.g.library_top(P0), Some(t.g.current(card)));
    assert!(with_subtype(&t, P0, "Goblin").is_empty());
    // A creature card isn't cast.
    let mut t = TestGame::new(2);
    let card = galvanoth_upkeep(&mut t, "Grizzly Bears");
    assert_eq!(t.zone(card), Zone::Library(P0));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}
