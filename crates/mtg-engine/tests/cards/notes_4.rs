//! Notes (CR 607.2e): a number noted by one ability and read by a linked one.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn sigardas_splendor_compares_with_the_last_noted_life_total() {
    cr!("607.2e", "614.12");
    ruling!(
        "Sigarda's Splendor",
        "You will note your life total as the second ability resolves, even if you don't draw a card."
    );
    supported("Sigarda's Splendor");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let s = t.hand(P0, "Sigarda's Splendor");
    t.cast(P0, s).go();
    t.resolve();
    // Noted 20 as it entered; now at 18: no card, and 18 is noted.
    t.g.player_mut(P0).life = 18;
    t.advance_to(P0, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // At 18 again next upkeep: 18 >= 18 (the last noted), so a card.
    t.advance_to(P0, Step::Draw);
    t.advance_to(P0, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
