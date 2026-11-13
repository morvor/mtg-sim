//! CR 601.2b, 601.2c, 601.2h (107.3a): X is announced with the costs and the targets it
//! implies, and the spell must then be castable with it: its total cost, the parts that
//! use X included ("Discard X cards", "Sacrifice X Mountains"), must be payable, and it
//! must have enough legal targets ("Tap X target creatures"). The default X (a player who
//! answers with the default) is the greatest value the spell could be cast with, and no
//! greater value is offered.

use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

/// The X recorded for the spell.
fn x_of(t: &TestGame, spell: ObjectId) -> Option<i32> {
    t.g.obj(spell).stack.as_ref().and_then(|s| s.x)
}

/// The greatest X `P0` was offered.
fn offered_max(t: &TestGame) -> i64 {
    t.asked()
        .iter()
        .rev()
        .find_map(|(p, d)| match d {
            Decision::ChooseX { max, .. } if *p == P0 => Some(*max),
            _ => None,
        })
        .expect("asked for X")
}

#[test]
fn the_default_x_of_a_discard_x_cost_is_the_number_of_cards_in_hand() {
    cr!("601.2b", "601.2h", "107.3a");
    interp!("default-numbers");
    // Conflagrate: "Flashback—{R}{R}, Discard X cards." Six Mountains, two cards in hand.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 6);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    let card = t.graveyard(P0, "Conflagrate");
    let spell = t.cast(P0, card).method(FLASHBACK).go();
    assert_eq!(x_of(&t, spell), Some(2));
    assert_eq!(offered_max(&t), 2);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn the_default_x_of_an_exile_x_cards_cost_is_the_number_of_cards_to_exile() {
    cr!("601.2b", "601.2h", "107.3a");
    interp!("default-numbers");
    // Flash of Insight: "Flashback—{1}{U}, Exile X blue cards from your graveyard." One
    // other blue card there.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 6);
    t.graveyard(P0, "Counterspell");
    t.graveyard(P0, "Lightning Bolt");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    let card = t.graveyard(P0, "Flash of Insight");
    let spell = t.cast(P0, card).method(FLASHBACK).go();
    assert_eq!(x_of(&t, spell), Some(1));
    // Summons of Saruman: "Flashback—{3}{U}{R}, Exile X cards from your graveyard." Two
    // other cards there, eight lands.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 4);
    t.lands(P0, "Mountain", 4);
    t.graveyard(P0, "Counterspell");
    t.graveyard(P0, "Grizzly Bears");
    let card = t.graveyard(P0, "Summons of Saruman");
    let spell = t.cast(P0, card).method(FLASHBACK).go();
    assert_eq!(x_of(&t, spell), Some(2));
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn the_default_x_of_a_remove_x_counters_cost_is_the_counters_there_are() {
    cr!("601.2b", "601.2h", "107.3a");
    interp!("default-numbers");
    // Light Up the Night: "Flashback—{3}{R}, Remove X loyalty counters from among
    // planeswalkers you control. If you cast this spell this way, X can't be 0." Jace
    // Beleren enters with three loyalty counters; eight Mountains.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 8);
    let jace = t.battlefield(P0, "Jace Beleren");
    let n = t.counters(jace, "loyalty") as i32;
    assert!(n > 0 && n < 4);
    let card = t.graveyard(P0, "Light Up the Night");
    let spell = t.cast(P0, card).method(FLASHBACK).target(P1).go();
    assert_eq!(x_of(&t, spell), Some(n));
}

#[test]
fn the_default_x_of_a_sacrifice_x_cost_is_the_permanents_there_are() {
    cr!("601.2b", "601.2h", "107.3a");
    interp!("default-numbers");
    // Firecat Blitz: "Flashback—{R}{R}, Sacrifice X Mountains." Two Mountains and four
    // Islands: six mana, but only two Mountains to sacrifice.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Island", 4);
    let card = t.graveyard(P0, "Firecat Blitz");
    let spell = t.cast(P0, card).method(FLASHBACK).go();
    assert_eq!(x_of(&t, spell), Some(2));
    assert_eq!(offered_max(&t), 2);
}

#[test]
fn the_default_x_of_x_targets_is_the_number_of_legal_targets() {
    cr!("601.2b", "601.2c", "107.3a");
    interp!("default-numbers");
    // Plant Tadpoles ({X}{U}{U} Sorcery — Omen): "Tap X target creatures." Six Islands
    // (X up to 4 by mana), two creatures.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 6);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Illithid Harvester // Plant Tadpoles");
    let spell = t
        .cast(P0, card)
        .method(CastMethod::Half(1))
        .targets(&[a.into(), b.into()])
        .go();
    assert_eq!(x_of(&t, spell), Some(2));
    assert_eq!(offered_max(&t), 2);
    t.resolve_all();
    assert!(t.g.obj(t.g.current(a)).tapped && t.g.obj(t.g.current(b)).tapped);
}
