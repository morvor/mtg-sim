//! "Each opponent chooses an artifact, a creature, an enchantment, and a planeswalker from
//! among the nonland permanents they control, then sacrifices the rest." (Ajani, Nacatl
//! Avenger's −4): like Cataclysm's "each player chooses ...", but only opponents choose
//! and sacrifice.

use mtg_engine::ability::{Destination, Effect, Sel};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

const AJANI: &str = "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger";

#[test]
fn only_opponents_choose_and_sacrifice_the_rest() {
    cr!("101.4", "101.4c");
    let c = card(AJANI);
    assert!(
        c.unsupported_text().is_empty(),
        "unsupported text: {:?}",
        c.unsupported_text()
    );
    let mut t = TestGame::new(2);
    // P0's Ajani, Nacatl Avenger (put onto the battlefield transformed), with enough
    // loyalty for −4, and two creatures of P0's own.
    let card = t.graveyard(P0, AJANI);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(card)]];
    let mut to = Destination::battlefield();
    to.transformed = true;
    t.g.exec(
        &Effect::Move {
            what: Sel::Target(0),
            to,
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.resolve_all();
    let ajani = t.g.current(card);
    assert_eq!(t.obj(ajani).chars.name, "Ajani, Nacatl Avenger");
    t.g.add_counters(Entity::Object(ajani), counters::LOYALTY, 2, None);
    let my_bears = t.battlefield(P0, "Grizzly Bears");
    let my_giant = t.battlefield(P0, "Hill Giant");
    // P1: an artifact creature, two more creatures, an enchantment, and a land.
    let thopter = t.battlefield(P1, "Ornithopter");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let island = t.battlefield(P1, "Island");
    // P1 keeps Ornithopter as their artifact, Hill Giant as their creature, and Glorious
    // Anthem as their enchantment (they have no planeswalker).
    t.answer_choose(P1, &[Entity::Object(thopter)]);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.answer_choose(P1, &[Entity::Object(anthem)]);
    let from = t.asked().len();
    t.activate(P0, ajani, 2, &[]).expect("can't activate −4");
    t.resolve_all();
    // Only P1 chose.
    assert!(t.asked()[from..].iter().all(|(p, _)| *p == P1));
    for kept in [thopter, giant, anthem, island] {
        assert!(t.on_battlefield(kept));
    }
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // P0 sacrificed nothing.
    assert!(t.on_battlefield(my_bears) && t.on_battlefield(my_giant));
    assert!(t.on_battlefield(ajani));
    assert_eq!(t.counters(ajani, counters::LOYALTY), 1);
}
