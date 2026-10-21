//! Loops (CR 608.2c): processes that repeat while a condition holds, and counts over all
//! their passes ("for each nonland card exiled this way").

use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn tokens(t: &TestGame, prefix: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| t.obj_now(**o).chars.name.starts_with(prefix))
        .count()
}

#[test]
fn rally_the_horde_repeats_until_the_last_card_is_a_land() {
    cr!("608.2c");
    supported("Rally the Horde");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let spell = t.hand(P0, "Rally the Horde");
    // From the top: Bears, Forest, Bolt (nonland last: repeat), Giant, Bears, Forest
    // (land last: stop).
    for name in [
        "Forest",
        "Grizzly Bears",
        "Hill Giant",
        "Lightning Bolt",
        "Forest",
        "Grizzly Bears",
    ] {
        t.library_top(P0, name);
    }
    t.cast(P0, spell).go();
    t.resolve();
    // Six cards exiled, four of them nonland.
    assert_eq!(tokens(&t, "Warrior"), 4);
}
