//! CR 905: the Conspiracy Draft casual variant — a draft of one card per pick with cards
//! that function during the draft, then a multiplayer game in which conspiracies start in
//! the command zone.

use crate::r100_common::{fillers, pregame};
use crate::r703_common::supported;
use crate::r900_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::{check_limited, DeckProblem};
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::draft::{Draft, DraftError, DraftStyle, PassDirection};
use mtg_engine::game::GameConfig;
use mtg_engine::multiplayer::setup::SetupError;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn pid(i: usize) -> PlayerId {
    PlayerId(i as u8)
}

fn draft_all(d: &mut Draft) {
    while !d.is_complete() {
        for p in 0..d.players() {
            if !d.pack(pid(p)).is_empty() {
                let _ = d.pick(pid(p), &[0]);
            }
        }
    }
}

/// A limited multiplayer game (not started) with 40-card decks.
fn conspiracy_pregame(n: usize) -> TestGame {
    pregame(
        GameConfig {
            skip_mulligans: true,
            starting_player: Some(P0),
            ..GameConfig::conspiracy_draft_game()
        },
        (0..n).map(|_| fillers(40)).collect(),
    )
}

#[test]
fn conspiracy_draft_is_a_draft_then_a_multiplayer_game() {
    cr!("905.1");
    let mut d = Draft::new(boosters(4, 3, 15), DraftStyle::Conspiracy, 1);
    draft_all(&mut d);
    let (pools, _) = d.finish();
    assert!(pools.iter().all(|p| p.len() == 45));
    // The decks are built from the pools; the game is a multiplayer game.
    let decks: Vec<Vec<Arc<CardDef>>> = pools
        .iter()
        .map(|p| {
            let mut deck: Vec<Arc<CardDef>> = p.iter().take(23).cloned().collect();
            deck.extend((0..17).map(|_| card("Forest")));
            assert!(check_limited(&deck, p).is_empty());
            deck
        })
        .collect();
    let config = GameConfig {
        skip_mulligans: true,
        ..GameConfig::conspiracy_draft_game()
    };
    assert_eq!(config.validate(4), Ok(()));
    let mut t = pregame(config, decks);
    t.g.start();
    assert_eq!(t.g.players_in_game().len(), 4);
}

#[test]
fn players_draft_one_card_from_each_pack() {
    cr!("905.1a");
    let mut d = Draft::new(boosters(3, 3, 5), DraftStyle::Conspiracy, 1);
    assert_eq!(d.pick(P0, &[0, 1]), Err(DraftError::InvalidPick));
    assert_eq!(d.pick(P0, &[4]), Ok(()));
    assert_eq!(d.pick(P0, &[0]), Err(DraftError::AlreadyDrafted));
    assert_eq!(names_of(&d.pool(P0)), vec!["P0R0C4"]);
    d.pick(P1, &[0]).unwrap();
    d.pick(P2, &[0]).unwrap();
    // The packs were passed; drafting continues until every card of the round is
    // drafted (five picks each), then the next round's packs are opened.
    assert_eq!(d.pack(P0).len(), 4);
    for _ in 0..4 {
        for p in 0..3 {
            d.pick(pid(p), &[0]).unwrap();
        }
    }
    assert_eq!(d.round, 2);
    assert_eq!(d.pool(P0).len(), 5);
    draft_all(&mut d);
    assert_eq!(d.round, 3);
    assert_eq!(d.pool(P2).len(), 15);
}

#[test]
fn conspiracy_packs_go_left_right_left() {
    cr!("905.1b");
    assert_eq!(Draft::direction(1), PassDirection::Left);
    assert_eq!(Draft::direction(2), PassDirection::Right);
    assert_eq!(Draft::direction(3), PassDirection::Left);
    let mut d = Draft::new(boosters(4, 3, 2), DraftStyle::Conspiracy, 1);
    let once = |d: &mut Draft| {
        for p in 0..4 {
            d.pick(pid(p), &[0]).unwrap();
        }
    };
    once(&mut d);
    // Left: P3's pack to P0, P0's to P1.
    assert_eq!(names_of(d.pack(P0)), vec!["P3R0C1"]);
    assert_eq!(names_of(d.pack(P1)), vec!["P0R0C1"]);
    once(&mut d);
    once(&mut d);
    // Right: P1's pack to P0, P0's to P3.
    assert_eq!(names_of(d.pack(P0)), vec!["P1R1C1"]);
    assert_eq!(names_of(d.pack(pid(3))), vec!["P0R1C1"]);
    once(&mut d);
    once(&mut d);
    assert_eq!(names_of(d.pack(P1)), vec!["P0R2C1"]);
}

/// A draft of three players whose first packs hold `first` for P0.
fn draft_with(first: Vec<Arc<CardDef>>) -> Draft {
    let mut packs = boosters(3, 1, first.len().max(3));
    for (i, c) in first.into_iter().enumerate() {
        packs[0][0][i] = c;
    }
    Draft::new(packs, DraftStyle::Conspiracy, 1)
}

#[test]
fn what_a_drafting_player_may_look_at() {
    cr!("905.1c");
    supported("Garbage Fire");
    let mut d = draft_with(vec![
        card("Cogwork Librarian"),
        card("Garbage Fire"),
        named("Secret Pick"),
        named("Another Card"),
    ]);
    // P0 drafts Cogwork Librarian (face up); the others draft their first cards.
    d.pick(P0, &[0]).unwrap();
    d.pick(P1, &[0]).unwrap();
    d.pick(P2, &[0]).unwrap();
    // P1 now has P0's pack: they see it, their own drafted card, and P0's face-up
    // Librarian; not P2's drafted card.
    assert!(d.can_see(P1, "Garbage Fire"));
    assert!(d.can_see(P1, "P1R0C0"));
    assert!(d.can_see(P1, "Cogwork Librarian"));
    assert!(!d.can_see(P1, "P2R0C0"));
    // P1 drafts Garbage Fire, revealing it: while it's revealed, everyone may see it.
    d.pick(P1, &[0]).unwrap();
    assert!(d.can_see(P2, "Garbage Fire"));
    assert!(d.can_see(P0, "Garbage Fire"));
    d.pick(P0, &[0]).unwrap();
    d.pick(P2, &[0]).unwrap();
    // Once it's turned face down (the packs have been passed), only its drafter sees it;
    // the noted information stays public.
    assert!(!d.can_see(P2, "Garbage Fire"));
    assert!(!d.can_see(P0, "Garbage Fire"));
    assert!(d.can_see(P1, "Garbage Fire"));
    assert!(d
        .info
        .notes
        .iter()
        .any(|n| n.player == P1 && n.card == "Garbage Fire" && n.number == Some(2)));
    // A card drafted without being revealed stays hidden.
    let theirs = names_of(&d.pool(P2));
    assert!(!d.can_see(P0, &theirs[1]));
}

#[test]
fn the_card_pool_and_the_deck() {
    cr!("905.1d");
    let mut d = Draft::new(boosters(3, 3, 15), DraftStyle::Conspiracy, 1);
    draft_all(&mut d);
    let pool = d.pool(P0);
    assert_eq!(pool.len(), 45);
    // A deck of cards from the pool and any number of basic lands.
    let mut deck: Vec<Arc<CardDef>> = pool.iter().take(20).cloned().collect();
    deck.extend((0..20).map(|_| card("Island")));
    assert!(check_limited(&deck, &pool).is_empty());
    // Not cards from someone else's pool.
    deck.push(d.pool(P1)[0].clone());
    assert!(check_limited(&deck, &pool)
        .iter()
        .any(|p| matches!(p, DeckProblem::NotInPool { .. })));
}

#[test]
fn some_cards_function_during_the_draft() {
    cr!("905.2", "905.2c");
    // Cogwork Librarian: "Draft this card face up."
    let mut d = draft_with(vec![card("Cogwork Librarian"), named("A"), named("B")]);
    d.pick(P0, &[0]).unwrap();
    assert!(d.drafted(P0)[0].face_up);
    // While it's face up, all players may look at it.
    assert!(d.can_see(P1, "Cogwork Librarian"));
    assert!(d.can_see(P2, "Cogwork Librarian"));
    // An effect instructs P0 to turn it face down: it's hidden again.
    d.turn_face_down(P0, 0).unwrap();
    assert!(!d.can_see(P1, "Cogwork Librarian"));
    assert_eq!(d.turn_face_down(P0, 0), Err(DraftError::NotFaceUp));
    // It remains face up only until the draft is complete.
    let mut d = draft_with(vec![card("Cogwork Librarian"), named("A"), named("B")]);
    d.pick(P0, &[0]).unwrap();
    draft_all(&mut d);
    let (pools, _) = d.finish();
    assert!(names_of(&pools[0]).contains(&"Cogwork Librarian".to_string()));
    // An ordinary card is drafted face down.
    let mut d = draft_with(vec![named("A"), named("B"), named("C")]);
    d.pick(P0, &[0]).unwrap();
    assert!(!d.drafted(P0)[0].face_up);
    assert!(!d.can_see(P1, "A"));
}

#[test]
fn during_a_draft_simultaneous_actions_happen_in_a_random_order() {
    cr!("905.2a");
    // No active player and no priority: players draft in any order, and the packs are
    // passed only once everyone has drafted.
    let mut d = Draft::new(boosters(3, 1, 3), DraftStyle::Conspiracy, 1);
    d.pick(P2, &[0]).unwrap();
    d.pick(P0, &[0]).unwrap();
    assert_eq!(d.pack(P0).len(), 2);
    assert!(names_of(d.pack(P0))[0].starts_with("P0"));
    d.pick(P1, &[0]).unwrap();
    assert!(names_of(d.pack(P0))[0].starts_with("P2"));
    // Players who want to act at the same time and can't agree act in a random order.
    let mut orders = Vec::new();
    for seed in 0..20 {
        let mut d = Draft::new(boosters(3, 1, 3), DraftStyle::Conspiracy, seed);
        let o = d.simultaneous_order(&[P0, P1, P2]);
        let mut sorted = o.clone();
        sorted.sort();
        assert_eq!(sorted, vec![P0, P1, P2]);
        if !orders.contains(&o) {
            orders.push(o);
        }
    }
    assert!(orders.len() > 1);
}

#[test]
fn noted_draft_information_is_used_during_the_game() {
    cr!("905.2b");
    supported("Garbage Fire");
    // Garbage Fire: "Reveal this card as you draft it and note how many cards you've
    // drafted this draft round, including this card." P0 drafts it as their second card.
    let mut packs = boosters(3, 1, 3);
    packs[2][0][1] = card("Garbage Fire");
    let mut d = Draft::new(packs, DraftStyle::Conspiracy, 1);
    for p in 0..3 {
        d.pick(pid(p), &[0]).unwrap();
    }
    // P0 now has P2's pack: Garbage Fire is its first card.
    assert_eq!(names_of(d.pack(P0))[0], "Garbage Fire");
    d.pick(P0, &[0]).unwrap();
    // Revealed as it's drafted, and the number is noted.
    assert!(d.can_see(P1, "Garbage Fire"));
    assert_eq!(d.info.notes.len(), 1);
    assert_eq!(d.info.notes[0].number, Some(2));
    assert_eq!(d.info.notes[0].player, P0);
    // Then it's turned face down and added to P0's pile; any player can still look at
    // the noted information.
    d.pick(P1, &[0]).unwrap();
    d.pick(P2, &[0]).unwrap();
    assert!(!d.can_see(P1, "Garbage Fire"));
    assert!(d.drafted(P0).iter().any(|c| c.card.name == "Garbage Fire" && !c.face_up));
    assert_eq!(d.info.notes[0].card, "Garbage Fire");
    draft_all(&mut d);
    let (_, info) = d.finish();
    // In the game: "Garbage Fire deals damage to target creature equal to the highest
    // number you noted for cards named Garbage Fire."
    let mut t = TestGame::new(3);
    t.g.start.draft = info;
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let fire = t.hand(P0, "Garbage Fire");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, fire).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    let fire2 = t.hand(P0, "Garbage Fire");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, fire2).target(giant).go();
    t.resolve();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj(giant).damage, 2);
    // Another player noted nothing: their Garbage Fire deals no damage.
    let bears2 = t.battlefield(P0, "Grizzly Bears");
    let fire3 = t.hand(P1, "Garbage Fire");
    t.lands(P1, "Mountain", 3);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, fire3).target(bears2).go();
    t.resolve();
    assert!(t.on_battlefield(bears2));
}

#[test]
fn a_conspiracy_draft_game_is_multiplayer_free_for_all() {
    cr!("905.3");
    let c = GameConfig::conspiracy_draft_game();
    assert!(c.validate(2).unwrap_err().contains(&SetupError::NotMultiplayer));
    assert_eq!(c.validate(4), Ok(()));
    assert!(c.attack_multiple_players);
    assert_eq!(c.range_of_influence, None);
    let t = TestGame::with_config(4, c);
    for q in [P1, P2, pid(3)] {
        assert!(t.g.are_opponents(P0, q));
    }
}

#[test]
fn conspiracies_go_from_the_sideboard_to_the_command_zone_as_the_game_starts() {
    cr!("905.4");
    let mut t = conspiracy_pregame(3);
    let side = t.g.add_to_sideboard(
        P0,
        vec![card("Power Play"), card("Iterative Analysis"), card("Grizzly Bears")],
    );
    // Any number of them: P0 puts one of the two conspiracies in.
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.g.start();
    assert_eq!(t.zone(side[0]), Zone::Command);
    assert_eq!(t.zone(side[1]), Zone::Outside(P0));
    assert_eq!(t.zone(side[2]), Zone::Outside(P0));
    // Before the decks were shuffled into libraries: it's not part of the library.
    assert_eq!(t.g.player(P0).library.len() + t.hand_size(P0), 40);
    // Power Play ("You are the starting player") already applied.
    assert_eq!(t.g.turn.starting_player, P0);
}

#[test]
fn hidden_agenda_conspiracies_start_face_down() {
    cr!("905.4a");
    supported("Brago's Favor");
    let mut t = conspiracy_pregame(3);
    let side = t.g.add_to_sideboard(P1, vec![card("Brago's Favor")]);
    t.g.start();
    let c = side[0];
    assert_eq!(t.zone(c), Zone::Command);
    assert!(t.obj(c).face_down);
    // Any time they have priority — even during another player's turn — they may turn it
    // face up.
    t.set_step(P0, Step::Upkeep);
    t.g.turn.priority = Some(P1);
    let action = Action::Special(SpecialAction::TurnFaceUp { obj: c });
    assert!(t.g.legal_actions(P1).contains(&action));
    t.g.perform_action(P1, action).unwrap();
    assert!(!t.obj(c).face_down);
}

#[test]
fn a_conspiracy_belongs_to_the_player_who_put_it_into_the_command_zone() {
    cr!("905.5");
    let mut t = conspiracy_pregame(3);
    let side = t.g.add_to_sideboard(pid(2), vec![card("Power Play")]);
    t.g.start();
    let c = side[0];
    assert_eq!((t.obj(c).owner, t.obj(c).controller), (pid(2), pid(2)));
    // It applies for them: they're the starting player.
    assert_eq!(t.g.turn.starting_player, pid(2));
}

#[test]
fn conspiracy_draft_starting_life_and_hands() {
    cr!("905.6");
    let mut t = conspiracy_pregame(4);
    t.g.start();
    for p in 0..4 {
        assert_eq!(t.life(pid(p)), 20);
        assert_eq!(t.hand_size(pid(p)), 7);
    }
}
