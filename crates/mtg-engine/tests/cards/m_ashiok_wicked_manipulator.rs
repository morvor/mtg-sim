//! Ashiok, Wicked Manipulator (hand-written, `src/cards/ashiok_wicked_manipulator.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn paying_life_exiles_cards_from_your_library_instead() {
    cr!("119.4", "614.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashiok, Wicked Manipulator");
    assert!(t.g.pay_life(P0, 3));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.library_size(P0), 27);
}

#[test]
fn with_too_few_cards_you_pay_the_life() {
    cr!("119.4");
    ruling!("Ashiok, Wicked Manipulator", "you'll just pay life as normal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashiok, Wicked Manipulator");
    let lib = t.g.players[0].library.clone();
    t.g.players[0].library = lib[..2].to_vec();
    assert!(t.g.pay_life(P0, 3));
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.library_size(P0), 2);
}

#[test]
fn a_life_cost_is_paid_by_exiling() {
    cr!("119.4", "118.3");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashiok, Wicked Manipulator");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Swamp");
    let s = t.hand(P0, "Snuff Out");
    // Snuff Out: "If you control a Swamp, you may pay 4 life rather than pay this spell's
    // mana cost."
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    let alt = t
        .g
        .cast_options(P0, s)
        .into_iter()
        .map(|o| o.method)
        .find(|m| matches!(m, mtg_engine::object::CastMethod::Alternative(_)))
        .expect("Snuff Out's alternative cost");
    t.cast(P0, s).method(alt).target(bear).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.library_size(P0), 26);
}
