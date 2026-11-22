//! The worked examples of CR 605 (mana abilities), played out as tests. The CR 605.2
//! example is cited by a test in `r600_mana_abilities.rs`.

use crate::r600_examples_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn mana_flare_mana_from_lands_tapped_while_casting_pays_for_the_spell() {
    cr!("605.4a", "601.2g");
    cr_example!("605.4a", "the additional mana is added immediately");
    supported("Mana Flare");
    let mut t = TestGame::new(2);
    // Mana Flare: "Whenever a player taps a land for mana, that player adds one mana of
    // any type that land produced."
    t.battlefield(P1, "Mana Flare");
    let forest = t.battlefield(P0, "Forest");
    // One Forest tapped while casting Grizzly Bears ({1}{G}) pays for all of it.
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert!(t.obj(forest).tapped);
    assert_eq!(t.stack_len(), 1);
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
