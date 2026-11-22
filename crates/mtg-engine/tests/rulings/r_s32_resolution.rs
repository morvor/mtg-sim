//! Rulings batch S32 — what happens to an instant or sorcery card as it finishes resolving
//! or is countered: Timetwister isn't in the graveyard while it shuffles graveyards
//! (CR 608.2n), Green Sun's Zenith countered goes to the graveyard (CR 701.6a), and a
//! wish can't get ante cards (CR 400.11a, 407).

use crate::r_s01_common::*;
use crate::r_s04_common::graveyard_names;
use crate::r_s25_common::cast_new;
use crate::r_s32_common::*;
use mtg_engine::card::card;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn timetwister_isnt_shuffled_into_the_library_by_its_own_effect() {
    cr!("608.2n", "701.24a");
    ruling!(
        "Timetwister",
        "This card won’t be put into your graveyard until after it’s finished resolving, which means it won’t be shuffled into your library as part of its own effect."
    );
    supported("Timetwister");
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P1, "Shock");
    let library = t.library_size(P0);
    cast_new(&mut t, P0, "Timetwister", &[]);
    t.resolve_all();
    // Hand (1 card) and graveyard (2 cards) shuffled in, then seven drawn.
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.library_size(P0), library + 3 - 7);
    // Timetwister is the only card in the graveyard; it isn't in the library.
    assert_eq!(graveyard_names(&t, P0), vec!["Timetwister"]);
    assert!(!in_library(&t, P0, "Timetwister"));
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn a_countered_zenith_goes_to_the_graveyard_not_the_library() {
    cr!("701.6a", "608.2n");
    ruling!(
        "Green Sun's Zenith",
        "If this spell doesn't resolve, none of its effects occur. In particular, it will go to the graveyard rather than to its owner's library."
    );
    supported("Green Sun's Zenith");
    // Resolving, it shuffles itself into the library.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 2);
    let gsz = t.hand(P0, "Green Sun's Zenith");
    t.cast(P0, gsz).x(1).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(in_library(&t, P0, "Green Sun's Zenith"));
    assert!(!t.in_graveyard(P0, "Green Sun's Zenith"));
    // Countered: no search, and it goes to the graveyard.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 2);
    let gsz = t.hand(P0, "Green Sun's Zenith");
    let spell = t.cast(P0, gsz).x(1).go();
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(spell)]);
    t.resolve_all();
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
    assert!(t.in_graveyard(P0, "Green Sun's Zenith"));
    assert!(!in_library(&t, P0, "Green Sun's Zenith"));
}

#[test]
fn living_wish_cant_get_ante_cards() {
    cr!("400.11", "400.11a", "407.2");
    ruling!(
        "Living Wish",
        "Can't acquire the Ante cards. They are considered still \"in the game\" as are cards in the library and the graveyard."
    );
    supported("Living Wish");
    // Anted Grizzly Bears, Llanowar Elves in the graveyard, Hill Giant in the library:
    // only the sideboard's Forest is outside the game.
    let mut t = TestGame::new(2);
    let anted = t.custom(P0, (*card("Grizzly Bears")).clone(), Zone::Ante);
    let dead = t.graveyard(P0, "Llanowar Elves");
    t.library_top(P0, "Hill Giant");
    let forest = t.custom(P0, (*card("Forest")).clone(), Zone::Outside(P0));
    let from = t.asked().len();
    cast_new(&mut t, P0, "Living Wish", &[]);
    t.resolve_all();
    for (_, d) in &t.asked()[from..] {
        if let mtg_engine::decision::Decision::ChooseEntities { candidates, .. } = d {
            assert!(!candidates.contains(&Entity::Object(anted)), "{candidates:?}");
            assert!(!candidates.contains(&Entity::Object(dead)), "{candidates:?}");
        }
    }
    assert_eq!(t.zone(anted), Zone::Ante);
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Hill Giant"));
    assert!(t.in_hand(P0, "Forest"), "{:?}", t.zone(forest));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}
