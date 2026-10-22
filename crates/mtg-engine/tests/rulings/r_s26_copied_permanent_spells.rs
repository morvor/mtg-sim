//! Rulings batch S26 — copies of permanent spells: a copy of a permanent spell becomes a
//! token as it resolves, but that token isn't "created" (CR 707.10f, 608.3f, 111.1).

use crate::r_s01_common::supported;
use crate::r_s26_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn verazols_copy_of_a_creature_spell_becomes_a_token_that_isnt_created() {
    cr!("707.10f", "608.3f", "111.1");
    ruling!(
        "Verazol, the Split Current",
        "The token that a resolving copy of a spell becomes isn't said to have been \"created.\""
    );
    supported("Verazol, the Split Current");
    supported("Anointed Procession");
    supported("Kavu Titan");
    let mut t = TestGame::new(2);
    let verazol = t.battlefield(P0, "Verazol, the Split Current");
    t.g.add_counters(Entity::Object(verazol), counters::PLUS1, 4, None);
    // "If an effect would create one or more tokens under your control, it creates twice
    // that many of those tokens instead."
    t.battlefield(P0, "Anointed Procession");
    t.lands(P0, "Forest", 5);
    let titan = t.hand(P0, "Kavu Titan");
    t.cast(P0, titan).kicked(true).go();
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    t.resolve_all();
    assert_eq!(t.counters(verazol, counters::PLUS1), 2);
    // One token (the resolved copy, kicked like the original), not two.
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Kavu Titan");
    assert_eq!(t.pt(toks[0]), (5, 5));
    assert!(t.obj_now(toks[0]).has_keyword(KeywordKind::Trample));
    assert_eq!(t.named_on_battlefield("Kavu Titan").len(), 2);
    // Tokens that are created are doubled.
    let before = t.g.battlefield.clone();
    crate::r_s02_common::create_token(&mut t, P0, "Rat");
    assert_eq!(new_tokens(&t, P0, &before).len(), 2);
}
