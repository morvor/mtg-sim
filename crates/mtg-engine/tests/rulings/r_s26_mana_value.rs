//! Rulings batch S26 — the mana values of tokens and copies: a token that isn't a copy
//! has no mana cost, so mana value 0; a copy has the mana cost of what it copies (CR
//! 202.3, 202.3a, 202.3e, 707.2, 111.4).

use crate::r_s01_common::supported;
use crate::r_s02_common::create_token;
use crate::r_s04_common::spell_targets;
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::token_copy;
use crate::r_s26_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s Clone enters as a copy of `what`.
fn clone_of(t: &mut TestGame, p: PlayerId, what: ObjectId) -> ObjectId {
    t.answer_choose(p, &[Entity::Object(what)]);
    let c = t.enter(p, "Clone");
    t.settle();
    t.g.current(c)
}

#[test]
fn a_token_that_isnt_a_copy_cant_pay_dreadsires_ward_cost() {
    cr!("202.3a", "111.4", "702.21a");
    ruling!(
        "Ulamog's Dreadsire",
        "The mana value of a token that isn't a copy of another object is 0."
    );
    supported("Ulamog's Dreadsire");
    // "Ward—Sacrifice a permanent with mana value 1 or greater."
    let mut t = TestGame::new(2);
    let dreadsire = t.battlefield(P0, "Ulamog's Dreadsire");
    // P0's own Eldrazi token has mana value 0.
    let before = t.g.battlefield.clone();
    t.activate(P0, dreadsire, 0, &[]).expect("activate");
    t.resolve_all();
    let eldrazi = new_tokens(&t, P0, &before)[0];
    assert_eq!(t.pt(eldrazi), (10, 10));
    assert_eq!(mv(&mut t, eldrazi), 0);
    // P1's only other permanent is a token that isn't a copy: they can't pay.
    let rat = create_token(&mut t, P1, "Rat");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(dreadsire).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.on_battlefield(rat));
    assert_eq!(t.obj_now(dreadsire).damage, 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // A token that's a copy of Grizzly Bears has mana value 2, so it can be sacrificed.
    // (P1 pays the ward cost, so "a permanent" in it is one P1 controls.)
    let bears = t.battlefield(P1, "Grizzly Bears");
    let copy = token_copy(&mut t, P1, bears)[0];
    crate::r_s02_common::destroy(&mut t, bears);
    assert_eq!(mv(&mut t, copy), 2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(dreadsire).go();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(!t.g.is_live(copy));
    assert!(t.on_battlefield(rat));
    assert_eq!(t.obj_now(dreadsire).damage, 3);
}

#[test]
fn smother_can_destroy_a_token_unless_its_copying_something_bigger() {
    cr!("202.3a", "707.2", "115.1");
    ruling!(
        "Smother",
        "A token has a mana value of 0, unless it is copying something else."
    );
    supported("Smother");
    let mut t = TestGame::new(2);
    let dreadsire = t.battlefield(P1, "Ulamog's Dreadsire");
    let before = t.g.battlefield.clone();
    t.activate(P1, dreadsire, 0, &[]).expect("activate");
    t.resolve_all();
    let eldrazi = new_tokens(&t, P1, &before)[0];
    let angel = t.battlefield(P1, "Serra Angel");
    let angel_copy = token_copy(&mut t, P1, angel)[0];
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bears_copy = token_copy(&mut t, P1, bears)[0];
    let legal = spell_targets(&mut t, P0, "Smother");
    assert!(legal.contains(&Entity::Object(eldrazi)));
    assert!(legal.contains(&Entity::Object(bears_copy)));
    assert!(!legal.contains(&Entity::Object(angel_copy)));
    assert!(!legal.contains(&Entity::Object(dreadsire)));
}

#[test]
fn smother_uses_only_the_printed_mana_cost() {
    cr!("202.3", "202.3e", "115.1");
    ruling!(
        "Smother",
        "A creature's mana value is determined solely by the mana symbols printed in its upper right corner (unless that creature is copying something else; see below). If its mana cost includes {X}, X is considered to be 0."
    );
    supported("Smother");
    supported("Endless One");
    supported("Kavu Titan");
    supported("Mutavault");
    let mut t = TestGame::new(2);
    // Endless One cast with X = 5: a 5/5 whose mana value on the battlefield is 0.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Forest", 5);
    let one = t.hand(P1, "Endless One");
    let one = {
        t.cast(P1, one).x(5).go();
        t.resolve_all();
        t.named_on_battlefield("Endless One")[0]
    };
    assert_eq!(t.pt(one), (5, 5));
    // Kavu Titan cast kicked (for five mana): still mana value 2.
    t.lands(P1, "Forest", 5);
    let titan = t.hand(P1, "Kavu Titan");
    t.cast(P1, titan).kicked(true).go();
    t.resolve_all();
    let titan = t.named_on_battlefield("Kavu Titan")[0];
    assert_eq!(t.pt(titan), (5, 5));
    // An animated Mutavault has no mana cost.
    let vault = t.battlefield(P1, "Mutavault");
    t.lands(P1, "Wastes", 1);
    activate_containing(&mut t, P1, vault, "becomes").expect("animate");
    t.resolve_all();
    assert!(t.obj_now(vault).is(CardType::Creature));
    let giant = t.battlefield(P1, "Hill Giant");
    let legal = spell_targets(&mut t, P0, "Smother");
    assert!(legal.contains(&Entity::Object(one)));
    assert!(legal.contains(&Entity::Object(titan)));
    assert!(legal.contains(&Entity::Object(vault)));
    assert!(!legal.contains(&Entity::Object(giant)));
}

#[test]
fn smother_uses_the_mana_value_of_what_a_creature_is_copying() {
    cr!("707.2", "202.3", "115.1");
    ruling!(
        "Smother",
        "If a creature is copying something else, its mana value is the mana value of whatever it's copying."
    );
    supported("Smother");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P0, "Serra Angel");
    // Clone's own mana value is 4; as a copy it has the copied mana cost.
    let small = clone_of(&mut t, P1, bears);
    let big = clone_of(&mut t, P1, angel);
    assert_eq!(mv(&mut t, small), 2);
    assert_eq!(mv(&mut t, big), 5);
    let legal = spell_targets(&mut t, P0, "Smother");
    assert!(legal.contains(&Entity::Object(small)));
    assert!(!legal.contains(&Entity::Object(big)));
    // Smother destroys the small one.
    t.lands(P0, "Swamp", 2);
    let smother = t.hand(P0, "Smother");
    t.cast(P0, smother).target(small).go();
    t.resolve_all();
    assert!(!t.g.is_live(small));
    assert!(t.on_battlefield(big));
}
