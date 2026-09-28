//! Rulings batch S18 — venture into the dungeon (CR 701.49, 309): dungeon cards, the
//! venture marker, room abilities, and completing a dungeon.

use crate::r_s01_common::*;
use crate::r_s04_common::stack_items;
use crate::r_s06_common::activate_containing;
use mtg_engine::card::card;
use mtg_engine::deck::{self, DeckProblem};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::dungeons;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

const MINE: &str = "Lost Mine of Phandelver";
const TOMB: &str = "Tomb of Annihilation";

/// The dungeon in `p`'s command zone and the room of their venture marker.
fn at(t: &TestGame, p: PlayerId) -> Option<(String, usize)> {
    dungeons::marker(&t.g, p).map(|(d, r)| (t.obj_now(d).chars.name.to_string(), r))
}

/// Dungeon cards in the command zone.
fn dungeons_in_command(t: &TestGame) -> usize {
    t.g.command
        .iter()
        .filter(|id| t.g.obj(**id).chars.card_types.contains(CardType::Dungeon))
        .count()
}

/// P0 discards a Radiant Solar ("{W}, Discard this card: Venture into the dungeon and you
/// gain 3 life.") from hand; the ability is on the stack.
fn solar_venture(t: &mut TestGame) {
    t.lands(P0, "Plains", 1);
    let solar = t.hand(P0, "Radiant Solar");
    activate_containing(t, P0, solar, "Venture").expect("Radiant Solar");
}

/// Answers P0's next choice among options with `i`.
fn option(t: &mut TestGame, i: usize) {
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn venturing_enters_a_new_dungeon_or_moves_to_the_next_room() {
    cr!("701.49a", "701.49b", "309.4a", "309.4c");
    ruling!(
        "Radiant Solar",
        "To venture into the dungeon, a player moves their venture marker into the next room of the dungeon they are currently in. If they aren't currently in a dungeon, that player instead chooses a dungeon card from outside the game, puts it into the command zone, and moves their venture marker onto the first room."
    );
    ruling!(
        "Radiant Solar",
        "Moving into a dungeon room will cause its room ability to trigger."
    );
    supported("Radiant Solar");
    let mut t = TestGame::new(2);
    assert_eq!(at(&t, P0), None);
    // Not in a dungeon: P0 chooses Lost Mine of Phandelver, which is put into the command
    // zone with the marker on Cave Entrance ("Scry 1"), whose ability triggers.
    solar_venture(&mut t);
    option(&mut t, 0);
    t.resolve();
    assert_eq!(at(&t, P0), Some((MINE.to_string(), 0)));
    assert_eq!(dungeons_in_command(&t), 1);
    assert_eq!(t.life(P0), 23);
    assert_eq!(stack_items(&t), vec!["ability: Cave Entrance — Scry 1"]);
    t.resolve_all();
    // In a dungeon: the marker moves to the next room (Mine Tunnels, "Create a Treasure
    // token"), whose ability triggers.
    solar_venture(&mut t);
    option(&mut t, 1);
    t.resolve();
    assert_eq!(at(&t, P0), Some((MINE.to_string(), 2)));
    assert_eq!(stack_items(&t), vec!["ability: Mine Tunnels — Create a Treasure token"]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(dungeons_in_command(&t), 1);
}

#[test]
fn no_one_can_respond_between_the_venture_choice_and_the_room_ability_triggering() {
    cr!("701.49a", "309.4c", "603.3");
    ruling!(
        "Radiant Solar",
        "Choosing the dungeon or room to venture into is part of resolving the venture into the dungeon keyword action. Once that choice is made, players may not respond until after the appropriate room ability has triggered."
    );
    // The players pass priority for real; P1 records the stack each time it gets
    // priority: after Radiant Solar's ability, the next time is with the room ability
    // (Tomb of Annihilation's Trapped Entry) already on the stack.
    let mut t = TestGame::new(2);
    solar_venture(&mut t);
    option(&mut t, 2);
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        |g| {
            g.stack
                .iter()
                .map(|id| {
                    matches!(
                        g.obj(*id).stack.as_deref().map(|si| &si.kind),
                        Some(StackKind::Triggered { .. })
                    )
                })
                .collect::<Vec<_>>()
        },
    );
    // (Until Trapped Entry — "Each player loses 1 life." — has resolved.)
    assert!(t
        .g
        .run_until(1000, |g| g.stack.is_empty() && g.player(P1).life == 19));
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 2, "{seen:?}");
    // First Radiant Solar's activated ability, then the triggered room ability.
    assert_eq!(seen[0], vec![false]);
    assert_eq!(seen[1], vec![true]);
    assert_eq!(at(&t, P0), Some((TOMB.to_string(), 0)));
    // Trapped Entry: each player loses 1 life.
    assert_eq!(t.life(P1), 19);
}

#[test]
fn only_one_dungeon_and_a_completed_one_is_removed_as_a_state_based_action() {
    cr!("309.2a", "309.6", "704.5t", "701.49c");
    ruling!(
        "Radiant Solar",
        "A player may only have one dungeon in the command zone at a time."
    );
    ruling!(
        "Radiant Solar",
        "Dungeons are removed from the game as a state-based action."
    );
    // Tomb of Annihilation: Trapped Entry → Oubliette → Cradle of the Death God.
    let mut t = TestGame::new(2);
    for (i, choice) in [2, 1].into_iter().enumerate() {
        solar_venture(&mut t);
        option(&mut t, choice);
        t.resolve_all();
        assert_eq!(dungeons_in_command(&t), 1, "venture {i}");
    }
    solar_venture(&mut t);
    t.resolve();
    // The marker is on the bottommost room, but its ability is still on the stack: the
    // dungeon stays.
    assert_eq!(at(&t, P0), Some((TOMB.to_string(), 4)));
    assert_eq!(stack_items(&t).len(), 1);
    assert_eq!(dungeons_in_command(&t), 1);
    // Venturing now completes it and starts a new one: still only one dungeon.
    solar_venture(&mut t);
    option(&mut t, 0);
    t.resolve();
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
    assert_eq!(at(&t, P0), Some((MINE.to_string(), 0)));
    assert_eq!(dungeons_in_command(&t), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("The Atropal").len(), 1);
    // Once the last room's ability has resolved, the state-based action removes the
    // dungeon (Tomb of Annihilation again, straight to the bottom through Oubliette).
    let mut t = TestGame::new(2);
    for choice in [Some(2), Some(1), None] {
        solar_venture(&mut t);
        if let Some(c) = choice {
            option(&mut t, c);
            t.resolve_all();
        } else {
            t.resolve();
        }
    }
    assert_eq!(at(&t, P0), Some((TOMB.to_string(), 4)));
    let tomb = dungeons::marker(&t.g, P0).unwrap().0;
    // Resolve the room ability without checking state-based actions: still there.
    t.g.resolve_top();
    assert_eq!(t.g.obj(tomb).zone, Zone::Command);
    t.settle();
    assert_eq!(dungeons_in_command(&t), 0);
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
    assert_eq!(at(&t, P0), None);
}

#[test]
fn dungeon_cards_arent_part_of_a_deck_and_any_dungeon_can_be_used() {
    cr!("309.2", "100.2a", "100.4a", "701.49a");
    ruling!(
        "Radiant Solar",
        "Dungeon cards are not part of a player's deck or sideboard. In both constructed and limited formats, players can use any dungeon card when they venture into the dungeon."
    );
    // A 60-card constructed deck listed with dungeon cards: they aren't counted.
    let deck = |n: usize| {
        let mut d = vec![card("Forest"); n];
        d.extend([card(MINE), card(TOMB), card("Dungeon of the Mad Mage")]);
        d
    };
    assert!(deck::check_constructed(&deck(60)).is_empty());
    assert!(matches!(
        deck::check_constructed(&deck(59))[..],
        [DeckProblem::TooFewCards { have: 59, min: 60 }]
    ));
    // Nor in a full sideboard.
    let side: Vec<_> = (0..15)
        .map(|_| card("Plains"))
        .chain([card(MINE)])
        .collect();
    let names = deck::NameEquivalence::default();
    assert!(deck::check_constructed_with(&deck(60), &side, &names).is_empty());
    // Limited: a dungeon isn't a card of the pool, and isn't needed in it.
    let pool = vec![card("Forest"); 40];
    assert!(deck::check_limited(&deck(40), &pool).is_empty());
    // A player with no dungeon cards chooses among all the dungeons.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    solar_venture(&mut t);
    option(&mut t, 1);
    t.resolve();
    let offered: Vec<Vec<String>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect();
    assert_eq!(
        offered,
        vec![vec![
            MINE.to_string(),
            "Dungeon of the Mad Mage".to_string(),
            TOMB.to_string()
        ]]
    );
    assert_eq!(at(&t, P0), Some(("Dungeon of the Mad Mage".to_string(), 0)));
}
