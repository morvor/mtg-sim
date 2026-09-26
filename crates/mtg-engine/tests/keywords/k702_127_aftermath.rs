//! CR 702.127 Aftermath.

use crate::common_k702_125_139::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const AFTERMATH: CastMethod = CastMethod::Keyword(KeywordKind::Aftermath);
const FIRST_HALF: CastMethod = CastMethod::Half(0);
const SECOND_HALF: CastMethod = CastMethod::Half(1);

#[test]
fn the_aftermath_half_is_cast_from_the_graveyard_then_exiled() {
    cr!("702.127", "702.127a");
    assert_supported_card("Cut // Ribbons");
    let mut t = TestGame::new(2);
    // Cut: {1}{R} sorcery, 4 damage to target creature. Ribbons: {X}{B}{B} sorcery,
    // aftermath, each opponent loses X life.
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Swamp", 4);
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Cut // Ribbons");
    let methods = cast_methods(&mut t, P0, card);
    assert!(methods.contains(&FIRST_HALF));
    assert!(!methods.contains(&SECOND_HALF) && !methods.contains(&AFTERMATH));
    t.cast(P0, card)
        .method(FIRST_HALF)
        .target(Entity::Object(giant))
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(t.in_graveyard(P0, "Cut // Ribbons"));
    // From the graveyard, only Ribbons can be cast.
    let in_gy = t.g.current(card);
    let methods = cast_methods(&mut t, P0, in_gy);
    assert_eq!(methods, vec![AFTERMATH]);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let spell = t.cast(P0, in_gy).method(AFTERMATH).go();
    assert_eq!(t.g.obj(spell).chars.name, "Ribbons");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Cut // Ribbons"));
    assert!(!t.in_graveyard(P0, "Cut // Ribbons"));
}

#[test]
fn a_countered_aftermath_spell_is_exiled() {
    cr!("702.127a");
    ruling!(
        "Cut // Ribbons",
        "A spell with aftermath cast from a graveyard will always be exiled afterward, whether it resolves, it's countered, or it leaves the stack in some other way."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let card = t.graveyard(P0, "Cut // Ribbons");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    let spell = t.cast(P0, card).method(AFTERMATH).go();
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(Entity::Object(spell)).go();
    t.resolve_all();
    assert!(t.in_exile("Cut // Ribbons"));
}

#[test]
fn another_permission_from_a_graveyard_allows_either_half() {
    cr!("702.127a");
    ruling!(
        "Cut // Ribbons",
        "If another effect allows you to cast a split card with aftermath from a graveyard, you may cast either half. If you cast the half that has aftermath, you'll exile the card if it would leave the stack."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Swamp", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.graveyard(P0, "Cut // Ribbons");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![card],
        mtg_engine::ability::Duration::EndOfTurn,
        false,
        None,
    );
    let methods = cast_methods(&mut t, P0, card);
    assert!(methods.contains(&FIRST_HALF));
    assert!(methods.contains(&SECOND_HALF));
    // Cut cast from the graveyard doesn't have aftermath: it goes back to the graveyard.
    t.cast(P0, card)
        .method(FIRST_HALF)
        .target(Entity::Object(giant))
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Cut // Ribbons"));
    // Ribbons cast with that permission is exiled.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let card = t.graveyard(P0, "Cut // Ribbons");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![card],
        mtg_engine::ability::Duration::EndOfTurn,
        false,
        None,
    );
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.cast(P0, card).method(SECOND_HALF).go();
    t.resolve_all();
    assert!(t.in_exile("Cut // Ribbons"));
}

#[test]
fn the_aftermath_half_cant_be_cast_from_another_zone() {
    cr!("702.127a");
    ruling!(
        "Cut // Ribbons",
        "If another effect allows you to cast a split card with aftermath from any zone other than a graveyard, you can't cast the half with aftermath."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Swamp", 2);
    t.battlefield(P1, "Hill Giant");
    let card = t.exile(P0, "Cut // Ribbons");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![card],
        mtg_engine::ability::Duration::EndOfTurn,
        false,
        None,
    );
    let methods = cast_methods(&mut t, P0, card);
    assert!(methods.contains(&FIRST_HALF));
    assert!(!methods.contains(&SECOND_HALF));
    assert!(!methods.contains(&AFTERMATH));
    assert!(t.cast(P0, card).method(SECOND_HALF).try_go().is_err());
    assert_eq!(t.zone(card), Zone::Exile);
}
