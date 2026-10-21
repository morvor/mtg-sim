//! CR 605.3a, 601.2g: a mana ability that costs mana (a filter, such as a Signet's
//! "{1}, {T}: Add {B}{G}") may be activated while paying a cost, its own cost paid first
//! with other mana. The automatic payment plans one such ability when a cost can't be
//! paid without it.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_signet_filters_a_forests_mana_to_cast_a_spell() {
    cr!("605.3a", "601.2g");
    let mut t = TestGame::new(2);
    let signet = t.battlefield(P0, "Golgari Signet");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert!(t.obj_now(signet).tapped);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // A Signet alone can't pay its own cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Golgari Signet");
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // Nor is {1}{G} paid with one Forest and a Signet when {2}{G} is needed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Golgari Signet");
    t.lands(P0, "Forest", 1);
    let giant = t.hand(P0, "Centaur Courser");
    assert!(t.cast(P0, giant).try_go().is_err());
}
