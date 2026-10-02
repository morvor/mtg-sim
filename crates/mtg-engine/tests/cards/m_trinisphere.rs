//! Trinisphere (hand-written, `src/cards/trinisphere.rs`): a minimum total cost of three
//! mana, applied after reductions (CR 601.2f).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn one_mana_spell_costs_three_while_untapped() {
    cr!("601.2f");
    ruling!("Trinisphere", "Finally, apply Trinisphere's effect");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Trinisphere");
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    t.lands(P0, "Mountain", 1);
    let bolt = t.g.current(bolt);
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // All three lands were tapped to pay {2}{R}.
    let untapped = t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .filter(|&l| !t.g.obj(l).tapped)
        .count();
    assert_eq!(untapped, 0);
    // The mana value of the spell is unchanged (it's still 1).
    assert_eq!(card("Lightning Bolt").front().chars.mana_cost.as_ref().unwrap().mana_value(), 1);
}

#[test]
fn no_effect_while_tapped_or_on_expensive_spells() {
    cr!("601.2f");
    let mut t = TestGame::new(2);
    let tri = t.battlefield(P1, "Trinisphere");
    t.g.objects[tri.0 as usize].tapped = true;
    t.g.recompute();
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Untapped: a four-mana spell still costs four.
    t.g.objects[tri.0 as usize].tapped = false;
    t.g.recompute();
    t.lands(P0, "Mountain", 4);
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}
