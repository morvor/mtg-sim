//! Rulings batch S22 — casting from the top of a library, and a top card that changes
//! while a spell is being cast: the new top card can't be looked at, and isn't revealed,
//! until the spell has become cast (CR 401.5, 601.2i).

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::zones;
use mtg_engine::*;

/// Whether P0 may look at the top card of their library now.
fn p0_sees_top(g: &Game) -> bool {
    g.library_top(P0).is_some_and(|top| can_look_at(g, P0, top))
}

/// Whether the top card of P0's library is revealed (P1 may see it) now.
fn top_revealed(g: &Game) -> bool {
    g.library_top(P0)
        .is_some_and(|top| zones::revealed_top(g, P0) == Some(top) && can_look_at(g, P1, top))
}

fn is_choose_x(d: &Decision) -> bool {
    matches!(d, Decision::ChooseX { .. })
}

fn is_choose_targets(d: &Decision) -> bool {
    matches!(d, Decision::ChooseTargets { .. })
}

/// P0, with `permission` on the battlefield, casts Endless One ({X}: "enters with X
/// +1/+1 counters") from the top of their library with X = 2; `look` is checked while
/// X is chosen (the card below is then the top card). Returns what `look` saw then and
/// after the spell became cast.
fn cast_endless_one_from_top(permission: &str, look: fn(&Game) -> bool) -> (Vec<bool>, bool) {
    let mut t = TestGame::new(2);
    let next = t.library_top(P0, "Hill Giant");
    let one = t.library_top(P0, "Endless One");
    t.battlefield(P0, permission);
    t.lands(P0, "Wastes", 2);
    let seen = watch(&mut t, P0, is_choose_x, look);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.cast(P0, one).go();
    let during = seen.lock().unwrap().clone();
    assert_eq!(t.g.library_top(P0), Some(next));
    let after = look(&t.g);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Endless One").len(), 1);
    (during, after)
}

#[test]
fn mystic_forge_the_next_card_cant_be_looked_at_until_the_spell_is_cast() {
    cr!("401.5", "601.2i", "601.3");
    ruling!(
        "Mystic Forge",
        "If the top card of your library changes while you're casting a spell, playing a land, or activating an ability, you can't look at the new top card until you finish doing so. This means that if you cast the top card of your library, you can't look at the next one until you're done paying for that spell."
    );
    supported("Mystic Forge");
    // Endless One is a colorless spell: Mystic Forge lets P0 cast it from the top.
    assert_eq!(
        cast_endless_one_from_top("Mystic Forge", p0_sees_top),
        (vec![false], true)
    );
}

#[test]
fn elven_chorus_the_next_card_cant_be_looked_at_until_the_spell_is_cast() {
    cr!("401.5", "601.2i", "601.3");
    ruling!(
        "Elven Chorus",
        "If the top card of your library changes while you're casting a spell, playing a land, or activating an ability, you can't look at the new top card until you finish doing so. This means that if you cast a spell from the top of your library, you can't look at the next one until you're done paying for that spell."
    );
    supported("Elven Chorus");
    assert_eq!(
        cast_endless_one_from_top("Elven Chorus", p0_sees_top),
        (vec![false], true)
    );
}

#[test]
fn garruks_horde_the_next_card_isnt_revealed_until_the_spell_is_cast() {
    cr!("401.5", "601.2i", "601.3");
    ruling!(
        "Garruk's Horde",
        "If the top card of your library changes while you’re casting a spell, playing a land, or activating an ability, the new top card won’t be revealed until you finish doing so."
    );
    supported("Garruk's Horde");
    assert_eq!(
        cast_endless_one_from_top("Garruk's Horde", top_revealed),
        (vec![false], true)
    );
}

#[test]
fn goblin_spy_the_next_card_isnt_revealed_until_the_spell_is_cast() {
    cr!("401.5", "601.2i");
    ruling!(
        "Goblin Spy",
        "If the top card of your library changes while you're casting a spell, playing a land, or activating an ability, the new top card won't be revealed until you finish doing so."
    );
    supported("Goblin Spy");
    // Goblin Spy reveals the top card; Elven Chorus lets P0 cast the creature on top.
    let mut t = TestGame::new(2);
    let next = t.library_top(P0, "Hill Giant");
    let one = t.library_top(P0, "Endless One");
    t.battlefield(P0, "Goblin Spy");
    t.battlefield(P0, "Elven Chorus");
    t.lands(P0, "Wastes", 2);
    assert!(top_revealed(&t.g));
    let seen = watch(&mut t, P0, is_choose_x, top_revealed);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.cast(P0, one).go();
    assert_eq!(t.g.library_top(P0), Some(next));
    assert_eq!(*seen.lock().unwrap(), vec![false]);
    assert!(top_revealed(&t.g));
}

/// P0, with `permission` on the battlefield, casts Lightning Bolt from the top of their
/// library at P1; whether P0 could look at the next card while choosing the target, and
/// after the spell became cast.
fn cast_bolt_from_top(permission: &str) -> (Vec<bool>, bool) {
    let mut t = TestGame::new(2);
    t.library_top(P0, "Hill Giant");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.battlefield(P0, permission);
    t.lands(P0, "Mountain", 1);
    let seen = watch(&mut t, P0, is_choose_targets, p0_sees_top);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    let during = seen.lock().unwrap().clone();
    let after = p0_sees_top(&t.g);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    (during, after)
}

#[test]
fn precognition_field_the_next_card_cant_be_looked_at_until_the_spell_is_cast() {
    cr!("401.5", "601.2i", "601.3");
    ruling!(
        "Precognition Field",
        "If the top card of your library changes while you’re casting a spell, playing a land, or activating an ability, you can’t look at the new top card until you finish doing so. This means that if you cast the top card of your library, you can’t look at the next one until you’re done paying for that spell."
    );
    supported("Precognition Field");
    assert_eq!(cast_bolt_from_top("Precognition Field"), (vec![false], true));
}

#[test]
fn madame_web_the_next_card_cant_be_looked_at_until_the_spell_is_cast() {
    cr!("401.5", "601.2i", "601.3");
    ruling!(
        "Madame Web, Clairvoyant",
        "If the top card of your library changes while you're casting a spell, playing a land, activating an ability, or taking a special action, you can't look at the new top card until you finish doing so. This means that if you cast the top card of your library, you can't look at the next one until you're done paying for that spell."
    );
    supported("Madame Web, Clairvoyant");
    // Lightning Bolt is a noncreature spell.
    assert_eq!(
        cast_bolt_from_top("Madame Web, Clairvoyant"),
        (vec![false], true)
    );
}
