//! CR 901: the Planechase casual variant — planar decks, the starting plane, the planar
//! controller, the planar die, planeswalking, players leaving, Two-Headed Giant and Grand
//! Melee Planechase, and the single planar deck option.

use crate::r100_common::{fillers, pregame};
use crate::r703_common::{oracle_card, run_effect, supported};
use crate::r900_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::casual::check_planar_deck;
use mtg_engine::deck::DeckProblem;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::multiplayer::grand_melee;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::planechase::{self, PlanarFace, PLANAR_DIE_ACTION};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pid(i: usize) -> PlayerId {
    PlayerId(i as u8)
}

const P4: PlayerId = PlayerId(4);

/// A started Planechase game: each player's deck is `fillers` plus `planar` cards.
fn started(n: usize, planar: &[&[&str]]) -> TestGame {
    let decks = (0..n)
        .map(|i| {
            let mut d = fillers(40);
            d.extend(cards(planar.get(i).copied().unwrap_or(&[])));
            d
        })
        .collect();
    let mut t = pregame(
        GameConfig {
            skip_mulligans: true,
            starting_player: Some(P0),
            ..GameConfig::planechase_game()
        },
        decks,
    );
    t.g.start();
    t
}

fn can_roll(t: &mut TestGame, p: PlayerId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Special(SpecialAction::Other { name, .. }) if name.as_str() == PLANAR_DIE_ACTION)
    })
}

fn stack_controller(t: &TestGame) -> PlayerId {
    t.obj(*t.g.stack.last().expect("stack")).controller
}

#[test]
fn planechase_adds_planes_and_phenomena_to_a_normal_game() {
    cr!("901.1");
    let t = started(2, &[&["Krosa", "Goldmeadow"], &["Tazeem", "Naar Isle"]]);
    // The normal rules of a game: life totals, opening hands, libraries.
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (7, 7));
    assert_eq!(t.library_size(P0), 33);
    // ... with the additions: a starting plane from the starting player's planar deck,
    // whose abilities affect the game.
    let names = face_up_names(&t);
    assert_eq!(names.len(), 1);
    assert!(["Krosa", "Goldmeadow"].contains(&names[0].as_str()));
    // Without the variant, the same decks' planes do nothing.
    let mut t = pregame(
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
        vec![
            {
                let mut d = fillers(40);
                d.extend(cards(&["Krosa"]));
                d
            },
            fillers(40),
        ],
    );
    t.g.start();
    assert!(face_up_names(&t).is_empty());
}

#[test]
fn planechase_is_two_player_or_free_for_all() {
    cr!("901.2");
    let c = GameConfig::planechase_game();
    assert_eq!(c.validate(2), Ok(()));
    assert_eq!(c.validate(5), Ok(()));
    assert!(c.attack_multiple_players);
    assert_eq!(c.range_of_influence, None);
    // Multiplayer: players compete as individuals, with no limited range of influence.
    let mut t = TestGame::with_config(4, GameConfig::planechase_game());
    for q in [P1, P2, P3] {
        assert!(t.g.are_opponents(P0, q));
    }
    assert_eq!(t.g.range_of_influence(P0), None);
    // Attack multiple players: P0 attacks P1 and P2 at once.
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P2))], &[]);
    assert_eq!((t.life(P1), t.life(P2)), (18, 18));
}

#[test]
fn a_planar_deck_has_ten_cards_at_most_two_phenomena_and_different_names() {
    cr!("901.3");
    let ok = cards(&TEN_PLANES);
    assert!(check_planar_deck(&ok, None).is_empty());
    // Two phenomena are fine; three aren't.
    let mut two = ok.clone();
    two.extend(cards(&PHENOMENA[..2]));
    assert!(check_planar_deck(&two, None).is_empty());
    let mut three = ok.clone();
    three.extend(cards(&PHENOMENA[..3]));
    assert!(check_planar_deck(&three, None)
        .iter()
        .any(|p| matches!(p, DeckProblem::TooManyPhenomena { have: 3, max: 2 })));
    // Nine cards are too few.
    assert!(check_planar_deck(&ok[..9], None)
        .iter()
        .any(|p| matches!(p, DeckProblem::TooFewCards { have: 9, min: 10 })));
    // Each card must have a different English name.
    let mut dup = ok.clone();
    dup.push(card("Krosa"));
    assert!(check_planar_deck(&dup, None)
        .iter()
        .any(|p| matches!(p, DeckProblem::TooManyCopies { name, .. } if name == "Krosa")));
    // Only plane and phenomenon cards.
    let mut other = ok.clone();
    other.push(card("Grizzly Bears"));
    assert!(check_planar_deck(&other, None)
        .iter()
        .any(|p| matches!(p, DeckProblem::WrongCardType { name, .. } if name == "Grizzly Bears")));
}

#[test]
fn the_planar_die_has_a_planeswalker_face_a_chaos_face_and_four_blanks() {
    cr!("901.3a");
    let faces: Vec<PlanarFace> = (1..=6).map(planechase::face_for).collect();
    let count = |f: PlanarFace| faces.iter().filter(|x| **x == f).count();
    assert_eq!(count(PlanarFace::Planeswalker), 1);
    assert_eq!(count(PlanarFace::Chaos), 1);
    assert_eq!(count(PlanarFace::Blank), 4);
    // Rolling it is a six-sided roll: over many rolls every face comes up.
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    let mut seen = Vec::new();
    for _ in 0..60 {
        let f = planechase::roll_planar_die(&mut t.g, P0);
        if !seen.contains(&f) {
            seen.push(f);
        }
        t.g.stack.clear();
        t.g.pending_triggers.clear();
    }
    assert_eq!(seen.len(), 3);
}

#[test]
fn planar_cards_stay_in_the_command_zone_in_the_deck_and_face_up() {
    cr!("901.4");
    let mut t = started(2, &[&["Krosa", "Goldmeadow", "Tazeem"]]);
    let planar: Vec<ObjectId> = t
        .g
        .command
        .iter()
        .copied()
        .filter(|id| {
            let c = t.obj(*id).card.as_ref().unwrap().front().chars.clone();
            c.is(CardType::Plane)
        })
        .collect();
    assert_eq!(planar.len(), 3);
    // None of them was put into the library.
    assert!(t
        .g
        .player(P0)
        .library
        .iter()
        .all(|c| !t.obj(*c).chars.is(CardType::Plane)));
    let face_up = planechase::face_up_planar_cards(&t.g)[0];
    // An effect can't move the face-up plane or a card of the planar deck out of the
    // command zone.
    for id in [face_up, planechase::planar_deck(&t.g, P0)[0]] {
        run_effect(
            &mut t,
            P1,
            None,
            Effect::Move {
                what: Sel::Target(0),
                to: Destination::zone(ZoneKind::Hand),
            },
            &[Entity::Object(id)],
        );
        assert_eq!(t.zone(id), Zone::Command);
    }
    // After planeswalking, every planar card is still in the command zone.
    planechase::planeswalk(&mut t.g, P0);
    for id in planar {
        assert_eq!(t.zone(t.g.current(id)), Zone::Command);
    }
}

#[test]
fn the_starting_plane_is_the_first_plane_turned_face_up_without_triggering_anything() {
    cr!("901.5");
    supported("Panopticon");
    // P0's planar deck: two phenomena and Panopticon ("When you planeswalk to Panopticon,
    // draw a card"), in a random order.
    let t = started(2, &[&["Mutual Epiphany", "Planewide Disaster", "Panopticon"]]);
    assert_eq!(face_up_names(&t), vec!["Panopticon"]);
    // The phenomena turned face up on the way went to the bottom face down; none of the
    // cards' abilities triggered: nobody drew cards, and nothing is waiting.
    let deck = planechase::planar_deck(&t.g, P0);
    assert_eq!(deck.len(), 2);
    assert!(deck.iter().all(|id| t.obj(*id).face_down));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (7, 7));
    assert!(t.g.pending_triggers.is_empty());
    assert_eq!(t.stack_len(), 0);
    // The starting player's planar deck is used.
    let t = started(2, &[&[], &["Krosa"]]);
    assert!(face_up_names(&t).is_empty());
    let t = started(2, &[&["Tazeem"], &["Krosa"]]);
    assert_eq!(face_up_names(&t), vec!["Tazeem"]);
}

#[test]
fn the_owner_and_the_controller_of_planar_cards() {
    cr!("901.6");
    ruling!("Krosa", "the next player in turn order");
    let mut t = planechase_game(4, false);
    // P1's plane is face up during P0's turn: P1 owns it, the planar controller (the
    // active player) controls it.
    let deck = add_planar_deck(&mut t, P1, &["Krosa"]);
    add_planar_deck(&mut t, P3, &["Goldmeadow"]);
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, deck[0]);
    t.g.recompute();
    assert_eq!((t.obj(deck[0]).owner, t.obj(deck[0]).controller), (P1, P0));
    t.set_step(P2, Step::Upkeep);
    assert_eq!(t.obj(deck[0]).controller, P2);
    // If the planar controller would leave the game, the next player in turn order that
    // wouldn't leave becomes the planar controller: P2 and P3 lose at the same time, so
    // it's P0.
    t.g.lose_game_simultaneously(&[P2, P3]);
    t.g.recompute();
    assert!(!t.player(P2).in_game() && !t.player(P3).in_game());
    assert_eq!(planechase::planar_controller(&t.g), Some(P0));
    assert_eq!(t.obj(deck[0]).controller, P0);
    // They keep the designation until a different player becomes the active player.
    t.set_step(P1, Step::Upkeep);
    assert_eq!(t.obj(deck[0]).controller, P1);
}

#[test]
fn face_up_planar_cards_abilities_function_from_the_command_zone() {
    cr!("901.7");
    let mut t = planechase_game(2, false);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shrine = add_custom_planar(&mut t, P0, plane("Shrine Realm", "{1}: You gain 2 life."));
    add_planar_deck(&mut t, P0, &["Krosa", "Mutual Epiphany", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    // Activated.
    t.lands(P0, "Plains", 1);
    t.activate(P0, shrine, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 22);
    // Static: Krosa's "All creatures get +2/+2".
    planechase::planeswalk(&mut t.g, P0);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    // Triggered: a phenomenon's encounter ability triggers.
    let hands = t.hand_size(P1);
    planechase::planeswalk(&mut t.g, P0);
    t.settle();
    assert_eq!(face_up_names(&t), vec!["Mutual Epiphany"]);
    t.resolve();
    assert_eq!(t.hand_size(P1), hands + 4);
}

#[test]
fn a_planar_card_turned_face_down_becomes_a_new_object() {
    cr!("901.7a");
    let mut t = planechase_game(2, false);
    let deck = add_planar_deck(&mut t, P0, &["Krosa", "Mutual Epiphany", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    planechase::planeswalk(&mut t.g, P0);
    assert!(!t.g.is_live(deck[0]));
    let krosa = t.g.current(deck[0]);
    assert_ne!(krosa, deck[0]);
    assert!(t.obj(krosa).face_down);
    // A phenomenon too.
    t.settle();
    t.resolve();
    t.settle();
    assert!(!t.g.is_live(deck[1]));
    assert!(t.obj(t.g.current(deck[1])).face_down);
}

#[test]
fn the_planeswalking_ability_has_no_source_and_the_roller_controls_it() {
    cr!("901.8");
    // Two-Headed Giant Planechase: P0 is the planar controller; their teammate P1 rolls.
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant_planechase(vec![0, 0, 1, 1]));
    add_planar_deck(&mut t, P0, &["Krosa"]);
    add_planar_deck(&mut t, P1, &["Tazeem"]);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(planechase::planar_controller(&t.g), Some(P0));
    let krosa = planechase::face_up_planar_cards(&t.g)[0];
    assert_eq!(t.obj(krosa).controller, P0);
    roll(&mut t, P1, PlanarFace::Planeswalker);
    // Controlled by the player whose roll made it trigger, not by the controller of the
    // face-up plane (an exception to CR 113.8).
    assert_eq!(t.stack_len(), 1);
    assert_eq!(stack_controller(&t), P1);
    // It has no source: it isn't an ability of the plane (or of any card).
    let ab = t.g.stack[0];
    let src = t.g.ability_source_of(ab);
    assert_ne!(src, krosa);
    assert!(t.obj(src).card.is_none());
    assert_eq!(t.zone(src), Zone::Nowhere);
    t.resolve();
    assert_eq!(face_up_names(&t), vec!["Tazeem"]);
}

#[test]
fn rolling_the_chaos_symbol_makes_chaos_ensue_and_the_active_player_gets_priority() {
    cr!("901.9b");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    roll(&mut t, P0, PlanarFace::Chaos);
    assert_eq!(chaos_count(&t), 1);
    // Goldmeadow's chaos ability is on the stack; the active player has priority.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.turn.priority, Some(P0));
    t.resolve();
    assert_eq!(tokens(&t, P0), 1);
}

fn tokens(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.controller == p)
        .count()
}

#[test]
fn rolling_the_planeswalker_symbol_triggers_the_planeswalking_ability() {
    cr!("901.9c");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow", "Krosa"]);
    planechase::set_starting_plane(&mut t.g);
    roll(&mut t, P0, PlanarFace::Planeswalker);
    // The planeswalking ability is on the stack and the active player gets priority;
    // P0 hasn't planeswalked yet.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.turn.priority, Some(P0));
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
    assert_eq!(planeswalks(&t), 0);
    t.resolve();
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
    assert_eq!(planeswalks(&t), 1);
}

#[test]
fn the_planar_die_triggers_die_roll_abilities_but_has_no_number() {
    cr!("901.9d");
    ruling!(
        "Component Pouch",
        "While playing Planechase, rolling the planar die will cause any ability that triggers whenever a player rolls one or more dice to trigger."
    );
    supported("Brazen Dwarf");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    // "Whenever you roll one or more dice, Brazen Dwarf deals 1 damage to each opponent."
    t.battlefield(P0, "Brazen Dwarf");
    // "Whenever you roll one or more dice, put a number of charge counters on Vexing
    // Puzzlebox equal to the result."
    let bx = t.battlefield(P0, "Vexing Puzzlebox");
    roll(&mut t, P0, PlanarFace::Blank);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.counters(bx, counters::CHARGE), 0);
}

#[test]
fn when_a_plane_owner_leaves_the_planar_controller_planeswalks() {
    cr!("901.10");
    supported("Panopticon");
    let mut t = planechase_game(3, false);
    // P1's plane is face up; P0 (the planar controller) has Panopticon on top of their
    // planar deck: "When you planeswalk to Panopticon, draw a card."
    let theirs = add_planar_deck(&mut t, P1, &["Krosa", "Tazeem"]);
    add_planar_deck(&mut t, P0, &["Panopticon", "Goldmeadow"]);
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, theirs[0]);
    t.g.recompute();
    let hand = t.hand_size(P0);
    // P1 leaves the game: the cards they own leave it, and — right away, not as a
    // state-based action — P0 turns the top card of their planar deck face up.
    t.g.turn.priority = Some(P1);
    t.g.perform_action(P1, Action::Concede).unwrap();
    assert_eq!(t.zone(theirs[0]), Zone::Nowhere);
    assert_eq!(face_up_names(&t), vec!["Panopticon"]);
    assert_eq!(planeswalks(&t), 1);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_planeswalking_ability_ceases_to_exist_when_a_plane_leaves_the_game() {
    cr!("901.10a");
    let mut t = planechase_game(3, false);
    let theirs = add_planar_deck(&mut t, P1, &["Krosa"]);
    add_planar_deck(&mut t, P0, &["Goldmeadow", "Tazeem"]);
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, theirs[0]);
    t.g.recompute();
    roll(&mut t, P0, PlanarFace::Planeswalker);
    assert_eq!(t.stack_len(), 1);
    // P1 leaves: their plane leaves the game; P0 planeswalks to Goldmeadow, and the
    // planeswalking ability ceases to exist.
    t.g.turn.priority = Some(P1);
    t.g.perform_action(P1, Action::Concede).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
    t.resolve_all();
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
    assert_eq!(planeswalks(&t), 1);
}

#[test]
fn abilities_from_phenomena_of_a_departed_player_stay_on_the_stack() {
    cr!("901.10b");
    let mut t = planechase_game(3, false);
    // P0, the active player and planar controller, encounters their own Mutual Epiphany.
    add_planar_deck(&mut t, P0, &["Mutual Epiphany"]);
    add_planar_deck(&mut t, P1, &["Krosa"]);
    planechase::planeswalk(&mut t.g, P0);
    t.settle();
    assert_eq!(face_up_names(&t), vec!["Mutual Epiphany"]);
    assert_eq!(stack_controller(&t), P0);
    let (h1, h2) = (t.hand_size(P1), t.hand_size(P2));
    // P0 leaves the game. P1 becomes the planar controller; the phenomenon (owned by P0)
    // leaves the game, so P1 planeswalks — but its encounter ability stays on the stack,
    // now controlled by P1.
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Concede).unwrap();
    assert_eq!(planechase::planar_controller(&t.g), Some(P1));
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(stack_controller(&t), P1);
    t.resolve();
    assert_eq!((t.hand_size(P1), t.hand_size(P2)), (h1 + 4, h2 + 4));
}

#[test]
fn planeswalking_ends_effects_and_triggers_abilities() {
    cr!("901.11");
    supported("Eloren Wilds");
    supported("Panopticon");
    let mut t = planechase_game(2, false);
    // Eloren Wilds: "Whenever chaos ensues, target player can't cast spells until a player
    // planeswalks."
    add_planar_deck(&mut t, P0, &["Eloren Wilds"]);
    add_planar_deck(&mut t, P1, &["Panopticon"]);
    planechase::set_starting_plane(&mut t.g);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    roll_effect(&mut t, P0, PlanarFace::Chaos);
    t.resolve_all();
    let bolt = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P1, bolt).target(Entity::Player(P0)).try_go().is_err());
    // A player planeswalks (to Panopticon, whose "When you planeswalk to Panopticon, draw
    // a card" triggers): the effect ends.
    let hand = t.hand_size(P1);
    planechase::planeswalk(&mut t.g, P1);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    assert!(t.cast(P1, bolt).target(Entity::Player(P0)).try_go().is_ok());
}

#[test]
fn chaotic_aether_lasts_until_a_player_planeswalks_away_from_a_plane() {
    cr!("901.11");
    supported("Chaotic Aether");
    ruling!(
        "Chaotic Aether",
        "Planeswalking away from a phenomenon (as you do when resolving Chaotic Aether's ability) doesn't cause Chaotic Aether's effect to expire."
    );
    ruling!(
        "Chaotic Aether",
        "While Chaotic Aether's effect applies, rolling any blank face of the planar die will cause chaos abilities to trigger."
    );
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow", "Chaotic Aether", "Krosa", "Tazeem"]);
    planechase::set_starting_plane(&mut t.g);
    // Planeswalk to Chaotic Aether; its ability resolves; the planar controller then
    // planeswalks away from the phenomenon (to Krosa).
    planechase::planeswalk(&mut t.g, P0);
    t.settle();
    t.resolve();
    t.settle();
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
    // A blank roll is a chaos roll.
    let before = chaos_count(&t);
    roll_effect(&mut t, P0, PlanarFace::Blank);
    assert_eq!(chaos_count(&t), before + 1);
    t.resolve_all();
    // Planeswalking away from a plane ends the effect.
    planechase::planeswalk(&mut t.g, P0);
    t.settle();
    let before = chaos_count(&t);
    roll_effect(&mut t, P0, PlanarFace::Blank);
    assert_eq!(chaos_count(&t), before);
}

#[test]
fn the_ways_a_player_planeswalks() {
    cr!("901.11a");
    let mut t = planechase_game(3, false);
    add_planar_deck(
        &mut t,
        P0,
        &["Goldmeadow", "Krosa", "Tazeem", "Mutual Epiphany", "Llanowar", "Lethe Lake"],
    );
    let theirs = add_planar_deck(&mut t, P1, &["Naar Isle"]);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(planeswalks(&t), 0);
    // The planeswalking ability.
    roll(&mut t, P0, PlanarFace::Planeswalker);
    t.resolve();
    assert_eq!(planeswalks(&t), 1);
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
    // An ability instructing a player to planeswalk.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::KeywordAction {
            action: KeywordAction::Planeswalk,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        },
        &[],
    );
    assert_eq!(planeswalks(&t), 2);
    assert_eq!(face_up_names(&t), vec!["Tazeem"]);
    // A phenomenon's triggered ability leaving the stack.
    planechase::planeswalk(&mut t.g, P0);
    t.settle();
    assert_eq!(face_up_names(&t), vec!["Mutual Epiphany"]);
    assert_eq!(planeswalks(&t), 3);
    t.resolve();
    t.settle();
    assert_eq!(planeswalks(&t), 4);
    assert_eq!(face_up_names(&t), vec!["Llanowar"]);
    // The owner of a face-up planar card leaving the game: P1's Naar Isle.
    let llanowar = planechase::face_up_planar_cards(&t.g)[0];
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, theirs[0]);
    t.g.turn.priority = Some(P1);
    t.g.perform_action(P1, Action::Concede).unwrap();
    assert_eq!(planeswalks(&t), 5);
    assert_eq!(face_up_names(&t), vec!["Lethe Lake"]);
    assert!(!t.g.is_live(llanowar));
}

#[test]
fn planeswalking_to_and_away_from_planes() {
    cr!("901.11b");
    let mut t = planechase_game(3, false);
    let away = plane(
        "Farewell Realm",
        "When you planeswalk away from ~, you gain 5 life.",
    );
    let to = plane("Welcome Realm", "When you planeswalk to ~, you gain 3 life.");
    // P1 owns a face-up plane that says what happens when you planeswalk away from it.
    let fare = add_custom_planar(&mut t, P1, away);
    add_custom_planar(&mut t, P0, to);
    add_planar_deck(&mut t, P0, &["Krosa"]);
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, fare);
    t.g.recompute();
    // The plane turned face up is the one planeswalked to; the one that leaves the game
    // (with its owner) is the one planeswalked away from.
    t.g.turn.priority = Some(P1);
    t.g.perform_action(P1, Action::Concede).unwrap();
    t.settle();
    t.resolve_all();
    assert_eq!(face_up_names(&t), vec!["Welcome Realm"]);
    assert_eq!(t.life(P0), 28);
    // Turned face down: planeswalked away from (Welcome Realm has no such ability; Krosa
    // is planeswalked to).
    planechase::planeswalk(&mut t.g, P0);
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
}

#[test]
fn planeswalking_away_from_several_face_up_planes() {
    cr!("901.11c");
    supported("Spatial Merging");
    let mut t = planechase_game(2, false);
    let deck = add_planar_deck(
        &mut t,
        P0,
        &["Goldmeadow", "Spatial Merging", "Krosa", "Mutual Epiphany", "Tazeem", "Naar Isle"],
    );
    planechase::set_starting_plane(&mut t.g);
    // Spatial Merging: reveal until two planes are revealed, planeswalk to both of them.
    planechase::planeswalk(&mut t.g, P0);
    t.settle();
    t.resolve();
    t.settle();
    let mut up = face_up_names(&t);
    up.sort();
    assert_eq!(up, vec!["Krosa", "Tazeem"]);
    // The phenomenon revealed on the way went to the bottom.
    assert_eq!(name_of(&t, *planechase::planar_deck(&t.g, P0).last().unwrap()), "Mutual Epiphany");
    // Planeswalking now moves away from both planes.
    planechase::planeswalk(&mut t.g, P0);
    assert_eq!(face_up_names(&t), vec!["Naar Isle"]);
    assert!(!t.g.is_live(deck[2]) && !t.g.is_live(deck[4]));
    assert!(t.obj(t.g.current(deck[2])).face_down);
    assert!(t.obj(t.g.current(deck[4])).face_down);
}

/// A Two-Headed Giant Planechase game: teams P0+P1 and P2+P3.
fn two_headed() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant_planechase(vec![0, 0, 1, 1]))
}

#[test]
fn two_headed_giant_planechase_uses_both_variants_rules() {
    cr!("901.12");
    let mut t = two_headed();
    assert!(planechase::is_planechase(&t.g));
    // Two-Headed Giant: a shared team life total of 30.
    assert_eq!(t.g.starting_life(P1), 30);
    assert_eq!(t.life(P1), 30);
    // Planechase: a starting plane, and the planar die can be rolled.
    add_planar_deck(&mut t, P0, &["Krosa"]);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
    assert!(can_roll(&mut t, P0));
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]));
    assert!(!planechase::is_planechase(&t.g));
    assert!(!can_roll(&mut t, P0));
}

#[test]
fn each_two_headed_giant_player_has_their_own_planar_deck() {
    cr!("901.12a");
    let mut t = two_headed();
    let a = add_planar_deck(&mut t, P0, &["Krosa", "Goldmeadow"]);
    let b = add_planar_deck(&mut t, P1, &["Tazeem", "Naar Isle"]);
    assert_eq!(planechase::planar_deck(&t.g, P0), a);
    assert_eq!(planechase::planar_deck(&t.g, P1), b);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
    // When P1 planeswalks, they turn the top card of their own planar deck face up.
    roll(&mut t, P1, PlanarFace::Planeswalker);
    t.resolve();
    assert_eq!(face_up_names(&t), vec!["Tazeem"]);
    assert_eq!(planechase::planar_deck(&t.g, P0).len(), 2);
}

#[test]
fn the_planar_controller_is_the_primary_player_of_the_active_team() {
    cr!("901.12b");
    // Three teams of two, seated together.
    let mut t = TestGame::with_config(
        6,
        GameConfig::two_headed_giant_planechase(vec![0, 0, 1, 1, 2, 2]),
    );
    let deck = add_planar_deck(&mut t, P3, &["Krosa"]);
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, deck[0]);
    t.g.recompute();
    let primary = t.g.primary_player(P1);
    assert_eq!(primary, P0);
    assert_eq!(planechase::planar_controller(&t.g), Some(P0));
    assert_eq!(t.obj(deck[0]).controller, P0);
    // With P1 representing the team's turn, it's still the primary player.
    t.set_step(P1, Step::Upkeep);
    assert_eq!(planechase::planar_controller(&t.g), Some(P0));
    // If the planar controller's team leaves the game, the primary player of the next
    // team in turn order becomes the planar controller — until another team is active.
    t.g.lose_game_simultaneously(&[P0, P1]);
    t.g.recompute();
    assert_eq!(planechase::planar_controller(&t.g), Some(P2));
    assert_eq!(t.obj(deck[0]).controller, P2);
    t.set_step(P4, Step::Upkeep);
    assert_eq!(planechase::planar_controller(&t.g), Some(P4));
}

#[test]
fn you_on_a_plane_means_both_members_of_the_planar_controllers_team() {
    cr!("901.12c");
    let mut t = two_headed();
    let mine = t.battlefield(P0, "Grizzly Bears");
    let mate = t.battlefield(P1, "Grizzly Bears");
    let foe = t.battlefield(P2, "Grizzly Bears");
    add_custom_planar(
        &mut t,
        P2,
        plane(
            "Team Realm",
            "Creatures you control get +1/+1.\nWhenever you roll the planar die, draw a card.",
        ),
    );
    let top = planechase::planar_deck(&t.g, P2)[0];
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, top);
    t.g.recompute();
    assert_eq!(t.obj(top).controller, P0);
    // "Creatures you control" includes the teammate's creatures.
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(mate), (3, 3));
    assert_eq!(t.pt(foe), (2, 2));
    // "Whenever you roll the planar die" triggers when the teammate rolls, and "draw a
    // card" applies to both of them.
    let (h0, h1, h2) = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    roll(&mut t, P1, PlanarFace::Blank);
    t.resolve_all();
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)),
        (h0 + 1, h1 + 1, h2)
    );
}

#[test]
fn each_member_of_the_active_team_may_roll_the_planar_die() {
    cr!("901.12d");
    let mut t = two_headed();
    add_planar_deck(&mut t, P0, &["Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    // Both members of the active team may roll; the other team's players may not.
    assert!(can_roll(&mut t, P0));
    assert!(can_roll(&mut t, P1));
    assert!(!can_roll(&mut t, P2));
    // Each player's cost counts only their own previous rolls: P0's first roll is free,
    // and so is P1's after it; P0's second costs {1}.
    roll(&mut t, P0, PlanarFace::Blank);
    assert!(can_roll(&mut t, P1));
    roll(&mut t, P1, PlanarFace::Blank);
    assert!(!can_roll(&mut t, P0));
    t.lands(P0, "Wastes", 1);
    assert!(can_roll(&mut t, P0));
    roll(&mut t, P0, PlanarFace::Blank);
    assert!(t.g.battlefield.iter().any(|id| t.obj(*id).tapped));
}

#[test]
fn plane_abilities_ignore_the_limited_range_of_influence_except_in_grand_melee() {
    cr!("901.13");
    // Free-for-All Planechase with a range of influence of 1.
    let mut t = TestGame::with_config(
        5,
        GameConfig {
            range_of_influence: Some(1),
            ..GameConfig::planechase_game()
        },
    );
    let far = t.battlefield(P2, "Grizzly Bears");
    add_planar_deck(&mut t, P0, &["Krosa"]);
    planechase::set_starting_plane(&mut t.g);
    t.g.recompute();
    assert_eq!(t.pt(far), (4, 4));
    // Grand Melee Planechase: Krosa affects only creatures within range.
    let mut t = TestGame::with_config(8, GameConfig::grand_melee_planechase());
    let near = t.battlefield(P1, "Grizzly Bears");
    let far = t.battlefield(P2, "Grizzly Bears");
    add_planar_deck(&mut t, P0, &["Krosa"]);
    add_planar_deck(&mut t, P4, &["Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    grand_melee::ensure(&mut t.g);
    t.g.recompute();
    assert_eq!(t.pt(near), (4, 4));
    assert_eq!(t.pt(far), (2, 2));
}

/// A Grand Melee Planechase game of eight players (two turn markers: P0 and P4), each of
/// P0 and P4 with a planar deck; the starting planes are set and the markers handed out.
fn grand_melee_planechase() -> TestGame {
    let mut t = TestGame::with_config(
        8,
        GameConfig {
            starting_player: Some(P0),
            ..GameConfig::grand_melee_planechase()
        },
    );
    add_planar_deck(&mut t, P0, &["Krosa", "Tazeem"]);
    add_planar_deck(&mut t, P4, &["Goldmeadow", "Naar Isle"]);
    planechase::set_starting_plane(&mut t.g);
    grand_melee::ensure(&mut t.g);
    t.g.recompute();
    t
}

fn face_up_of(t: &TestGame, name: &str) -> ObjectId {
    planechase::face_up_planar_cards(&t.g)
        .into_iter()
        .find(|id| t.obj(*id).chars.name == name)
        .unwrap_or_else(|| panic!("{name} isn't face up"))
}

#[test]
fn grand_melee_planechase_has_several_face_up_planes() {
    cr!("901.14");
    let mut t = grand_melee_planechase();
    let mut up = face_up_names(&t);
    up.sort();
    assert_eq!(up, vec!["Goldmeadow", "Krosa"]);
    // P0 planeswalks away from their plane only.
    planechase::planeswalk(&mut t.g, P0);
    let mut up = face_up_names(&t);
    up.sort();
    assert_eq!(up, vec!["Goldmeadow", "Tazeem"]);
    assert_eq!(t.obj(face_up_of(&t, "Tazeem")).controller, P0);
    assert_eq!(t.obj(face_up_of(&t, "Goldmeadow")).controller, P4);
}

#[test]
fn each_player_starting_with_a_turn_marker_sets_a_starting_plane() {
    cr!("901.14a");
    let t = grand_melee_planechase();
    assert_eq!(grand_melee::markers(&t.g).len(), 2);
    let pcs = planechase::planar_controllers(&t.g);
    assert_eq!(pcs, vec![P0, P4]);
    assert_eq!(t.obj(face_up_of(&t, "Krosa")).controller, P0);
    assert_eq!(t.obj(face_up_of(&t, "Goldmeadow")).controller, P4);
    // Each set a starting plane from their own planar deck.
    assert_eq!(t.obj(face_up_of(&t, "Goldmeadow")).owner, P4);
}

#[test]
fn a_planar_controller_leaving_that_removes_a_turn_marker_isnt_replaced() {
    cr!("901.14b");
    let mut t = grand_melee_planechase();
    // Turn marker 2 has passed from P4 to P5, who now controls P4's Goldmeadow.
    t.g.multiplayer.grand_melee.as_mut().unwrap().markers[1].holder = pid(5);
    t.g.recompute();
    let gold = face_up_of(&t, "Goldmeadow");
    assert_eq!(t.obj(gold).controller, pid(5));
    // P5 leaves: seven players need only one turn marker. P5 stops being a planar
    // controller without anyone taking over; Goldmeadow goes to the bottom of its
    // owner's planar deck, and no one planeswalks.
    let before = planeswalks(&t);
    t.g.turn.priority = Some(pid(5));
    t.g.perform_action(pid(5), Action::Concede).unwrap();
    t.g.recompute();
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
    let deck = planechase::planar_deck(&t.g, P4);
    assert_eq!(name_of(&t, *deck.last().unwrap()), "Goldmeadow");
    assert_eq!(planeswalks(&t), before);
    assert_eq!(planechase::planar_controllers(&t.g), vec![P0]);
}

#[test]
fn the_single_planar_deck_option() {
    cr!("901.15", "901.15a");
    // A communal planar deck: at least forty cards or ten per player, whichever is
    // smaller; no more phenomena than twice the number of players.
    let ten = cards(&TEN_PLANES);
    assert!(check_planar_deck(&ten, Some(1)).is_empty());
    assert!(check_planar_deck(&ten, Some(2))
        .iter()
        .any(|p| matches!(p, DeckProblem::TooFewCards { min: 20, .. })));
    // Five players: forty cards is the smaller number (not fifty).
    assert!(check_planar_deck(&ten, Some(5))
        .iter()
        .any(|p| matches!(p, DeckProblem::TooFewCards { min: 40, .. })));
    // Two players may have four phenomena, not five; names are all different.
    let mut deck = cards(&TEN_PLANES);
    deck.extend(cards(&PHENOMENA));
    let problems = check_planar_deck(&deck, Some(1));
    assert!(problems
        .iter()
        .any(|p| matches!(p, DeckProblem::TooManyPhenomena { have: 4, max: 2 })));
    deck.extend(cards(&["Krosa"]));
    assert!(check_planar_deck(&deck, Some(2))
        .iter()
        .any(|p| matches!(p, DeckProblem::TooManyCopies { name, .. } if name == "Krosa")));
}

#[test]
fn the_planar_controller_owns_the_communal_planar_deck() {
    cr!("901.15b");
    let mut t = planechase_game(3, true);
    let deck = add_planar_deck(&mut t, P2, &["Krosa", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    t.g.recompute();
    for id in planechase::face_up_planar_cards(&t.g)
        .into_iter()
        .chain(planechase::planar_deck(&t.g, P0))
    {
        assert_eq!(t.obj(id).owner, P0);
    }
    t.set_step(P1, Step::Upkeep);
    assert_eq!(t.obj(t.g.current(deck[1])).owner, P1);
}

#[test]
fn rules_referring_to_a_players_planar_deck_use_the_communal_deck() {
    cr!("901.15c");
    let mut t = planechase_game(3, true);
    let deck = add_planar_deck(&mut t, P2, &["Krosa", "Goldmeadow", "Tazeem"]);
    for p in [P0, P1, P2] {
        assert_eq!(planechase::planar_deck(&t.g, p), deck);
    }
    // P0 sets the starting plane from the communal deck; P1 planeswalks from it.
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
    t.set_step(P1, Step::PrecombatMain);
    planechase::planeswalk(&mut t.g, P1);
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
}
