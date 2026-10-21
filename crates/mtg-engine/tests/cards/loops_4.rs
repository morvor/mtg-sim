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

fn tyrant(t: &mut TestGame) {
    supported("Tyrant of Discord");
    t.lands(P0, "Mountain", 7);
    let tyrant = t.hand(P0, "Tyrant of Discord");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, tyrant).go();
    t.resolve_all();
}

#[test]
fn tyrant_of_discord_keeps_sacrificing_until_a_land() {
    cr!("608.2c");
    ruling!(
        "Tyrant of Discord",
        "the opponent will keep sacrificing permanents until they sacrifice a land"
    );
    // Only nonland permanents: all of them go.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Ornithopter");
    tyrant(&mut t);
    assert!(![a, b, c].iter().any(|o| t.on_battlefield(*o)));
    // Only lands: the first one sacrificed stops it.
    let mut t = TestGame::new(2);
    let lands = t.lands(P1, "Forest", 3);
    tyrant(&mut t);
    assert_eq!(lands.iter().filter(|o| t.on_battlefield(**o)).count(), 2);
}
