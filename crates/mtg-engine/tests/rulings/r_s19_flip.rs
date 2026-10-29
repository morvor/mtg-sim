//! Rulings on flip cards (CR 710): the Ascendants of Saviors of Kamigawa.

use crate::r_s01_common::*;
use crate::r_s03_common::{choice_candidates, in_hand_with_mana};
use mtg_engine::flip;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn an_ascendant_is_legendary_unflipped_and_flipped() {
    cr!("710.1a", "710.2", "704.5j");
    ruling!(
        "Kuon, Ogre Ascendant // Kuon's Essence",
        "Each Ascendant is legendary in both its unflipped and flipped forms"
    );
    supported("Kuon, Ogre Ascendant // Kuon's Essence");
    supported("Time of Need");
    // Time of Need: "Search your library for a legendary creature card, reveal it, put it
    // into your hand, then shuffle." It can find Kuon in the library.
    let mut t = TestGame::new(2);
    let kuon = t.library_top(P0, "Kuon, Ogre Ascendant // Kuon's Essence");
    let bears = t.library_top(P0, "Grizzly Bears");
    let spell = in_hand_with_mana(&mut t, P0, "Time of Need");
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(kuon)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let offered = choice_candidates(&t, from, "");
    assert!(offered.iter().any(|c| c.contains(&Entity::Object(kuon))));
    assert!(offered.iter().all(|c| !c.contains(&Entity::Object(bears))));
    assert!(t.in_hand(P0, "Kuon, Ogre Ascendant"));
    // Flipped, Kuon's Essence is a legendary enchantment: two of them under one player's
    // control are subject to the legend rule.
    let a = t.battlefield(P0, "Kuon, Ogre Ascendant // Kuon's Essence");
    let b = t.battlefield(P0, "Kuon, Ogre Ascendant // Kuon's Essence");
    assert!(t.obj(a).chars.is_legendary());
    assert!(flip::flip(&mut t.g, a) && flip::flip(&mut t.g, b));
    t.g.recompute();
    let c = &t.obj(a).chars;
    assert_eq!(c.name, "Kuon's Essence");
    assert!(c.is_legendary() && c.is(CardType::Enchantment) && !c.is_creature());
    t.settle();
    assert_eq!(t.named_on_battlefield("Kuon's Essence").len(), 1);
}
