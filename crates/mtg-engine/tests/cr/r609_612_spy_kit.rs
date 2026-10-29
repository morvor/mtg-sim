//! CR 612.7: Spy Kit's "all names of nonlegendary creature cards" changes the text that
//! represents the object's name.

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn equipped_creature_has_all_nonlegendary_creature_card_names() {
    // CR 612.7: Spy Kit ("Equipped creature gets +1/+1 and has all names of nonlegendary
    // creature cards in addition to its name"): the equipped creature has the name of
    // each nonlegendary creature card in the Oracle card reference.
    cr!("612.7");
    ruling!(
        "Spy Kit",
        "includes the names of all nonlegendary creature cards in the Oracle card reference, including the back faces of double-faced cards"
    );
    ruling!(
        "Spy Kit",
        "the equipped creature won’t gain the names of tokens"
    );
    let mut t = TestGame::new(2);
    // "Creatures named Grizzly Bears get +2/+2."
    t.custom(
        P1,
        permanent(
            "Bear Anthem",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::and(vec![
                    Filter::Type(CardType::Creature),
                    Filter::Named("Grizzly Bears".into()),
                ]),
                vec![pt(2, 2)],
            )],
        ),
        Zone::Battlefield,
    );
    let kit = t.battlefield(P0, "Spy Kit");
    let memnite = t.battlefield(P0, "Memnite");
    t.recompute();
    assert!(!t.obj_now(memnite).chars.has_name("Grizzly Bears"));
    assert_eq!(t.pt(memnite), (1, 1));
    assert!(t.g.attach(kit, memnite.into()));
    t.recompute();
    let c = &t.obj_now(memnite).chars;
    // It keeps its own name and gains the others.
    assert_eq!(c.name, "Memnite");
    assert!(c.has_name("Memnite"));
    assert!(c.has_name("Grizzly Bears"));
    assert!(c.has_name("Insectile Aberration"));
    // Not legendary creature cards, noncreature cards, or tokens.
    assert!(!c.has_name("Isamaru, Hound of Konda"));
    assert!(!c.has_name("Lightning Bolt"));
    assert!(!c.has_name("Zombie"));
    // +1/+1 from Spy Kit, +2/+2 as a creature named Grizzly Bears.
    assert_eq!(t.pt(memnite), (4, 4));
    // It shares a name with a Grizzly Bears.
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(t
        .obj_now(memnite)
        .chars
        .shares_name_with(&t.obj_now(bears).chars));
    // Unattached, it has only its own name again.
    assert!(t.g.attach(kit, bears.into()));
    t.recompute();
    assert!(!t.obj_now(memnite).chars.has_name("Grizzly Bears"));
    assert_eq!(t.pt(memnite), (1, 1));
}
