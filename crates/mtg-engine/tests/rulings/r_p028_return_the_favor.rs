//! Rulings batch P028 — Return the Favor ("Spree. + {1} — Copy target instant spell,
//! sorcery spell, activated ability, or triggered ability. You may choose new targets for
//! the copy. + {1} — Change the target of target spell or ability with a single
//! target."): copies keep X, modes, division and the number of targets (CR 707.10); "with
//! a single target" counts the times a target was chosen (CR 115.9a).

use crate::r_p028_common::*;
use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::supported;
use crate::r_s07_common::chosen_modes;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn return_the_favor_s_copy_keeps_x_of_a_spell_and_of_an_ability() {
    cr!("707.10", "107.3", "702.172a");
    ruling!(
        "Return the Favor",
        "If a spell that's copied has an X whose value was determined as it was cast, the copy will have the same value of X. The same is true for an ability that's copied and an X whose value was determined as the ability was activated."
    );
    supported("Return the Favor");
    // Blaze with X = 2 at P1.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Blaze");
    let blaze = t.cast(P0, card).x(2).target(P1).go();
    (RETURN_THE_FAVOR.start)(&mut t, blaze);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(x_of(&t, copies[0]), Some(2));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Crypt Rats ("{X}: This creature deals X damage to each creature and each player.
    // Spend only black mana on X.") with X = 1: the copy deals 1 too.
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Crypt Rats");
    t.lands(P0, "Swamp", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, rats, 0, &[]).unwrap();
    let ability = *t.g.stack.last().unwrap();
    (RETURN_THE_FAVOR.start)(&mut t, ability);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, ability);
    assert_eq!(x_of(&t, copy), Some(1));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (18, 18));
}

#[test]
fn return_the_favor_s_copy_keeps_the_modes() {
    cr!("707.10", "700.2");
    ruling!(
        "Return the Favor",
        "If the copied spell or ability is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode or modes. You can't choose different ones."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Izzet Charm");
    let card = t.hand(P0, "Izzet Charm");
    let charm = t.cast(P0, card).modes(&[1]).target(giant).go();
    (RETURN_THE_FAVOR.start)(&mut t, charm);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(chosen_modes(&t, copies[0]), vec![1]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

#[test]
fn return_the_favor_s_copy_keeps_the_division_and_number_of_targets() {
    cr!("707.10", "707.10c", "601.2d");
    ruling!(
        "Return the Favor",
        "If the copied spell or ability divides damage or distributes counters among a number of targets, the division and number of targets can't be changed. If you choose new targets, you must choose the same number of targets."
    );
    // Electrolyze: 1 damage to the Bears and 1 to P1; the copy gets the Giant instead of
    // the Bears and still has two targets, 1 damage each.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    lands_for_cost(&mut t, P0, "Electrolyze");
    let card = t.hand(P0, "Electrolyze");
    let e = t
        .cast(P0, card)
        .targets(&[obj(bears), Entity::Player(P1)])
        .go();
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Return the Favor");
    t.cast(P0, card).modes(&[0]).target(e).go();
    change_copy_targets(&mut t, P0, &[Some(obj(giant)), None]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(
        targets_of(&t, copies[0]),
        vec![obj(giant), Entity::Player(P1)]
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.obj_now(giant).damage, 1);
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn return_the_favor_cant_redirect_a_spell_that_had_several_targets() {
    cr!("115.9a", "115.7");
    ruling!(
        "Return the Favor",
        "If a spell or ability targets multiple things, you can't target it with Return the Favor's second mode, even if all but one of those targets have become illegal."
    );
    // Electrolyze at the Bears and P1; the Bears die: the spell still had two targets.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    lands_for_cost(&mut t, P0, "Electrolyze");
    let card = t.hand(P0, "Electrolyze");
    let e = t
        .cast(P0, card)
        .targets(&[obj(bears), Entity::Player(P1)])
        .go();
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.settle();
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Return the Favor");
    // The second mode can't be chosen (the answer is rejected; the default is the first
    // mode, copying it).
    if let Ok(id) = t.cast(P0, card).modes(&[1]).target(e).try_go() {
        assert_ne!(chosen_modes(&t, id), vec![1]);
    }
    t.resolve_all();
    // A spell with a single target can be.
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    let card = t.hand(P0, "Return the Favor");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    let id = t.cast(P0, card).modes(&[1]).target(bolt).go();
    assert_eq!(chosen_modes(&t, id), vec![1]);
}
