//! Choosing objects as an instruction (CR 608.2d; not targeted, CR 115.10), by you or by
//! other players in APNAP order (CR 101.4), and referring to the chosen objects and to
//! the rest afterward.

use mtg_engine::decision::Decision;
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

/// The players asked to choose objects since `from`, in order.
fn choosers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect()
}

#[test]
fn duneblast_choose_up_to_one_creature_destroy_the_rest() {
    cr!("608.2d", "115.10");
    ruling!(
        "Duneblast",
        "If you don't choose a creature, then all creatures will be destroyed."
    );
    compiles("Duneblast");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    let hexproof = t.battlefield(P1, "Gladecover Scout");
    let spell = t.hand(P0, "Duneblast");
    t.answer_choose(P0, &[Entity::Object(hexproof)]);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(hexproof), "the choice doesn't target");
    // None chosen: all of them.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    let spell = t.hand(P0, "Duneblast");
    t.answer_choose(P0, &[]);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn divine_reckoning_each_player_keeps_a_creature() {
    cr!("608.2d", "101.4");
    ruling!(
        "Divine Reckoning",
        "Starting with the player whose turn it is, each player chooses a creature in turn order."
    );
    compiles("Divine Reckoning");
    let mut t = TestGame::new(3);
    t.lands(P0, "Plains", 4);
    let a0 = t.battlefield(P0, "Grizzly Bears");
    let b0 = t.battlefield(P0, "Llanowar Elves");
    let a1 = t.battlefield(P1, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Llanowar Elves");
    let a2 = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(b0)]);
    t.answer_choose(P1, &[Entity::Object(a1)]);
    let spell = t.hand(P0, "Divine Reckoning");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(choosers(&t, from), vec![P0, P1, P2]);
    assert!(!t.on_battlefield(a0));
    assert!(t.on_battlefield(b0));
    assert!(t.on_battlefield(a1));
    assert!(!t.on_battlefield(b1));
    assert!(t.on_battlefield(a2));
}

#[test]
fn dredge_the_mire_each_opponent_chooses_a_creature_card_in_their_graveyard() {
    cr!("608.2d", "101.4");
    compiles("Dredge the Mire");
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 4);
    let b1 = t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Llanowar Elves");
    t.graveyard(P2, "Forest");
    t.graveyard(P0, "Ornithopter");
    t.answer_choose(P1, &[Entity::Object(b1)]);
    let spell = t.hand(P0, "Dredge the Mire");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve();
    // P2 has no creature card: only P1 chooses.
    assert_eq!(choosers(&t, from), vec![P1]);
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj_now(bears[0]).controller, P0);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(t.in_graveyard(P0, "Ornithopter"));
}

#[test]
fn scrounge_target_opponent_chooses_an_artifact_card_in_their_graveyard() {
    cr!("608.2d");
    compiles("Scrounge");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.graveyard(P1, "Ornithopter");
    t.graveyard(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Scrounge");
    let from = t.asked().len();
    t.cast(P0, spell).target(P1).go();
    t.resolve();
    assert_eq!(choosers(&t, from), vec![P1]);
    let thopter = t.named_on_battlefield("Ornithopter");
    assert_eq!(thopter.len(), 1);
    assert_eq!(t.obj_now(thopter[0]).controller, P0);
}

#[test]
fn goblin_war_cry_other_creatures_than_the_chosen_one_cant_block() {
    cr!("608.2d", "611.2c");
    compiles("Goblin War Cry");
    for (blocker_is_kept, life) in [(false, 17), (true, 20)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 3);
        let attacker = t.battlefield(P0, "Hill Giant");
        let kept = t.battlefield(P1, "Grizzly Bears");
        let other = t.battlefield(P1, "Llanowar Elves");
        t.answer_choose(P1, &[Entity::Object(kept)]);
        let spell = t.hand(P0, "Goblin War Cry");
        t.cast(P0, spell).target(P1).go();
        t.resolve();
        // Only the chosen creature can block.
        let blocker = if blocker_is_kept { kept } else { other };
        t.attack(&[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
        assert_eq!(t.life(P1), life);
    }
}

#[test]
fn thran_tome_target_opponent_chooses_one_of_those_cards() {
    cr!("608.2d");
    ruling!(
        "Thran Tome",
        "you draw two cards regardless of how many were actually revealed"
    );
    compiles("Thran Tome");
    let mut t = TestGame::new(2);
    let tome = t.battlefield(P0, "Thran Tome");
    t.lands(P0, "Plains", 5);
    let a = t.library_top(P0, "Grizzly Bears");
    let b = t.library_top(P0, "Forest");
    let c = t.library_top(P0, "Ornithopter");
    t.answer_choose(P1, &[Entity::Object(b)]);
    let hand = t.hand_size(P0);
    t.activate(P0, tome, 0, &[Entity::Player(P1)]).expect("activates");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert_eq!(t.hand_size(P0), hand + 2);
    let _ = (a, c);
}

#[test]
fn urzas_sylex_each_player_chooses_six_lands() {
    cr!("608.2d", "101.4");
    compiles("Urza's Sylex");
    let mut t = TestGame::new(2);
    let sylex = t.battlefield(P0, "Urza's Sylex");
    t.lands(P0, "Plains", 7);
    t.lands(P1, "Forest", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, sylex, 0, &[]).expect("activates");
    t.resolve_all();
    // P0 keeps six of seven Plains; P1 keeps three Forests; the creature is destroyed.
    assert_eq!(t.g.permanents_controlled_by(P0).len(), 6);
    assert_eq!(t.g.permanents_controlled_by(P1).len(), 3);
    assert!(!t.on_battlefield(bears));
}
