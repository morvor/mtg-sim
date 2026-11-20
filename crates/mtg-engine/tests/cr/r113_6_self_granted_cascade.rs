//! A card's own "This spell has cascade as long as ..." (Bloodbraid Marauder's delirium
//! ability) functions only on the stack. CR 113.6: "Abilities of an instant or sorcery
//! spell usually function only while that object is on the stack. Abilities of all other
//! objects usually function only while that object is on the battlefield. The exceptions
//! are as follows:" / 113.6b "An ability that states which zones it functions in functions
//! only from those zones." / 112.1 "A spell is a card on the stack. ..." / 702.85a
//! "Cascade is a triggered ability that functions only while the spell with cascade is on
//! the stack. ..." The ability speaks of the card as a spell, so it gives the card cascade
//! only on the stack, as it's cast (the card's ruling: "The delirium ability will check
//! your graveyard at the moment you finish casting Bloodbraid Marauder to determine if it
//! has cascade."). Before small-fixes-6 it functioned in every zone: Bloodbraid Marauder
//! had cascade in a hand, a library, a graveyard, exile and on the battlefield.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Four card types in `p`'s graveyard (land, creature, instant, artifact): delirium.
fn delirium(t: &mut TestGame, p: PlayerId) {
    for n in ["Forest", "Grizzly Bears", "Lightning Bolt", "Ornithopter"] {
        t.graveyard(p, n);
    }
}

#[test]
fn a_self_granted_cascade_functions_only_on_the_stack() {
    cr!("113.6", "113.6b", "702.85a");
    interp!("self-granted-cascade-functions-on-the-stack");
    ruling!(
        "Bloodbraid Marauder",
        "The delirium ability will check your graveyard at the moment you finish casting Bloodbraid Marauder to determine if it has cascade."
    );
    // With delirium, Bloodbraid Marauder doesn't have cascade outside the stack.
    let mut t = TestGame::new(2);
    delirium(&mut t, P0);
    let cards = [
        t.hand(P0, "Bloodbraid Marauder"),
        t.library_top(P0, "Bloodbraid Marauder"),
        t.graveyard(P0, "Bloodbraid Marauder"),
        t.exile(P0, "Bloodbraid Marauder"),
        t.battlefield(P0, "Bloodbraid Marauder"),
    ];
    t.g.recompute();
    for c in cards {
        assert!(
            !t.obj(c).chars.has_keyword(KeywordKind::Cascade),
            "in {:?}",
            t.zone(c)
        );
    }
    // Cast, it has cascade and cascades into Ornithopter.
    t.library_top(P0, "Ornithopter");
    t.lands(P0, "Mountain", 3);
    let spell = t.cast(P0, cards[0]).go();
    t.settle();
    t.g.recompute();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(t.obj(spell).chars.has_keyword(KeywordKind::Cascade));
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    // On the battlefield it no longer has cascade.
    let m = t.g.current(spell);
    assert_eq!(t.zone(m), Zone::Battlefield);
    assert!(!t.obj(m).chars.has_keyword(KeywordKind::Cascade));
}
