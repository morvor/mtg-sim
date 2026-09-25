//! Attack and block restrictions with conditions: "can't attack unless defending player
//! ..." checked against the player each creature would attack (CR 508.1c, 508.5), board
//! conditions, and evasion against "and/or" lists of creatures.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
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
fn defending_player_is_the_player_it_would_attack() {
    cr!("508.1c", "508.5");
    compiles("Sea Monster");
    let mut t = TestGame::new(3);
    let monster = t.battlefield(P0, "Sea Monster");
    t.battlefield(P1, "Island");
    let jace = t.battlefield(P2, "Jace Beleren");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.can_attack(monster));
    assert!(t.can_attack_target(monster, Entity::Player(P1)));
    assert!(!t.can_attack_target(monster, Entity::Player(P2)));
    assert!(!t.can_attack_target(monster, Entity::Object(jace)));
}

#[test]
fn can_attack_only_the_monarch_or_their_planeswalkers() {
    cr!("508.1c", "724.1");
    ruling!(
        "Crown-Hunter Hireling",
        "can attack only the monarch or a planeswalker controlled by the monarch"
    );
    compiles("Crown-Hunter Hireling");
    let mut t = TestGame::new(3);
    let ogre = t.battlefield(P0, "Crown-Hunter Hireling");
    let jace = t.battlefield(P2, "Jace Beleren");
    t.g.monarch = Some(P2);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!t.can_attack_target(ogre, Entity::Player(P1)));
    assert!(t.can_attack_target(ogre, Entity::Player(P2)));
    assert!(t.can_attack_target(ogre, Entity::Object(jace)));
}

#[test]
fn more_creatures_than_the_defending_or_attacking_player() {
    cr!("508.1c", "509.1b");
    compiles("Goblin Goon");
    let mut t = TestGame::new(2);
    let goon = t.battlefield(P0, "Goblin Goon");
    t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    // One creature each: not more.
    assert!(!t.can_attack_target(goon, Entity::Player(P1)));
    t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert!(t.can_attack_target(goon, Entity::Player(P1)));
    // Blocking: compared with the attacking player's creatures.
    let mut t = TestGame::new(2);
    let goon = t.battlefield(P1, "Goblin Goon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(goon, bears)]);
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(bears));
}

#[test]
fn you_control_a_one_one_creature() {
    cr!("508.1c");
    ruling!(
        "Lovestruck Beast // Heart's Desire",
        "You don't have to attack with a 1/1 creature for Lovestruck Beast to be able to attack."
    );
    compiles("Lovestruck Beast // Heart's Desire");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Lovestruck Beast // Heart's Desire");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!t.can_attack(beast));
    t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert!(t.can_attack(beast));
}

#[test]
fn board_conditions() {
    cr!("508.1c", "509.1b");
    ruling!("Glacial Crasher", "It doesn't matter who controls the Mountain.");
    compiles("Glacial Crasher");
    compiles("Lupine Prototype");
    let mut t = TestGame::new(2);
    let crasher = t.battlefield(P0, "Glacial Crasher");
    let proto = t.battlefield(P0, "Lupine Prototype");
    t.hand(P0, "Island");
    t.hand(P1, "Island");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!t.can_attack(crasher));
    assert!(!t.can_attack(proto));
    t.battlefield(P1, "Mountain");
    t.g.recompute();
    assert!(t.can_attack(crasher));
    // "unless a player has no cards in hand"
    let card = t.g.player(P1).hand[0];
    t.g.move_object(
        card,
        mtg_engine::object::Zone::Graveyard(P1),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.recompute();
    assert!(t.can_attack(proto));
}

#[test]
fn cant_be_blocked_except_by_and_or_lists() {
    cr!("509.1b");
    compiles("Elven Riders");
    compiles("Sacred Knight");
    // "except by Walls and/or creatures with flying"
    for (blocker, damage) in [("Grizzly Bears", 3), ("Wall of Stone", 0), ("Suntail Hawk", 0)] {
        let mut t = TestGame::new(2);
        let riders = t.battlefield(P0, "Elven Riders");
        let b = t.battlefield(P1, blocker);
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(riders, Entity::Player(P1))], &[(b, riders)]);
        assert_eq!(t.life(P1), 20 - damage, "blocked by {blocker}");
    }
    // "can't be blocked by black and/or red creatures"
    for (blocker, damage) in [("Goblin Piker", 3), ("Grizzly Bears", 0)] {
        let mut t = TestGame::new(2);
        let knight = t.battlefield(P0, "Sacred Knight");
        let b = t.battlefield(P1, blocker);
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(knight, Entity::Player(P1))], &[(b, knight)]);
        assert_eq!(t.life(P1), 20 - damage, "blocked by {blocker}");
    }
}

#[test]
fn creatures_with_power_or_toughness_one_or_less_cant_be_blocked() {
    cr!("509.1b");
    compiles("Tetsuko Umezawa, Fugitive");
    for (attacker, damage) in [("Llanowar Elves", 1), ("Grizzly Bears", 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Tetsuko Umezawa, Fugitive");
        let a = t.battlefield(P0, attacker);
        let wall = t.battlefield(P1, "Wall of Stone");
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(a, Entity::Player(P1))], &[(wall, a)]);
        assert_eq!(t.life(P1), 20 - damage, "{attacker}");
    }
}
