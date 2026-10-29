//! CR 207: the text box — rules text, reminder and flavor text, flavor words, the chaos
//! symbol, and Cryptic Spires' circled colors.

use crate::r105_util::{card_from_text, colors, pool_count};
use crate::r107_planechase::{add_planar_deck, planechase_game, roll};
use crate::r703_common::supported;
use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::{check_commander, circle_colors, DeckProblem};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::planechase::{self, PlanarFace};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn kinds(def: &CardDef) -> Vec<&'static str> {
    def.front()
        .chars
        .abilities
        .iter()
        .map(|a| match &a.kind {
            AbilityKind::Keyword(_) => "keyword",
            AbilityKind::Activated(x) if x.is_mana_ability => "mana",
            AbilityKind::Activated(_) => "activated",
            AbilityKind::Triggered(_) => "triggered",
            AbilityKind::Static(_) => "static",
            AbilityKind::Spell(_) => "spell",
            _ => "other",
        })
        .collect()
}

#[test]
fn the_text_box_defines_the_cards_abilities() {
    cr!("207.1");
    // Serra Angel: "Flying, vigilance". Llanowar Elves: "{T}: Add {G}."
    let angel = card("Serra Angel");
    assert!(angel.front().chars.has_keyword(KeywordKind::Flying));
    assert!(angel.front().chars.has_keyword(KeywordKind::Vigilance));
    assert_eq!(kinds(&card("Llanowar Elves")), vec!["mana"]);
    assert_eq!(kinds(&card("Lightning Bolt")), vec!["spell"]);
    assert_eq!(kinds(&card("Glorious Anthem")), vec!["static"]);
    // A vanilla creature's text box is empty: it has no abilities.
    let bears = card("Grizzly Bears");
    assert!(bears.front().chars.abilities.is_empty());
    assert!(bears.front().chars.rules_text.is_empty());
    // The abilities work: Llanowar Elves taps for {G}.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, elves, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, ManaType::G), 1);
}

#[test]
fn italicized_text_has_no_game_function() {
    cr!("207.2", "207.2a", "207.2b");
    // Reminder text: "Flying (This creature can't be blocked except by creatures with
    // flying or reach.)" is just flying.
    let def = card_from_text(
        "Reminded Bird",
        "{1}{U}",
        "Creature — Bird",
        Some((1, 1)),
        "Flying (This creature can't be blocked except by creatures with flying or reach.)",
    );
    assert_eq!(kinds(&def), vec!["keyword"]);
    // Primal Clay's reminder text about defender adds nothing.
    supported("Primal Clay");
    let clay = card("Primal Clay");
    assert!(clay
        .front()
        .chars
        .rules_text
        .contains("(A creature with defender can't attack.)"));
    assert_eq!(clay.front().chars.abilities.len(), 1);
    assert!(clay
        .front()
        .chars
        .abilities
        .iter()
        .all(|a| !a.text.contains("can't attack")));
    // Reminder text on its own line about an aspect of the card other than an ability:
    // Dryad Arbor's text box is only reminder text, so it has no abilities of its own (its
    // mana ability comes from its Forest land type).
    let arbor = card("Dryad Arbor");
    assert!(arbor.front().chars.rules_text.starts_with('('));
    assert!(arbor.is_fully_supported());
    assert!(arbor.front().chars.abilities.is_empty());
    // Flavor text isn't rules text: Grizzly Bears has flavor text but no abilities.
    assert!(card("Grizzly Bears").front().chars.abilities.is_empty());
}

#[test]
fn flavor_words_have_no_rules_meaning() {
    cr!("207.2d");
    // Owlbear: "Keen Senses — When this creature enters, draw a card." "Keen Senses" is
    // a flavor word: the ability is an ordinary enters trigger.
    supported("Owlbear");
    let owlbear = card("Owlbear");
    assert_eq!(kinds(&owlbear), vec!["keyword", "triggered"]);
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    t.enter(P0, "Owlbear");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // The same ability without the flavor word works the same way.
    let plain = card_from_text(
        "Plain Owlbear",
        "{3}{G}{G}",
        "Creature — Bird Bear",
        Some((4, 4)),
        "Trample\nWhen this creature enters, draw a card.",
    );
    assert_eq!(kinds(&plain), kinds(&owlbear));
}

#[test]
fn the_chaos_symbol_marks_an_ordinary_chaos_trigger() {
    // CR 207.4: the chaos symbol precedes a plane's "whenever chaos ensues" triggered
    // ability; the symbol has no rules meaning of its own.
    cr!("207.4");
    let goldmeadow = card("Goldmeadow");
    let chaos = goldmeadow
        .front()
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains("chaos ensues"))
        .expect("chaos ability");
    assert!(matches!(chaos.kind, AbilityKind::Triggered(_)));
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow", "The Fourth Sphere"]);
    planechase::set_starting_plane(&mut t.g);
    roll(&mut t, P0, PlanarFace::Chaos);
    // It triggered and uses the stack like any triggered ability.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Goat Token").len(), 1);
}

#[test]
fn cryptic_spires_circled_colors_are_printed_rules_text() {
    cr!("207.5");
    ruling!(
        "Cryptic Spires",
        "Once a Cryptic Spires has two colors circled, those colors are considered to be printed on the card."
    );
    ruling!(
        "Cryptic Spires",
        "The circled colors only affect what colors of mana the last ability produces as it resolves. They do not make Cryptic Spires those colors. It is still colorless."
    );
    ruling!(
        "Cryptic Spires",
        "If another permanent becomes a copy of Cryptic Spires, its mana ability will produce the same colors"
    );
    supported("Cryptic Spires");
    let printed = card("Cryptic Spires");
    // Colors must be two different colors.
    assert!(circle_colors(&printed, [Color::White, Color::White]).is_none());
    assert!(circle_colors(&card("Forest"), [Color::White, Color::Blue]).is_none());
    let wu = circle_colors(&printed, [Color::White, Color::Blue]).unwrap();
    assert!(wu.front().chars.rules_text.contains("{W} or {U}"));
    assert!(
        circle_colors(&wu, [Color::Red, Color::Green]).is_none(),
        "circled once"
    );
    // It taps for either circled color.
    let mut t = TestGame::new(2);
    let spires = t.custom(P0, wu.clone(), Zone::Battlefield);
    assert_eq!(colors(&t, spires), ColorSet::NONE);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, spires, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    assert_eq!(pool_count(&t, P0, ManaType::U), 1);
    // A copy of it has the circled colors too.
    let forest = t.battlefield(P0, "Forest");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(forest)], vec![Entity::Object(spires)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert_eq!(t.obj_now(forest).chars.name, "Cryptic Spires");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, P0, ManaType::W), 1);
    // Uncircled, the ability produces no mana.
    let bare = t.custom(P0, (*printed).clone(), Zone::Battlefield);
    let before = t.g.player(P0).mana_pool.total();
    let _ = t.activate(P0, bare, 0, &[]);
    assert_eq!(t.g.player(P0).mana_pool.total(), before);
    // The circled colors are part of its color identity (CR 903.4).
    assert_eq!(printed.color_identity, ColorSet::NONE);
    assert!(wu.color_identity.contains(Color::White) && wu.color_identity.contains(Color::Blue));
    let isamaru = card("Isamaru, Hound of Konda");
    let mut deck = vec![isamaru.clone(), Arc::new(wu)];
    deck.extend((0..98).map(|_| card("Plains")));
    assert!(check_commander(&deck, &isamaru, &[], false).iter().any(
        |p| matches!(p, DeckProblem::OutsideColorIdentity { name } if name == "Cryptic Spires")
    ));
    deck[1] = printed;
    assert!(check_commander(&deck, &isamaru, &[], false).is_empty());
}
