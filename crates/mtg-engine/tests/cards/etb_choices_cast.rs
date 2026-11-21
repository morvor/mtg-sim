//! Abilities of a permanent that refer to how it was cast ("if it was kicked", "if you
//! cast it from your hand") see the permanent's cast info (CR 607.2i, 603.4).

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn soldiers(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Soldier"))
        .count()
}

#[test]
fn kicked_etb_trigger() {
    cr!("607.2i", "603.4", "702.33d");
    assert_supported("Phyrexian Warhorse");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.lands(P0, "Plains", 1);
    let h = t.hand(P0, "Phyrexian Warhorse");
    t.cast(P0, h).kicked(true).go();
    t.resolve_all();
    assert!(t.on_battlefield(h));
    assert_eq!(soldiers(&t), 1);
    // Not kicked: no token.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let h = t.hand(P0, "Phyrexian Warhorse");
    t.cast(P0, h).kicked(false).go();
    t.resolve_all();
    assert_eq!(soldiers(&t), 0);
}

#[test]
fn cast_from_hand_etb_trigger() {
    cr!("603.4", "601.2");
    assert_supported("Feasting Troll King");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let k = t.hand(P0, "Feasting Troll King");
    t.cast(P0, k).go();
    t.resolve_all();
    let foods = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.chars.has_subtype("Food"))
            .count()
    };
    assert_eq!(foods(&t), 3);
    // Put onto the battlefield without being cast: no Food.
    let mut t = TestGame::new(2);
    t.enter(P0, "Feasting Troll King");
    t.resolve_all();
    assert_eq!(foods(&t), 0);
}
