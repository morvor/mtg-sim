//! The dig grammar (`oracle/patterns/dig_grammar.rs`): who controls the cards a player
//! puts onto the battlefield, "that creature" after a dig in a triggered ability, and a
//! card exiled face down that may be cast if it's a creature spell.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Replaces `p`'s library with the named cards; the last one named ends up on top.
fn library(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    t.g.players[p.0 as usize].library.clear();
    names.iter().map(|n| t.library_top(p, n)).collect()
}

#[test]
fn polymorph_the_creatures_controller_puts_the_card_onto_the_battlefield() {
    cr!("110.2a", "701.20a");
    assert_supported("Polymorph");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Top first: Forest, Serra Angel, Island.
    library(&mut t, P1, &["Island", "Serra Angel", "Forest"]);
    let s = t.hand(P0, "Polymorph");
    t.cast(P0, s).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let angel = t.named_on_battlefield("Serra Angel");
    assert_eq!(angel.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(angel[0]).controller, P1);
    // The Forest was shuffled back in; the Island wasn't revealed.
    assert_eq!(t.library_size(P1), 2);
}

#[test]
fn clear_the_land_each_player_puts_their_own_lands_onto_the_battlefield() {
    cr!("110.2a", "701.20a");
    assert_supported("Clear the Land");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    library(&mut t, P1, &["Island", "Swamp", "Shock"]);
    library(&mut t, P0, &["Plains", "Lightning Bolt"]);
    let s = t.hand(P0, "Clear the Land");
    t.cast(P0, s).go();
    t.resolve();
    let swamp = t.named_on_battlefield("Swamp");
    let plains = t.named_on_battlefield("Plains");
    assert_eq!((swamp.len(), plains.len()), (1, 1), "{}", t.dump_log());
    assert_eq!(t.obj_now(swamp[0]).controller, P1);
    assert_eq!(t.obj_now(plains[0]).controller, P0);
    assert!(t.obj_now(swamp[0]).tapped);
    // The nonland cards were exiled.
    assert!(t.in_exile("Shock") && t.in_exile("Lightning Bolt"));
    assert_eq!(t.library_size(P0) + t.library_size(P1), 0);
}

#[test]
fn lost_in_the_woods_removes_the_attacking_creature_not_the_revealed_card() {
    cr!("506.4", "701.20a");
    assert_supported("Lost in the Woods");
    for (top, damage) in [("Forest", 0), ("Island", 2)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lost in the Woods");
        let other = if top == "Forest" { "Island" } else { "Forest" };
        let ids = library(&mut t, P0, &[other, top]);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P1, Step::BeginningOfCombat);
        t.attack(&[(bears, Entity::Player(P0))], &[]);
        assert_eq!(t.life(P0), 20 - damage, "{top}: {}", t.dump_log());
        // Then the revealed card went to the bottom.
        assert_eq!(t.g.player(P0).library[0], ids[1], "{top}");
    }
}

#[test]
fn vivien_champion_of_the_wilds_may_cast_the_face_down_card_only_if_its_a_creature() {
    cr!("406.3a", "601.3");
    assert_supported("Vivien, Champion of the Wilds");
    for (pick, castable) in [("Grizzly Bears", true), ("Shock", false)] {
        let mut t = TestGame::new(2);
        let vivien = t.battlefield(P0, "Vivien, Champion of the Wilds");
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Mountain", 1);
        let ids = library(&mut t, P0, &["Island", "Shock", "Grizzly Bears", "Forest"]);
        let chosen = if pick == "Shock" { ids[1] } else { ids[2] };
        t.answer_choose(P0, &[Entity::Object(chosen)]);
        t.activate(P0, vivien, 1, &[]).unwrap();
        t.resolve();
        let exiled: Vec<ObjectId> = t.g.exile.clone();
        assert_eq!(exiled.len(), 1, "{}", t.dump_log());
        assert!(t.obj_now(exiled[0]).face_down);
        // Shock needs a target.
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let r = t.cast(P0, exiled[0]).try_go();
        assert_eq!(r.is_ok(), castable, "{pick}: {r:?}");
    }
}

#[test]
fn unexpected_results_returns_itself_only_with_the_land_put_onto_the_battlefield() {
    cr!("701.20a", "608.2c");
    assert_supported("Unexpected Results");
    for put in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        t.lands(P0, "Island", 2);
        // A one-card library: the shuffle can't move it.
        library(&mut t, P0, &["Plains"]);
        let s = t.hand(P0, "Unexpected Results");
        t.answer_yes(P0, put);
        t.cast(P0, s).go();
        t.resolve();
        assert_eq!(t.named_on_battlefield("Plains").len(), usize::from(put), "{put}");
        assert_eq!(t.in_hand(P0, "Unexpected Results"), put, "{put}: {}", t.dump_log());
        assert_eq!(t.in_graveyard(P0, "Unexpected Results"), !put, "{put}");
    }
    // A nonland card may be cast; the spell then goes to the graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 2);
    library(&mut t, P0, &["Grizzly Bears"]);
    let s = t.hand(P0, "Unexpected Results");
    t.answer_yes(P0, true);
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Unexpected Results"));
}

#[test]
fn murmurs_from_beyond_the_opponent_picks_the_card_for_the_graveyard() {
    cr!("701.20a", "608.2c");
    assert_supported("Murmurs from Beyond");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    // Top first: Shock, Forest, Grizzly Bears, Plains.
    let ids = library(&mut t, P0, &["Plains", "Grizzly Bears", "Forest", "Shock"]);
    t.answer_choose(P1, &[Entity::Object(ids[2])]);
    let s = t.hand(P0, "Murmurs from Beyond");
    t.cast(P0, s).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Forest"), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Shock") && t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.library_size(P0), 1);
}

#[test]
fn erratic_mutation_uses_the_revealed_cards_mana_value_and_bottoms_them() {
    cr!("701.20a", "608.2c");
    assert_supported("Erratic Mutation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let giant = t.battlefield(P1, "Hill Giant");
    // Top first: Forest, Llanowar Elves (mana value 1), Shock.
    let ids = library(&mut t, P0, &["Shock", "Llanowar Elves", "Forest"]);
    let s = t.hand(P0, "Erratic Mutation");
    t.cast(P0, s).target(giant).go();
    t.resolve();
    assert_eq!(t.pt(giant), (4, 2), "{}", t.dump_log());
    // Shock wasn't revealed: it's on top, the revealed cards under it.
    let lib = t.g.player(P0).library.clone();
    assert_eq!(lib.len(), 3);
    assert_eq!(*lib.last().unwrap(), ids[0]);
}
