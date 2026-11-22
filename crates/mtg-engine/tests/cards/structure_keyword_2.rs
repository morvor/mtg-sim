//! Structure-coverage tests (round 2): keyword structures shared by many cards that no
//! other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_seed_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState};
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn jetfire_casts_converted_for_its_more_than_meets_the_eye_cost() {
    cr!("702.162a", "712.8c");
    // "More Than Meets the Eye {3}{U}" on a {4}{U} front face.
    supported("Jetfire, Ingenious Scientist // Jetfire, Air Guardian");

    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let card = t.hand(P0, "Jetfire, Ingenious Scientist");
    assert!(
        t.cast(P0, card)
            .method(CastMethod::Keyword(KeywordKind::MoreThanMeetsTheEye))
            .try_go()
            .is_err(),
        "{{3}}{{U}} needs four mana"
    );
    t.lands(P0, "Island", 1);
    assert!(
        t.cast(P0, card).try_go().is_err(),
        "its {{4}}{{U}} mana cost needs five"
    );
    let spell = t
        .cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::MoreThanMeetsTheEye))
        .go();
    assert_eq!(t.obj_now(spell).face, FaceState::Back);
    assert_eq!(t.obj_now(spell).chars.name, "Jetfire, Air Guardian");
    t.resolve_all();
    let v = t.named_on_battlefield("Jetfire, Air Guardian");
    assert_eq!(v.len(), 1, "it enters converted");
    let o = t.obj_now(v[0]);
    assert!(o.chars.has_subtype("Vehicle"));
    assert!(o.has_keyword(KeywordKind::Flying));
    // Living metal: a creature during its controller's turn.
    assert!(o.is(CardType::Creature));
}

/// Vona's Hunger: "Ascend. Each opponent sacrifices a creature of their choice. If you
/// have the city's blessing, instead each opponent sacrifices half the creatures they
/// control of their choice, rounded up."
fn vonas_hunger(permanents: usize) -> (TestGame, usize) {
    supported("Vona's Hunger");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Plains", permanents - 3);
    for _ in 0..5 {
        t.battlefield(P1, "Grizzly Bears");
    }
    let s = t.hand(P0, "Vona's Hunger");
    t.cast(P0, s).go();
    assert!(!t.g.player(P0).has_citys_blessing, "not until it resolves");
    t.resolve_all();
    let left = t.named_on_battlefield("Grizzly Bears").len();
    (t, left)
}

#[test]
fn vonas_hunger_ascends_as_it_resolves() {
    cr!("702.131a", "702.131c");
    // Ten permanents: the city's blessing, then 3 of 5 creatures (half, rounded up).
    let (t, left) = vonas_hunger(10);
    assert!(t.g.player(P0).has_citys_blessing);
    assert!(!t.g.player(P1).has_citys_blessing);
    assert_eq!(left, 2);
    // Nine permanents: no blessing, one creature.
    let (t, left) = vonas_hunger(9);
    assert!(!t.g.player(P0).has_citys_blessing);
    assert_eq!(left, 4);
}
