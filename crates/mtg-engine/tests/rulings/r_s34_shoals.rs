//! Rulings batch S34 — the Shoals' alternative cost, "exile a [color] card with mana value
//! X from your hand rather than pay this spell's mana cost" (CR 118.9, 107.3a): X is
//! announced as the spell is cast (CR 601.2b) and a card with that mana value is exiled
//! (CR 601.2h). The spell's mana value on the stack includes that X although no mana was
//! spent on it (CR 202.3e).

use crate::r_s01_common::*;
use crate::r_s08_common::mana_value;
use crate::r_s34_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn nourishing_shoal_cast_by_exiling_a_card_has_x_in_its_mana_value() {
    cr!("107.3a", "118.9", "202.3e", "601.2b", "601.2h");
    ruling!(
        "Nourishing Shoal",
        "If a spell has {X} in its mana cost, include the value chosen for that X when determining the mana value of that spell, even if it was cast for an alternative cost and no mana was spent on X."
    );
    supported("Nourishing Shoal");
    // Nourishing Shoal ({X}{G}{G}): "You may exile a green card with mana value X from
    // your hand rather than pay this spell's mana cost. You gain X life." P0 has no lands:
    // exiling Thragtusk (mana value 5) makes X 5. P1's Kaervek sees mana value 7.
    let mut t = TestGame::new(2);
    kaervek(&mut t);
    let shoal = t.hand(P0, "Nourishing Shoal");
    let tusk = t.hand(P0, "Thragtusk");
    // Lightning Bolt isn't green: it can't pay the cost.
    t.hand(P0, "Lightning Bolt");
    let alt = alternative(&mut t, shoal);
    aim_kaervek_at_p0(&mut t);
    t.answer_choose(P0, &[Entity::Object(tusk)]);
    let spell = t.cast(P0, shoal).method(alt).x(5).go();
    assert!(t.in_exile("Thragtusk"));
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(mana_value(&t, spell), 7);
    t.resolve();
    assert_eq!(t.life(P0), 13);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn nourishing_shoal_s_x_must_match_a_green_card_in_hand() {
    cr!("107.3a", "118.9", "601.2b", "601.2h");
    supported("Nourishing Shoal");
    // Without another green card in hand, the alternative cost can't be paid; with only
    // Thragtusk (mana value 5), X can't be announced as 3: X is 5.
    let mut t = TestGame::new(2);
    let shoal = t.hand(P0, "Nourishing Shoal");
    t.hand(P0, "Lightning Bolt");
    let methods = crate::r_s07_common::cast_methods(&mut t, P0, shoal);
    assert!(methods
        .iter()
        .all(|m| !matches!(m, mtg_engine::object::CastMethod::Alternative(_))));
    t.hand(P0, "Thragtusk");
    let alt = alternative(&mut t, shoal);
    let spell = t.cast(P0, shoal).method(alt).x(3).go();
    assert!(t.in_exile("Thragtusk"));
    assert_eq!(mana_value(&t, spell), 7);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

/// P1 casts Endless One with X = 3 (a spell with mana value 3); P0 responds with
/// Disrupting Shoal, exiling `exiled` with X = `x`. Returns (Endless One, the Shoal).
fn shoal_vs_endless_one(t: &mut TestGame, exiled: &str, x: i64) -> (ObjectId, ObjectId) {
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 3);
    let one = t.hand(P1, "Endless One");
    let one = t.cast(P1, one).x(3).go();
    assert_eq!(mana_value(t, one), 3);
    let shoal = t.hand(P0, "Disrupting Shoal");
    let card = t.hand(P0, exiled);
    let alt = alternative(t, shoal);
    t.answer_choose(P0, &[Entity::Object(card)]);
    let spell = t.cast(P0, shoal).method(alt).x(x).target(one).go();
    assert!(t.in_exile(exiled));
    (one, spell)
}

#[test]
fn disrupting_shoal_counters_a_spell_whose_mana_value_with_x_is_its_x() {
    cr!("107.3a", "202.3e", "608.2c", "701.6a");
    ruling!(
        "Disrupting Shoal",
        "If a spell has {X} in its mana cost, include the value chosen for that X when determining the mana value of that spell, even if it was cast for an alternative cost and no mana was spent on X."
    );
    supported("Disrupting Shoal");
    // Disrupting Shoal ({X}{U}{U}): "... Counter target spell if its mana value is X."
    // Exiling Cancel ({1}{U}{U}) makes X 3: the Shoal's own mana value is 5, and Endless
    // One cast with X = 3 has mana value 3, so it's countered.
    let mut t = TestGame::new(2);
    let (one, shoal) = shoal_vs_endless_one(&mut t, "Cancel", 3);
    assert_eq!(mana_value(&t, shoal), 5);
    t.resolve();
    assert!(!t.g.stack.contains(&one));
    assert!(t.in_graveyard(P1, "Endless One"));
}

#[test]
fn disrupting_shoal_does_nothing_unless_the_spell_s_mana_value_is_x() {
    cr!("107.3a", "608.2c", "115.1");
    ruling!(
        "Disrupting Shoal",
        "Disrupting Shoal can target any spell, but does nothing unless that spell's mana value is X."
    );
    supported("Disrupting Shoal");
    // Exiling Opt ({U}) makes X 1: Endless One (mana value 3) can be targeted, but it
    // isn't countered.
    let mut t = TestGame::new(2);
    let (one, shoal) = shoal_vs_endless_one(&mut t, "Opt", 1);
    assert_eq!(mana_value(&t, shoal), 3);
    t.resolve();
    assert!(t.g.stack.contains(&one));
    t.resolve_all();
    let one = t.g.current(one);
    assert!(t.on_battlefield(one));
    assert_eq!(t.pt(one), (3, 3));
}

#[test]
fn disrupting_shoal_can_exile_a_card_with_x_in_its_mana_cost_as_mana_value_without_x() {
    cr!("107.3g", "202.3e", "107.3a");
    supported("Disrupting Shoal");
    supported("Blue Sun's Zenith");
    // Blue Sun's Zenith ({X}{U}{U}{U}) has mana value 3 in hand (X is 0 there), so it can
    // be exiled for X = 3, countering Endless One cast with X = 3.
    let mut t = TestGame::new(2);
    let (one, _) = shoal_vs_endless_one(&mut t, "Blue Sun's Zenith", 3);
    t.resolve();
    assert!(!t.g.stack.contains(&one));
    assert!(t.in_graveyard(P1, "Endless One"));
}
