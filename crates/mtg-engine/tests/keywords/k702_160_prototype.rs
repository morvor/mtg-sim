//! CR 702.160 Prototype (see also `tests/cr/r718_prototype_cards.rs`).

use crate::common_k702_153_167::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::{CardType, Color};
use mtg_engine::*;

const PROTOTYPED: CastMethod = CastMethod::Keyword(KeywordKind::Prototype);

#[test]
fn a_prototyped_spell_uses_the_alternative_power_toughness_and_mana_cost() {
    cr!("702.160", "702.160a");
    ruling!(
        "Goring Warplow",
        "Regardless of how it was cast, a prototype card always has the same name, abilities, types, and so on."
    );
    assert_supported("Goring Warplow");
    // Goring Warplow: {6} 5/4 deathtouch; "Prototype {1}{B} — 1/1".
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let card = t.hand(P0, "Goring Warplow");
    // Two lands can't pay {6}, but can pay the prototype's {1}{B}.
    assert!(t.cast(P0, card).try_go().is_err());
    let spell = t.cast(P0, card).method(PROTOTYPED).go();
    let s = t.g.obj(spell);
    assert_eq!(s.chars.mana_cost.as_ref().unwrap().to_string(), "{1}{B}");
    assert_eq!(t.g.mana_value_of(spell), 2);
    assert!(s.chars.colors.contains(Color::Black));
    t.resolve();
    let w = named(&t, P0, "Goring Warplow")[0];
    assert_eq!(t.pt(w), (1, 1));
    // Same name, abilities and types.
    assert!(has_kw(&t, w, KeywordKind::Deathtouch));
    assert!(t.obj(w).is(CardType::Artifact) && t.obj(w).is(CardType::Creature));
    // Leaving the battlefield, it resumes its normal characteristics.
    t.g.move_object(
        w,
        Zone::Hand(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    )
    .unwrap();
    let back = t.g.current(w);
    assert_eq!(t.g.mana_value_of(back), 6);
    assert!(t.g.obj(back).chars.colors.is_colorless());
    // Cast normally, it's a colorless 5/4.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let card = t.hand(P0, "Goring Warplow");
    t.cast(P0, card).go();
    t.resolve();
    let w = named(&t, P0, "Goring Warplow")[0];
    assert_eq!(t.pt(w), (5, 4));
    assert!(t.obj(w).chars.colors.is_colorless());
}
