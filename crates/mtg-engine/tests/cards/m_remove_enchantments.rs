//! Remove Enchantments (hand-written, `src/cards/remove_enchantments.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn returns_yours_and_destroys_the_others() {
    cr!("608.2b", "701.8a");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // An opponent's Aura on your creature is destroyed.
    let theirs = t.battlefield(P1, "Pacifism");
    t.g.objects[theirs.0 as usize].attached_to = Some(Entity::Object(bears));
    // Your Aura on an opponent's (non-attacking) creature: you both own and control it.
    let giant = t.battlefield(P1, "Hill Giant");
    let weak = t.battlefield(P0, "Weakness");
    t.g.objects[weak.0 as usize].attached_to = Some(Entity::Object(giant));
    // An opponent's enchantment that isn't attached to your things is untouched.
    let their_anthem = t.battlefield(P1, "Glorious Anthem");
    t.g.recompute();
    t.lands(P0, "Plains", 1);
    let s = t.hand(P0, "Remove Enchantments");
    t.cast(P0, s).go();
    t.resolve();
    assert!(!t.on_battlefield(anthem));
    assert!(!t.on_battlefield(weak));
    assert!(t.in_hand(P0, "Glorious Anthem"));
    assert!(t.in_hand(P0, "Weakness"));
    assert!(t.in_graveyard(P1, "Pacifism"));
    assert!(t.on_battlefield(their_anthem));
}
