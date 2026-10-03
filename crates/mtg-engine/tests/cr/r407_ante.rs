//! CR 407: ante, an optional variation (`GameConfig::ante`).

use crate::r100_common::*;
use crate::r703_common::{run_effect, supported};
use mtg_engine::ability::*;
use mtg_engine::ante;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::DeckProblem;
use mtg_engine::game::GameConfig;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

fn ante_config() -> GameConfig {
    GameConfig {
        ante: true,
        ..Default::default()
    }
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

fn ante_game() -> TestGame {
    TestGame::with_config(2, ante_config())
}

#[test]
fn ante_cards_are_recognized_by_their_reminder() {
    cr!("407.3");
    for name in ["Contract from Below", "Darkpact", "Demonic Attorney"] {
        supported(name);
        assert!(ante::is_ante_card(&card(name)), "{name}");
    }
    assert!(!ante::is_ante_card(&card("Grizzly Bears")));
}

#[test]
fn playing_for_ante_is_an_optional_variation() {
    cr!("407.1", "407.2");
    // Without ante, nothing is put into the ante zone.
    let mut t = pregame(
        GameConfig::default(),
        vec![distinct("A", 20), distinct("B", 20)],
    );
    t.g.start();
    assert!(t.g.ante.is_empty());
    assert_eq!(t.library_size(P0), 13);
    // Playing for ante, each player antes one card from their deck.
    let mut t = pregame(ante_config(), vec![distinct("A", 20), distinct("B", 20)]);
    t.g.start();
    assert_eq!(t.g.ante.len(), 2);
    let owners: Vec<PlayerId> = t.g.ante.iter().map(|id| t.obj(*id).owner).collect();
    assert!(owners.contains(&P0) && owners.contains(&P1));
    assert!(t.g.ante.iter().all(|id| t.zone(*id) == Zone::Ante));
}

#[test]
fn each_player_antes_a_random_card_before_drawing_and_anyone_may_examine_it() {
    cr!("407.2");
    let mut t = pregame(ante_config(), vec![distinct("A", 20), distinct("B", 20)]);
    t.g.start();
    // The anted card came from the deck before the opening hands were drawn: seven cards
    // in hand, 20 - 1 - 7 in the library.
    for p in [P0, P1] {
        assert_eq!(t.hand_size(p), 7);
        assert_eq!(t.library_size(p), 12);
    }
    // It happens before anyone draws: a player whose deck is a single card antes it and
    // draws nothing.
    let mut t = pregame(ante_config(), vec![distinct("A", 1), distinct("B", 20)]);
    t.g.start();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.g.ante.iter().any(|id| t.obj(*id).chars.name == "A 0"));
    // Cards in the ante zone may be examined by any player.
    for id in t.g.ante.clone() {
        for p in [P0, P1] {
            assert!(mtg_engine::facedown::can_look_at(&t.g, p, id));
        }
    }
    // The card is random: different games ante different cards.
    let mut anted = std::collections::BTreeSet::new();
    for seed in 0..12 {
        let mut t = pregame(
            GameConfig {
                seed,
                ..ante_config()
            },
            vec![distinct("A", 20), distinct("B", 20)],
        );
        t.g.start();
        for id in &t.g.ante {
            anted.insert(t.obj(*id).chars.name.to_string());
        }
    }
    assert!(anted.len() > 4, "anted cards: {anted:?}");
}

#[test]
fn the_winner_becomes_the_owner_of_all_the_cards_in_the_ante() {
    cr!("407.2");
    let mut t = ante_game();
    let mine = t.custom(P0, CardDef::custom(named("My Stake")), Zone::Ante);
    let theirs = t.custom(P1, CardDef::custom(named("Their Stake")), Zone::Ante);
    assert_eq!(t.obj(theirs).owner, P1);
    t.g.player_loses(P1);
    assert!(t.g.result.is_some());
    assert_eq!(t.obj(mine).owner, P0);
    assert_eq!(t.obj(theirs).owner, P0);
    // Not playing for ante, ownership doesn't change.
    let mut t = TestGame::new(2);
    let theirs = t.custom(P1, CardDef::custom(named("Their Stake")), Zone::Ante);
    t.g.player_loses(P1);
    assert_eq!(t.obj(theirs).owner, P1);
}

fn named(name: &str) -> Characteristics {
    Characteristics {
        name: SmolStr::new(name),
        rules_text: Arc::from(""),
        ..Default::default()
    }
}

#[test]
fn ante_cards_cant_be_in_decks_or_sideboards_when_not_playing_for_ante() {
    cr!("407.3");
    let deck: Vec<Arc<CardDef>> = copies("Grizzly Bears", 4)
        .into_iter()
        .chain([card("Contract from Below")])
        .collect();
    assert_eq!(
        ante::check_deck(&deck, &[], false),
        vec![DeckProblem::AnteCard {
            name: "Contract from Below".into()
        }]
    );
    assert_eq!(
        ante::check_deck(&copies("Grizzly Bears", 4), &[card("Darkpact")], false),
        vec![DeckProblem::AnteCard {
            name: "Darkpact".into()
        }]
    );
    assert!(ante::check_deck(&deck, &[card("Darkpact")], true).is_empty());
}

#[test]
fn ante_cards_cant_be_brought_into_the_game_from_outside_it_without_ante() {
    cr!("407.3");
    supported("Burning Wish");
    for playing_for_ante in [false, true] {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                ante: playing_for_ante,
                ..Default::default()
            },
        );
        t.custom(P0, (*card("Contract from Below")).clone(), Zone::Outside(P0));
        t.lands(P0, "Mountain", 2);
        let wish = t.hand(P0, "Burning Wish");
        t.cast(P0, wish).go();
        t.resolve();
        assert_eq!(
            t.in_hand(P0, "Contract from Below"),
            playing_for_ante,
            "playing for ante: {playing_for_ante}"
        );
    }
}

#[test]
fn only_ante_cards_put_cards_into_the_ante_zone() {
    cr!("407.3", "407.4");
    let mut t = ante_game();
    let top = t.library_top(P0, "Grizzly Bears");
    // A card without the ante reminder can't ante a card...
    let other = t.battlefield(P0, "Hill Giant");
    run_effect(
        &mut t,
        P0,
        Some(other),
        Effect::Custom(ante::ANTE_TOP.into()),
        &[],
    );
    assert_eq!(t.zone(top), Zone::Library(P0));
    assert!(t.g.ante.is_empty());
    // ...but Contract from Below can: "Discard your hand, ante the top card of your
    // library, then draw seven cards."
    t.lands(P0, "Swamp", 1);
    let contract = t.hand(P0, "Contract from Below");
    t.cast(P0, contract).go();
    t.resolve();
    assert_eq!(t.zone(top), Zone::Ante);
    assert_eq!(t.g.ante.len(), 1);
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn only_ante_cards_take_cards_out_of_the_ante_zone_or_change_owners() {
    cr!("407.3");
    supported("Darkpact");
    let mut t = ante_game();
    let stake = t.custom(P1, CardDef::custom(named("Their Stake")), Zone::Ante);
    // Another effect can't move a card out of the ante zone.
    let other = t.battlefield(P0, "Hill Giant");
    run_effect(
        &mut t,
        P0,
        Some(other),
        Effect::Move {
            what: Sel::All(Filter::InZone(ZoneKind::Ante)),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[],
    );
    assert_eq!(t.zone(stake), Zone::Ante);
    // Nor make a player own a card in it.
    run_effect(
        &mut t,
        P0,
        Some(other),
        Effect::Seq(vec![
            Effect::Store {
                var: vars::IT,
                sel: Sel::All(Filter::InZone(ZoneKind::Ante)),
            },
            Effect::Custom(ante::GAIN_OWNERSHIP.into()),
        ]),
        &[],
    );
    assert_eq!(t.obj(stake).owner, P1);
    // Darkpact: "You own target card in the ante. Exchange that card with the top card of
    // your library."
    let my_top = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let pact = t.hand(P0, "Darkpact");
    t.cast(P0, pact).target(stake).go();
    t.resolve();
    let now = t.g.current(stake);
    assert_eq!(t.obj(now).owner, P0);
    assert_eq!(t.g.library_top(P0), Some(now));
    assert_eq!(t.zone(my_top), Zone::Ante);
}

#[test]
fn only_the_owner_of_an_object_can_ante_it() {
    cr!("407.4");
    supported("Demonic Attorney");
    let mut t = ante_game();
    let mine = t.library_top(P0, "Grizzly Bears");
    let theirs = t.library_top(P1, "Hill Giant");
    // Anteing another player's card does nothing, even for an ante card.
    let attorney = t.hand(P0, "Demonic Attorney");
    assert_eq!(ante::ante(&mut t.g, P0, theirs, Some(attorney)), None);
    assert_eq!(t.zone(theirs), Zone::Library(P1));
    // "Each player antes the top card of their library": each owner antes their own.
    t.lands(P0, "Swamp", 3);
    t.cast(P0, attorney).go();
    t.resolve();
    assert_eq!(t.zone(mine), Zone::Ante);
    assert_eq!(t.zone(theirs), Zone::Ante);
    assert_eq!(t.obj_now(theirs).owner, P1);
}

