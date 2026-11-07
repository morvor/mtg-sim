//! The worked examples of CR 710 (flip cards), played out as tests.

use super::r709_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const LAVARUNNER: &str = "Akki Lavarunner // Tok-Tok, Volcano Born";

#[test]
fn a_legendary_search_cant_find_akki_lavarunner_and_a_legendary_anthem_only_affects_tok_tok() {
    cr!("710.1b");
    cr_example!("710.2", "Akki Lavarunner");
    supported(LAVARUNNER);
    // "Search your library for a legendary card": Akki Lavarunner in the library has only
    // its normal (nonlegendary) characteristics, so it can't be found.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let akki_in_library = t.library_top(P0, LAVARUNNER);
    let isamaru = t.library_top(P0, "Isamaru, Hound of Konda");
    let tutor = t.custom(
        P0,
        oracle_card(
            "Legend Seeker",
            "Sorcery",
            "{0}",
            None,
            "Search your library for a legendary card, reveal it, put it into your hand, then shuffle.",
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, tutor).go();
    t.resolve_all();
    let searches: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(searches.len(), 1, "one search choice");
    assert!(searches[0].contains(&Entity::Object(isamaru)));
    assert!(!searches[0].contains(&Entity::Object(akki_in_library)));
    assert!(!t.in_hand(P0, "Akki Lavarunner"));

    // "Legendary creatures get +2/+2" doesn't affect Akki Lavarunner but affects Tok-Tok.
    let mut t = TestGame::new(2);
    t.custom(
        P1,
        oracle_card(
            "Legends' Banner",
            "Enchantment",
            "{0}",
            None,
            "Legendary creatures get +2/+2.",
        ),
        Zone::Battlefield,
    );
    let akki = t.battlefield(P0, LAVARUNNER);
    assert_eq!(t.pt(akki), (1, 1));
    // "Whenever this creature deals damage to an opponent, flip it."
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(akki, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.obj(akki).flipped);
    assert_eq!(t.obj(akki).chars.name, "Tok-Tok, Volcano Born");
    // Tok-Tok is a 2/2 legendary creature: 4/4.
    assert_eq!(t.pt(akki), (4, 4));
}
