//! "Choose target opponent/player." as a sentence of its own (CR 115.1a, 601.2c): the
//! player is chosen as the spell or ability is put on the stack, and "that player",
//! "they" and "the chosen player" refer to them afterward.

use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn curious_herd_the_number_of_artifacts_that_player_controls() {
    cr!("115.1a", "601.2c");
    compiles("Curious Herd");
    let mut t = TestGame::new(3);
    t.lands(P0, "Forest", 4);
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P2, "Ornithopter");
    let spell = t.hand(P0, "Curious Herd");
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!(t.g.battlefield.iter().filter(|o| t.g.obj(**o).chars.subtypes.iter().any(|s| s == "Beast")).count(), 2);
}

#[test]
fn haunt_the_network_the_chosen_player_loses_life() {
    cr!("115.1a", "601.2c");
    compiles("Haunt the Network");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Swamp", 2);
    let spell = t.hand(P0, "Haunt the Network");
    t.cast(P0, spell).target(P2).go();
    t.resolve();
    // Two Thopters: the chosen player loses 2 life and you gain 2.
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn the_fall_of_kroog_that_player_and_creatures_they_control() {
    cr!("115.1a", "601.2c");
    compiles("The Fall of Kroog");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let land = t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let mine = t.battlefield(P0, "Llanowar Elves");
    let spell = t.hand(P0, "The Fall of Kroog");
    t.cast(P0, spell).targets(&[Entity::Player(P1), Entity::Object(land)]).go();
    t.resolve();
    assert!(!t.on_battlefield(land));
    assert_eq!(t.life(P1), 17);
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(elves));
    assert!(t.on_battlefield(mine));
}

#[test]
fn courageous_resolve_fateful_hour_three_instructions() {
    // "you can't lose life this turn, you can't lose the game this turn, and your opponents
    // can't win the game this turn": a series of instructions, all under the condition.
    cr!("704.5a", "120.3a");
    compiles("Courageous Resolve");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    t.g.player_mut(P0).life = 5;
    let spell = t.hand(P0, "Courageous Resolve");
    t.cast(P0, spell).go();
    t.resolve_all();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 5, "{}", t.dump_log());
    // At 0 life (set directly) the player still doesn't lose this turn.
    t.g.player_mut(P0).life = 0;
    t.settle();
    assert!(!t.has_lost(P0));
}

#[test]
fn courageous_resolve_above_six_life_does_nothing_more() {
    cr!("120.3a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    t.g.player_mut(P0).life = 6;
    let spell = t.hand(P0, "Courageous Resolve");
    t.cast(P0, spell).go();
    t.resolve_all();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 3);
}
