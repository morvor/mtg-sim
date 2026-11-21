//! Rulings batch S27 — "if {R} was spent to cast this spell" looks at all the mana spent
//! on the spell's total cost, not only on its hybrid symbol, and only at whether any of
//! that color was spent (CR 601.2f, 601.2h).

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::*;

/// P0 casts Torrent of Souls ({4}{B/R}: "Return up to one target creature card from your
/// graveyard to the battlefield if {B} was spent to cast this spell. Creatures target
/// player controls get +2/+0 and gain haste until end of turn if {R} was spent to cast
/// this spell.") paying with `lands`, targeting Grizzly Bears in P0's graveyard and P0.
/// Returns (the Bears, P0's Hill Giant).
fn torrent_of_souls(lands: &[(&str, usize)]) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    for (land, n) in lands {
        t.lands(P0, land, *n);
    }
    let torrent = t.hand(P0, "Torrent of Souls");
    t.cast(P0, torrent)
        .target(bears)
        .target(Entity::Player(P0))
        .go();
    assert_eq!(tapped_lands(&t, P0), 5);
    t.resolve_all();
    (t, bears, giant)
}

#[test]
fn torrent_of_souls_counts_mana_spent_on_its_generic_cost() {
    cr!("601.2f", "601.2h", "107.4e");
    ruling!(
        "Torrent of Souls",
        "This spell cares about what mana was spent to pay its total cost, not just what mana was spent to pay the hybrid mana symbol in its cost."
    );
    supported("Torrent of Souls");
    // A Swamp and a Mountain (one pays {B/R}, the other part of {4}) and three Wastes:
    // both {B} and {R} were spent, so it does both.
    let (t, bears, giant) = torrent_of_souls(&[("Swamp", 1), ("Mountain", 1), ("Wastes", 3)]);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.pt(giant), (5, 3));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
}

#[test]
fn torrent_of_souls_checks_only_whether_red_was_spent() {
    cr!("601.2h", "608.2h");
    ruling!(
        "Torrent of Souls",
        "This spell checks on resolution to see if any mana of the appropriate colors were spent to pay its cost. It doesn’t matter how much mana of that color was spent; the effect isn’t multiplied."
    );
    supported("Torrent of Souls");
    // Five Mountains: +2/+0 once, and no {B} was spent, so the Bears stay put.
    let (t, _, giant) = torrent_of_souls(&[("Mountain", 5)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.pt(giant), (5, 3));
}

#[test]
fn unnerving_assault_counts_mana_spent_on_its_generic_cost() {
    cr!("601.2f", "601.2h", "107.4e");
    ruling!(
        "Unnerving Assault",
        "The spell cares about what mana was spent to pay its total cost, not just what mana was spent to pay the hybrid part of its cost."
    );
    supported("Unnerving Assault");
    // {2}{U/R}: "Creatures your opponents control get -1/-0 until end of turn if {U} was
    // spent to cast this spell, and creatures you control get +1/+0 until end of turn if
    // {R} was spent to cast this spell." An Island, a Mountain and a Wastes: whichever
    // pays the hybrid symbol, the other color paid part of {2}.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let assault = t.hand(P0, "Unnerving Assault");
    t.cast(P0, assault).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.pt(giant), (4, 3));
    assert_eq!(t.pt(bears), (1, 2));
    // Paid with an Island and two Wastes: only {U} was spent.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let assault = t.hand(P0, "Unnerving Assault");
    t.cast(P0, assault).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (3, 3));
    assert_eq!(t.pt(bears), (1, 2));
}
