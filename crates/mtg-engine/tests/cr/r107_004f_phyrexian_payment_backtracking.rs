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
