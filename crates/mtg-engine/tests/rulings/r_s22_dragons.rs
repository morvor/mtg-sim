//! Rulings batch S22 — Dragons of Tarkir's "As an additional cost to cast this spell, you
//! may reveal a Dragon card from your hand." with "If you revealed a Dragon card or
//! controlled a Dragon as you cast this spell, ...": an optional additional cost
//! (CR 601.2b), or a Dragon controlled as the spell was cast (CR 601.2i). Either one is
//! enough, and both give nothing more.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts Draconic Roar ("Draconic Roar deals 3 damage to target creature. If you
/// revealed a Dragon card or controlled a Dragon as you cast this spell, Draconic Roar
/// deals 3 damage to that creature's controller.") at P1's Hill Giant, with `dragons_in_hand`
/// Dragon cards in hand (revealing one if `reveal`), controlling a Dragon if
/// `control_dragon` (which dies before Draconic Roar resolves if `dragon_dies`). Returns
/// P1's life total afterward.
fn roar(dragons_in_hand: usize, reveal: bool, control_dragon: bool, dragon_dies: bool) -> i32 {
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let dragon = control_dragon.then(|| t.battlefield(P0, "Shivan Dragon"));
    let cards: Vec<ObjectId> = (0..dragons_in_hand)
        .map(|_| t.hand(P0, "Shivan Dragon"))
        .collect();
    t.lands(P0, "Mountain", 2);
    let roar = t.hand(P0, "Draconic Roar");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(reveal));
    if reveal {
        let all: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
        t.answer_choose(P0, &all);
    }
    let spell = t.cast(P0, roar).target(Entity::Object(giant)).go();
    let paid = &t.obj(spell).stack.as_ref().expect("a spell").cast.paid;
    assert_eq!(paid.iter().filter(|p| p.as_str() == "reveal").count(), reveal as usize);
    if let (Some(d), true) = (dragon, dragon_dies) {
        destroy(&mut t, d);
    }
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // The revealed card stays in P0's hand.
    assert_eq!(
        t.g.find_in_zone(mtg_engine::object::Zone::Hand(P0), "Shivan Dragon")
            .len(),
        dragons_in_hand
    );
    t.life(P1)
}

#[test]
fn draconic_roar_revealing_and_controlling_a_dragon_dont_add_up() {
    cr!("601.2b", "601.2i", "608.2c");
    ruling!(
        "Draconic Roar",
        "You can’t reveal more than one Dragon card to multiply the bonus. There is also no additional benefit for both revealing a Dragon card as an additional cost and controlling a Dragon as you cast the spell."
    );
    supported("Draconic Roar");
    supported("Shivan Dragon");
    // Neither: no damage to P1.
    assert_eq!(roar(1, false, false, false), 20);
    // Revealing a Dragon card: 3 damage.
    assert_eq!(roar(1, true, false, false), 17);
    // Controlling a Dragon: 3 damage, even if it's gone by the time Draconic Roar
    // resolves (it was controlled as the spell was cast).
    assert_eq!(roar(0, false, true, false), 17);
    assert_eq!(roar(0, false, true, true), 17);
    // Both: still 3.
    assert_eq!(roar(1, true, true, false), 17);
    // Two Dragon cards in hand: only one is revealed; 3 damage.
    assert_eq!(roar(2, true, false, false), 17);
}
