//! Rulings batch S26 — Vesuvan Duplimancy ("Whenever you cast a spell that targets only a
//! single artifact or creature you control, create a token that's a copy of that artifact
//! or creature, except it's not legendary."): the token copies what the target copies
//! (CR 707.3, 707.9b), for a spell with a single target (CR 115.9c).

use crate::r_s01_common::supported;
use crate::r_s26_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::Supertype;
use mtg_engine::*;

#[test]
fn vesuvan_duplimancy_copies_what_the_target_is_copying() {
    cr!("707.3", "707.9b", "115.9c");
    ruling!(
        "Vesuvan Duplimancy",
        "If the copied permanent is copying something else (for example, if the copied creature is a Clone), then the token enters the battlefield as whatever that permanent copied."
    );
    supported("Vesuvan Duplimancy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vesuvan Duplimancy");
    // P0's Clone is a copy of P1's Kiki-Jiki (a legendary creature).
    let kiki = t.battlefield(P1, "Kiki-Jiki, Mirror Breaker");
    t.answer_choose(P0, &[Entity::Object(kiki)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    let clone = t.g.current(clone);
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    let before = t.g.battlefield.clone();
    t.cast(P0, growth).target(clone).go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Kiki-Jiki, Mirror Breaker");
    assert!(!tok.chars.supertypes.contains(Supertype::Legendary));
    assert_eq!(t.pt(toks[0]), (2, 2));
    // A spell with a target P0 doesn't control doesn't trigger it.
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    let before = t.g.battlefield.clone();
    t.cast(P0, growth).target(kiki).go();
    t.resolve_all();
    assert!(new_tokens(&t, P0, &before).is_empty());
}
