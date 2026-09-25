//! Putting objects into libraries (patterns in
//! `src/oracle/patterns/control_exile_library.rs`): on top, on the bottom, Nth from the
//! top (CR 401.7), several cards at once (CR 401.4), and a choice of top or bottom made by
//! the owner or by the spell's controller.

use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// Position of an object in its owner's library, counted from the top (0 = top).
fn from_top(t: &TestGame, p: PlayerId, id: ObjectId) -> Option<usize> {
    let now = t.g.current(id);
    t.g.player(p).library.iter().rev().position(|c| *c == now)
}

#[test]
fn library_placement_cards_compile() {
    assert_compiles(&[
        "Grasp of Phantoms",
        "Fallow Earth",
        "Temporal Spring",
        "Vanishment",
        "Warrant // Warden",
        "Isolation at Orthanc",
        "Misleading Motes",
        "Temporal Cleansing",
        "Run Ashore",
        "Not Forgotten",
        "Footbottom Feast",
        "False Mourning",
        "Meldweb Curator",
        "Barkform Harvester",
        "Chrome Companion",
        "Reito Sentinel",
        "Nightscape Apprentice",
    ]);
}

#[test]
fn put_target_creature_on_top_of_its_owners_library() {
    cr!("400.3", "401.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    // P0 controls a creature P1 owns: it goes to its owner's library.
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bear.0 as usize].controller = P0;
    t.g.objects[bear.0 as usize].base_controller = P0;
    t.g.recompute();
    let lib0 = t.library_size(P0);
    let grasp = t.hand(P0, "Grasp of Phantoms");
    t.cast(P0, grasp).target(bear).go();
    t.resolve_all();
    assert_eq!(t.zone(bear), Zone::Library(P1));
    assert_eq!(from_top(&t, P1, bear), Some(0));
    assert_eq!(t.library_size(P0), lib0);
}

#[test]
fn second_from_the_top_or_the_bottom_of_a_short_library() {
    cr!("401.7");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 8);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Isolation at Orthanc");
    t.cast(P0, spell).target(bear).go();
    t.resolve_all();
    assert_eq!(from_top(&t, P1, bear), Some(1));
    // With no other cards in the library, "second from the top" is the bottom.
    t.g.players[1].library.clear();
    let ox = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Isolation at Orthanc");
    t.cast(P0, spell).target(ox).go();
    t.resolve_all();
    assert_eq!(t.g.player(P1).library, vec![t.g.current(ox)]);
}

#[test]
fn owner_chooses_top_or_bottom() {
    cr!("401.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 8);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let motes = t.hand(P0, "Misleading Motes");
    t.cast(P0, motes).target(bear).go();
    // The creature's owner (not the caster) chooses: the bottom.
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    assert_eq!(t.g.player(P1).library.first(), Some(&t.g.current(bear)));
    let chooser = t
        .asked()
        .into_iter()
        .find(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseOption { .. }))
        .map(|(p, _)| p);
    assert_eq!(chooser, Some(P1));

    // "Into their library second from the top or on the bottom".
    let wall = t.battlefield(P1, "Wall of Wood");
    let cleansing = t.hand(P0, "Temporal Cleansing");
    t.cast(P0, cleansing).target(wall).go();
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert_eq!(from_top(&t, P1, wall), Some(1));
}

#[test]
fn your_choice_of_top_or_bottom_of_its_owners_library() {
    cr!("400.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let card_in_gy = t.graveyard(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Not Forgotten");
    t.cast(P0, spell).target(card_in_gy).go();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    assert_eq!(
        t.g.player(P1).library.first(),
        Some(&t.g.current(card_in_gy))
    );
    // "Create a 1/1 white Spirit creature token with flying."
    let spirits =
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Spirit"))
            .count();
    assert_eq!(spirits, 1);
}

#[test]
fn several_cards_on_top_are_arranged_by_their_owner() {
    cr!("401.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let bear = t.graveyard(P0, "Grizzly Bears");
    let wall = t.graveyard(P0, "Wall of Wood");
    let feast = t.hand(P0, "Footbottom Feast");
    t.cast(P0, feast)
        .targets(&[Entity::Object(bear), Entity::Object(wall)])
        .go();
    // Top first: the Wall, then the Bears. "Draw a card" then draws the Wall.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.resolve_all();
    assert!(t.in_hand(P0, "Wall of Wood"));
    assert_eq!(from_top(&t, P0, bear), Some(0));
}

#[test]
fn activated_bottom_of_library_from_a_graveyard() {
    cr!("400.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    let companion = t.battlefield(P0, "Chrome Companion");
    let card_in_gy = t.graveyard(P1, "Grizzly Bears");
    t.activate(P0, companion, 0, &[Entity::Object(card_in_gy)])
        .unwrap();
    t.resolve_all();
    assert_eq!(
        t.g.player(P1).library.first(),
        Some(&t.g.current(card_in_gy))
    );
    assert_eq!(t.graveyard_size(P1), 0);
    // "Whenever this creature becomes tapped, you gain 1 life."
    assert_eq!(t.life(P0), 21);
}

#[test]
fn put_target_creature_you_control_on_top() {
    cr!("602.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let apprentice = t.battlefield(P0, "Nightscape Apprentice");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, apprentice, 0, &[Entity::Object(bear)])
        .unwrap();
    t.resolve_all();
    assert_eq!(from_top(&t, P0, bear), Some(0));
    assert!(t.on_battlefield(other));
}
