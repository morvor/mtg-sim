//! CR 726: the initiative.

use mtg_engine::designations::take_initiative;
use mtg_engine::dungeons;
use mtg_engine::monarch_initiative::{
    is_inherent, INITIATIVE_STEAL, INITIATIVE_TAKEN, INITIATIVE_UPKEEP,
};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The controller and source of the top object of the stack.
fn top(t: &TestGame) -> (PlayerId, ObjectId) {
    let id = *t.g.stack.last().expect("empty stack");
    let o = t.g.obj(id);
    let src = match o.stack.as_deref().map(|s| &s.kind) {
        Some(mtg_engine::object::StackKind::Triggered { source, .. }) => *source,
        _ => id,
    };
    (o.controller, src)
}

/// The dungeon `p` is in and the room of their venture marker.
fn at(t: &TestGame, p: PlayerId) -> Option<(String, usize)> {
    dungeons::marker(&t.g, p).map(|(d, r)| (t.obj_now(d).chars.name.to_string(), r))
}

/// `p` takes the initiative (as an effect would make them).
fn take(t: &mut TestGame, p: PlayerId) {
    take_initiative(&mut t.g, p);
    t.g.flush_events();
}

#[test]
fn there_is_no_initiative_until_a_player_takes_it() {
    cr!("726.1", "726.2");
    ruling!(
        "Aarakocra Sneak",
        "There is no initiative in a game until an effect instructs a player to take the initiative. Once a player is instructed to do this, they have the initiative until another player takes the initiative."
    );
    ruling!(
        "Aarakocra Sneak",
        "If you aren't in a dungeon when instructed to venture into Undercity, you will put Undercity into the command zone and move your venture marker to Secret Entrance (the first room)."
    );
    let mut t = TestGame::new(2);
    assert_eq!(t.g.initiative, None);
    // No one ventures at the beginning of their upkeep without the initiative.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert!(t.g.stack.is_empty());
    t.advance_to(P0, Step::PrecombatMain);
    // "When this creature enters, you take the initiative."
    t.enter(P0, "Aarakocra Sneak");
    t.resolve_all();
    assert_eq!(t.g.initiative, Some(P0));
    // Taking the initiative made P0 venture into Undercity.
    assert_eq!(at(&t, P0), Some(("Undercity".to_string(), 0)));
}

#[test]
fn the_player_with_the_initiative_ventures_at_the_beginning_of_their_upkeep() {
    cr!("726.2");
    ruling!(
        "Aarakocra Sneak",
        "First, whenever a player takes the initiative, and at the beginning of the upkeep of the player with the initiative, that player ventures into Undercity."
    );
    let mut t = TestGame::new(2);
    take(&mut t, P0);
    // "Whenever a player takes the initiative, that player ventures into Undercity."
    t.settle();
    let (controller, src) = top(&t);
    assert_eq!(controller, P0);
    assert!(is_inherent(&t.g, src, INITIATIVE_TAKEN));
    t.resolve_all();
    assert_eq!(at(&t, P0), Some(("Undercity".to_string(), 0)));
    // Not at the beginning of another player's upkeep.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert!(t.g.stack.is_empty());
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    let (controller, src) = top(&t);
    assert_eq!(controller, P0);
    assert!(is_inherent(&t.g, src, INITIATIVE_UPKEEP));
    t.resolve_all();
    // Secret Entrance leads to Forge or Lost Well.
    let (name, room) = at(&t, P0).unwrap();
    assert_eq!(name, "Undercity");
    assert!(room > 0);
}

#[test]
fn creatures_dealing_combat_damage_to_the_player_with_the_initiative_take_it() {
    cr!("726.2", "726.3");
    ruling!(
        "Aarakocra Sneak",
        "Second, whenever one or more creatures a player controls deal combat damage to the player who has the initiative, the first player takes the initiative."
    );
    ruling!(
        "Aarakocra Sneak",
        "Only one player can have the initiative at a time. As one player takes the initiative, any other player that had the initiative ceases to have it."
    );
    let mut t = TestGame::new(2);
    take(&mut t, P0);
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Savannah Lions");
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P0)), (b, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    // One ability for the two creatures, controlled by the player with the initiative.
    assert_eq!(t.g.stack.len(), 1);
    let (controller, src) = top(&t);
    assert_eq!(controller, P0);
    assert!(is_inherent(&t.g, src, INITIATIVE_STEAL));
    t.resolve();
    assert_eq!(t.g.initiative, Some(P1));
    // Then P1 ventures into Undercity because they took the initiative.
    t.resolve_all();
    assert_eq!(at(&t, P1), Some(("Undercity".to_string(), 0)));
}

#[test]
fn if_the_player_with_the_initiative_leaves_the_active_player_takes_it() {
    cr!("726.4");
    ruling!(
        "Aarakocra Sneak",
        "If the player with the initiative leaves the game, the active player takes the initiative at the same time that player leaves the game. If the active player is leaving the game or if there is no active player, the next player in turn order takes the initiative."
    );
    let mut t = TestGame::new(3);
    take(&mut t, P2);
    t.resolve_all();
    t.take_action(P2, Action::Concede);
    assert_eq!(t.g.initiative, Some(P0));
    // Taking it this way is taking the initiative: P0 ventures into Undercity.
    t.resolve_all();
    assert_eq!(at(&t, P0), Some(("Undercity".to_string(), 0)));
    // During P1's turn, P1 leaves while having the initiative: the next player in turn
    // order still in the game (P3, as P2 already left) takes it.
    let mut t = TestGame::new(4);
    t.set_step(P1, Step::PrecombatMain);
    take(&mut t, P1);
    t.resolve_all();
    t.take_action(P2, Action::Concede);
    t.take_action(P1, Action::Concede);
    assert_eq!(t.g.initiative, Some(P3));
}

#[test]
fn taking_the_initiative_again_ventures_again_without_a_second_designation() {
    cr!("726.5", "726.3");
    ruling!(
        "Aarakocra Sneak",
        "A player who currently has the initiative may take the initiative again. This causes that player to venture into Undercity again, but does not cause them to have multiple initiative designations."
    );
    let mut t = TestGame::new(2);
    take(&mut t, P0);
    t.resolve_all();
    assert_eq!(at(&t, P0), Some(("Undercity".to_string(), 0)));
    take(&mut t, P0);
    t.settle();
    let (_, src) = top(&t);
    assert!(is_inherent(&t.g, src, INITIATIVE_TAKEN));
    t.resolve_all();
    assert_eq!(t.g.initiative, Some(P0));
    let (_, room) = at(&t, P0).unwrap();
    assert!(room > 0);
}
