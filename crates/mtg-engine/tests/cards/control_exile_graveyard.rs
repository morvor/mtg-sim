//! "Would be put into a graveyard ... instead" replacement effects (patterns in
//! `src/oracle/patterns/control_exile_graveyard.rs`, CR 614.1a).

use mtg_engine::card::{CardDef, Layout};
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// A sorcery compiled from oracle text with the real compiler.
fn sorcery(name: &str, text: &str) -> CardDef {
    let tl = TypeLine::parse("Sorcery");
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(
        compiled.unsupported.is_empty(),
        "{:?}",
        compiled.unsupported
    );
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        mana_cost: mtg_engine::mana::ManaCost::parse("{0}"),
        card_types: tl.card_types,
        abilities: compiled.abilities,
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

#[test]
fn graveyard_replacement_cards_compile() {
    assert_compiles(&[
        "Lunarch Veteran // Luminous Phantom",
        "Darksteel Colossus",
        "Blightsteel Colossus",
        "Rest in Peace",
        "Leyline of the Void",
        "Dryad Militant",
    ]);
}

#[test]
fn shuffled_into_its_owners_library_from_anywhere() {
    cr!("614.1a", "701.24a");
    ruling!(
        "Darksteel Colossus",
        "it's sacrificed or if its toughness is 0 or less"
    );
    // From the battlefield: sacrificed.
    let mut t = TestGame::new(2);
    t.lands(P1, "Swamp", 2);
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    let lib = t.library_size(P0);
    let edict = t.hand(P1, "Diabolic Edict");
    t.cast(P1, edict).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.zone(colossus), Zone::Library(P0));
    assert_eq!(t.library_size(P0), lib + 1);
    assert_eq!(t.graveyard_size(P0), 0);

    // From a hand: discarded.
    let mut t = TestGame::new(2);
    let colossus = t.hand(P0, "Darksteel Colossus");
    t.g.discard(P0, colossus, None);
    t.resolve_all();
    assert_eq!(t.zone(colossus), Zone::Library(P0));

    // From a library: milled.
    let mut t = TestGame::new(2);
    let colossus = t.library_top(P0, "Darksteel Colossus");
    t.g.mill(P0, 1);
    t.resolve_all();
    assert_eq!(t.zone(colossus), Zone::Library(P0));
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn disturb_back_face_is_exiled_instead() {
    cr!("614.1a", "712.8d", "712.8e");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let veteran = t.battlefield(P0, "Lunarch Veteran // Luminous Phantom");
    assert!(mtg_engine::dfc::transform(&mut t.g, veteran));
    t.g.recompute();
    assert_eq!(t.obj_now(veteran).chars.name.as_str(), "Luminous Phantom");
    let bolt = t.hand(P1, "Lightning Bolt");
    let phantom = t.g.current(veteran);
    t.cast(P1, bolt).target(phantom).go();
    t.resolve_all();
    assert_eq!(t.zone(veteran), Zone::Exile);
    assert_eq!(t.graveyard_size(P0), 0);

    // The front face has no such ability.
    let front = t.battlefield(P0, "Lunarch Veteran // Luminous Phantom");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(front).go();
    t.resolve_all();
    assert_eq!(t.zone(front), Zone::Graveyard(P0));
}

#[test]
fn rest_in_peace_exiles_cards_and_tokens() {
    cr!("614.1a", "700.4");
    ruling!(
        "Rest in Peace",
        "abilities that trigger whenever a creature dies won't trigger"
    );
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    t.enter(P0, "Rest in Peace");
    t.resolve_all();
    // "When this enchantment enters, exile all graveyards."
    assert_eq!(t.graveyard_size(P1), 0);
    let artist = t.battlefield(P1, "Blood Artist");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bear).go();
    t.resolve_all();
    assert_eq!(t.zone(bear), Zone::Exile);
    // The creature never died: no Blood Artist trigger.
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
    // The resolved spell is exiled too, and so are discarded cards.
    assert_eq!(t.zone(bolt), Zone::Exile);
    let card_in_hand = t.hand(P1, "Grizzly Bears");
    t.g.discard(P1, card_in_hand, None);
    assert_eq!(t.zone(card_in_hand), Zone::Exile);
    assert!(t.on_battlefield(artist));
}

#[test]
fn rest_in_peace_destroyed_by_a_spell() {
    cr!("614.1a", "608.2n");
    ruling!("Rest in Peace", "If Rest in Peace is destroyed by a spell");
    let mut t = TestGame::new(2);
    let rip = t.battlefield(P0, "Rest in Peace");
    t.lands(P1, "Forest", 2);
    let naturalize = t.hand(P1, "Naturalize");
    t.cast(P1, naturalize).target(rip).go();
    t.resolve_all();
    assert_eq!(t.zone(rip), Zone::Exile);
    assert_eq!(t.zone(naturalize), Zone::Graveyard(P1));
}

#[test]
fn leyline_of_the_void_exiles_opponents_cards_but_tokens_still_die() {
    cr!("614.1a", "111.7");
    ruling!("Leyline of the Void", "Tokens can still die");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of the Void");
    // An opponent's card is exiled; your own isn't.
    let theirs = t.hand(P1, "Grizzly Bears");
    t.g.discard(P1, theirs, None);
    assert_eq!(t.zone(theirs), Zone::Exile);
    let mine = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, mine, None);
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
    // An opponent's token still dies (it isn't a card), so "dies" abilities trigger.
    t.battlefield(P1, "Blood Artist");
    t.lands(P1, "Plains", 2);
    let alarm = t.hand(P1, "Raise the Alarm");
    t.cast(P1, alarm).go();
    t.resolve_all();
    let token =
        t.g.battlefield
            .iter()
            .copied()
            .find(|id| t.g.obj(*id).chars.has_subtype("Soldier"))
            .unwrap();
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(token).go();
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve_all();
    assert!(!t.g.is_live(token));
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 21);
}

#[test]
fn exile_instead_for_the_rest_of_the_turn() {
    cr!("614.1a", "611.2a");
    let mut t = TestGame::new(2);
    let spell = t.custom(
        P0,
        sorcery(
            "Graveyard Ward",
            "If a card would be put into your graveyard from anywhere this turn, exile that card instead.",
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, spell).go();
    t.resolve_all();
    // The effect exists by the time the spell is put into the graveyard (CR 608.2n).
    assert!(t.in_exile("Graveyard Ward"));
    let card_in_hand = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, card_in_hand, None);
    assert_eq!(t.zone(card_in_hand), Zone::Exile);
    let theirs = t.hand(P1, "Grizzly Bears");
    t.g.discard(P1, theirs, None);
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
    // It ends at end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    let later = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, later, None);
    assert_eq!(t.zone(later), Zone::Graveyard(P0));
}
