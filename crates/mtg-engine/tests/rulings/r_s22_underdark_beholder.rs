//! Rulings batch S22 — Underdark Beholder ("Underdark Beholder enters the battlefield with
//! ten eyestalk counters on it. If Underdark Beholder would be dealt damage, remove that
//! many eyestalk counters from it instead. If you can't, sacrifice it. Whenever Underdark
//! Beholder attacks, reveal cards from the top of your library until you reveal an
//! instant, sorcery, or enchantment card with converted mana cost less than the number of
//! eyestalk counters on Underdark Beholder. You may cast it without paying its mana cost.
//! Shuffle your library."): the revealed card is cast without paying its mana cost
//! (CR 118.9): no alternative cost, additional costs paid (CR 118.8a, 118.9a).

use crate::r_s01_common::*;
use crate::r_s06_common::damage;
use crate::r_s22_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Underdark Beholder enters the battlefield under P0's control (with its eyestalk
/// counters) and can attack this turn.
fn beholder(t: &mut TestGame) -> ObjectId {
    let b = crate::r_s05_common::enter(t, P0, "Underdark Beholder");
    t.g.objects[b.0 as usize].summoning_sick = false;
    b
}

/// Puts `name` on top of P0's library, over a land (revealed first and passed over), with
/// Underdark Beholder on P0's battlefield.
fn place(t: &mut TestGame, name: &str) -> ObjectId {
    beholder(t);
    stack_library(t, P0, &["Forest", name])[1]
}

fn run(t: &mut TestGame, _card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let b = t.named_on_battlefield("Underdark Beholder")[0];
    t.answer_yes(P0, true);
    answers(t);
    attack_p1_unblocked(t, b);
    t.clear_answers();
}

#[test]
fn underdark_beholder_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9", "118.9a", "118.8a", "601.2b", "608.2g");
    ruling!(
        "Underdark Beholder",
        "If you cast a card “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Tormenting Voice, you must pay those to cast the card."
    );
    supported("Underdark Beholder");
    // Burst Lightning (kicker {4}) may be kicked; Tormenting Voice ("As an additional cost
    // to cast this spell, discard a card.") can't be cast without a card to discard;
    // Cyclonic Rift can't be cast for its overload cost.
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place,
        run,
    });
    // Tormenting Voice is cast by discarding P0's only card, Mountain.
    let mut t = TestGame::new(2);
    let voice = place(&mut t, "Tormenting Voice");
    t.hand(P0, "Mountain");
    run(&mut t, voice, &|_| {});
    assert!(t.in_graveyard(P0, "Mountain"));
    assert!(t.in_graveyard(P0, "Tormenting Voice"));
    assert_eq!(t.hand_size(P0), 2);
    // Underdark Beholder (6/6) dealt its combat damage.
    assert_eq!(t.life(P1), 20 - 6);
}

#[test]
fn underdark_beholder_reveals_a_card_with_mana_value_less_than_its_eyestalk_counters() {
    cr!("614.1c", "122.6", "701.20a", "202.3");
    supported("Underdark Beholder");
    let mut t = TestGame::new(2);
    let b = beholder(&mut t);
    assert_eq!(t.counters(b, "eyestalk"), 10);
    // With two eyestalk counters, Tormenting Voice (mana value 2) is passed over and
    // Burst Lightning (mana value 1) is cast.
    let source = t.battlefield(P1, "Hill Giant");
    damage(&mut t, source, 8, b);
    assert_eq!(t.counters(b, "eyestalk"), 2);
    stack_library(&mut t, P0, &["Tormenting Voice", "Burst Lightning"]);
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Bool(false),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    run(&mut t, b, &|_| {});
    assert!(t.in_graveyard(P0, "Burst Lightning"));
    assert!(!t.in_graveyard(P0, "Tormenting Voice"));
    assert_eq!(t.life(P1), 20 - 6 - 2);
}

#[test]
fn underdark_beholder_damage_removes_eyestalk_counters_or_it_is_sacrificed() {
    cr!("614.1a", "701.21a", "510.2");
    supported("Underdark Beholder");
    // Blocked by Hill Giant (3/3): three eyestalk counters are removed instead of the
    // damage being dealt; Underdark Beholder has no damage marked and survives.
    let mut t = TestGame::new(2);
    let b = beholder(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(b, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, b)]);
    assert!(t.on_battlefield(b));
    assert_eq!(t.counters(b, "eyestalk"), 7);
    assert_eq!(crate::r_s07_common::damage_on(&t, b), 0);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Eight damage with seven counters: it can't remove that many, so it's sacrificed.
    let source = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, source, 8, b);
    assert!(t.in_graveyard(P0, "Underdark Beholder"));
    // Exactly as many damage as counters: they're all removed and it stays.
    let mut t = TestGame::new(2);
    let b = beholder(&mut t);
    let source = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, source, 10, b);
    assert!(t.on_battlefield(b));
    assert_eq!(t.counters(b, "eyestalk"), 0);
}
