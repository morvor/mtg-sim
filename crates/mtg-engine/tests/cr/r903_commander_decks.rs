//! CR 903.4–903.5: color identity and the Commander deck construction rules.

use crate::r200_common::with_interchangeable_names;
use crate::r703_common::oracle_card;
use mtg_engine::card::{card, CardDef};
use mtg_engine::commander_rules::{computed_color_identity, object_color_identity, with_chosen_color};
use mtg_engine::deck::{check_commander, DeckProblem};
use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn colors(cs: &[Color]) -> ColorSet {
    let mut s = ColorSet::NONE;
    for c in cs {
        s.insert(*c);
    }
    s
}

/// A Commander deck: the commander, `extra`, and basic lands up to 100 cards.
fn deck(commander: &Arc<CardDef>, extra: Vec<Arc<CardDef>>, land: &str) -> Vec<Arc<CardDef>> {
    let mut d = vec![commander.clone()];
    let n = 99 - extra.len();
    d.extend(extra);
    d.extend((0..n).map(|_| card(land)));
    d
}

fn has<F: Fn(&DeckProblem) -> bool>(problems: &[DeckProblem], f: F) -> bool {
    problems.iter().any(f)
}

fn outside_identity(problems: &[DeckProblem], name: &str) -> bool {
    has(problems, |p| matches!(p, DeckProblem::OutsideColorIdentity { name: n } if n == name))
}

/// The options Command Tower offers `p` when activated.
fn tower_options(t: &mut TestGame, p: PlayerId) -> Vec<String> {
    let tower = t.battlefield(p, "Command Tower");
    let before = t.asked().len();
    t.activate(p, tower, 0, &[]).unwrap();
    t.asked()[before..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .unwrap_or_else(|| {
            t.g.player(p)
                .mana_pool
                .mana
                .iter()
                .map(|m| format!("{:?}", m.ty))
                .collect()
        })
}

fn commander(t: &mut TestGame, p: PlayerId, name: &str, zone: Zone) -> ObjectId {
    let id = match zone {
        Zone::Battlefield => t.battlefield(p, name),
        Zone::Library(_) => t.library_top(p, name),
        _ => t.command(p, name),
    };
    t.g.objects[id.0 as usize].is_commander = true;
    id
}

#[test]
fn color_identity_comes_from_mana_symbols_and_defined_colors() {
    cr!("903.4");
    ruling!(
        "Surgeon General Commander",
        "color identity is all five colors, thanks to the mana symbols in its rules text"
    );
    ruling!(
        "God-Eternal Oketra",
        "a commander's color identity isn't affected by color words (such as black) appearing in its text"
    );
    ruling!(
        "Bismuth Mindrender",
        "Devoid doesn't affect the color identity of the card"
    );
    // Mana symbols in its rules text: {T}: Add {W}, {U}, {B}, {R}, or {G}.
    assert_eq!(computed_color_identity(&card("Surgeon General Commander")), ColorSet::ALL);
    // A color word isn't a mana symbol.
    assert_eq!(
        computed_color_identity(&card("God-Eternal Oketra")),
        colors(&[Color::White])
    );
    // A colorless card (devoid) still has the colors of its mana cost.
    assert_eq!(
        computed_color_identity(&card("Bismuth Mindrender")),
        colors(&[Color::Black])
    );
    // A color indicator defines colors too (Dryad Arbor is green).
    assert_eq!(
        computed_color_identity(&card("Dryad Arbor")),
        colors(&[Color::Green])
    );
    // It decides what a deck may contain.
    let oketra = card("God-Eternal Oketra");
    let problems = check_commander(
        &deck(&oketra, vec![card("Surgeon General Commander")], "Plains"),
        &oketra,
        &[],
        false,
    );
    assert!(outside_identity(&problems, "Surgeon General Commander"));
}

#[test]
fn color_identity_is_established_before_the_game_begins() {
    cr!("903.4a");
    ruling!(
        "Scuttlemutt",
        "the color identities of cards are determined as the game begins"
    );
    ruling!(
        "War Room",
        "Color identity is set before the game begins and doesn't change during the game, even if your commander is in a hidden zone"
    );
    let mut t = TestGame::with_config(2, GameConfig::commander_game());
    let niv = commander(&mut t, P0, "Niv-Mizzet, Parun", Zone::Battlefield);
    // An effect makes the commander green: its color identity is still blue and red.
    crate::r703_common::run_effect(
        &mut t,
        P1,
        None,
        mtg_engine::ability::Effect::Modify {
            what: mtg_engine::ability::Sel::Target(0),
            mods: vec![mtg_engine::ability::Modification::SetColors(colors(&[
                Color::Green,
            ]))],
            duration: mtg_engine::ability::Duration::EndOfTurn,
        },
        &[Entity::Object(niv)],
    );
    assert_eq!(t.obj(niv).chars.colors, colors(&[Color::Green]));
    assert_eq!(
        object_color_identity(&t.g, niv),
        Some(colors(&[Color::Blue, Color::Red]))
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert_eq!(tower_options(&mut t, P0), vec!["U", "R"]);
    // In a hidden zone (the library) too.
    let mut t = TestGame::with_config(2, GameConfig::commander_game());
    commander(&mut t, P0, "Niv-Mizzet, Parun", Zone::Library(P0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    assert_eq!(tower_options(&mut t, P0), vec!["U", "R"]);
    assert_eq!(
        t.g.player(P0).mana_pool.mana.iter().map(|m| m.ty).collect::<Vec<_>>(),
        vec![ManaType::R]
    );
}

#[test]
fn a_color_chosen_before_the_game_is_part_of_the_commanders_color_identity() {
    cr!("903.4b");
    let piper = card("The Prismatic Piper");
    assert_eq!(computed_color_identity(&piper), ColorSet::NONE);
    // The choice applies during deck construction: with blue chosen, Islands and blue
    // cards may be in the deck; red ones may not.
    let blue_piper = with_chosen_color(&piper, Color::Blue).expect("choose a color");
    let d = deck(&blue_piper.clone().into(), vec![card("Unsummon")], "Island");
    assert!(check_commander(&d, &blue_piper, &[], false).is_empty());
    let d = deck(&blue_piper.clone().into(), vec![card("Lightning Bolt")], "Island");
    assert!(outside_identity(
        &check_commander(&d, &blue_piper, &[], false),
        "Lightning Bolt"
    ));
    // Only a card with such an ability.
    assert!(with_chosen_color(&card("Isamaru, Hound of Konda"), Color::Blue).is_none());
    // And throughout the game: the owner chooses as the game begins, and Command Tower
    // makes blue mana.
    let mut t = TestGame::with_config(2, GameConfig::commander_game());
    let p = commander(&mut t, P0, "The Prismatic Piper", Zone::Command);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    mtg_engine::opening_hand::before_shuffle_actions(&mut t.g);
    t.g.recompute();
    assert_eq!(object_color_identity(&t.g, p), Some(colors(&[Color::Blue])));
    assert_eq!(tower_options(&mut t, P0), vec!["U"]);
}

#[test]
fn reminder_text_is_ignored_for_color_identity() {
    cr!("903.4c");
    // Syndic of Tithes: {1}{W}, "Extort (Whenever you cast a spell, you may pay {W/B}. ...)"
    assert_eq!(
        computed_color_identity(&card("Syndic of Tithes")),
        colors(&[Color::White])
    );
    // A card whose black mana symbol is in its rules text proper is black.
    let with_text = oracle_card(
        "Pay Black",
        "Creature — Cleric",
        "{1}{W}",
        Some((2, 2)),
        "{B}: This creature gets +1/+1 until end of turn.",
    );
    assert_eq!(
        computed_color_identity(&with_text),
        colors(&[Color::White, Color::Black])
    );
    let mut reminder = with_text.clone();
    reminder.faces[0].chars.rules_text = Arc::from("(You may pay {B}. This is reminder text.)");
    reminder.faces[0].chars.abilities.clear();
    assert_eq!(computed_color_identity(&reminder), colors(&[Color::White]));
    // So Syndic of Tithes may be in a mono-white deck.
    let isamaru = card("Isamaru, Hound of Konda");
    let d = deck(&isamaru, vec![card("Syndic of Tithes")], "Plains");
    assert!(check_commander(&d, &isamaru, &[], false).is_empty());
    let d = deck(&isamaru, vec![Arc::new(with_text)], "Plains");
    assert!(outside_identity(&check_commander(&d, &isamaru, &[], false), "Pay Black"));
}

#[test]
fn the_back_face_counts_for_color_identity() {
    cr!("903.4d");
    ruling!(
        "Ral, Monsoon Mage",
        "a double-faced card's color identity is determined by the mana costs and mana symbols in the rules text of both faces combined"
    );
    // Ral, Monsoon Mage ({1}{R}) transforms into Ral, Leyline Prodigy (blue and red color
    // indicator); its front face alone is red.
    let ral = card("Ral, Monsoon Mage");
    assert_eq!(ral.front().chars.colors, colors(&[Color::Red]));
    assert_eq!(
        computed_color_identity(&ral),
        colors(&[Color::Blue, Color::Red])
    );
    // Westvale Abbey (a colorless land) transforms into a black creature.
    assert_eq!(
        computed_color_identity(&card("Westvale Abbey")),
        colors(&[Color::Black])
    );
    // So Ral can't be in a mono-red deck.
    let cmdr = card("Krenko, Mob Boss");
    let d = deck(&cmdr, vec![ral.clone()], "Mountain");
    assert!(outside_identity(&check_commander(&d, &cmdr, &[], false), &ral.name));
}

#[test]
fn an_adventurers_adventure_counts_for_color_identity() {
    cr!("903.4e");
    // Mosswood Dreadknight ({1}{G}) // Dread Whispers ({1}{B}).
    let knight = card("Mosswood Dreadknight");
    assert_eq!(
        computed_color_identity(&knight),
        colors(&[Color::Black, Color::Green])
    );
    let cmdr = card("Titania, Protector of Argoth");
    let d = deck(&cmdr, vec![knight.clone()], "Forest");
    assert!(outside_identity(&check_commander(&d, &cmdr, &[], false), &knight.name));
}

#[test]
fn a_commander_deck_follows_the_deck_construction_rules() {
    cr!("903.5");
    let isamaru = card("Isamaru, Hound of Konda");
    let ok = deck(&isamaru, vec![card("Savannah Lions")], "Plains");
    assert!(check_commander(&ok, &isamaru, &[], false).is_empty());
    // Each rule is checked: too many cards, a duplicate, a color outside the identity,
    // a sideboard.
    let mut bad = deck(&isamaru, vec![card("Savannah Lions"), card("Savannah Lions")], "Plains");
    bad.push(card("Lightning Bolt"));
    let problems = check_commander(&bad, &isamaru, &[card("Plains")], false);
    assert!(has(&problems, |p| matches!(p, DeckProblem::TooManyCards { .. })));
    assert!(has(&problems, |p| matches!(p, DeckProblem::TooManyCopies { .. })));
    assert!(outside_identity(&problems, "Lightning Bolt"));
    assert!(has(&problems, |p| matches!(p, DeckProblem::SideboardNotAllowed)));
}

#[test]
fn a_commander_deck_has_exactly_one_hundred_cards() {
    cr!("903.5a");
    let isamaru = card("Isamaru, Hound of Konda");
    let mut d = deck(&isamaru, vec![], "Plains");
    assert_eq!(d.len(), 100);
    assert!(check_commander(&d, &isamaru, &[], false).is_empty());
    d.pop();
    assert!(has(&check_commander(&d, &isamaru, &[], false), |p| matches!(
        p,
        DeckProblem::TooFewCards { have: 99, min: 100 }
    )));
    d.push(card("Plains"));
    d.push(card("Plains"));
    assert!(has(&check_commander(&d, &isamaru, &[], false), |p| matches!(
        p,
        DeckProblem::TooManyCards { have: 101, max: 100 }
    )));
}

#[test]
fn a_commander_deck_has_one_of_each_card_other_than_basic_lands() {
    cr!("903.5b");
    let isamaru = card("Isamaru, Hound of Konda");
    // 98 Plains are fine.
    assert!(check_commander(&deck(&isamaru, vec![], "Plains"), &isamaru, &[], false).is_empty());
    // Two Savannah Lions aren't.
    let d = deck(&isamaru, vec![card("Savannah Lions"), card("Savannah Lions")], "Plains");
    assert!(has(&check_commander(&d, &isamaru, &[], false), |p| matches!(
        p,
        DeckProblem::TooManyCopies { name, have: 2, max: 1 } if name == "Savannah Lions"
    )));
    // Cards with interchangeable names have the same English name.
    let twin = Arc::new(with_interchangeable_names("Silvercoat Lion", &["Savannah Lions"]));
    let d = deck(&isamaru, vec![card("Savannah Lions"), twin], "Plains");
    assert!(has(&check_commander(&d, &isamaru, &[], false), |p| matches!(
        p,
        DeckProblem::TooManyCopies { have: 2, .. }
    )));
}

#[test]
fn every_card_must_be_within_the_commanders_color_identity() {
    cr!("903.5c");
    let niv = card("Niv-Mizzet, Parun");
    // Blue, red, blue-red and colorless cards: yes.
    let d = deck(
        &niv,
        vec![card("Unsummon"), card("Lightning Bolt"), card("Sol Ring"), card("Izzet Charm")],
        "Island",
    );
    assert!(check_commander(&d, &niv, &[], false).is_empty());
    // A card with any other color: no, even a hybrid card with blue in it.
    let d = deck(&niv, vec![card("Wilt-Leaf Liege"), card("Kitchen Finks")], "Island");
    let problems = check_commander(&d, &niv, &[], false);
    assert!(outside_identity(&problems, "Wilt-Leaf Liege"));
    assert!(outside_identity(&problems, "Kitchen Finks"));
}

/// A land card with basic land types and no rules text.
fn typed_land(name: &str, types: &str) -> Arc<CardDef> {
    Arc::new(oracle_card(name, &format!("Land — {types}"), "", None, ""))
}

#[test]
fn a_card_with_basic_land_types_must_make_only_the_commanders_colors() {
    cr!("903.5d");
    let isamaru = card("Isamaru, Hound of Konda");
    // A Plains Island could produce blue mana: not with a mono-white commander, even
    // though it has no mana symbols (its color identity is colorless).
    let dual = typed_land("Test Tundra", "Plains Island");
    assert_eq!(computed_color_identity(&dual), ColorSet::NONE);
    let d = deck(&isamaru, vec![dual.clone()], "Plains");
    assert!(has(&check_commander(&d, &isamaru, &[], false), |p| matches!(
        p,
        DeckProblem::ManaOutsideColorIdentity { name } if name == "Test Tundra"
    )));
    // With a white-blue commander it's fine.
    let cmdr = card("Brago, King Eternal");
    let d = deck(&cmdr, vec![dual], "Plains");
    assert!(check_commander(&d, &cmdr, &[], false).is_empty());
    // Dryad Arbor (a Forest) can't be in a mono-white deck.
    let d = deck(&isamaru, vec![card("Dryad Arbor")], "Plains");
    assert!(has(&check_commander(&d, &isamaru, &[], false), |p| matches!(
        p,
        DeckProblem::ManaOutsideColorIdentity { name } if name == "Dryad Arbor"
    )));
}

#[test]
fn commander_games_do_not_use_sideboards() {
    cr!("903.5e");
    let isamaru = card("Isamaru, Hound of Konda");
    let d = deck(&isamaru, vec![], "Plains");
    assert!(check_commander(&d, &isamaru, &[], false).is_empty());
    assert!(has(
        &check_commander(&d, &isamaru, &[card("Savannah Lions")], false),
        |p| matches!(p, DeckProblem::SideboardNotAllowed)
    ));
}
