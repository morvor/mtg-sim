//! Rulings batch P022 — Hofri Ghostforge's Spirit tokens ("create a token that's a copy of
//! that creature, except ... it has "When this token leaves the battlefield, return the
//! exiled card to its owner's graveyard.""): the granted ability is linked to Hofri's
//! ability on the tokens that ability created (CR 607.1d, 607.2a), even if Hofri left the
//! battlefield before it resolved; a copy of a Spirit token has the ability (CR 707.9a)
//! but not the link.

use crate::r_p022_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

const HOFRI: &str = "Hofri Ghostforge";

/// P0's Hofri and P0's creatures `names`; each of them dies in turn. Returns the Spirit
/// token Hofri's ability made for each.
fn spirits_of(t: &mut TestGame, names: &[&str]) -> Vec<ObjectId> {
    supported(HOFRI);
    t.battlefield(P0, HOFRI);
    let mut spirits = Vec::new();
    for name in names {
        let c = t.battlefield(P0, name);
        let before = t.g.battlefield.clone();
        kill(t, c);
        t.resolve_all();
        let tok = one(new_tokens(t, P0, &before));
        assert_eq!(name_of(t, tok), *name);
        assert!(has_subtype(t, tok, "Spirit"));
        assert!(t.in_exile(name));
        spirits.push(tok);
    }
    spirits
}

#[test]
fn a_hofri_spirit_returns_the_card_even_if_hofri_died_with_the_creature() {
    cr!("607.1d", "607.2a", "603.10a");
    supported(HOFRI);
    supported("Wrath of God");
    // Hofri and a Hill Giant die together: Hofri's ability triggers for the Giant (it
    // looks back in time) and resolves after Hofri left the battlefield.
    let mut t = TestGame::new(2);
    let hofri = t.battlefield(P0, HOFRI);
    t.battlefield(P0, "Hill Giant");
    let tok = one(cast_spell(
        &mut t,
        P0,
        "Wrath of God",
        "{2}{W}{W}",
        &[],
        None,
        None,
    ));
    assert!(!t.on_battlefield(hofri));
    assert!(has_subtype(&t, tok, "Spirit"));
    assert!(t.in_exile("Hill Giant"));
    // The token is still linked to Hofri's ability: it returns the card.
    kill(&mut t, tok);
    t.resolve_all();
    assert!(!t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn a_token_copy_of_a_hofri_spirit_has_the_ability_but_returns_nothing() {
    cr!("707.9a", "607.1d", "707.3");
    ruling!("Hofri Ghostforge", "If another creature enters the battlefield as a copy of the Spirit token or another effect creates a copy of the Spirit token, it will have the triggered ability granted by Hofri, but will not be linked to Hofri's ability and will not return the card to the graveyard when it leaves the battlefield.");
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    let spirit = one(spirits_of(&mut t, &["Hill Giant"]));
    // Cackling Counterpart: "Create a token that's a copy of target creature you
    // control."
    let copy = one(cast_spell(
        &mut t,
        P0,
        "Cackling Counterpart",
        "{1}{U}{U}",
        &[obj(spirit)],
        None,
        None,
    ));
    assert_eq!(name_of(&t, copy), "Hill Giant");
    assert!(has_subtype(&t, copy, "Spirit"));
    assert_eq!(abilities_with(&t, copy, "return the exiled card"), 1);
    // The copy leaves: its trigger returns nothing.
    kill(&mut t, copy);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    // The Spirit leaves: the card returns.
    kill(&mut t, spirit);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn hofri_spirits_from_one_resolution_each_return_the_card() {
    cr!("607.1d", "607.2a", "614.1a");
    ruling!("Hofri Ghostforge", "Hofri Ghostforge's last ability is linked to the ability it grants the Spirit token. If a replacement effect causes you to create more than one Spirit token with this effect, they will all have that ability, and the exiled card will be returned to the graveyard if any of them leaves the battlefield.");
    supported(HOFRI);
    for leaves in [0, 1] {
        // Doubling Season: two Spirit tokens; either one leaving returns the card.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Doubling Season");
        t.battlefield(P0, HOFRI);
        let giant = t.battlefield(P0, "Hill Giant");
        let before = t.g.battlefield.clone();
        kill(&mut t, giant);
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), 2);
        assert!(toks
            .iter()
            .all(|x| abilities_with(&t, *x, "return the exiled card") == 1));
        kill(&mut t, toks[leaves]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Hill Giant"), "token {leaves}");
    }
}

#[test]
fn each_hofri_spirit_returns_only_the_card_exiled_as_it_was_made() {
    cr!("607.2a", "607.1d");
    ruling!("Hofri Ghostforge", "Each Spirit token you create refers only to the card you exile during the resolution of the ability that created that token. When it leaves the battlefield, you return only that card to your graveyard, not any other card exiled by Hofri.");
    let mut t = TestGame::new(2);
    let spirits = spirits_of(&mut t, &["Hill Giant", "Grizzly Bears"]);
    // The Bears' Spirit leaves: only the Bears returns.
    kill(&mut t, spirits[1]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_exile("Hill Giant"));
    // Then the Giant's.
    kill(&mut t, spirits[0]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn a_hofri_spirit_made_from_a_copy_of_a_spirit_returns_only_its_own_card() {
    cr!("607.1d", "607.2a", "707.3");
    ruling!("Hofri Ghostforge", "Each Spirit token you create refers only to the card you exile during the resolution of the ability that created that token. When it leaves the battlefield, you return only that card to your graveyard, not any other card exiled by Hofri.");
    let mut t = TestGame::new(2);
    let spirit = one(spirits_of(&mut t, &["Hill Giant"]));
    // P0's Clone enters as a copy of the Spirit and dies: Hofri exiles the Clone card and
    // makes a token copy of it, which has the ability the Clone copied (unlinked) and the
    // one Hofri's exception gives it.
    let clone = clone_of(&mut t, P0, spirit);
    let before = t.g.battlefield.clone();
    kill(&mut t, clone);
    t.resolve_all();
    let second = one(new_tokens(&t, P0, &before));
    assert!(t.in_exile("Clone") && t.in_exile("Hill Giant"));
    assert_eq!(abilities_with(&t, second, "return the exiled card"), 2);
    // It leaves: only the Clone card returns; the first Spirit's card stays exiled.
    kill(&mut t, second);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Clone"));
    assert!(t.in_exile("Hill Giant"));
    assert!(t.on_battlefield(spirit));
}
