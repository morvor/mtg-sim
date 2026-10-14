//! Cards from outside the game (CR 400.11b, 108.3b): a card from outside the game or a
//! face-up exiled card into your hand, outside cards shuffled into or put on top of your
//! library.

use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

#[test]
fn coax_takes_an_eldrazi_from_outside_the_game_or_face_up_exile() {
    cr!("400.11b", "108.3b", "701.20a");
    assert_supported(&["Coax from the Blind Eternities"]);
    // From outside the game.
    let mut t = TestGame::new(2);
    let side = t
        .g
        .add_to_sideboard(P0, vec![card("Eldrazi Devastator"), card("Grizzly Bears")]);
    t.lands(P0, "Island", 3);
    let coax = t.hand(P0, "Coax from the Blind Eternities");
    t.answer_yes(P0, true);
    // The Grizzly Bears isn't an Eldrazi: choosing it is invalid, so the Eldrazi is put
    // into the hand.
    t.answer_choose(P0, &[Entity::Object(side[1])]);
    t.cast(P0, coax).go();
    t.resolve();
    assert!(t.in_hand(P0, "Eldrazi Devastator"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    // A face-up Eldrazi card you own in exile.
    let mut t = TestGame::new(2);
    let ex = t.exile(P0, "Eldrazi Devastator");
    t.lands(P0, "Island", 3);
    let coax = t.hand(P0, "Coax from the Blind Eternities");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(ex)]);
    t.cast(P0, coax).go();
    t.resolve();
    assert!(t.in_hand(P0, "Eldrazi Devastator"));
}

#[test]
fn research_shuffles_up_to_four_outside_cards_into_your_library() {
    cr!("400.11b", "701.24a");
    let c = card("Research // Development");
    assert!(!c
        .unsupported_text()
        .iter()
        .any(|t| t.contains("outside the game")));
    let mut t = TestGame::new(2);
    let side = t.g.add_to_sideboard(
        P0,
        vec![card("Grizzly Bears"), card("Hill Giant"), card("Ornithopter")],
    );
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    let r = t.hand(P0, "Research // Development");
    let before = t.library_size(P0);
    t.answer_choose(
        P0,
        &[Entity::Object(side[0]), Entity::Object(side[1])],
    );
    t.cast(P0, r).method(CastMethod::Half(0)).go();
    t.resolve();
    assert_eq!(t.library_size(P0), before + 2);
    assert_eq!(t.zone(side[2]), Zone::Outside(P0));
}

#[test]
fn the_ravens_warning_puts_an_outside_card_on_top_of_your_library() {
    cr!("400.11b", "714.2b");
    assert_supported(&["The Raven's Warning"]);
    let mut t = TestGame::new(2);
    let side = t
        .g
        .add_to_sideboard(P0, vec![card("Grizzly Bears"), card("Hill Giant")]);
    let saga = t.battlefield(P0, "The Raven's Warning");
    t.g.add_counters(Entity::Object(saga), "lore", 2, None);
    t.resolve_all();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(side[1])]);
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.resolve_all();
    let top = t.g.library_top(P0).expect("a library");
    assert_eq!(t.g.obj(top).chars.name, "Hill Giant");
    assert_eq!(t.zone(side[0]), Zone::Outside(P0));
}
