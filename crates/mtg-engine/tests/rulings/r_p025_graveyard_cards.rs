//! Rulings batch P025 — copies of creature cards in a graveyard copy what's printed on the
//! card, not anything about the permanent it was before it went there (CR 400.7, 707.2),
//! and use the card's last known information if it left the graveyard (CR 608.2h).

use crate::r_p025_common::*;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s dressed-up Hill Giant (see [`dressed_giant`]) is destroyed; returns the card in
/// `p`'s graveyard.
fn dead_dressed_giant(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let giant = dressed_giant(t, p);
    t.g.destroy(giant, None);
    t.settle();
    let card = t.g.current(giant);
    assert_eq!(t.zone(card), Zone::Graveyard(p));
    card
}

/// `p`'s Clone copying a Serra Angel is destroyed; returns the Clone card in `p`'s
/// graveyard.
fn dead_clone(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let clone = cloned_angel(t, p);
    t.g.destroy(clone, None);
    t.settle();
    let card = t.g.current(clone);
    assert_eq!(t.zone(card), Zone::Graveyard(p));
    assert_eq!(name_of(t, card), "Clone");
    card
}

/// `p` activates `source`'s "{X}: ... becomes a copy of target creature card in your
/// graveyard with mana value X" ability for `card` with X = `x`, and it resolves.
fn x_copy(t: &mut TestGame, p: PlayerId, source: ObjectId, card: ObjectId, x: i64) {
    t.lands(p, "Wastes", x as usize);
    t.answer(p, DecisionKind::X, Answer::Number(x));
    t.answer_targets(p, &[obj(card)]);
    activate_containing(t, p, source, "becomes a copy").expect("activation");
    t.resolve_all();
}

#[test]
fn lazav_the_multifarious_copies_only_the_printed_card() {
    cr!("707.2", "707.9b", "400.7");
    ruling!(
        "Lazav, the Multifarious",
        "Lazav copies exactly what was printed on the original card and nothing else, except the characteristics it specifically modifies. It doesn't copy any information about the object the card was before it was put into your graveyard."
    );
    supported("Lazav, the Multifarious");
    let mut t = TestGame::new(2);
    let lazav = t.battlefield(P0, "Lazav, the Multifarious");
    let card = dead_dressed_giant(&mut t, P0);
    x_copy(&mut t, P0, lazav, card, 4);
    let o = t.obj_now(lazav);
    assert_eq!(o.chars.name, "Lazav, the Multifarious");
    assert_eq!(o.chars.colors, red());
    assert!(has_subtype(&t, lazav, "Giant") && legendary(&t, lazav));
    assert_eq!(t.pt(lazav), (3, 3));
    // A Clone card that was a Serra Angel on the battlefield: Lazav becomes a 0/0 Clone
    // (named Lazav, with its +1/+1 counter a 1/1), not a Serra Angel.
    let mut t = TestGame::new(2);
    let lazav = t.battlefield(P0, "Lazav, the Multifarious");
    plus1(&mut t, lazav, 1);
    let card = dead_clone(&mut t, P0);
    x_copy(&mut t, P0, lazav, card, 4);
    let o = t.obj_now(lazav);
    assert_eq!(o.chars.name, "Lazav, the Multifarious");
    assert!(legendary(&t, lazav) && !has_kw(&t, lazav, KeywordKind::Flying));
    assert!(!has_subtype(&t, lazav, "Angel"));
    assert_eq!(t.pt(lazav), (1, 1));
}

#[test]
fn likeness_looter_copies_only_the_printed_card() {
    cr!("707.2", "707.9b", "400.7", "704.5f");
    ruling!(
        "Likeness Looter",
        "Likeness Looter copies exactly what was printed on the original card and nothing else, except the characteristics it specifically modifies. It doesn't copy any information about the object the card was before it was put into your graveyard."
    );
    supported("Likeness Looter");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let looter = t.battlefield(P0, "Likeness Looter");
    // The Clone card was a Serra Angel on the battlefield; in the graveyard it's a 0/0
    // Clone. The Looter becomes a 0/0 Clone with flying and dies.
    let card = dead_clone(&mut t, P0);
    x_copy(&mut t, P0, looter, card, 4);
    assert!(!t.on_battlefield(looter));
    assert!(t.in_graveyard(P0, "Likeness Looter"));
    // And with a +1/+1 counter it survives as a 1/1 Clone with flying.
    let looter = t.battlefield(P0, "Likeness Looter");
    plus1(&mut t, looter, 1);
    x_copy(&mut t, P0, looter, card, 4);
    assert_eq!(name_of(&t, looter), "Clone");
    assert!(has_kw(&t, looter, KeywordKind::Flying));
    assert_eq!(t.pt(looter), (1, 1));
}

#[test]
fn lazav_familiar_stranger_copies_only_the_printed_card() {
    cr!("707.2", "400.7");
    ruling!(
        "Lazav, Familiar Stranger",
        "Lazav copies exactly what was printed on the original card and nothing else. It doesn't copy any information about the object the card was before it was put into a graveyard."
    );
    supported("Lazav, Familiar Stranger");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let lazav = t.battlefield(P0, "Lazav, Familiar Stranger");
    let card = dead_dressed_giant(&mut t, P1);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    // Targeting an opponent is a crime.
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    let o = t.obj_now(lazav);
    assert_eq!(o.chars.name, "Hill Giant");
    assert_eq!(o.chars.colors, red());
    // A printed 3/3 with Lazav's own +1/+1 counter.
    assert_eq!(t.counters(lazav, counters::PLUS1), 1);
    assert_eq!(t.pt(lazav), (4, 4));
}

#[test]
fn lazotep_convert_copies_the_printed_card_and_keeps_its_other_colors_and_types() {
    cr!("707.2", "707.9b", "400.7", "712.14a");
    ruling!(
        "Invasion of Amonkhet // Lazotep Convert",
        "Lazotep Convert copies exactly what was printed on the original creature card, with the noted exceptions."
    );
    ruling!(
        "Invasion of Amonkhet // Lazotep Convert",
        "Lazotep Convert has received an update to its official rules text to clarify that it keeps its other colors and types as part of its copy effect."
    );
    supported("Invasion of Amonkhet // Lazotep Convert");
    let mut t = TestGame::new(2);
    let card = dead_dressed_giant(&mut t, P1);
    t.answer_choose(P0, &[obj(card)]);
    let c = crate::r_s17_common::enter_transformed(
        &mut t,
        P0,
        "Invasion of Amonkhet // Lazotep Convert",
    );
    t.resolve_all();
    let o = t.obj_now(c);
    assert_eq!(o.chars.name, "Hill Giant");
    // A red Giant that's also a black Zombie, 4/4.
    assert!(o.chars.colors.contains(Color::Red) && o.chars.colors.contains(Color::Black));
    assert!(!o.chars.colors.contains(Color::Green));
    assert!(has_subtype(&t, c, "Giant") && has_subtype(&t, c, "Zombie"));
    assert_eq!(t.pt(c), (4, 4));
    assert!(o.counters.values().all(|n| *n == 0));
}

#[test]
fn lazav_dimir_mastermind_copies_the_card_not_what_the_clone_copied() {
    cr!("707.2", "707.9b", "400.7", "704.5f");
    ruling!(
        "Lazav, Dimir Mastermind",
        "Notably, copy effects that applied to the creature card when it was on the battlefield won't be copied by Lazav."
    );
    supported("Lazav, Dimir Mastermind");
    let mut t = TestGame::new(2);
    let lazav = t.battlefield(P0, "Lazav, Dimir Mastermind");
    // P1's Clone copying a Serra Angel dies: Lazav becomes a 0/0 Clone (named Lazav,
    // with hexproof) and dies.
    t.answer_yes(P0, true);
    dead_clone(&mut t, P1);
    t.resolve_all();
    assert!(!t.on_battlefield(lazav));
    assert!(t.in_graveyard(P0, "Lazav, Dimir Mastermind"));
}

#[test]
fn lazav_dimir_mastermind_uses_the_cards_last_known_information() {
    cr!("707.2", "608.2h");
    ruling!(
        "Lazav, Dimir Mastermind",
        "If that card is no longer in the graveyard when Lazav's ability resolves, use the characteristics of that card as it last existed in the graveyard."
    );
    supported("Lazav, Dimir Mastermind");
    let mut t = TestGame::new(2);
    let lazav = t.battlefield(P0, "Lazav, Dimir Mastermind");
    t.answer_yes(P0, true);
    let card = dead_dressed_giant(&mut t, P1);
    assert_eq!(triggers_on_stack(&t).len(), 1);
    // In response, the Hill Giant card is exiled from the graveyard.
    t.g.move_object(card, Zone::Exile, MoveCause::Effect, Some(P1));
    t.resolve_all();
    let o = t.obj_now(lazav);
    assert_eq!(o.chars.name, "Lazav, Dimir Mastermind");
    assert!(has_subtype(&t, lazav, "Giant") && legendary(&t, lazav));
    assert!(o.has_keyword(KeywordKind::Hexproof));
    assert_eq!(t.pt(lazav), (3, 3));
}
