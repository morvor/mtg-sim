//! CR 107.4f, 119.4, 601.2h: paying a cost with hybrid and Phyrexian symbols from a mana
//! pool. A way of paying that would need more life than the player has is abandoned, and
//! the life it tentatively counted doesn't stay counted against the next way tried.

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn hybrid_and_two_phyrexian_symbols_paid_with_the_one_payment_that_fits_the_life_total() {
    cr!("107.4f", "119.4", "601.2h");
    let mut t = TestGame::new(2);
    t.g.players[0].life = 3;
    // {B/G}{G/P}{G/P} with {G}{B} in the pool and 3 life: paying the hybrid with {G}
    // leaves both Phyrexian symbols to life (4, too much); paying it with {B} lets {G} pay
    // one Phyrexian symbol and 2 life the other.
    t.g.players[0].mana_pool.add_type(ManaType::G, 1);
    t.g.players[0].mana_pool.add_type(ManaType::B, 1);
    let def = CB::new("Sap of Life")
        .instant()
        .cost("{B/G}{G/P}{G/P}")
        .spell(Body::effect(gain(1)))
        .build();
    let s = t.custom(P0, def, Zone::Hand(P0));
    assert!(t.cast(P0, s).try_go().is_ok());
    assert_eq!(t.life(P0), 1);
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
}

#[test]
fn an_abandoned_way_of_paying_doesnt_count_phyrexian_symbols_paid_with_life() {
    cr!("107.4f", "601.2h", "702.150a");
    let mut t = TestGame::new(2);
    t.g.players[0].life = 3;
    // The same cost on a compleated planeswalker with five loyalty: the payment that fits
    // pays one Phyrexian symbol with life, so it enters with 5 - 2 loyalty counters (the
    // abandoned way, which paid one with life before running out, doesn't count).
    t.g.players[0].mana_pool.add_type(ManaType::G, 1);
    t.g.players[0].mana_pool.add_type(ManaType::B, 1);
    let mut def = crate::r703_common::oracle_card(
        "Test Compleated Walker",
        "Legendary Planeswalker — Test",
        "{B/G}{G/P}{G/P}",
        None,
        "Compleated",
    );
    def.faces[0].chars.loyalty = Some(5);
    let w = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, w).go();
    assert_eq!(t.life(P0), 1);
    t.resolve_all();
    assert!(t.on_battlefield(w));
    assert_eq!(t.counters(w, "loyalty"), 3);
}
