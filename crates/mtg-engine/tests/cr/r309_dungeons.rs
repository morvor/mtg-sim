//! CR 309: dungeons — nontraditional cards brought into the command zone by venturing
//! into the dungeon, rooms and the venture marker, room abilities, and completing a
//! dungeon.

use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::dungeons;
use mtg_engine::game::{Game, GameConfig};
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const MINE: &str = "Lost Mine of Phandelver";

fn venture(t: &mut TestGame, p: PlayerId) {
    keyword_action(t, p, KeywordAction::Venture, 1);
}

fn choose(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// A game where P0 owns a Lost Mine of Phandelver outside the game.
fn with_own_dungeon() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let mine = t.custom(
        P0,
        (*mtg_engine::card::card(MINE)).clone(),
        Zone::Outside(P0),
    );
    (t, mine)
}

fn dungeons_in_command(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.command
        .iter()
        .copied()
        .filter(|id| t.obj(*id).owner == p && t.obj(*id).is(CardType::Dungeon))
        .collect()
}

#[test]
fn a_dungeon_is_a_nontraditional_card_that_isnt_part_of_a_deck() {
    cr!("309.1", "309.2");
    let card = mtg_engine::card::card(MINE);
    assert!(mtg_engine::variants::is_nontraditional(&card));
    // Listed with a deck, it doesn't go into the library: it begins outside the game.
    let mut deck = vec![mtg_engine::card::card("Forest"); 20];
    deck.push(card);
    let g = Game::new(GameConfig::default(), vec![deck, vec![]], vec![]);
    assert_eq!(g.player(P0).library.len(), 20);
    let mine: Vec<_> = g
        .objects
        .iter()
        .filter(|o| o.base.name.as_str() == MINE)
        .collect();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].zone, Zone::Outside(P0));
    // It's brought into the game by venturing into the dungeon.
    let (mut t, mine) = with_own_dungeon();
    venture(&mut t, P0);
    assert_eq!(dungeons_in_command(&t, P0), vec![t.g.current(mine)]);
}

#[test]
fn venturing_without_a_dungeon_puts_one_the_player_owns_into_the_command_zone() {
    cr!("309.2a", "309.4", "309.4a");
    // P0 owns a single dungeon card outside the game: that's the one.
    let (mut t, mine) = with_own_dungeon();
    venture(&mut t, P0);
    let d = t.g.current(mine);
    assert_eq!(t.zone(d), Zone::Command);
    assert_eq!(t.obj(d).owner, P0);
    // The venture marker is on its topmost room.
    assert_eq!(dungeons::marker(&t.g, P0), Some((d, 0)));
    // A player without dungeon cards of their own chooses among the dungeons available.
    choose(&mut t, P1, 0);
    venture(&mut t, P1);
    let theirs = dungeons_in_command(&t, P1);
    assert_eq!(theirs.len(), 1);
    assert_eq!(dungeons::marker(&t.g, P1), Some((theirs[0], 0)));
}

#[test]
fn a_dungeon_stays_in_the_command_zone_until_it_leaves_the_game() {
    cr!("309.2b", "309.2c");
    let (mut t, mine) = with_own_dungeon();
    venture(&mut t, P0);
    t.resolve_all();
    let d = t.g.current(mine);
    // It's not a permanent, and it can't be cast.
    assert!(t.g.permanents().all(|o| o.id != d));
    assert!(t.cast(P0, d).try_go().is_err());
    // It can't be moved out of the command zone by an effect.
    for to in [
        Destination::zone(ZoneKind::Hand),
        Destination::zone(ZoneKind::Exile),
        Destination::battlefield(),
    ] {
        run_effect(
            &mut t,
            P0,
            None,
            Effect::Move {
                what: Sel::Target(0),
                to,
            },
            &[Entity::Object(d)],
        );
        assert_eq!(t.g.current(d), d);
        assert_eq!(t.zone(d), Zone::Command);
    }
    // It stays across turns.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(d), Zone::Command);
    assert_eq!(dungeons::marker(&t.g, P0), Some((d, 0)));
}

#[test]
fn a_player_owns_only_one_dungeon_in_the_command_zone_at_a_time() {
    cr!("309.3");
    let (mut t, mine) = with_own_dungeon();
    // A second dungeon card P0 owns is outside the game too.
    t.custom(
        P0,
        (*mtg_engine::card::card(MINE)).clone(),
        Zone::Outside(P0),
    );
    venture(&mut t, P0);
    choose(&mut t, P0, 0);
    venture(&mut t, P0);
    venture(&mut t, P0);
    // The later ventures moved the venture marker instead of bringing in another dungeon.
    assert_eq!(dungeons_in_command(&t, P0).len(), 1);
    let (d, room) = dungeons::marker(&t.g, P0).unwrap();
    assert_eq!(d, t.g.current(mine));
    assert!(room > 0);
}

#[test]
fn room_abilities_trigger_as_the_marker_moves_in_and_are_controlled_by_the_owner() {
    cr!("309.4c");
    supported(MINE);
    let (mut t, mine) = with_own_dungeon();
    // It's P1's turn; P0 ventures.
    t.set_step(P1, Step::PrecombatMain);
    venture(&mut t, P0);
    t.settle();
    let d = t.g.current(mine);
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    let o = t.obj(top);
    assert_eq!(o.controller, P0);
    assert!(matches!(
        o.stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { source, .. }) if *source == d
    ));
    t.resolve_all();
    // Moving P1's venture marker doesn't trigger P0's rooms.
    choose(&mut t, P1, 0);
    venture(&mut t, P1);
    t.settle();
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).controller, P1);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn venturing_moves_the_marker_along_an_arrow_the_player_chooses() {
    cr!("309.5", "309.5a");
    let (mut t, mine) = with_own_dungeon();
    venture(&mut t, P0);
    t.resolve_all();
    let d = t.g.current(mine);
    // Cave Entrance leads to Goblin Lair (room 1) or Mine Tunnels (room 2).
    choose(&mut t, P0, 1);
    venture(&mut t, P0);
    assert_eq!(dungeons::marker(&t.g, P0), Some((d, 2)));
    // Mine Tunnels — Create a Treasure token.
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 1);
    // The other arrow, in another game.
    let (mut t, mine) = with_own_dungeon();
    venture(&mut t, P0);
    t.resolve_all();
    choose(&mut t, P0, 0);
    venture(&mut t, P0);
    assert_eq!(dungeons::marker(&t.g, P0), Some((t.g.current(mine), 1)));
}

/// Ventures P0 from the topmost room to the bottommost one without resolving anything.
fn to_bottom(t: &mut TestGame) {
    venture(t, P0);
    choose(t, P0, 1);
    venture(t, P0);
    choose(t, P0, 0);
    venture(t, P0);
    venture(t, P0);
}

#[test]
fn venturing_from_the_bottommost_room_completes_the_dungeon_and_starts_another() {
    cr!("309.5b", "309.7");
    let (mut t, mine) = with_own_dungeon();
    to_bottom(&mut t);
    let d = t.g.current(mine);
    assert_eq!(dungeons::marker(&t.g, P0), Some((d, 6)));
    // The bottommost room's ability hasn't left the stack yet: the dungeon is still there.
    t.settle();
    assert_eq!(t.zone(d), Zone::Command);
    assert_eq!(t.g.player(P0).dungeons_completed, 0);
    // Venturing again removes it from the game (completing it), then P0 puts a dungeon
    // they own from outside the game into the command zone, marker on the topmost room.
    venture(&mut t, P0);
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
    let now = dungeons_in_command(&t, P0);
    assert_eq!(now.len(), 1);
    assert_eq!(t.obj(now[0]).base.name.as_str(), MINE);
    assert_eq!(dungeons::marker(&t.g, P0), Some((now[0], 0)));
    assert!(t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, mtg_engine::events::Event::Custom { name, player: Some(p), .. }
            if name.as_str() == dungeons::COMPLETED && *p == P0)));
}

#[test]
fn a_dungeon_is_removed_once_its_bottommost_rooms_ability_has_left_the_stack() {
    cr!("309.6", "309.7");
    let (mut t, mine) = with_own_dungeon();
    to_bottom(&mut t);
    let d = t.g.current(mine);
    t.settle();
    // Four room abilities wait; the dungeon stays while any of them does.
    while t.stack_len() > 1 {
        t.g.resolve_top();
        t.settle();
        assert_eq!(t.zone(d), Zone::Command);
    }
    t.resolve();
    assert_eq!(t.zone(d), Zone::Outside(P0));
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
    assert_eq!(dungeons::marker(&t.g, P0), None);
}
