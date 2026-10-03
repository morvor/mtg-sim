//! Round-trip follow-up `roundtrip-tail-3` (cards M–R): in-game tests for compiler
//! misreads the round trip found (each shows the behavior the corrected compilation has
//! and the old one didn't), and renderer checks.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn enchant_artifact_or_non_aura_enchantment_enchants_a_plain_artifact() {
    cr!("702.5a", "303.4a");
    // Puppet Crafting: "Enchant artifact or non-Aura enchantment". The quality was read
    // as "artifact enchantment or non-Aura enchantment", so a plain artifact couldn't be
    // enchanted.
    supported("Puppet Crafting");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    let aura = t.hand(P0, "Puppet Crafting");
    t.lands(P0, "Forest", 2);
    t.cast(P0, aura).target(stone).go();
    t.resolve();
    assert!(t.named_on_battlefield("Puppet Crafting").len() == 1);
    let o = t.obj_now(stone);
    assert!(o.is_creature(), "Mind Stone should be a creature");
    assert_eq!(t.pt(stone), (5, 5));
}
