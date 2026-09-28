//! Rulings batch S25 — the characteristics of tokens: a token's name is its subtypes plus
//! "Token" unless the effect names it (CR 111.4); a token that isn't a copy has no mana
//! cost and mana value 0, and a token that's a copy has the copied mana cost (CR 202.3a,
//! 707.2); entering tapped isn't part of a Powerstone's definition, so a copy of one
//! enters untapped (CR 707.2, 111.10h).

use crate::r_s01_common::{supported, tokens};
use crate::r_s02_common::target_candidates;
use crate::r_s06_common::activate_containing;
use crate::r_s08_common::mana_value;
use crate::r_s13_common::add;
use crate::r_s17_common::token_copy;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts Raise the Alarm ("Create two 1/1 white Soldier creature tokens."). Returns
/// the two tokens.
fn two_soldiers(t: &mut TestGame, p: PlayerId) -> Vec<ObjectId> {
    let before = tokens(t, p);
    cast_new(t, p, "Raise the Alarm", &[]);
    t.resolve_all();
    tokens(t, p)
        .into_iter()
        .filter(|id| !before.contains(id))
        .collect()
}

/// A token copy of a new `name` that P0 controls (the card itself then leaves).
fn token_copy_of(t: &mut TestGame, name: &str) -> ObjectId {
    let card = t.battlefield(P0, name);
    let tok = token_copy(t, P0, card)[0];
    crate::r_s02_common::destroy(t, card);
    tok
}

#[test]
fn a_token_s_name_is_its_subtypes_plus_token() {
    cr!("111.4");
    ruling!(
        "Maelstrom Pulse",
        "Unless a token is a copy of another permanent or was explicitly given a name by the effect that created it, its name is the subtypes it was given when it was created plus the word \"Token.\""
    );
    supported("Maelstrom Pulse");
    // "Destroy target nonland permanent and all other permanents with the same name as that
    // permanent."
    let mut t = TestGame::new(2);
    let soldiers = [two_soldiers(&mut t, P1), two_soldiers(&mut t, P1)].concat();
    for s in &soldiers {
        assert_eq!(name_now(&t, *s), "Soldier Token");
    }
    // Repel Intruders' Kithkin Soldiers are named "Kithkin Soldier Token".
    cast_new(&mut t, P1, "Repel Intruders", &[]);
    t.resolve_all();
    assert_eq!(tokens(&t, P1).len(), 6);
    cast_new(&mut t, P0, "Maelstrom Pulse", &[Entity::Object(soldiers[0])]);
    t.resolve_all();
    let left = tokens(&t, P1);
    assert_eq!(left.len(), 2);
    for k in left {
        assert_eq!(name_now(&t, k), "Kithkin Soldier Token");
    }
}

#[test]
fn a_token_copy_has_the_mana_cost_of_what_it_copies() {
    cr!("202.3a", "707.2");
    ruling!(
        "Abrupt Decay",
        "The mana value of a token that isn't a copy of another object is 0. A token that is a copy of another object has the same mana cost as that object."
    );
    supported("Abrupt Decay");
    // "Destroy target nonland permanent with mana value 3 or less."
    let mut t = TestGame::new(2);
    let soldier = two_soldiers(&mut t, P0)[0];
    let giant = token_copy_of(&mut t, "Hill Giant");
    assert_eq!(mana_value(&t, soldier), 0);
    assert_eq!(mana_value(&t, giant), 4);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Abrupt Decay", &[Entity::Object(soldier)]);
    let offered = target_candidates(&t, P0, from);
    assert!(offered[0].contains(&Entity::Object(soldier)));
    assert!(!offered[0].contains(&Entity::Object(giant)));
    t.resolve_all();
    assert!(!t.on_battlefield(soldier));
}

#[test]
fn tokens_that_arent_copies_have_mana_value_0() {
    cr!("202.3a", "707.2");
    ruling!(
        "Ratchet Bomb",
        "Tokens that aren't a copy of something else don't have a mana cost. Anything without a mana cost normally has a mana value of 0."
    );
    supported("Ratchet Bomb");
    // "{T}, Sacrifice this artifact: Destroy each nonland permanent with mana value equal to
    // the number of charge counters on this artifact." (None here.)
    let mut t = TestGame::new(2);
    let bomb = t.battlefield(P0, "Ratchet Bomb");
    let soldiers = two_soldiers(&mut t, P1);
    let bears = token_copy_of(&mut t, "Grizzly Bears");
    assert!(t.obj_now(soldiers[0]).chars.mana_cost.is_none());
    activate_containing(&mut t, P0, bomb, "Destroy each").unwrap();
    t.resolve_all();
    assert!(soldiers.iter().all(|s| !t.on_battlefield(*s)));
    assert!(t.on_battlefield(bears));
}

#[test]
fn a_creature_token_has_mana_value_0_unless_it_copies_a_creature() {
    cr!("202.3a", "707.2");
    ruling!(
        "Gaze of Granite",
        "The mana value of a creature token is 0 unless that token is a copy of another creature, in which case it copies that creature's mana cost."
    );
    supported("Gaze of Granite");
    // "Destroy each nonland permanent with mana value X or less." (X = 1.)
    let mut t = TestGame::new(2);
    let soldiers = two_soldiers(&mut t, P1);
    let giant = token_copy_of(&mut t, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Gaze of Granite");
    t.cast(P0, card).x(1).go();
    t.resolve_all();
    assert!(soldiers.iter().all(|s| !t.on_battlefield(*s)));
    assert!(!t.on_battlefield(elves));
    assert!(t.on_battlefield(giant));
}

#[test]
fn a_token_copying_something_has_its_mana_value() {
    cr!("202.3a", "707.2");
    ruling!(
        "Blast Zone",
        "Every token has mana value 0 unless it is copying something or was created with a specific mana cost."
    );
    supported("Blast Zone");
    // "{3}, {T}, Sacrifice this land: Destroy each nonland permanent with mana value equal
    // to the number of charge counters on this land." (Two here.)
    let mut t = TestGame::new(2);
    let zone = t.battlefield(P0, "Blast Zone");
    add(&mut t, zone, counters::CHARGE, 2);
    let soldiers = two_soldiers(&mut t, P1);
    let bears = token_copy_of(&mut t, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, zone, "Destroy each").unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(soldiers.iter().all(|s| t.on_battlefield(*s)));
}

#[test]
fn a_copy_of_a_token_with_x_in_its_mana_cost_has_x_0() {
    cr!("202.3e", "107.3g", "707.2");
    ruling!(
        "Oltec Matterweaver",
        "If the copied token has {X} in its mana cost, X is 0. (Most tokens don’t have a mana cost unless they’re copying something else.)"
    );
    supported("Oltec Matterweaver");
    supported("Engineered Explosives");
    // "Whenever you cast a creature spell, choose one — • Create a 1/1 colorless Gnome
    // artifact creature token. • Create a token that's a copy of target artifact token you
    // control."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oltec Matterweaver");
    let first = token_copy_of(&mut t, "Engineered Explosives");
    let before = tokens(&t, P0);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[Entity::Object(first)]);
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    let new: Vec<ObjectId> = tokens(&t, P0)
        .into_iter()
        .filter(|id| !before.contains(id))
        .collect();
    assert_eq!(new.len(), 1);
    assert_eq!(name_now(&t, new[0]), "Engineered Explosives");
    let cost = t.obj_now(new[0]).chars.mana_cost.clone().unwrap();
    assert!(format!("{cost}").contains("{X}"));
    assert_eq!(mana_value(&t, new[0]), 0);
}

/// The Powerstone tokens P0 controls.
fn powerstones(t: &TestGame) -> Vec<ObjectId> {
    tokens(t, P0)
        .into_iter()
        .filter(|id| t.obj(*id).chars.has_subtype("Powerstone"))
        .collect()
}

/// P0 copies the tapped Powerstone token `stone` with Saheeli's Artistry ("Create a token
/// that's a copy of target artifact."): the copy enters untapped.
fn copy_of_a_tapped_powerstone_is_untapped(t: &mut TestGame, stone: ObjectId) {
    assert!(t.obj_now(stone).tapped);
    lands_for_cost(t, P0, "Saheeli's Artistry");
    let card = t.hand(P0, "Saheeli's Artistry");
    t.cast(P0, card).modes(&[0]).target(stone).go();
    t.resolve_all();
    let stones = powerstones(t);
    assert_eq!(stones.len(), 2);
    let copy = *stones.iter().find(|id| **id != stone).unwrap();
    assert!(!t.obj_now(copy).tapped);
}

#[test]
fn a_copy_of_a_powerstone_doesnt_enter_tapped() {
    cr!("707.2", "111.10h");
    ruling!(
        "Stern Lesson",
        "Although all the cards in The Brothers' War that create Powerstone tokens create a tapped Powerstone token, entering the battlefield tapped isn't part of the token's definition. Notably, if you create a token that is a copy of a Powerstone token, the token copy won't enter the battlefield tapped."
    );
    supported("Stern Lesson");
    // "Draw two cards, then discard a card. Create a tapped Powerstone token."
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Stern Lesson", &[]);
    t.resolve_all();
    let stone = powerstones(&t)[0];
    copy_of_a_tapped_powerstone_is_untapped(&mut t, stone);
}

#[test]
fn a_copy_of_a_devastation_powerstone_doesnt_enter_tapped() {
    cr!("707.2", "111.10h");
    ruling!(
        "Terisiare's Devastation",
        "Although all the cards in The Brothers’ War that create Powerstone tokens create a tapped Powerstone token, entering the battlefield tapped isn’t part of the token’s definition. Notably, if you create a token that is a copy of a Powerstone token, the token copy won’t enter the battlefield tapped."
    );
    supported("Terisiare's Devastation");
    // "You lose X life and create X tapped Powerstone tokens. Then all creatures get -1/-1
    // until end of turn for each artifact you control."
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Terisiare's Devastation");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Terisiare's Devastation");
    t.cast(P0, card).x(1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    let stone = powerstones(&t)[0];
    copy_of_a_tapped_powerstone_is_untapped(&mut t, stone);
}
