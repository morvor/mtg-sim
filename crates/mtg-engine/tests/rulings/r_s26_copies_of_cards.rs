//! Rulings batch S26 — token copies of cards in a graveyard (or exiled from one) copy
//! only the card, as modified by the copy effect's exceptions (CR 707.2, 707.9b, 400.7).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s06_common::attach_new;
use crate::r_s26_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn god_pharaohs_gift_token_copies_only_the_card() {
    cr!("707.2", "707.9b", "400.7");
    ruling!(
        "God-Pharaoh's Gift",
        "The token copies exactly what was printed on the original card and nothing else, except the characteristics it specifically modifies. It doesn't copy any information about the object the card was before it was put into your graveyard."
    );
    supported("God-Pharaoh's Gift");
    // "At the beginning of combat on your turn, you may exile a creature card from your
    // graveyard. If you do, create a token that's a copy of that card, except it's a 4/4
    // black Zombie. It gains haste until end of turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "God-Pharaoh's Gift");
    // A Serra Angel that had counters, an Aura and a color change before it died.
    let angel = t.battlefield(P0, "Serra Angel");
    attach_new(&mut t, P0, "Holy Strength", angel);
    dress_up(&mut t, angel);
    destroy(&mut t, angel);
    let card = t.g.current(angel);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.in_exile("Serra Angel"));
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Serra Angel");
    assert!(fresh(&t, toks[0]));
    // The exceptions: a 4/4 black Zombie; the rest is the card's (flying, vigilance).
    assert_eq!(t.pt(toks[0]), (4, 4));
    assert_eq!(tok.chars.colors, ColorSet::single(Color::Black));
    assert!(tok.chars.subtypes.iter().any(|s| s == "Zombie"));
    assert!(!tok.chars.subtypes.iter().any(|s| s == "Angel"));
    assert!(tok.has_keyword(KeywordKind::Flying) && tok.has_keyword(KeywordKind::Vigilance));
    assert!(tok.has_keyword(KeywordKind::Haste));
}
