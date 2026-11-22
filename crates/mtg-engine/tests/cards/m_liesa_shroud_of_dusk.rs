//! Liesa, Shroud of Dusk (hand-written, `src/cards/liesa_shroud_of_dusk.rs`).

use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_commander_tax_is_paid_in_life() {
    cr!("903.8");
    ruling!("Liesa, Shroud of Dusk", "You can't choose to pay mana to cover the \"commander tax,\"");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    );
    let liesa = t.command(P0, "Liesa, Shroud of Dusk");
    t.g.objects[liesa.0 as usize].is_commander = true;
    t.g.players[0].commander_names.push("Liesa, Shroud of Dusk".into());
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 3);
    t.cast(P0, liesa).go();
    t.resolve_all();
    let life = t.life(P0);
    // It returns to the command zone; cast again for {2}{W}{W}{B} and 2 life.
    t.answer_yes(P0, true);
    t.g.destroy(t.g.current(liesa), None);
    t.settle();
    assert_eq!(t.zone(liesa), Zone::Command);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 3);
    let liesa = t.g.current(liesa);
    t.cast(P0, liesa).go();
    // Liesa's "Whenever a player casts a spell, they lose 2 life" isn't on the battlefield.
    assert_eq!(t.life(P0), life - 2);
    t.resolve_all();
    assert!(t.on_battlefield(liesa));
}
