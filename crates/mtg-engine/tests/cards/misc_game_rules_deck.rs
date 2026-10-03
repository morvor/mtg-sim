//! Abilities that modify the deck construction rules (CR 113.6n), compiled by
//! `oracle/patterns/misc_game_rules_deck.rs`: "[This card] can be your commander."
//! (CR 903.3a) — and which cards can be commanders otherwise (CR 903.3, 903.12c) — and
//! "A deck can have any number of cards named ~." on a sorcery.

use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::{check_commander, check_constructed, max_copies, DeckProblem};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::kw::partner::can_be_commander;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;
use std::sync::Arc;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// A Commander deck: the commander plus `n` copies of `land`.
fn deck(commander: &Arc<CardDef>, land: &str, n: usize) -> Vec<Arc<CardDef>> {
    std::iter::once(commander.clone())
        .chain((0..n).map(|_| card(land)))
        .collect()
}

fn invalid_commander(problems: &[DeckProblem]) -> bool {
    problems
        .iter()
        .any(|p| matches!(p, DeckProblem::InvalidCommanders { .. }))
}

#[test]
fn a_planeswalker_that_says_it_can_be_your_commander_can_lead_a_deck() {
    cr!("903.3", "903.3a", "113.6n");
    compiles("Freyalise, Llanowar's Fury");
    let freyalise = card("Freyalise, Llanowar's Fury");
    assert!(can_be_commander(&freyalise, false));
    assert!(check_commander(&deck(&freyalise, "Forest", 99), &freyalise, &[], false).is_empty());
    // Another legendary planeswalker can't (outside Brawl).
    let jace = card("Jace Beleren");
    assert!(!can_be_commander(&jace, false));
    assert!(invalid_commander(&check_commander(
        &deck(&jace, "Island", 99),
        &jace,
        &[],
        false
    )));
}

#[test]
fn a_commander_is_a_legendary_creature_vehicle_or_spacecraft_card() {
    cr!("903.3");
    let isamaru = card("Isamaru, Hound of Konda");
    assert!(can_be_commander(&isamaru, false));
    // A legendary Vehicle card.
    let parhelion = card("Parhelion II");
    assert!(can_be_commander(&parhelion, false));
    assert!(check_commander(&deck(&parhelion, "Plains", 99), &parhelion, &[], false).is_empty());
    // A nonlegendary creature card can't be a commander.
    let bears = card("Grizzly Bears");
    assert!(!can_be_commander(&bears, false));
    assert!(invalid_commander(&check_commander(
        &deck(&bears, "Forest", 99),
        &bears,
        &[],
        false
    )));
    // Nor can a legendary noncreature card without such an ability.
    let mox = card("Mox Amber");
    assert!(!can_be_commander(&mox, false));
    // A Vehicle must be legendary too.
    assert!(!can_be_commander(&card("Smuggler's Copter"), false));
    // A legendary Spacecraft card only with a power/toughness box.
    assert!(can_be_commander(&card("The Seriema"), false));
    assert!(!can_be_commander(&card("The Eternity Elevator"), false));
}

#[test]
fn brawl_commanders_may_be_planeswalkers() {
    cr!("903.12c", "903.12d");
    let jace = card("Jace Beleren");
    assert!(can_be_commander(&jace, true));
    assert!(check_commander(&deck(&jace, "Island", 59), &jace, &[], true).is_empty());
}

/// A game not yet started, with these decks.
fn pregame(config: GameConfig, decks: Vec<Vec<Arc<CardDef>>>) -> TestGame {
    use std::collections::VecDeque;
    use std::sync::Mutex;
    let n = decks.len();
    let script = Arc::new(Mutex::new(Script {
        queues: vec![VecDeque::new(); n],
        asked: vec![],
    }));
    let agents: Vec<Box<dyn mtg_engine::decision::Agent>> = (0..n)
        .map(|i| {
            Box::new(ScriptedAgent {
                player: PlayerId(i as u8),
                script: script.clone(),
            }) as Box<dyn mtg_engine::decision::Agent>
        })
        .collect();
    let g = mtg_engine::game::Game::new(config, decks, agents);
    TestGame { g, script }
}

#[test]
fn the_planeswalker_commander_starts_in_the_command_zone() {
    cr!("903.6");
    let freyalise = card("Freyalise, Llanowar's Fury");
    let mut t = pregame(
        GameConfig {
            variant: Variant::Commander,
            skip_mulligans: true,
            ..Default::default()
        },
        vec![deck(&freyalise, "Forest", 39), deck(&card("Isamaru, Hound of Konda"), "Plains", 39)],
    );
    assert!(t.g.designate_commander(P0, "Freyalise, Llanowar's Fury"));
    t.g.start();
    let ids = t.g.find_in_zone(Zone::Command, "Freyalise, Llanowar's Fury");
    assert_eq!(ids.len(), 1);
    assert!(t.g.obj(ids[0]).is_commander);
}

#[test]
fn the_planeswalker_commander_is_cast_from_the_command_zone() {
    cr!("903.8");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    );
    let freyalise = t.command(P0, "Freyalise, Llanowar's Fury");
    t.g.objects[freyalise.0 as usize].is_commander = true;
    t.g.players[P0.idx()]
        .commander_names
        .push("Freyalise, Llanowar's Fury".into());
    // {3}{G}{G}
    t.lands(P0, "Forest", 5);
    t.cast(P0, freyalise).go();
    t.resolve_all();
    let now = t.g.current(freyalise);
    assert!(t.on_battlefield(now));
    assert_eq!(t.zone(now), Zone::Battlefield);
    assert!(t.g.obj(now).is_commander);
}

#[test]
fn a_sorcery_that_says_any_number_ignores_the_four_copy_limit() {
    cr!("100.2a", "113.6n");
    ruling!(
        "Dragon's Approach",
        "lets you ignore the \"four-of\" rule"
    );
    let approach = card("Dragon's Approach");
    assert!(approach
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("any number of cards named")));
    assert_eq!(max_copies(&approach), None);
    let mut d: Vec<Arc<CardDef>> = (0..20).map(|_| approach.clone()).collect();
    d.extend((0..40).map(|_| card("Mountain")));
    assert!(check_constructed(&d).is_empty());
    // Other sorceries keep the limit.
    let mut d: Vec<Arc<CardDef>> = (0..5).map(|_| card("Divination")).collect();
    d.extend((0..55).map(|_| card("Island")));
    assert!(check_constructed(&d)
        .iter()
        .any(|p| matches!(p, DeckProblem::TooManyCopies { .. })));
}
