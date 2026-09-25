//! CR 103: starting the game — the starting player, the additional steps before shuffling,
//! starting life totals and hands, mulligans, and the first turn.

use crate::r100_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::Decision;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::start::StickerSheet;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

fn config() -> GameConfig {
    GameConfig::default()
}

/// A deck of `n` distinctly named vanilla cards.
fn distinct(prefix: &str, n: usize) -> Vec<Arc<CardDef>> {
    (0..n)
        .map(|i| {
            Arc::new(CardDef::custom(Characteristics {
                name: SmolStr::new(format!("{prefix} {i}")),
                rules_text: Arc::from(""),
                ..Default::default()
            }))
        })
        .collect()
}

/// Runs a started game until `active`'s precombat main phase begins with priority.
fn to_main(t: &mut TestGame, active: PlayerId) {
    let ok = t.g.run_until(2000, |g| {
        g.turn.active == active
            && g.turn.step == Step::PrecombatMain
            && g.turn.stage == Stage::Priority
    });
    assert!(ok, "did not reach {active}'s main phase");
}

// ---------------------------------------------------------------------------
// CR 103.1: the starting player
// ---------------------------------------------------------------------------

#[test]
fn a_chosen_player_chooses_who_takes_the_first_turn() {
    cr!("103.1");
    // In a match, the loser of the previous game (here P1) chooses.
    let mut t = pregame(
        GameConfig {
            first_turn_chooser: Some(P1),
            ..config()
        },
        vec![fillers(20), fillers(20)],
    );
    t.answer_choose(P1, &[Entity::Player(P0)]);
    t.g.start();
    assert_eq!(t.g.start.chooser, Some(P1));
    assert_eq!(t.g.turn.starting_player, P0);
    // Left to the default, the chooser takes the first turn themself.
    let mut t = pregame(
        GameConfig {
            first_turn_chooser: Some(P1),
            ..config()
        },
        vec![fillers(20), fillers(20)],
    );
    t.g.start();
    assert_eq!(t.g.turn.starting_player, P1);
    // Otherwise the chooser is determined at random.
    let mut choosers = std::collections::BTreeSet::new();
    for seed in 0..20 {
        let mut t = pregame(
            GameConfig {
                seed,
                ..config()
            },
            vec![fillers(20), fillers(20)],
        );
        t.g.start();
        choosers.insert(t.g.start.chooser.unwrap());
        assert_eq!(Some(t.g.turn.starting_player), t.g.start.chooser);
    }
    assert_eq!(choosers.len(), 2);
}

#[test]
fn the_archenemy_takes_the_first_turn() {
    cr!("103.1b");
    for seed in 0..6 {
        let mut deck2 = fillers(20);
        deck2.push(card("What's Yours Is Now Mine"));
        let mut t = pregame(
            GameConfig {
                variant: Variant::Archenemy,
                seed,
                ..config()
            },
            vec![fillers(20), fillers(20), deck2],
        );
        t.g.start();
        assert_eq!(t.g.archenemy(), Some(P2));
        assert_eq!(t.g.turn.starting_player, P2);
        assert_eq!(t.g.turn.active, P2);
    }
}

#[test]
fn power_play_makes_its_controller_the_starting_player() {
    cr!("103.1c", "103.2e");
    // "You are the starting player. If multiple players would be the starting player, one
    // of those players is chosen at random."
    let mut t = pregame(
        GameConfig {
            first_turn_chooser: Some(P0),
            ..config()
        },
        vec![fillers(20), fillers(20)],
    );
    t.g.add_to_sideboard(P1, vec![card("Power Play")]);
    t.g.start();
    // P0 chose themself, but Power Play supersedes that.
    assert_eq!(t.g.turn.starting_player, P1);
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(t.g.turn.number, 1);
    // Several Power Plays: one of those players at random, never anyone else.
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..16 {
        let mut t = pregame(
            GameConfig {
                seed,
                first_turn_chooser: Some(P0),
                ..config()
            },
            vec![fillers(20), fillers(20), fillers(20)],
        );
        t.g.add_to_sideboard(P1, vec![card("Power Play")]);
        t.g.add_to_sideboard(P2, vec![card("Power Play")]);
        t.g.start();
        seen.insert(t.g.turn.starting_player);
    }
    assert_eq!(seen, [P1, P2].into_iter().collect());
}

// ---------------------------------------------------------------------------
// CR 103.2: additional steps
// ---------------------------------------------------------------------------

#[test]
fn sideboards_are_set_aside_and_the_deck_is_the_starting_deck() {
    cr!("103.2", "103.2a");
    let mut t = pregame(config(), vec![distinct("Main", 20), fillers(20)]);
    let side = t.g.add_to_sideboard(P0, distinct("Side", 5));
    t.g.start();
    // The starting deck is the 20 main-deck cards; the sideboard stays outside the game.
    assert_eq!(t.g.start.starting_decks[&P0].len(), 20);
    for s in side {
        assert_eq!(t.g.obj(s).zone, Zone::Outside(P0));
    }
    let in_game: Vec<String> = t
        .g
        .player(P0)
        .library
        .iter()
        .chain(t.g.player(P0).hand.iter())
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect();
    assert_eq!(in_game.len(), 20);
    assert!(in_game.iter().all(|n| n.starts_with("Main")));
}

#[test]
fn a_companion_can_be_revealed_only_if_the_starting_deck_fulfills_its_condition() {
    cr!("103.2b");
    // Lurrus: "Companion — Each permanent card in your starting deck has mana value 2 or
    // less."
    let cheap = || {
        let mut d = copies("Grizzly Bears", 10);
        d.extend(copies("Forest", 10));
        d.extend(copies("Giant Growth", 5));
        d
    };
    let mut t = pregame(config(), vec![cheap(), fillers(20)]);
    let side = t.g.add_to_sideboard(P0, vec![card("Lurrus of the Dream-Den"), card("Hill Giant")]);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.g.start();
    assert_eq!(t.g.start.companions.get(&P0), Some(&side[0]));
    // The revealed card remains outside the game.
    assert_eq!(t.g.obj(side[0]).zone, Zone::Outside(P0));
    // A deck with a three-mana permanent doesn't fulfill it: no reveal is offered.
    let mut deck = cheap();
    deck.push(card("Hill Giant"));
    let mut t = pregame(config(), vec![deck, fillers(20)]);
    let side = t.g.add_to_sideboard(P0, vec![card("Lurrus of the Dream-Den")]);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.g.start();
    assert!(t.g.start.companions.is_empty());
    // At most one companion.
    let mut t = pregame(config(), vec![cheap(), fillers(20)]);
    let side = t.g.add_to_sideboard(
        P0,
        vec![card("Lurrus of the Dream-Den"), card("Lurrus of the Dream-Den")],
    );
    t.answer_choose(P0, &[Entity::Object(side[0]), Entity::Object(side[1])]);
    t.g.start();
    assert!(t.g.start.companions.len() <= 1);
}

#[test]
fn commanders_start_face_up_in_the_command_zone() {
    cr!("103.2c", "103.4c");
    let mut t = pregame(
        GameConfig {
            variant: Variant::Commander,
            ..config()
        },
        vec![
            {
                let mut d = fillers(30);
                d.push(card("Isamaru, Hound of Konda"));
                d
            },
            fillers(30),
        ],
    );
    assert!(t.g.designate_commander(P0, "Isamaru, Hound of Konda"));
    t.g.start();
    let cmdr = t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda");
    assert_eq!(cmdr.len(), 1);
    assert!(!t.g.obj(cmdr[0]).face_down);
    assert!(t.g.obj(cmdr[0]).is_commander);
    assert_eq!(t.g.player(P0).library.len() + t.g.player(P0).hand.len(), 30);
    assert_eq!(t.life(P0), 40);
    assert_eq!(t.life(P1), 40);
}

fn sheets(n: usize) -> Vec<StickerSheet> {
    (0..n)
        .map(|i| StickerSheet {
            name: SmolStr::new(format!("Sheet {i}")),
            stickers: vec![],
        })
        .collect()
}

#[test]
fn constructed_players_reveal_all_sticker_sheets_and_choose_three_at_random() {
    cr!("103.2d");
    let mut picks = std::collections::BTreeSet::new();
    for seed in 0..8 {
        let mut t = pregame(
            GameConfig {
                seed,
                ..config()
            },
            vec![fillers(20), fillers(20)],
        );
        t.g.start.sticker_sheets.insert(P0, sheets(10));
        t.g.start();
        assert_eq!(t.g.start.revealed_sticker_sheets[&P0].len(), 10);
        let chosen = t.g.start.chosen_sticker_sheets[&P0].clone();
        assert_eq!(chosen.len(), 3);
        assert_eq!(t.g.accessible_sticker_sheets(P0).len(), 3);
        picks.insert(chosen);
        // A player without sticker sheets has none.
        assert!(t.g.accessible_sticker_sheets(P1).is_empty());
    }
    assert!(picks.len() > 1, "chosen at random");
}

#[test]
fn limited_players_choose_up_to_three_sticker_sheets_and_reveal_them() {
    cr!("103.2d");
    let mut t = pregame(
        GameConfig {
            limited: true,
            ..config()
        },
        vec![fillers(40), fillers(40)],
    );
    t.g.start.sticker_sheets.insert(P0, sheets(5));
    // Choose "Sheet 3", then "Sheet 0", then stop.
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.g.start();
    assert_eq!(t.g.start.chosen_sticker_sheets[&P0], vec![3, 0]);
    assert_eq!(t.g.start.revealed_sticker_sheets[&P0], vec![3, 0]);
    let names: Vec<String> = t
        .g
        .accessible_sticker_sheets(P0)
        .iter()
        .map(|s| s.name.to_string())
        .collect();
    assert_eq!(names, vec!["Sheet 3", "Sheet 0"]);
}

#[test]
fn conspiracies_go_from_the_sideboard_to_the_command_zone() {
    cr!("103.2e");
    let mut t = pregame(
        GameConfig {
            limited: true,
            ..config()
        },
        vec![fillers(40), fillers(40)],
    );
    // A conspiracy listed with the deck isn't part of it (CR 315.3).
    let mut t2 = pregame(
        config(),
        vec![
            {
                let mut d = fillers(40);
                d.push(card("Power Play"));
                d
            },
            fillers(40),
        ],
    );
    assert_eq!(t2.g.player(P0).library.len(), 40);
    assert_eq!(t2.g.player(P0).sideboard.len(), 1);
    t2.g.start();
    assert_eq!(t2.g.find_in_zone(Zone::Command, "Power Play").len(), 1);
    // Players may put any number of them into the command zone; hidden agenda ones go face
    // down.
    let side = t.g.add_to_sideboard(
        P0,
        vec![
            card("Double Stroke"),
            card("Power Play"),
            card("Brago's Favor"),
        ],
    );
    t.answer_choose(P0, &[Entity::Object(side[0]), Entity::Object(side[1])]);
    t.g.start();
    assert_eq!(t.g.obj(side[0]).zone, Zone::Command);
    assert!(t.g.obj(side[0]).face_down, "hidden agenda");
    assert_eq!(t.g.obj(side[1]).zone, Zone::Command);
    assert!(!t.g.obj(side[1]).face_down);
    assert_eq!(t.g.obj(side[2]).zone, Zone::Outside(P0));
}

// ---------------------------------------------------------------------------
// CR 103.3: shuffling
// ---------------------------------------------------------------------------

#[test]
fn each_players_deck_is_shuffled_and_becomes_their_library() {
    cr!("103.3", "100.2");
    let mut t = pregame(
        GameConfig {
            skip_mulligans: true,
            ..config()
        },
        vec![distinct("A", 30), distinct("B", 30)],
    );
    let before: Vec<String> = names(&t, &t.g.player(P0).library.clone());
    t.g.start();
    let p0 = t.g.player(P0);
    let mut after: Vec<ObjectId> = p0.library.clone();
    after.extend(p0.hand.iter().copied());
    let after_names = names(&t, &after);
    assert_ne!(after_names, before, "the order changed");
    let mut a = after_names.clone();
    let mut b = before.clone();
    a.sort();
    b.sort();
    assert_eq!(a, b, "the same cards");
    // Each player's cards are their own.
    for id in after {
        assert_eq!(t.g.obj(id).owner, P0);
    }
}

#[test]
fn supplementary_decks_are_shuffled_too() {
    cr!("103.3a");
    let planes = [
        "The Great Aerie",
        "Strixhaven",
        "The Windy City",
        "Shy Town",
        "The Pro Tour",
        "Horizon Boughs",
    ];
    let mut orders = std::collections::BTreeSet::new();
    for seed in 0..6 {
        let deck: Vec<Arc<CardDef>> = fillers(20)
            .into_iter()
            .chain(planes.iter().map(|n| card(n)))
            .collect();
        let mut t = pregame(
            GameConfig {
                variant: Variant::Planechase,
                starting_player: Some(P1),
                skip_mulligans: true,
                seed,
                ..config()
            },
            vec![deck, fillers(20)],
        );
        t.g.start();
        // Face-down cards have no characteristics; compare the objects' order.
        let order = mtg_engine::planechase::planar_deck(&t.g, P0);
        assert_eq!(order.len(), planes.len());
        orders.insert(order);
    }
    assert!(orders.len() > 1);
}

// ---------------------------------------------------------------------------
// CR 103.4: starting life totals
// ---------------------------------------------------------------------------

fn started(config: GameConfig, n: usize) -> TestGame {
    let mut t = pregame(
        GameConfig {
            skip_mulligans: true,
            ..config
        },
        (0..n).map(|_| fillers(40)).collect(),
    );
    t.g.start();
    t
}

#[test]
fn each_player_starts_at_twenty_life() {
    cr!("103.4");
    let t = started(config(), 2);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn two_headed_giant_teams_start_at_thirty_life() {
    cr!("103.4a");
    let t = started(
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..config()
        },
        4,
    );
    assert_eq!(t.g.starting_life(P0), 30);
    assert_eq!(t.life(P2), 30);
}

#[test]
fn a_vanguard_modifies_starting_life_and_hand_size() {
    cr!("103.4b", "103.5a");
    // Titania (vanguard): hand modifier +2, life modifier -5.
    let mut deck = fillers(40);
    deck.push(card("Titania"));
    let mut t = pregame(
        GameConfig {
            variant: Variant::Vanguard,
            skip_mulligans: true,
            ..config()
        },
        vec![deck, fillers(40)],
    );
    t.g.start();
    assert_eq!(t.life(P0), 15);
    assert_eq!(t.hand_size(P0), 9);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), 7);
    // A mulligan draws a new hand of the starting hand size (CR 902.5a).
    let mut deck = fillers(40);
    deck.push(card("Titania"));
    let mut t = pregame(
        GameConfig {
            variant: Variant::Vanguard,
            starting_player: Some(P0),
            ..config()
        },
        vec![deck, fillers(40)],
    );
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P0), 8, "nine cards, one on the bottom");
}

#[test]
fn commander_brawl_and_archenemy_starting_life() {
    cr!("103.4c", "103.4d", "103.4e");
    let t = started(
        GameConfig {
            variant: Variant::Commander,
            ..config()
        },
        4,
    );
    assert_eq!(t.life(P3), 40);
    let t = started(
        GameConfig {
            variant: Variant::Commander,
            brawl: true,
            ..config()
        },
        2,
    );
    assert_eq!((t.life(P0), t.life(P1)), (25, 25));
    let t = started(
        GameConfig {
            variant: Variant::Commander,
            brawl: true,
            ..config()
        },
        3,
    );
    assert_eq!(t.life(P2), 30);
    let mut deck = fillers(40);
    deck.push(card("What's Yours Is Now Mine"));
    let mut t = pregame(
        GameConfig {
            variant: Variant::Archenemy,
            skip_mulligans: true,
            ..config()
        },
        vec![fillers(40), deck, fillers(40)],
    );
    t.g.start();
    assert_eq!(t.life(P1), 40, "the archenemy");
    assert_eq!((t.life(P0), t.life(P2)), (20, 20));
}

// ---------------------------------------------------------------------------
// CR 103.5: opening hands and mulligans
// ---------------------------------------------------------------------------

#[test]
fn london_mulligan() {
    cr!("103.5");
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P1),
            ..config()
        },
        vec![distinct("A", 40), distinct("B", 40)],
    );
    // P1 (starting) mulligans twice; P0 once.
    t.answer(P1, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer(P1, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P1), 5);
    assert_eq!(t.hand_size(P0), 6);
    assert_eq!(t.g.player(P1).mulligans, 2);
    assert_eq!(t.library_size(P1), 35);
    // The starting player declares first, then the others; everyone who mulligans does so
    // before anyone declares again; kept hands aren't asked about again.
    let asked: Vec<(PlayerId, &'static str)> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::Mulligan { .. } => Some((p, "declare")),
            Decision::PutOnBottom { .. } => Some((p, "bottom")),
            _ => None,
        })
        .collect();
    assert_eq!(
        asked,
        vec![
            (P1, "declare"),
            (P0, "declare"),
            (P1, "bottom"),
            (P0, "bottom"),
            (P1, "declare"),
            (P0, "declare"),
            (P1, "bottom"),
            (P1, "declare"),
        ]
    );
}

#[test]
fn a_player_can_mulligan_until_their_opening_hand_would_be_empty() {
    cr!("103.5");
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P0),
            ..config()
        },
        vec![fillers(40), fillers(40)],
    );
    for _ in 0..10 {
        t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    }
    t.g.start();
    assert_eq!(t.g.player(P0).mulligans, 7);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn actions_any_time_a_player_could_mulligan() {
    cr!("103.5b");
    // Serum Powder: "Any time you could mulligan and this card is in your hand, you may
    // exile all the cards from your hand, then draw that many cards."
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P0),
            ..config()
        },
        vec![copies("Serum Powder", 40), fillers(40)],
    );
    // First declaration: use Serum Powder (then P0 still declares), then mulligan; after
    // the mulligan, use it again (not in the first round).
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer_yes(P0, true);
    t.g.start();
    assert_eq!(t.g.exile.len(), 7 + 6);
    assert_eq!(t.hand_size(P0), 6);
    let asked: Vec<&'static str> = t
        .asked()
        .into_iter()
        .filter(|(p, _)| *p == P0)
        .filter_map(|(_, d)| match d {
            Decision::YesNo { .. } => Some("powder"),
            Decision::Mulligan { .. } => Some("declare"),
            _ => None,
        })
        .take(4)
        .collect();
    assert_eq!(asked, vec!["powder", "declare", "powder", "declare"]);
}

#[test]
fn the_first_mulligan_is_free_in_multiplayer_and_brawl_games() {
    cr!("103.5c", "100.1b");
    // A game that begins with three players is a multiplayer game.
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P0),
            ..config()
        },
        vec![fillers(40), fillers(40), fillers(40)],
    );
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer(P1, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer(P1, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P0), 7, "the first mulligan is free");
    assert_eq!(t.hand_size(P1), 6, "the second one counts");
    // Two-player Brawl: also free.
    let mut t = pregame(
        GameConfig {
            variant: Variant::Commander,
            brawl: true,
            starting_player: Some(P0),
            ..config()
        },
        vec![fillers(40), fillers(40)],
    );
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P0), 7);
    // Two-player Commander without Brawl: not free.
    let mut t = pregame(
        GameConfig {
            variant: Variant::Commander,
            starting_player: Some(P0),
            ..config()
        },
        vec![fillers(40), fillers(40)],
    );
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P0), 6);
    // Nor in an ordinary two-player game.
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P0),
            ..config()
        },
        vec![fillers(40), fillers(40)],
    );
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P0), 6);
}

// ---------------------------------------------------------------------------
// CR 103.6–103.8
// ---------------------------------------------------------------------------

#[test]
fn opening_hand_actions_start_with_the_starting_player() {
    cr!("103.6");
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P1),
            skip_mulligans: true,
            ..config()
        },
        vec![
            copies("Leyline of Sanctity", 20),
            copies("Leyline of Sanctity", 20),
        ],
    );
    t.g.start();
    let order: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(order.first(), Some(&P1));
    assert_eq!(order.last(), Some(&P0));
}

#[test]
fn the_starting_player_turns_the_starting_plane_face_up() {
    cr!("103.7");
    for seed in 0..6 {
        let deck: Vec<Arc<CardDef>> = fillers(20)
            .into_iter()
            .chain(["Chaotic Aether", "Interplanar Tunnel", "Strixhaven"].map(card))
            .collect();
        let mut other: Vec<Arc<CardDef>> = fillers(20);
        other.push(card("The Great Aerie"));
        let mut t = pregame(
            GameConfig {
                variant: Variant::Planechase,
                starting_player: Some(P0),
                skip_mulligans: true,
                seed,
                ..config()
            },
            vec![deck, other],
        );
        t.g.start();
        // Phenomena went to the bottom until a plane was turned face up — from the starting
        // player's planar deck.
        let up = mtg_engine::planechase::face_up_planar_cards(&t.g);
        assert_eq!(names(&t, &up), vec!["Strixhaven"], "seed {seed}");
        let deck = mtg_engine::planechase::planar_deck(&t.g, P0);
        assert_eq!(deck.len(), 2);
    }
}

#[test]
fn the_starting_player_takes_the_first_turn() {
    cr!("103.8");
    let t = started(
        GameConfig {
            starting_player: Some(P1),
            ..config()
        },
        2,
    );
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(t.g.turn.number, 1);
}

#[test]
fn in_a_two_player_game_the_starting_player_skips_their_first_draw_step() {
    cr!("103.8a", "100.1a");
    let mut deck = fillers(40);
    deck.push(card("Howling Mine"));
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P0),
            skip_mulligans: true,
            ..config()
        },
        vec![fillers(40), fillers(40)],
    );
    t.g.start();
    // Howling Mine ("At the beginning of each player's draw step, ...") would trigger if
    // there were a draw step.
    let mine = t.g.create_card_object(card("Howling Mine"), P0, Zone::Battlefield);
    t.g.battlefield.push(mine);
    to_main(&mut t, P0);
    assert!(!t.g.turn.step_log.contains(&Step::Draw));
    assert_eq!(t.hand_size(P0), 7);
    to_main(&mut t, P1);
    assert!(t.g.turn.step_log.contains(&Step::Draw));
    assert_eq!(t.hand_size(P1), 9, "draw step draw plus Howling Mine");
}

#[test]
fn in_other_multiplayer_games_no_one_skips_their_first_draw() {
    cr!("103.8c", "100.1b");
    let mut t = started(
        GameConfig {
            starting_player: Some(P0),
            ..config()
        },
        3,
    );
    // Even if a player leaves before the first draw step, the game began with three
    // players: it's still a multiplayer game.
    t.g.player_loses(P2);
    to_main(&mut t, P0);
    assert!(t.g.turn.step_log.contains(&Step::Draw));
    assert_eq!(t.hand_size(P0), 8);
}
