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

#[test]
fn keeper_of_the_flame_condition_only_as_you_activate() {
    cr!("115.1a", "602.2b", "608.2b");
    ruling!(
        "Keeper of the Flame",
        "It is only necessary that the condition be true as you activate the ability."
    );
    ruling!(
        "Keeper of the Flame",
        "A different opposing player may be targeted each time the ability is activated."
    );
    compiles("Keeper of the Flame");
    let mut t = TestGame::new(3);
    let keeper = t.battlefield(P0, "Keeper of the Flame");
    t.lands(P0, "Mountain", 1);
    t.g.player_mut(P0).life = 15;
    t.g.player_mut(P2).life = 10;
    // P2 has less life than P0: not a legal target; P1 is.
    t.activate(P0, keeper, 0, &[Entity::Player(P2)]).expect("activates");
    // The life totals change before it resolves: it still resolves.
    t.g.player_mut(P0).life = 30;
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    assert_eq!(t.life(P2), 10);
}

#[test]
fn keeper_of_the_flame_cant_be_activated_without_an_opponent_with_more_life() {
    cr!("115.1a", "602.2b");
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Keeper of the Flame");
    t.lands(P0, "Mountain", 1);
    t.g.player_mut(P1).life = 20;
    assert!(t.activate(P0, keeper, 0, &[Entity::Player(P1)]).is_err());
}

#[test]
fn keepers_of_the_beasts_and_mind_compare_with_you() {
    cr!("115.1a");
    compiles("Keeper of the Beasts");
    compiles("Keeper of the Mind");
    compiles("Keeper of the Light");
    // Keeper of the Beasts: an opponent who controls more creatures than you.
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Keeper of the Beasts");
    t.lands(P0, "Forest", 1);
    assert!(t.activate(P0, keeper, 0, &[Entity::Player(P1)]).is_err());
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, keeper, 0, &[Entity::Player(P1)]).expect("activates");
    t.resolve_all();
    let beasts = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.subtypes.iter().any(|s| s == "Beast"))
        .count();
    assert_eq!(beasts, 1, "{}", t.dump_log());
    // Keeper of the Mind: at least two more cards in hand than you.
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Keeper of the Mind");
    t.lands(P0, "Island", 1);
    t.hand(P1, "Forest");
    assert!(t.activate(P0, keeper, 0, &[Entity::Player(P1)]).is_err());
    t.hand(P1, "Forest");
    let before = t.hand_size(P0);
    t.activate(P0, keeper, 0, &[Entity::Player(P1)]).expect("activates");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 1);
}

#[test]
fn skull_rend_those_players_each_discard_at_random() {
    cr!("701.9b", "120.3a");
    compiles("Skull Rend");
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 3);
    for p in [P1, P2] {
        t.hand(p, "Forest");
        t.hand(p, "Island");
        t.hand(p, "Plains");
    }
    let mine = t.hand_size(P0);
    let spell = t.hand(P0, "Skull Rend");
    t.cast(P0, spell).go();
    t.resolve_all();
    for p in [P1, P2] {
        assert_eq!(t.life(p), 18);
        assert_eq!(t.hand_size(p), 1, "{}", t.dump_log());
    }
    assert_eq!(t.hand_size(P0), mine);
}

#[test]
fn stuffy_doll_deals_that_much_damage_to_the_chosen_player() {
    cr!("607.2d", "614.12");
    compiles("Stuffy Doll");
    let mut t = TestGame::new(3);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let doll = t.enter(P0, "Stuffy Doll");
    t.resolve_all();
    t.g.obj_mut(doll).summoning_sick = false;
    // "{T}: Stuffy Doll deals 1 damage to itself."
    t.activate(P0, doll, 0, &[]).expect("activates");
    t.resolve_all();
    assert_eq!(t.life(P2), 19, "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn saskia_combat_damage_is_dealt_again_to_the_chosen_player() {
    cr!("607.2d", "510.2");
    compiles("Saskia the Unyielding");
    let mut t = TestGame::new(3);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.enter(P0, "Saskia the Unyielding");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18, "{}", t.dump_log());
}

#[test]
fn cinderheart_giant_damage_to_a_random_creature_an_opponent_controls() {
    cr!("608.2d", "120.3");
    compiles("Cinderheart Giant");
    compiles("Scab-Clan Giant");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Cinderheart Giant");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    // Kill the Giant (7/6) with damage: "When ~ dies, it deals 7 damage to a creature an
    // opponent controls chosen at random."
    t.g.obj_mut(giant).damage = 5;
    t.cast(P0, bolt).target(giant).go();
    t.resolve_all();
    assert!(!t.on_battlefield(theirs), "{}", t.dump_log());
    assert!(t.on_battlefield(mine));
}
