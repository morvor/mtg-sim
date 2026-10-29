//! CR 903.12: the Brawl option of the Commander variant.

use crate::r100_common::{fillers, pregame};
use crate::r703_common::run_effect;
use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::{check_commander, check_format_legality, DeckProblem};
use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

/// A 60-card Brawl deck: `commander` and 59 copies of `land`.
fn brawl_deck(commander: &str, land: &str) -> Vec<Arc<CardDef>> {
    std::iter::once(card(commander))
        .chain((0..59).map(|_| card(land)))
        .collect()
}

/// A Brawl game (not started) of `n` players; P0's commander is Beza, the Bounding
/// Spring.
fn brawl_pregame(n: usize, mulligans: bool) -> TestGame {
    let mut decks = vec![brawl_deck("Beza, the Bounding Spring", "Plains")];
    decks.extend((1..n).map(|_| fillers(60)));
    let mut t = pregame(
        GameConfig {
            skip_mulligans: !mulligans,
            starting_player: Some(P0),
            ..GameConfig::brawl_game()
        },
        decks,
    );
    assert!(t.g.designate_commander(P0, "Beza, the Bounding Spring"));
    t
}

fn make_commander(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let name = t.obj(id).card.as_ref().unwrap().name.clone();
    let owner = t.obj(id).owner;
    t.g.players[owner.idx()].commander_names.push(name);
}

#[test]
fn brawl_is_a_style_of_commander_game() {
    cr!("903.12");
    let c = GameConfig::brawl_game();
    assert!(c.brawl);
    let mut t = brawl_pregame(2, false);
    assert!(t.g.is_brawl());
    t.g.start();
    // A Commander game: the commander starts in the command zone.
    assert_eq!(t.g.find_in_zone(Zone::Command, "Beza, the Bounding Spring").len(), 1);
    // A non-Brawl Commander game isn't a Brawl game.
    let t = TestGame::with_config(2, GameConfig::commander_game());
    assert!(!t.g.is_brawl());
}

#[test]
fn brawl_uses_the_commander_rules_as_modified() {
    cr!("903.12a");
    let mut t = TestGame::with_config(2, GameConfig::brawl_game());
    // The commander tax (CR 903.8).
    let beza = t.command(P0, "Beza, the Bounding Spring");
    make_commander(&mut t, beza);
    assert_eq!(mtg_engine::kw::partner::commander_tax(&t.g, P0, beza), 0);
    t.lands(P0, "Plains", 4);
    t.cast(P0, beza).go();
    t.resolve_all();
    let on_bf = t.named_on_battlefield("Beza, the Bounding Spring")[0];
    // Returning to the command zone instead of the hand (CR 903.9b).
    t.answer_yes(P0, true);
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(on_bf)],
    );
    let back = t.g.find_in_zone(Zone::Command, "Beza, the Bounding Spring");
    assert_eq!(back.len(), 1);
    assert_eq!(mtg_engine::kw::partner::commander_tax(&t.g, P0, back[0]), 2);
    // But not the commander damage rule (CR 903.12h): 21 combat damage from a commander
    // doesn't make a player lose.
    t.g.players[1].commander_damage.insert("Beza, the Bounding Spring".into(), 25);
    t.settle();
    assert!(!t.has_lost(P1));
}

#[test]
fn brawl_decks_usually_use_standard_cards() {
    cr!("903.12b");
    // A deck of cards from the Standard format is legal there; the format's card
    // restrictions can be checked along with the Brawl deck rules.
    let beza = card("Beza, the Bounding Spring");
    let mut d = brawl_deck("Beza, the Bounding Spring", "Plains");
    d[1] = card("Savannah Lions");
    assert!(check_commander(&d, &beza, &[], true).is_empty());
    assert!(check_format_legality(&d, &[], "standard").is_empty());
    // Isamaru, Hound of Konda isn't a Standard card.
    d[2] = card("Isamaru, Hound of Konda");
    assert!(check_commander(&d, &beza, &[], true).is_empty());
    assert!(check_format_legality(&d, &[], "standard")
        .iter()
        .any(|p| matches!(p, DeckProblem::NotLegalInFormat { name, .. } if name == "Isamaru, Hound of Konda")));
}

#[test]
fn a_colorless_brawl_commander_allows_basic_lands_of_one_type() {
    cr!("903.12e");
    let karn = card("Karn, Legacy Reforged");
    // Any number of basic lands of one basic land type.
    let d = brawl_deck("Karn, Legacy Reforged", "Plains");
    assert!(check_commander(&d, &karn, &[], true).is_empty());
    // Not two types.
    let mut d = brawl_deck("Karn, Legacy Reforged", "Plains");
    for c in d.iter_mut().skip(30) {
        *c = card("Island");
    }
    assert!(check_commander(&d, &karn, &[], true)
        .iter()
        .any(|p| matches!(p, DeckProblem::BasicLandTypes { .. })));
    // Outside Brawl, basic lands must still fit the color identity (CR 903.5c, 903.5d).
    let d: Vec<Arc<CardDef>> = std::iter::once(card("Karn, Legacy Reforged"))
        .chain((0..99).map(|_| card("Plains")))
        .collect();
    assert!(!check_commander(&d, &karn, &[], false).is_empty());
    // A commander with a color doesn't get the exception.
    let beza = card("Beza, the Bounding Spring");
    let mut d = brawl_deck("Beza, the Bounding Spring", "Plains");
    d[1] = card("Island");
    assert!(!check_commander(&d, &beza, &[], true).is_empty());
}

#[test]
fn brawl_starting_life_totals() {
    cr!("903.12f");
    let mut t = brawl_pregame(2, false);
    t.g.start();
    assert_eq!((t.life(P0), t.life(P1)), (25, 25));
    let mut t = brawl_pregame(4, false);
    t.g.start();
    assert_eq!((t.life(P0), t.life(P3)), (30, 30));
}

fn mulligans_asked(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::Mulligan { .. }))
        .count()
}

#[test]
fn the_first_brawl_mulligan_is_free() {
    cr!("903.12g");
    // Two players: the first mulligan draws seven cards and puts none on the bottom.
    let mut t = brawl_pregame(2, true);
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P0), 7);
    // The second one counts as the first: one card on the bottom.
    let mut t = brawl_pregame(2, true);
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P0), 6);
    // It doesn't count toward the number of mulligans either: a player who keeps
    // mulliganing may take one more than in a two-player Commander game.
    let mut t = brawl_pregame(2, true);
    for _ in 0..10 {
        t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    }
    t.g.start();
    assert_eq!(mulligans_asked(&t, P0), 8);
    let mut t = pregame(
        GameConfig {
            starting_player: Some(P0),
            ..GameConfig::commander_game()
        },
        vec![fillers(100), fillers(100)],
    );
    for _ in 0..10 {
        t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    }
    t.g.start();
    assert_eq!(mulligans_asked(&t, P0), 7);
}
