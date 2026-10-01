//! Rulings batch P202 — bestow (CR 702.103) on Eidolon of Countless Battles, and
//! changeling (CR 702.73) on Formless Genesis.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_bestowed_eidolon_counts_as_an_aura_not_a_creature() {
    cr!("702.103b", "702.103d", "613.4c");
    ruling!(
        "Eidolon of Countless Battles",
        "A permanent with bestow is either a creature or an Aura, not both (although it's an enchantment either way). It will contribute just +1/+1 toward the bonus given by Eidolon of Countless Battles."
    );
    supported("Eidolon of Countless Battles");
    // "This creature and enchanted creature each get +1/+1 for each creature you control
    // and +1/+1 for each Aura you control."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let c = t.hand(P0, "Eidolon of Countless Battles");
    t.cast(P0, c)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(bears)
        .go();
    t.resolve_all();
    let eidolon = t.named_on_battlefield("Eidolon of Countless Battles")[0];
    let o = t.obj_now(eidolon);
    assert!(o.chars.has_subtype("Aura") && !o.is(CardType::Creature));
    assert!(o.is(CardType::Enchantment));
    // One creature (the Bears) and one Aura (the Eidolon): +2/+2.
    assert_eq!(t.pt(bears), (4, 4));

    // Unattached, it's a creature and not an Aura: it counts once, as a creature.
    destroy(&mut t, bears);
    let o = t.obj_now(eidolon);
    assert!(o.is(CardType::Creature) && !o.chars.has_subtype("Aura"));
    assert_eq!(t.pt(eidolon), (1, 1));
}

#[test]
fn formless_genesis_is_every_creature_type_in_every_zone() {
    cr!("702.73a", "604.3", "205.3m");
    ruling!(
        "Formless Genesis",
        "Changeling is a characteristic-defining ability. It functions in all zones."
    );
    ruling!(
        "Formless Genesis",
        "The subtype Shapeshifter that appears on the type line is mostly there to reinforce the flavor. A spell or token with changeling is just as much an Elf, a Dwarf, a Sliver, a Goat, a Coward, and a Zombie as it is a Shapeshifter."
    );
    // (Formless Genesis's changeling compiles; its token-making effect doesn't.)
    let types = ["Shapeshifter", "Elf", "Dwarf", "Sliver", "Goat", "Coward", "Zombie"];
    let mut t = TestGame::new(2);
    let in_library = t.library_top(P0, "Formless Genesis");
    let in_graveyard = t.graveyard(P0, "Formless Genesis");
    let in_exile = t.exile(P0, "Formless Genesis");
    let in_hand = t.hand(P0, "Formless Genesis");
    t.g.recompute();
    for id in [in_library, in_graveyard, in_exile, in_hand] {
        let o = t.obj(id);
        for ty in types {
            assert!(o.chars.has_subtype(ty), "{ty} in {:?}", o.zone);
        }
    }
    // As a spell, too.
    t.lands(P0, "Forest", 3);
    let spell = t.cast(P0, in_hand).go();
    assert_eq!(t.obj(spell).zone, Zone::Stack);
    for ty in types {
        assert!(t.obj(spell).chars.has_subtype(ty));
    }
    assert!(!t.obj(spell).chars.has_subtype("Aura"));
}
