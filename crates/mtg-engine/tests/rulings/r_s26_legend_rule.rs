//! Rulings batch S26 — token copies of legendary permanents that are copying something
//! else (CR 707.3), and static abilities that exempt permanents from the legend rule
//! (CR 704.5j).

use crate::r_s01_common::supported;
use crate::r_s26_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn cadrics_token_copies_what_the_legendary_permanent_is_copying() {
    cr!("707.3", "704.5j", "707.5");
    ruling!(
        "Cadric, Soul Kindler",
        "If the copied permanent is copying something else, the token enters the battlefield as whatever that permanent is copying."
    );
    supported("Cadric, Soul Kindler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cadric, Soul Kindler");
    t.lands(P0, "Wastes", 1);
    let kiki = t.battlefield(P1, "Kiki-Jiki, Mirror Breaker");
    // A Clone enters as a copy of Kiki-Jiki: a nontoken legendary permanent entering.
    t.answer_choose(P0, &[Entity::Object(kiki)]);
    let before = t.g.battlefield.clone();
    let clone = t.enter(P0, "Clone");
    t.answer_yes(P0, true);
    t.resolve_all();
    let clone = t.g.current(clone);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Kiki-Jiki, Mirror Breaker");
    assert!(tok.chars.is_legendary());
    assert!(tok.has_keyword(KeywordKind::Haste));
    assert_eq!(t.pt(toks[0]), (2, 2));
    // The legend rule doesn't apply to the token: both of P0's Kiki-Jikis stay.
    assert!(t.on_battlefield(clone) && t.on_battlefield(toks[0]));
}

#[test]
fn the_legend_rule_ignores_exempt_permanents() {
    cr!("704.5j");
    supported("Mirror Gallery");
    // Cadric: only tokens are exempt, so two nontoken Kiki-Jikis still meet the rule.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cadric, Soul Kindler");
    t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
    t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
    t.settle();
    assert_eq!(t.named_on_battlefield("Kiki-Jiki, Mirror Breaker").len(), 1);
    // Mirror Gallery: the legend rule doesn't apply at all.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
    t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
    t.settle();
    assert_eq!(t.named_on_battlefield("Kiki-Jiki, Mirror Breaker").len(), 2);
}
