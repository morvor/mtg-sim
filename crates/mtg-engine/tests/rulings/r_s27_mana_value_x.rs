//! Rulings batch S27 — {X} in the mana cost of an object that isn't a spell on the stack
//! is 0 (CR 107.3g, 202.3e): a permanent's mana value, a revealed card's, and the mana
//! cost paid for a scavenge ability (CR 107.3h).

use crate::r_s01_common::*;
use crate::r_s04_common::{activate_named, next_upkeep, spell_targets};
use crate::r_s08_common::mana_value;
use crate::r_s25_common::cast_new;
use crate::r_s27_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Endless One ({X} 0/0, "This creature enters with X +1/+1 counters on it."), cast by P0
/// with X = `x`: an X/X whose mana value on the battlefield is 0.
fn endless_one(t: &mut TestGame, x: usize) -> ObjectId {
    t.lands(P0, "Wastes", x);
    let card = t.hand(P0, "Endless One");
    t.cast(P0, card).x(x as i64).go();
    t.resolve_all();
    let id = t.g.current(card);
    assert_eq!(t.pt(id), (x as i32, x as i32));
    assert_eq!(mana_value(&t, id), 0);
    id
}

#[test]
fn abrupt_decay_destroys_a_permanent_whose_x_is_0() {
    cr!("107.3g", "202.3e", "115.1");
    ruling!(
        "Abrupt Decay",
        "If a permanent has {X} in its mana cost, X is considered to be 0."
    );
    supported("Abrupt Decay");
    supported("Endless One");
    // "Destroy target nonland permanent with mana value 3 or less." A 5/5 Endless One has
    // mana value 0.
    let mut t = TestGame::new(2);
    let one = endless_one(&mut t, 5);
    cast_new(&mut t, P0, "Abrupt Decay", &[Entity::Object(one)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"));
}

#[test]
fn epic_downfall_cant_target_a_creature_whose_x_is_0() {
    cr!("107.3g", "202.3e", "115.1");
    ruling!(
        "Epic Downfall",
        "If a creature on the battlefield has {X} in its mana cost, X is considered to be 0."
    );
    supported("Epic Downfall");
    // "Exile target creature with mana value 3 or greater." The 5/5 Endless One (mana
    // value 0) isn't a legal target; Hill Giant (mana value 4) is.
    let mut t = TestGame::new(2);
    let one = endless_one(&mut t, 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let targets = spell_targets(&mut t, P0, "Epic Downfall");
    assert!(!targets.contains(&Entity::Object(one)));
    assert!(targets.contains(&Entity::Object(giant)));
}

#[test]
fn repeal_for_x_0_returns_a_permanent_whose_x_is_0() {
    cr!("107.3g", "202.3e", "107.3a");
    ruling!(
        "Repeal",
        "If a permanent on the battlefield has {X} in its mana cost, X is considered to be 0."
    );
    supported("Repeal");
    // "Return target nonland permanent with mana value X to its owner's hand. Draw a
    // card." With X = 0, it returns the 4/4 Endless One.
    let mut t = TestGame::new(2);
    let one = endless_one(&mut t, 4);
    let hand = t.hand_size(P0);
    t.lands(P0, "Island", 1);
    let repeal = t.hand(P0, "Repeal");
    t.cast(P0, repeal).x(0).target(one).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Endless One"));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn dark_tutelage_a_revealed_card_s_x_is_0() {
    cr!("107.3g", "202.3e");
    ruling!(
        "Dark Tutelage",
        "If the mana cost of the revealed card includes {X}, X is considered to be 0."
    );
    supported("Dark Tutelage");
    supported("Blaze");
    // "At the beginning of your upkeep, reveal the top card of your library and put that
    // card into your hand. You lose life equal to its mana value." Blaze ({X}{R}) has
    // mana value 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dark Tutelage");
    t.library_top(P0, "Blaze");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.in_hand(P0, "Blaze"));
    assert_eq!(t.life(P0), 19);
}

#[test]
fn a_scavenged_card_s_x_is_0() {
    cr!("107.3h", "702.97a");
    ruling!(
        "Young Deathclaws",
        "If the creature card you scavenge has {X} in its mana cost, X is 0."
    );
    supported("Young Deathclaws");
    supported("Slumbering Trudge");
    // Young Deathclaws: "Each creature card in your graveyard has scavenge. The scavenge
    // cost is equal to its mana cost." Slumbering Trudge ({X}{G} 6/6) is scavenged for
    // {G}: X is 0 even though P0 has more mana and would choose X = 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Deathclaws");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let trudge = t.graveyard(P0, "Slumbering Trudge");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_named(&mut t, P0, trudge, "Scavenge", 0).expect("scavenge");
    assert_eq!(tapped_lands(&t, P0), 1);
    t.resolve_all();
    assert!(t.in_exile("Slumbering Trudge"));
    assert_eq!(t.counters(bears, "+1/+1"), 6);
}

#[test]
fn narset_a_discarded_card_s_x_is_0() {
    cr!("107.3g", "202.3e", "603.12");
    ruling!(
        "Narset of the Ancient Way",
        "If a card you discard has {X} in its mana cost, X is considered to be 0."
    );
    supported("Narset of the Ancient Way");
    // "−2: Draw a card, then you may discard a card. When you discard a nonland card this
    // way, Narset deals damage equal to that card's mana value to target creature or
    // planeswalker." Blaze ({X}{R}) has mana value 1.
    let mut t = TestGame::new(2);
    let narset = t.battlefield(P0, "Narset of the Ancient Way");
    let giant = t.battlefield(P1, "Hill Giant");
    let blaze = t.hand(P0, "Blaze");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(blaze)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.activate(P0, narset, 1, &[]).expect("activate −2");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blaze"));
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 1);
    assert_eq!(t.counters(narset, "loyalty"), 2);
    // Discarding a land: no damage.
    let mut t = TestGame::new(2);
    let narset = t.battlefield(P0, "Narset of the Ancient Way");
    let giant = t.battlefield(P1, "Hill Giant");
    let forest = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.activate(P0, narset, 1, &[]).expect("activate −2");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 0);
}
