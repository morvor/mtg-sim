//! An instant's or sorcery's ability-word paragraph ("Metalcraft — If ..., exile that
//! creature.") continues the spell's instructions (CR 113.3a, 207.2c, 608.2c): "that
//! creature" is the creature an earlier paragraph targeted, not the spell.

use crate::r_s01_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn metalcraft_exiles_the_creature_dispatch_tapped() {
    cr!("113.3a", "207.2c", "608.2c");
    ruling!(
        "Dispatch",
        "If you control three or more artifacts when Dispatch resolves, you'll tap the creature, then exile it."
    );
    supported("Dispatch");
    // "Tap target creature. Metalcraft — If you control three or more artifacts, exile
    // that creature."
    for artifacts in [2, 3] {
        let mut t = TestGame::new(2);
        for _ in 0..artifacts {
            t.battlefield(P0, "Ornithopter");
        }
        let giant = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Plains", 1);
        let c = t.hand(P0, "Dispatch");
        t.cast(P0, c).target(giant).go();
        t.resolve_all();
        // Dispatch itself is put into its owner's graveyard either way.
        assert!(t.in_graveyard(P0, "Dispatch"));
        assert!(!t.in_exile("Dispatch"));
        if artifacts == 3 {
            assert!(!t.on_battlefield(giant));
            assert!(t.in_exile("Hill Giant"));
        } else {
            assert!(t.on_battlefield(giant));
            assert!(t.obj(giant).tapped);
        }
    }
}

#[test]
fn delirium_gives_the_targeted_creature_double_strike() {
    cr!("113.3a", "207.2c", "608.2c");
    supported("Violent Urge");
    // "Target creature gets +1/+0 and gains first strike until end of turn. Delirium — If
    // there are four or more card types among cards in your graveyard, that creature gains
    // double strike until end of turn."
    for delirium in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.graveyard(P0, "Island");
        t.graveyard(P0, "Lightning Bolt");
        t.graveyard(P0, "Divination");
        if delirium {
            t.graveyard(P0, "Ornithopter");
        }
        t.lands(P0, "Mountain", 1);
        let c = t.hand(P0, "Violent Urge");
        t.cast(P0, c).target(bears).go();
        t.resolve_all();
        let o = t.obj(bears);
        assert_eq!(t.pt(bears), (3, 2));
        assert!(o.chars.has_keyword(KeywordKind::FirstStrike));
        assert_eq!(o.chars.has_keyword(KeywordKind::DoubleStrike), delirium);
    }
}
