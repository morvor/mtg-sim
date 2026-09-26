//! CR 206: the expansion symbol — names "originally printed" in an expansion (City in a
//! Bottle, Golgothian Sylex, Apocalypse Chime) and printings in constructed decks.

use crate::r105_util::matches;
use crate::r300_common::{can_cast, can_play_land};
use crate::r703_common::supported;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::deck::{check_format_legality, DeckProblem};
use mtg_engine::names::originally_printed_in;
use mtg_engine::testing::*;
use mtg_engine::*;

fn printed_in(set: &str) -> Filter {
    Filter::NameOriginallyPrintedIn(set.into())
}

#[test]
fn names_originally_printed_in_a_set() {
    // CR 206.3: cards that affected a set's expansion symbol now refer to names
    // originally printed in that set — whatever the printing of the card.
    cr!("206.3");
    assert!(originally_printed_in("Arabian Nights", "Kird Ape"));
    assert!(originally_printed_in("Antiquities", "Mishra's Factory"));
    assert!(originally_printed_in("Homelands", "Reveka, Wizard Savant"));
    assert!(!originally_printed_in("Arabian Nights", "Grizzly Bears"));
    assert!(!originally_printed_in("Antiquities", "Kird Ape"));
    let mut t = TestGame::new(2);
    // Kird Ape's current printing (Eternal Masters) has another expansion symbol; its
    // name was originally printed in Arabian Nights.
    assert_ne!(
        mtg_data::cards().by_name("Kird Ape").unwrap().set,
        "arn"
    );
    let ape = t.hand(P0, "Kird Ape");
    let ape_bf = t.battlefield(P1, "Kird Ape");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let f = printed_in("Arabian Nights");
    assert!(matches(&t, ape, &f, P0));
    assert!(matches(&t, ape_bf, &f, P0));
    assert!(!matches(&t, bears, &f, P0));
    // An object named by an effect changes its name: the filter follows its name.
    let renamed = t.battlefield(P1, "Grizzly Bears");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(renamed)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetName("Kird Ape".into())],
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert!(matches(&t, renamed, &f, P0));
}

#[test]
fn city_in_a_bottle_and_arabian_nights_names() {
    cr!("206.3", "206.3a");
    ruling!(
        "City in a Bottle",
        "even if the physical card representing that permanent is a reprint with a different expansion symbol"
    );
    supported("City in a Bottle");
    let mut t = TestGame::new(2);
    let city = t.battlefield(P0, "City in a Bottle");
    let mine = t.battlefield(P0, "Kird Ape");
    let theirs = t.battlefield(P1, "Kird Ape");
    let djinn = t.battlefield(P1, "Juzám Djinn");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Its state trigger: each player sacrifices those permanents.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Kird Ape"));
    assert!(t.in_graveyard(P1, "Kird Ape"));
    assert!(t.in_graveyard(P1, "Juzám Djinn"));
    assert!(!t.g.is_live(mine) && !t.g.is_live(theirs) && !t.g.is_live(djinn));
    // Other permanents stay, including City in a Bottle itself (also an Arabian Nights
    // name: "other").
    assert!(t.g.is_live(bears) && t.g.is_live(city));
    // Players can't cast spells or play lands with those names.
    let ape = t.hand(P0, "Kird Ape");
    let library = t.hand(P0, "Library of Alexandria");
    let giant = t.hand(P0, "Hill Giant");
    let forest = t.hand(P0, "Forest");
    t.lands(P0, "Mountain", 4);
    assert!(!can_cast(&mut t, P0, ape));
    assert!(t.cast(P0, ape).try_go().is_err());
    assert!(can_cast(&mut t, P0, giant));
    assert!(!can_play_land(&mut t, P0, library));
    assert!(t.play_land(P0, library).is_err());
    assert!(can_play_land(&mut t, P0, forest));
    // Without City in a Bottle, they can.
    let mut t = TestGame::new(2);
    let library = t.hand(P0, "Library of Alexandria");
    assert!(can_play_land(&mut t, P0, library));
}

#[test]
fn golgothian_sylex_and_antiquities_names() {
    cr!("206.3", "206.3b");
    supported("Golgothian Sylex");
    let mut t = TestGame::new(2);
    let sylex = t.battlefield(P0, "Golgothian Sylex");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P1, "Mishra's Factory");
    t.battlefield(P1, "Strip Mine");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.activate(P0, sylex, 0, &[]).unwrap();
    t.resolve_all();
    // Each nontoken permanent with an Antiquities name is sacrificed by its controller —
    // the Sylex's own name is one of them.
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(t.in_graveyard(P1, "Mishra's Factory"));
    assert!(t.in_graveyard(P1, "Strip Mine"));
    assert!(t.in_graveyard(P0, "Golgothian Sylex"));
    assert!(t.g.is_live(bears));
}

#[test]
fn apocalypse_chime_and_homelands_names() {
    cr!("206.3", "206.3c");
    supported("Apocalypse Chime");
    let mut t = TestGame::new(2);
    let chime = t.battlefield(P0, "Apocalypse Chime");
    t.battlefield(P1, "Ebony Rhino");
    t.battlefield(P1, "Serra Aviary");
    t.battlefield(P0, "Leaping Lizard");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.activate(P0, chime, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ebony Rhino"));
    assert!(t.in_graveyard(P1, "Serra Aviary"));
    assert!(t.in_graveyard(P0, "Leaping Lizard"));
    assert!(t.g.is_live(bears));
}

#[test]
fn any_printing_of_a_card_allowed_in_a_format_may_be_played() {
    // CR 206.4: Kird Ape was first printed in Arabian Nights, which Modern doesn't use,
    // but it was reprinted in sets Modern allows, so any printing of it may be in a
    // Modern deck. Juzám Djinn never was.
    cr!("206.4");
    let ape = card("Kird Ape");
    let djinn = card("Juzám Djinn");
    let deck: Vec<_> = (0..4).map(|_| ape.clone()).collect();
    assert!(check_format_legality(&deck, &[], "modern").is_empty());
    let deck: Vec<_> = (0..4).map(|_| djinn.clone()).collect();
    assert!(matches!(
        check_format_legality(&deck, &[], "modern").as_slice(),
        [DeckProblem::NotLegalInFormat { .. }]
    ));
    assert!(check_format_legality(&deck, &[], "legacy").is_empty());
}
