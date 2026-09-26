//! CR 702.139 Companion.

use crate::common_k702_125_139::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::{Action, Decision, SpecialAction};
use mtg_engine::events::MoveCause;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::sync::Arc;

fn copies(name: &str, n: usize) -> Vec<Arc<CardDef>> {
    (0..n).map(|_| card(name)).collect()
}

fn config() -> GameConfig {
    GameConfig {
        starting_player: Some(P0),
        ..Default::default()
    }
}

/// A 60-card deck fulfilling Lurrus of the Dream-Den's condition (each permanent card has
/// mana value 2 or less).
fn lurrus_deck() -> Vec<Arc<CardDef>> {
    let mut d = copies("Grizzly Bears", 20);
    d.extend(copies("Forest", 24));
    d.extend(copies("Giant Growth", 16));
    d
}

/// Starts a game where P0 plays `deck` with `side` outside the game and reveals the
/// sideboard card at `reveal` (if any) as their companion.
fn start_with(
    config: GameConfig,
    deck: Vec<Arc<CardDef>>,
    side: Vec<Arc<CardDef>>,
    reveal: Option<usize>,
) -> (TestGame, Vec<ObjectId>) {
    let mut t = pregame(config, vec![deck, copies("Forest", 60)]);
    let side = t.g.add_to_sideboard(P0, side);
    if let Some(i) = reveal {
        t.answer_choose(P0, &[Entity::Object(side[i])]);
    }
    t.g.start();
    (t, side)
}

/// Whether P0 is offered to reveal a companion before the game.
fn offered(t: &TestGame) -> bool {
    t.asked().iter().any(|(p, d)| {
        *p == P0
            && matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("companion"))
    })
}

fn specials(t: &mut TestGame, p: PlayerId) -> Vec<SpecialAction> {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Special(s) => Some(s),
            _ => None,
        })
        .collect()
}

fn to_hand(card: ObjectId) -> SpecialAction {
    SpecialAction::CompanionToHand { card }
}

#[test]
fn a_revealed_companion_is_put_into_hand_once_for_three_as_a_special_action() {
    cr!("702.139", "702.139a");
    ruling!(
        "Lurrus of the Dream-Den",
        "Paying {3} to put your companion into your hand is a special action. It doesn't use the stack and players can't respond to it."
    );
    let (mut t, side) = start_with(
        config(),
        lurrus_deck(),
        vec![card("Lurrus of the Dream-Den")],
        Some(0),
    );
    let lurrus = side[0];
    assert!(offered(&t));
    assert_eq!(t.g.companion_of(P0), Some(lurrus));
    // It remains revealed outside the game.
    assert_eq!(t.zone(lurrus), Zone::Outside(P0));
    t.advance_to(P0, Step::PrecombatMain);
    let lands = t.lands(P0, "Mountain", 3);
    // Not during an opponent's turn, not outside a main phase, not with a nonempty stack.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!specials(&mut t, P0).contains(&to_hand(lurrus)));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!specials(&mut t, P0).contains(&to_hand(lurrus)));
    t.set_step(P0, Step::PostcombatMain);
    assert!(specials(&mut t, P0).contains(&to_hand(lurrus)));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    assert!(!specials(&mut t, P0).contains(&to_hand(lurrus)));
    t.resolve_all();
    assert!(specials(&mut t, P0).contains(&to_hand(lurrus)));
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(to_hand(lurrus)))
        .unwrap();
    assert!(t.in_hand(P0, "Lurrus of the Dream-Den"));
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
    assert_eq!(t.stack_len(), 0);
    // Once per game.
    assert!(specials(&mut t, P0)
        .iter()
        .all(|s| !matches!(s, SpecialAction::CompanionToHand { .. })));
}

#[test]
fn only_a_companion_whose_condition_the_starting_deck_fulfills_can_be_revealed() {
    cr!("702.139a");
    // A three-mana permanent breaks Lurrus's condition.
    let mut deck = lurrus_deck();
    deck.pop();
    deck.push(card("Hill Giant"));
    let (t, side) = start_with(
        config(),
        deck,
        vec![card("Lurrus of the Dream-Den")],
        Some(0),
    );
    assert!(!offered(&t));
    assert_eq!(t.g.companion_of(P0), None);
    assert_eq!(t.zone(side[0]), Zone::Outside(P0));
    // Nor can a card without companion be revealed.
    let (t, _) = start_with(config(), lurrus_deck(), vec![card("Hill Giant")], Some(0));
    assert!(!offered(&t));
    assert_eq!(t.g.companion_of(P0), None);
}

#[test]
fn the_companion_conditions() {
    cr!("702.139a");
    let revealable = |deck: Vec<Arc<CardDef>>, companion: &str| -> bool {
        let (t, side) = start_with(config(), deck, vec![card(companion)], Some(0));
        t.g.companion_of(P0) == Some(side[0])
    };
    let lands = || copies("Forest", 24);
    let with = |mut base: Vec<Arc<CardDef>>, extra: Vec<Arc<CardDef>>| {
        base.extend(extra);
        base
    };
    // Lutri: each nonland card has a different name.
    let singletons: Vec<Arc<CardDef>> = [
        "Grizzly Bears",
        "Hill Giant",
        "Giant Growth",
        "Lightning Bolt",
        "Llanowar Elves",
        "Craw Wurm",
    ]
    .iter()
    .map(|n| card(n))
    .collect();
    assert!(revealable(
        with(lands(), singletons.clone()),
        "Lutri, the Spellchaser"
    ));
    assert!(!revealable(
        with(
            with(lands(), singletons.clone()),
            copies("Grizzly Bears", 1)
        ),
        "Lutri, the Spellchaser"
    ));
    // Gyruda: only even mana values (lands are 0). Obosh: odd mana values and lands.
    let even = with(lands(), copies("Grizzly Bears", 20));
    assert!(revealable(even.clone(), "Gyruda, Doom of Depths"));
    assert!(!revealable(even.clone(), "Obosh, the Preypiercer"));
    let odd = with(lands(), copies("Llanowar Elves", 20));
    assert!(revealable(odd.clone(), "Obosh, the Preypiercer"));
    assert!(!revealable(odd.clone(), "Gyruda, Doom of Depths"));
    // Zirda: each permanent card has an activated ability (a basic land's mana ability
    // counts, and so does a keyword that is one).
    assert!(revealable(odd.clone(), "Zirda, the Dawnwaker"));
    assert!(!revealable(even.clone(), "Zirda, the Dawnwaker"));
    // Umori: the nonland cards share a card type.
    assert!(revealable(even.clone(), "Umori, the Collector"));
    assert!(!revealable(
        with(even.clone(), copies("Giant Growth", 1)),
        "Umori, the Collector"
    ));
    // Yorion: at least twenty cards more than the minimum deck size (60).
    assert!(revealable(
        with(lands(), copies("Grizzly Bears", 56)),
        "Yorion, Sky Nomad"
    ));
    assert!(!revealable(
        with(lands(), copies("Grizzly Bears", 55)),
        "Yorion, Sky Nomad"
    ));
    // Jegantha: no card has two of the same mana symbol ({G}{G} does).
    assert!(revealable(even.clone(), "Jegantha, the Wellspring"));
    assert!(!revealable(
        with(even.clone(), copies("Craw Wurm", 1)),
        "Jegantha, the Wellspring"
    ));
    // Kaheera: each creature card is a Cat, Elemental, Nightmare, Dinosaur, or Beast.
    assert!(revealable(
        with(lands(), copies("Savannah Lions", 20)),
        "Kaheera, the Orphanguard"
    ));
    assert!(!revealable(even.clone(), "Kaheera, the Orphanguard"));
    // Keruga: mana value 3 or greater and land cards.
    assert!(revealable(
        with(lands(), copies("Hill Giant", 20)),
        "Keruga, the Macrosage"
    ));
    assert!(!revealable(even, "Keruga, the Macrosage"));
}

#[test]
fn the_starting_deck_is_the_deck_after_sideboard_cards_are_set_aside() {
    cr!("702.139b");
    ruling!(
        "Lurrus of the Dream-Den",
        "The requirements of the companion ability apply only to your starting deck. They do not apply to your sideboard."
    );
    // A three-mana permanent in the sideboard doesn't matter.
    let (t, side) = start_with(
        config(),
        lurrus_deck(),
        vec![card("Hill Giant"), card("Lurrus of the Dream-Den")],
        Some(1),
    );
    assert_eq!(t.g.companion_of(P0), Some(side[1]));
}

#[test]
fn in_commander_the_commander_is_part_of_the_starting_deck() {
    cr!("702.139b", "702.139d");
    ruling!(
        "Lurrus of the Dream-Den",
        "You may have one companion in the Commander variant. Your deck, including your commander, must meet its companion requirement."
    );
    let commander = || GameConfig {
        variant: Variant::Commander,
        starting_player: Some(P0),
        ..Default::default()
    };
    // A two-mana commander: Lurrus can be revealed, and put into hand from outside the
    // game during the game.
    let mut deck = copies("Forest", 60);
    deck.extend(copies("Grizzly Bears", 39));
    deck.push(card("Isamaru, Hound of Konda"));
    let mut t = pregame(commander(), vec![deck.clone(), copies("Forest", 100)]);
    assert!(t.g.designate_commander(P0, "Isamaru, Hound of Konda"));
    let side =
        t.g.add_to_sideboard(P0, vec![card("Lurrus of the Dream-Den")]);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.g.start();
    assert_eq!(t.g.companion_of(P0), Some(side[0]));
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(to_hand(side[0])))
        .unwrap();
    assert!(t.in_hand(P0, "Lurrus of the Dream-Den"));
    // A three-mana commander breaks the condition, though it's not in the library.
    let mut deck = copies("Forest", 60);
    deck.extend(copies("Grizzly Bears", 39));
    deck.push(card("Hill Giant"));
    let mut t = pregame(commander(), vec![deck, copies("Forest", 100)]);
    assert!(t.g.designate_commander(P0, "Hill Giant"));
    let side =
        t.g.add_to_sideboard(P0, vec![card("Lurrus of the Dream-Den")]);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.g.start();
    assert_eq!(t.g.companion_of(P0), None);
}

#[test]
fn a_companion_put_into_hand_remains_in_the_game() {
    cr!("702.139c");
    ruling!(
        "Yorion, Sky Nomad",
        "For example, if it's discarded, countered, or destroyed, it's put into your graveyard, remaining in the game."
    );
    let (mut t, side) = start_with(
        config(),
        lurrus_deck(),
        vec![card("Lurrus of the Dream-Den")],
        Some(0),
    );
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(to_hand(side[0])))
        .unwrap();
    let in_hand = t.g.current(side[0]);
    t.g.move_object(in_hand, Zone::Graveyard(P0), MoveCause::Discard, Some(P0));
    assert!(t.in_graveyard(P0, "Lurrus of the Dream-Den"));
    assert!(!t.g.player(P0).sideboard.contains(&t.g.current(side[0])));
    // It's a card in the game now: no special action for it, even back in the hand.
    let now = t.g.current(side[0]);
    t.g.move_object(now, Zone::Hand(P0), MoveCause::Return, Some(P0));
    assert!(specials(&mut t, P0)
        .iter()
        .all(|s| !matches!(s, SpecialAction::CompanionToHand { .. })));
}

#[test]
fn a_minimum_deck_size_condition_depends_on_the_format() {
    cr!("702.139a");
    ruling!(
        "Yorion, Sky Nomad",
        "Your minimum deck size is forty cards for Limited events (such as Booster Draft and Sealed Deck) and sixty cards for Constructed events (such as Standard or casual freeform play). Certain variants may have other minimums. The Commander variant requires exactly one hundred cards, so Yorion can never be your chosen companion in a Commander game."
    );
    let deck = |n: usize| {
        let mut d = copies("Forest", 24);
        d.extend(copies("Grizzly Bears", n - 24));
        d
    };
    // Limited: a sixty-card deck is twenty more than the minimum of forty.
    let limited = GameConfig {
        limited: true,
        starting_player: Some(P0),
        ..Default::default()
    };
    let (t, side) = start_with(limited.clone(), deck(60), vec![card("Yorion, Sky Nomad")], Some(0));
    assert_eq!(t.g.companion_of(P0), Some(side[0]));
    let (t, _) = start_with(limited, deck(59), vec![card("Yorion, Sky Nomad")], Some(0));
    assert_eq!(t.g.companion_of(P0), None);
    // Constructed: sixty cards aren't enough.
    let (t, _) = start_with(config(), deck(60), vec![card("Yorion, Sky Nomad")], Some(0));
    assert_eq!(t.g.companion_of(P0), None);
    // Commander: exactly one hundred cards, never twenty more than the minimum.
    let mut d = copies("Forest", 60);
    d.extend(copies("Grizzly Bears", 39));
    d.push(card("Isamaru, Hound of Konda"));
    let mut t = pregame(
        GameConfig {
            variant: Variant::Commander,
            starting_player: Some(P0),
            ..Default::default()
        },
        vec![d, copies("Forest", 100)],
    );
    assert!(t.g.designate_commander(P0, "Isamaru, Hound of Konda"));
    let side = t.g.add_to_sideboard(P0, vec![card("Yorion, Sky Nomad")]);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.g.start();
    assert_eq!(t.g.companion_of(P0), None);
}

#[test]
fn a_repeated_mana_symbol_condition_compares_exact_symbols() {
    cr!("702.139a");
    ruling!(
        "Jegantha, the Wellspring",
        "If any one card has the same symbol twice, such as {X}{X}{R} or {(r/g)}{(r/g)}, the companion condition isn't satisfied."
    );
    let revealable = |extra: &str| -> bool {
        let mut d = copies("Forest", 24);
        d.extend(copies("Grizzly Bears", 35));
        d.push(card(extra));
        let (t, side) = start_with(config(), d, vec![card("Jegantha, the Wellspring")], Some(0));
        t.g.companion_of(P0) == Some(side[0])
    };
    // {3}{R}: one generic symbol.
    assert!(revealable("Hill Giant"));
    // {X}{X} and {R/W}{R/W}{R/W}.
    assert!(!revealable("Hangarback Walker"));
    assert!(!revealable("Boros Reckoner"));
}

#[test]
fn a_shared_card_type_condition_needs_one_type_every_card_has() {
    cr!("702.139a");
    ruling!(
        "Umori, the Collector",
        "For example, if every nonland card is an artifact creature, enchantment creature, or creature, it is satisfied; but if you have an artifact creature, an artifact, and a creature, it is not satisfied"
    );
    let revealable = |nonland: &[&str]| -> bool {
        let mut d = copies("Forest", 24);
        for n in nonland {
            d.extend(copies(n, 12));
        }
        let (t, side) = start_with(config(), d, vec![card("Umori, the Collector")], Some(0));
        t.g.companion_of(P0) == Some(side[0])
    };
    // Artifact creature and creature: they share creature.
    assert!(revealable(&["Ornithopter", "Grizzly Bears", "Hill Giant"]));
    // Artifact creature, artifact, and creature: no one type.
    assert!(!revealable(&["Ornithopter", "Mind Stone", "Grizzly Bears"]));
}
