//! CR 100: general — decks, deck construction (constructed, limited, Commander),
//! supplementary decks, sideboards, minimum deck sizes, and matches.

use crate::r100_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::*;
use mtg_engine::decision::{Action, Decision};
use mtg_engine::game::{Game, GameConfig, GameResult, Variant};
use mtg_engine::match_play::Match;
use mtg_engine::object::Zone;
use mtg_engine::planechase::{self, PlanarFace};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn deck(cards: &[(&str, usize)]) -> Vec<Arc<CardDef>> {
    let mut d = Vec::new();
    for (n, k) in cards {
        d.extend(std::iter::repeat_n(card(n), *k));
    }
    d
}

// ---------------------------------------------------------------------------
// Decks and deck construction
// ---------------------------------------------------------------------------

#[test]
fn constructed_decks_have_sixty_cards_and_at_most_four_of_a_card() {
    cr!("100.2a");
    assert!(check_constructed(&deck(&[("Grizzly Bears", 4), ("Forest", 56)])).is_empty());
    assert_eq!(
        check_constructed(&deck(&[("Grizzly Bears", 4), ("Forest", 55)])),
        vec![DeckProblem::TooFewCards { have: 59, min: 60 }]
    );
    assert_eq!(
        check_constructed(&deck(&[("Grizzly Bears", 5), ("Forest", 55)])),
        vec![DeckProblem::TooManyCopies {
            name: "Grizzly Bears".into(),
            have: 5,
            max: 4
        }]
    );
    // Any number of basic lands.
    assert!(check_constructed(&deck(&[("Forest", 60)])).is_empty());
    // Cards with interchangeable names count as the same name (CR 201.3b).
    let names = NameEquivalence(vec![("Grizzly Bears".into(), "Bear Cub".into())]);
    let d = deck(&[("Grizzly Bears", 3), ("Bear Cub", 2), ("Forest", 55)]);
    assert!(check_constructed(&d).is_empty());
    assert_eq!(
        check_constructed_with(&d, &[], &names),
        vec![DeckProblem::TooManyCopies {
            name: "Grizzly Bears".into(),
            have: 5,
            max: 4
        }]
    );
}

#[test]
fn limited_decks_have_forty_cards_from_the_pool() {
    cr!("100.2b");
    let pool = deck(&[("Grizzly Bears", 2), ("Hill Giant", 1), ("Lightning Bolt", 1)]);
    // As many duplicates as the product had, plus any number of basic lands.
    let ok = deck(&[("Grizzly Bears", 2), ("Hill Giant", 1), ("Mountain", 37)]);
    assert!(check_limited(&ok, &pool).is_empty());
    let dup = deck(&[("Grizzly Bears", 3), ("Mountain", 37)]);
    assert_eq!(
        check_limited(&dup, &pool),
        vec![DeckProblem::NotInPool {
            name: "Grizzly Bears".into(),
            have: 3,
            available: 2
        }]
    );
    let small = deck(&[("Grizzly Bears", 2), ("Mountain", 37)]);
    assert_eq!(
        check_limited(&small, &pool),
        vec![DeckProblem::TooFewCards { have: 39, min: 40 }]
    );
}

#[test]
fn commander_decks_have_additional_restrictions() {
    cr!("100.2c", "100.5");
    // Isamaru is white: exactly 100 cards, singleton except basic lands, within its color
    // identity.
    let cmdr = card("Isamaru, Hound of Konda");
    let mut d = vec![cmdr.clone()];
    d.extend(deck(&[("Plains", 99)]));
    assert!(check_commander(&d, &cmdr, &[], false).is_empty());
    let mut big = d.clone();
    big.push(card("Plains"));
    assert_eq!(
        check_commander(&big, &cmdr, &[], false),
        vec![DeckProblem::TooManyCards { have: 101, max: 100 }]
    );
    let mut dup = vec![cmdr.clone()];
    dup.extend(deck(&[("Savannah Lions", 2), ("Plains", 97)]));
    assert_eq!(
        check_commander(&dup, &cmdr, &[], false),
        vec![DeckProblem::TooManyCopies {
            name: "Savannah Lions".into(),
            have: 2,
            max: 1
        }]
    );
    let mut off = vec![cmdr.clone()];
    off.extend(deck(&[("Lightning Bolt", 1), ("Plains", 98)]));
    assert_eq!(
        check_commander(&off, &cmdr, &[], false),
        vec![DeckProblem::OutsideColorIdentity {
            name: "Lightning Bolt".into()
        }]
    );
    // A card's own ability can allow more copies (Relentless Rats) — needs a black
    // commander.
    let black = card("Sheoldred, the Apocalypse");
    let mut rats = vec![black.clone()];
    rats.extend(deck(&[("Relentless Rats", 30), ("Swamp", 69)]));
    assert!(check_commander(&rats, &black, &[], false).is_empty());
    // No sideboards.
    assert_eq!(
        check_commander(&d, &cmdr, &deck(&[("Plains", 1)]), false),
        vec![DeckProblem::SideboardNotAllowed]
    );
}

#[test]
fn supplementary_decks_of_nontraditional_cards_are_separate() {
    cr!("100.2d");
    // Planes aren't part of the deck for its construction rules...
    let mut d = deck(&[("Forest", 59)]);
    d.push(card("Strixhaven"));
    d.push(card("The Great Aerie"));
    assert_eq!(
        check_constructed(&d),
        vec![DeckProblem::TooFewCards { have: 59, min: 60 }]
    );
    d.push(card("Forest"));
    assert!(check_constructed(&d).is_empty());
    // ... and in the game they form a planar deck in the command zone, not part of the
    // library.
    let t = pregame(
        GameConfig {
            variant: Variant::Planechase,
            ..Default::default()
        },
        vec![d, fillers(60)],
    );
    assert_eq!(t.g.player(P0).library.len(), 60);
    assert_eq!(planechase::planar_deck(&t.g, P0).len(), 2);
}

#[test]
fn casual_variants_use_specialized_dice() {
    // The planar die of Planechase: its faces come up at random.
    cr!("100.3");
    let mut faces = std::collections::BTreeSet::new();
    for seed in 0..30 {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                variant: Variant::Planechase,
                seed,
                ..Default::default()
            },
        );
        let f = planechase::roll_planar_die(&mut t.g, P0);
        faces.insert(format!("{f:?}"));
    }
    for f in [PlanarFace::Blank, PlanarFace::Chaos, PlanarFace::Planeswalker] {
        assert!(faces.contains(&format!("{f:?}")), "{f:?}");
    }
}

// ---------------------------------------------------------------------------
// Sideboards
// ---------------------------------------------------------------------------

#[test]
fn constructed_sideboards_have_at_most_fifteen_cards_and_share_the_four_card_limit() {
    cr!("100.4a");
    let main = deck(&[("Grizzly Bears", 3), ("Forest", 57)]);
    assert!(check_constructed_with(&main, &deck(&[("Hill Giant", 15)]), &Default::default())
        .iter()
        .all(|p| matches!(p, DeckProblem::TooManyCopies { .. })));
    assert_eq!(
        check_constructed_with(&main, &deck(&[("Forest", 16)]), &Default::default()),
        vec![DeckProblem::SideboardTooLarge { have: 16, max: 15 }]
    );
    // Three in the deck and two in the sideboard is five.
    assert_eq!(
        check_constructed_with(&main, &deck(&[("Grizzly Bears", 2)]), &Default::default()),
        vec![DeckProblem::TooManyCopies {
            name: "Grizzly Bears".into(),
            have: 5,
            max: 4
        }]
    );
}

#[test]
fn a_limited_players_unused_pool_cards_are_their_sideboard() {
    cr!("100.4b");
    let pool = deck(&[("Grizzly Bears", 2), ("Hill Giant", 1), ("Lightning Bolt", 1)]);
    let d = deck(&[("Grizzly Bears", 1), ("Hill Giant", 1), ("Mountain", 38)]);
    let side = limited_sideboard(&pool, &d);
    let mut names: Vec<String> = side.iter().map(|c| c.name.to_string()).collect();
    names.sort();
    assert_eq!(names, vec!["Grizzly Bears", "Lightning Bolt"]);
}

#[test]
fn a_two_headed_giant_teams_unused_pool_cards_are_the_teams_sideboard() {
    cr!("100.4c");
    let pool = deck(&[("Grizzly Bears", 2), ("Hill Giant", 2), ("Lightning Bolt", 1)]);
    let a = deck(&[("Grizzly Bears", 1), ("Plains", 39)]);
    let b = deck(&[("Hill Giant", 2), ("Mountain", 38)]);
    let side = team_sideboard(&pool, &[a, b]);
    let mut names: Vec<String> = side.iter().map(|c| c.name.to_string()).collect();
    names.sort();
    assert_eq!(names, vec!["Grizzly Bears", "Lightning Bolt"]);
}

#[test]
fn other_team_variants_assign_each_unused_card_to_one_players_sideboard() {
    cr!("100.4d");
    let pool = deck(&[("Grizzly Bears", 2), ("Hill Giant", 1), ("Lightning Bolt", 1)]);
    let decks = vec![
        deck(&[("Grizzly Bears", 1), ("Forest", 39)]),
        deck(&[("Hill Giant", 1), ("Mountain", 39)]),
    ];
    let sides = vec![deck(&[("Lightning Bolt", 1)]), deck(&[("Grizzly Bears", 1)])];
    assert!(check_team_pool(&pool, &decks, &sides).is_empty());
    // The same card can't be in two sideboards, and every card must be assigned.
    let both = vec![deck(&[("Lightning Bolt", 1)]), deck(&[("Lightning Bolt", 1)])];
    assert!(check_team_pool(&pool, &decks, &both)
        .contains(&DeckProblem::PoolMismatch {
            name: "Lightning Bolt".into()
        }));
    assert!(check_team_pool(&pool, &decks, &both)
        .contains(&DeckProblem::PoolMismatch {
            name: "Grizzly Bears".into()
        }));
}

#[test]
fn there_is_no_maximum_size_for_non_commander_decks() {
    cr!("100.5");
    assert!(check_constructed(&deck(&[("Grizzly Bears", 4), ("Forest", 296)])).is_empty());
    assert!(check_limited(
        &deck(&[("Forest", 250)]),
        &deck(&[("Grizzly Bears", 1)])
    )
    .is_empty());
}

// ---------------------------------------------------------------------------
// Matches
// ---------------------------------------------------------------------------

/// Concedes at its first priority if `concede` is set; otherwise passes.
struct Quitter {
    concede: bool,
}

impl Agent for Quitter {
    fn decide(&mut self, _g: &Game, _p: PlayerId, d: &Decision) -> Answer {
        match d {
            Decision::Priority { .. } if self.concede => Answer::Action(Action::Concede),
            Decision::Priority { .. } => Answer::Action(Action::Pass),
            Decision::Mulligan { .. } => Answer::Bool(false),
            _ => Answer::Default,
        }
    }
}

fn agents(conceders: &[bool]) -> Vec<Box<dyn Agent>> {
    conceders
        .iter()
        .map(|c| Box::new(Quitter { concede: *c }) as Box<dyn Agent>)
        .collect()
}

#[test]
fn a_two_player_match_is_played_until_a_player_wins_two_games() {
    cr!("100.6a", "103.1");
    let mut m = Match::new(
        GameConfig::default(),
        vec![fillers(40), fillers(40)],
        vec![vec![], vec![]],
    );
    assert_eq!(m.games_to_win, 2);
    // Game 1: P0 concedes. Game 2: P1 concedes. Game 3: P0 concedes.
    for (i, losers) in [[true, false], [false, true], [true, false]].iter().enumerate() {
        assert!(!m.is_over());
        m.play_game(agents(losers), i as u64);
    }
    assert!(m.is_over());
    assert_eq!(m.winner(), Some(P1));
    assert_eq!(m.wins, vec![1, 2]);
    // The loser of the previous game chose who took the first turn (by default,
    // themself).
    assert_eq!(m.choosers[1], Some(P0));
    assert_eq!(m.choosers[2], Some(P1));
    assert_eq!(m.starting_players[1], P0);
    assert_eq!(m.starting_players[2], P1);
}

#[test]
fn after_a_drawn_game_the_same_player_chooses_again() {
    cr!("103.1");
    let mut m = Match::new(
        GameConfig::default(),
        vec![fillers(40), fillers(40)],
        vec![vec![], vec![]],
    );
    m.results.push(GameResult::Win(vec![P0]));
    m.choosers.push(None);
    assert_eq!(m.next_chooser(), Some(P1));
    m.results.push(GameResult::Draw);
    m.choosers.push(Some(P1));
    assert_eq!(m.next_chooser(), Some(P1));
    let g = m.next_game(agents(&[false, false]), 3);
    assert_eq!(g.config.first_turn_chooser, Some(P1));
}

#[test]
fn a_multiplayer_match_is_one_game() {
    cr!("100.6a");
    let mut m = Match::new(
        GameConfig::default(),
        vec![fillers(40), fillers(40), fillers(40)],
        vec![vec![], vec![], vec![]],
    );
    m.play_game(agents(&[true, true, false]), 1);
    assert!(m.is_over());
    assert_eq!(m.results.len(), 1);
    assert_eq!(m.winner(), Some(P2));
}

#[test]
fn players_may_modify_their_decks_with_sideboards_between_games() {
    cr!("100.4");
    let mut m = Match::new(
        GameConfig::default(),
        vec![deck(&[("Grizzly Bears", 4), ("Forest", 36)]), fillers(40)],
        vec![deck(&[("Hill Giant", 2)]), vec![]],
    );
    m.play_game(agents(&[true, false]), 0);
    assert!(m.sideboard(P0, "Grizzly Bears", "Hill Giant"));
    assert!(!m.sideboard(P0, "Lightning Bolt", "Hill Giant"));
    let g = m.next_game(agents(&[false, false]), 1);
    let named = |g: &Game, z: &[ObjectId], n: &str| {
        z.iter().filter(|id| g.obj(**id).chars.name == n).count()
    };
    assert_eq!(named(&g, &g.player(P0).library, "Hill Giant"), 1);
    assert_eq!(named(&g, &g.player(P0).library, "Grizzly Bears"), 3);
    // The sideboard is outside the game.
    assert_eq!(named(&g, &g.player(P0).sideboard, "Grizzly Bears"), 1);
    assert_eq!(named(&g, &g.player(P0).sideboard, "Hill Giant"), 1);
    for id in &g.player(P0).sideboard {
        assert_eq!(g.obj(*id).zone, Zone::Outside(P0));
    }
}
