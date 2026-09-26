//! CR 903.13: the Commander Draft option — a draft of two cards per pick, then a
//! multiplayer Commander game with decks built from the drafted card pools.

use crate::r100_common::pregame;
use crate::r900_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::commander_rules::{check_commander_draft_deck, COMMANDER_LEGENDS, COMMANDER_MASTERS};
use mtg_engine::deck::DeckProblem;
use mtg_engine::draft::{Draft, DraftError, DraftStyle, PassDirection};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn pid(i: usize) -> PlayerId {
    PlayerId(i as u8)
}

/// Every player drafts the first cards of the pack in front of them until the draft is
/// complete.
fn draft_all(d: &mut Draft, picks: usize) {
    while !d.is_complete() {
        for p in 0..d.players() {
            let n = d.pack(pid(p)).len().min(picks);
            if n > 0 {
                let idx: Vec<usize> = (0..n).collect();
                let _ = d.pick(pid(p), &idx);
            }
        }
    }
}

#[test]
fn commander_draft_is_a_draft_then_a_multiplayer_commander_game() {
    cr!("903.13", "903.13a");
    // Four players draft; each builds a Commander deck from their pool and they play a
    // multiplayer Commander game.
    let mut packs = boosters(4, 3, 8);
    packs[0][0][0] = card("Isamaru, Hound of Konda");
    let mut d = Draft::new(packs, DraftStyle::Commander, 1);
    draft_all(&mut d, 2);
    let (pools, _) = d.finish();
    assert!(pools.iter().all(|p| p.len() == 24));
    assert!(names_of(&pools[0]).contains(&"Isamaru, Hound of Konda".to_string()));
    let decks: Vec<Vec<Arc<CardDef>>> = pools
        .iter()
        .map(|pool| {
            let mut deck = pool.clone();
            deck.extend((deck.len()..60).map(|_| card("Plains")));
            deck
        })
        .collect();
    let config = GameConfig {
        skip_mulligans: true,
        starting_player: Some(P0),
        ..GameConfig::commander_game()
    };
    assert_eq!(config.validate(4), Ok(()));
    let mut t = pregame(config, decks);
    assert!(t.g.designate_commander(P0, "Isamaru, Hound of Konda"));
    t.g.start();
    assert_eq!(t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda").len(), 1);
    assert!(t.g.players_in_game().len() == 4);
}

#[test]
fn players_draft_two_cards_from_each_pack_until_all_are_drafted() {
    cr!("903.13b");
    let mut d = Draft::new(boosters(3, 3, 6), DraftStyle::Commander, 1);
    assert_eq!(d.round, 1);
    // Two cards: not one, not three.
    assert_eq!(d.pick(P0, &[0]), Err(DraftError::InvalidPick));
    assert_eq!(d.pick(P0, &[0, 1, 2]), Err(DraftError::InvalidPick));
    assert_eq!(d.pick(P0, &[0, 1]), Ok(()));
    assert_eq!(d.drafted(P0).len(), 2);
    // Then the player waits for the pack passed to them.
    assert_eq!(d.pick(P0, &[0, 1]), Err(DraftError::AlreadyDrafted));
    assert_eq!(d.pack(P0).len(), 4);
    d.pick(P1, &[0, 1]).unwrap();
    d.pick(P2, &[0, 1]).unwrap();
    // Everyone drafted: the packs were passed.
    assert_ne!(names_of(d.pack(P0))[0], "P0R0C2");
    assert_eq!(d.pack(P0).len(), 4);
    // Until every card of the round is drafted; then the next round begins.
    for _ in 0..2 {
        for p in 0..3 {
            d.pick(pid(p), &[0, 1]).unwrap();
        }
    }
    assert_eq!(d.round, 2);
    draft_all(&mut d, 2);
    assert!(d.is_complete());
    assert_eq!(d.round, 3);
    for p in 0..3 {
        assert_eq!(d.pool(pid(p)).len(), 18);
    }
}

#[test]
fn packs_go_left_in_the_first_and_third_rounds_and_right_in_the_second() {
    cr!("903.13c");
    assert_eq!(Draft::direction(1), PassDirection::Left);
    assert_eq!(Draft::direction(2), PassDirection::Right);
    assert_eq!(Draft::direction(3), PassDirection::Left);
    let mut d = Draft::new(boosters(3, 3, 4), DraftStyle::Commander, 1);
    let round = |d: &mut Draft| {
        for p in 0..3 {
            d.pick(pid(p), &[0, 1]).unwrap();
        }
    };
    // Round 1: P0's pack goes to P1 (on P0's left), P2's to P0.
    round(&mut d);
    assert!(names_of(d.pack(P1))[0].starts_with("P0R0"));
    assert!(names_of(d.pack(P0))[0].starts_with("P2R0"));
    assert_eq!(d.passed_by(P1), Some(P0));
    round(&mut d);
    // Round 2: to the right — P0's pack goes to P2.
    assert_eq!(d.round, 2);
    round(&mut d);
    assert!(names_of(d.pack(P2))[0].starts_with("P0R1"));
    assert!(names_of(d.pack(P0))[0].starts_with("P1R1"));
    round(&mut d);
    // Round 3: to the left again.
    assert_eq!(d.round, 3);
    round(&mut d);
    assert!(names_of(d.pack(P1))[0].starts_with("P0R2"));
}

#[test]
fn a_drafting_player_sees_only_their_pack_and_their_drafted_cards() {
    cr!("903.13d");
    let mut d = Draft::new(boosters(3, 1, 4), DraftStyle::Commander, 1);
    for p in 0..3 {
        d.pick(pid(p), &[0, 1]).unwrap();
    }
    // P0 sees the pack in front of them (P2's leftovers) and the two cards they drafted.
    let seen = names_of(&d.visible(P0));
    assert_eq!(seen.len(), 4);
    assert!(d.can_see(P0, "P0R0C0") && d.can_see(P0, "P0R0C1"));
    assert!(d.can_see(P0, "P2R0C2"));
    // Not the cards others drafted, nor the packs others are drafting from.
    assert!(!d.can_see(P0, "P1R0C0"));
    assert!(!d.can_see(P0, "P0R0C2"));
}

#[test]
fn the_pool_may_include_prismatic_pipers_as_commanders() {
    cr!("903.13e");
    // The cards a player drafted are their card pool.
    let mut d = Draft::new(boosters(3, 1, 4), DraftStyle::Commander, 1);
    draft_all(&mut d, 2);
    assert_eq!(d.pool(P0).len(), 4);
    // With Commander Legends (or Commander Masters) boosters, up to two cards named The
    // Prismatic Piper may be added to the pool — only as commanders.
    let piper = card("The Prismatic Piper");
    let pool: Vec<Arc<CardDef>> = (0..60).map(|_| card("Wastes")).collect();
    let mut deck = vec![piper.clone(), piper.clone()];
    deck.extend(pool.iter().cloned());
    let two = [piper.clone(), piper.clone()];
    assert!(check_commander_draft_deck(&deck, &two, &pool, &[COMMANDER_LEGENDS]).is_empty());
    assert!(check_commander_draft_deck(&deck, &two, &pool, &[COMMANDER_MASTERS]).is_empty());
    // Not with other boosters.
    assert!(check_commander_draft_deck(&deck, &two, &pool, &["dom"])
        .iter()
        .any(|p| matches!(p, DeckProblem::NotInCardPool { name } if name == "The Prismatic Piper")));
    // Not as a card in the deck that isn't a commander.
    let isamaru = card("Isamaru, Hound of Konda");
    let mut pool2 = pool.clone();
    pool2.push(isamaru.clone());
    let mut deck2 = vec![isamaru.clone(), piper.clone()];
    deck2.extend(pool.iter().cloned());
    assert!(check_commander_draft_deck(&deck2, &[isamaru], &pool2, &[COMMANDER_LEGENDS])
        .iter()
        .any(|p| matches!(p, DeckProblem::NotInCardPool { name } if name == "The Prismatic Piper")));
}

#[test]
fn commander_draft_deck_construction() {
    cr!("903.13f");
    let isamaru = card("Isamaru, Hound of Konda");
    let lions = card("Savannah Lions");
    let mut pool = vec![isamaru.clone(), lions.clone(), lions.clone(), lions.clone()];
    pool.extend((0..60).map(|_| card("Plains")));
    let build = |n_lions: usize, n_plains: usize| {
        let mut d = vec![isamaru.clone()];
        d.extend((0..n_lions).map(|_| lions.clone()));
        d.extend((0..n_plains).map(|_| card("Plains")));
        d
    };
    let cmdrs = [isamaru.clone()];
    // At least 60 cards, no maximum; any number of cards with the same name from the
    // pool.
    assert!(check_commander_draft_deck(&build(3, 56), &cmdrs, &pool, &[]).is_empty());
    assert!(check_commander_draft_deck(&build(3, 96), &cmdrs, &pool, &[]).is_empty());
    assert!(check_commander_draft_deck(&build(3, 55), &cmdrs, &pool, &[])
        .iter()
        .any(|p| matches!(p, DeckProblem::TooFewCards { min: 60, .. })));
    // But only as many as the pool has.
    assert!(check_commander_draft_deck(&build(4, 56), &cmdrs, &pool, &[])
        .iter()
        .any(|p| matches!(p, DeckProblem::NotInCardPool { name } if name == "Savannah Lions")));
    // The other Commander rules apply: color identity.
    let mut pool_b = pool.clone();
    pool_b.push(card("Lightning Bolt"));
    let mut d = build(3, 56);
    d.push(card("Lightning Bolt"));
    assert!(check_commander_draft_deck(&d, &cmdrs, &pool_b, &[])
        .iter()
        .any(|p| matches!(p, DeckProblem::OutsideColorIdentity { .. })));
    // With Commander Masters boosters, a card that can be a commander by itself with one
    // or fewer colors in its color identity is considered to have partner.
    let krenko = card("Krenko, Mob Boss");
    let two = [isamaru.clone(), krenko.clone()];
    let mut pool_k = pool.clone();
    pool_k.push(krenko.clone());
    let mut d = build(3, 55);
    d.push(krenko.clone());
    assert!(check_commander_draft_deck(&d, &two, &pool_k, &[COMMANDER_MASTERS]).is_empty());
    assert!(check_commander_draft_deck(&d, &two, &pool_k, &[COMMANDER_LEGENDS])
        .iter()
        .any(|p| matches!(p, DeckProblem::InvalidCommanders { .. })));
    // It can partner with a two-color card that has partner itself (Akiri, Line-Slinger:
    // red and white): Krenko is considered to have partner.
    let akiri = card("Akiri, Line-Slinger");
    let two = [krenko.clone(), akiri.clone()];
    let mut pool_a = pool.clone();
    pool_a.extend([krenko.clone(), akiri.clone()]);
    let mut d = vec![krenko.clone(), akiri.clone()];
    d.extend((0..58).map(|_| card("Plains")));
    assert!(check_commander_draft_deck(&d, &two, &pool_a, &[COMMANDER_MASTERS]).is_empty());
    assert!(check_commander_draft_deck(&d, &two, &pool_a, &[COMMANDER_LEGENDS])
        .iter()
        .any(|p| matches!(p, DeckProblem::InvalidCommanders { .. })));
    // Not a two-color card.
    let niv = card("Niv-Mizzet, Parun");
    let two = [isamaru.clone(), niv.clone()];
    let mut pool_n = pool.clone();
    pool_n.push(niv.clone());
    let mut d = build(3, 55);
    d.push(niv);
    assert!(check_commander_draft_deck(&d, &two, &pool_n, &[COMMANDER_MASTERS])
        .iter()
        .any(|p| matches!(p, DeckProblem::InvalidCommanders { .. })));
}

#[test]
fn commander_draft_games_follow_the_commander_rules() {
    cr!("903.13g");
    // A 60-card drafted deck, in a Commander game.
    let mut deck = vec![card("Isamaru, Hound of Konda")];
    deck.extend((0..59).map(|_| card("Plains")));
    let mut other = vec![card("Krenko, Mob Boss")];
    other.extend((0..59).map(|_| card("Mountain")));
    let mut t = pregame(
        GameConfig {
            skip_mulligans: true,
            starting_player: Some(P0),
            ..GameConfig::commander_game()
        },
        vec![deck, other, (0..60).map(|_| card("Plains")).collect()],
    );
    assert!(t.g.designate_commander(P0, "Isamaru, Hound of Konda"));
    assert!(t.g.designate_commander(P1, "Krenko, Mob Boss"));
    t.g.start();
    // CR 903.6, 903.7: commanders start in the command zone; 40 life, seven cards.
    let isamaru = t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda");
    assert_eq!(isamaru.len(), 1);
    assert_eq!((t.life(P0), t.hand_size(P0), t.library_size(P0)), (40, 7, 52));
    // CR 903.8: cast from the command zone.
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P0, "Plains", 1);
    t.cast(P0, isamaru[0]).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
    // CR 903.10a: commander damage.
    t.g.players[2]
        .commander_damage
        .insert("Isamaru, Hound of Konda".into(), 21);
    t.settle();
    assert!(t.has_lost(pid(2)));
}
