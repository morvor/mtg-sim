//! CR 729: subgames (and restarting a subgame, CR 727.6).

use crate::r107_planechase::{add_planar_deck, planechase_game};
use mtg_engine::designations::become_monarch;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::planechase;
use mtg_engine::subgame::{self, Subgame};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Leaves `p` owning only `n` cards in their library (the rest cease to exist).
fn shrink_library(t: &mut TestGame, p: PlayerId, n: usize) {
    let lib = t.g.players[p.idx()].library.clone();
    for id in &lib[n..] {
        t.g.objects[id.0 as usize].zone = Zone::Nowhere;
    }
    t.g.players[p.idx()].library.truncate(n);
}

fn names(t: &Game, ids: &[ObjectId]) -> Vec<String> {
    let mut v: Vec<String> = ids
        .iter()
        .map(|i| t.obj(*i).chars.name.to_string())
        .collect();
    v.sort();
    v
}

/// Casts Shahrazad for P0 and lets it resolve.
fn cast_shahrazad(t: &mut TestGame) {
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Shahrazad");
    t.cast(P0, s).go();
    t.resolve_all();
}

/// Starts a subgame from `t` and wraps it in a test harness sharing `t`'s script.
fn start_subgame(t: &mut TestGame) -> (TestGame, Vec<(ObjectId, ObjectId)>) {
    let Subgame { mut game, outside } = subgame::begin(&mut t.g);
    game.logging = true;
    game.start();
    (
        TestGame {
            g: game,
            script: t.script.clone(),
        },
        outside,
    )
}

fn end_subgame(t: &mut TestGame, s: TestGame, outside: Vec<(ObjectId, ObjectId)>) {
    subgame::finish(&mut t.g, Subgame { game: s.g, outside });
}

#[test]
fn shahrazad_plays_a_subgame_and_the_main_game_resumes() {
    cr!("729.1", "729.1a", "729.3", "729.5");
    ruling!(
        "Shahrazad",
        "At the start of the sub-game both players draw their initial hand (usually 7 cards). If one player has fewer cards than required, that player loses."
    );
    ruling!(
        "Shahrazad",
        "At the end of a subgame, each player puts all cards they own that are in the subgame into their library in the main game, then shuffles them."
    );
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    // P1 has five cards in their library: they lose the subgame as it begins.
    shrink_library(&mut t, P1, 5);
    let bear = t.hand(P0, "Grizzly Bears");
    let lib0 = t.library_size(P0);
    cast_shahrazad(&mut t);
    let sub = t.g.subgames.last.as_ref().expect("a subgame was played");
    assert_eq!(sub.result, Some(GameResult::Win(vec![P0])));
    // The subgame ended in its first turn: P1 lost when state-based actions were first
    // checked, during the first upkeep.
    assert_eq!(sub.turn.number, 1);
    assert_eq!(sub.turn.step, Step::Upkeep);
    // The main game resumed where it left off: Shahrazad finished resolving, and the
    // player who didn't win lost half their life, rounded up.
    assert_eq!(t.g.turn.number, 1);
    assert_eq!(t.g.turn.step, Step::PrecombatMain);
    assert!(t.in_graveyard(P0, "Shahrazad"));
    assert_eq!((t.life(P0), t.life(P1)), (20, 10));
    // Every card went back into its owner's library; the main-game hand is untouched.
    assert_eq!(t.library_size(P0), lib0);
    assert_eq!(t.library_size(P1), 5);
    assert_eq!(t.zone(bear), Zone::Hand(P0));
    assert!(t.g.result.is_none());
}

#[test]
fn nothing_of_the_main_game_has_meaning_in_the_subgame() {
    cr!("729.1b", "729.4b");
    ruling!(
        "Shahrazad",
        "Events in a Shahrazad sub-game do not normally trigger abilities in the main game. And continuous effects in the main game do not carry over into the sub-game."
    );
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    shrink_library(&mut t, P1, 5);
    // "Whenever an opponent draws a card, Underworld Dreams deals 1 damage to that player."
    t.battlefield(P1, "Underworld Dreams");
    // "Players can't gain life."
    t.battlefield(P1, "Erebos, God of the Dead");
    become_monarch(&mut t.g, P0);
    t.g.set_day(true);
    t.g.players[0].life = 12;
    t.g.add_counters(Entity::Player(P0), counters::POISON, 3, None);
    cast_shahrazad(&mut t);
    let sub = t.g.subgames.last.as_ref().unwrap();
    // No monarch, no day or night, starting life totals and no counters in the subgame.
    assert_eq!(sub.monarch, None);
    assert_eq!(sub.day, None);
    assert_eq!(sub.player(P0).life, 20);
    assert_eq!(sub.player(P0).poison(), 0);
    assert_eq!(sub.player(P0).hand.len(), 7);
    // P0 drew seven cards in the subgame without Underworld Dreams triggering.
    assert_eq!(t.life(P0), 12);
    assert!(t.g.stack.is_empty() && t.g.pending_triggers.is_empty());
    // The main-game counters are still there.
    assert_eq!(t.g.player(P0).poison(), 3);
    assert_eq!(t.g.monarch, Some(P0));

    // Counters a player gets in a subgame cease to exist when it ends; effects in it
    // don't affect the main game.
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    let (mut s, outside) = start_subgame(&mut t);
    s.g.add_counters(Entity::Player(P0), counters::POISON, 4, None);
    become_monarch(&mut s.g, P1);
    s.g.players[0].life = 3;
    end_subgame(&mut t, s, outside);
    assert_eq!(t.g.player(P0).poison(), 0);
    assert_eq!(t.g.monarch, None);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_subgame_has_its_own_zones_made_from_the_main_game_libraries() {
    cr!("729.2", "729.4");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    let in_hand = t.hand(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.graveyard(P0, "Lava Spike");
    t.g.add_to_sideboard(P0, vec![card("Shock")]);
    let main_lib: Vec<String> = names(&t.g, &t.g.player(P0).library.clone());
    let (s, outside) = start_subgame(&mut t);
    // The main game's libraries are now the subgame's (shuffled, seven cards drawn).
    assert!(t.g.player(P0).library.is_empty());
    let mut sub_cards: Vec<ObjectId> = s.g.player(P0).library.clone();
    sub_cards.extend(s.g.player(P0).hand.clone());
    assert_eq!(names(&s.g, &sub_cards), main_lib);
    // No other main-game card moved: they're outside the subgame, with the cards
    // outside the main game.
    assert_eq!(t.zone(in_hand), Zone::Hand(P0));
    assert!(s.g.battlefield.is_empty());
    let out = names(&s.g, &s.g.player(P0).sideboard.clone());
    for n in ["Grizzly Bears", "Hill Giant", "Lava Spike", "Shock"] {
        assert!(out.contains(&n.to_string()), "{n} is outside the subgame");
    }
    assert_eq!(s.g.turn.number, 1);
    end_subgame(&mut t, s, outside);
    assert_eq!(names(&t.g, &t.g.player(P0).library.clone()), main_lib);
    // Which player goes first is determined at random: no player chooses.
    let mut first = std::collections::BTreeSet::new();
    for seed in 0..12u64 {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                skip_mulligans: true,
                ..Default::default()
            },
        );
        t.g.rng = ChaCha8Rng::seed_from_u64(seed);
        // Whoever would be asked would choose to go second.
        t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![Entity::Player(P1)]));
        t.answer(P1, DecisionKind::Entities, Answer::Entities(vec![Entity::Player(P0)]));
        let asked = t.asked().len();
        let (s, outside) = start_subgame(&mut t);
        assert!(!t.asked()[asked..].iter().any(|(_, d)| matches!(
            d,
            mtg_engine::decision::Decision::ChooseEntities { prompt, .. } if prompt.contains("first turn")
        )));
        first.insert(s.g.turn.starting_player);
        end_subgame(&mut t, s, outside);
    }
    assert_eq!(first.len(), 2);
}

#[test]
fn cards_brought_in_from_the_main_game_trigger_main_game_abilities_later() {
    cr!("729.4", "729.4a");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    // "Whenever one or more creature cards leave your graveyard, create a 1/1 black Bat
    // creature token with flying."
    t.battlefield(P0, "Desecrated Tomb");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let (mut s, outside) = start_subgame(&mut t);
    s.set_step(P0, Step::PrecombatMain);
    // In the subgame, P0 casts Living Wish ("Choose a creature or land card you own from
    // outside the game, reveal it, and put it into your hand.") and gets the main-game
    // Grizzly Bears.
    let wish = s.hand(P0, "Living Wish");
    s.lands(P0, "Forest", 2);
    let proxy = outside.iter().find(|(_, m)| *m == bears).unwrap().0;
    s.answer_choose(P0, &[Entity::Object(proxy)]);
    s.cast(P0, wish).go();
    s.resolve_all();
    assert!(s.in_hand(P0, "Grizzly Bears"));
    end_subgame(&mut t, s, outside);
    // The Bears left the main-game graveyard (into the library with the other cards);
    // the Tomb's ability waits to be put onto the stack until the main game continues.
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.g.find_in_zone(Zone::Library(P0), "Grizzly Bears").is_empty());
    assert!(t.g.stack.is_empty());
    t.settle();
    assert_eq!(t.g.stack.len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bat Token").len(), 1);
}

#[test]
fn every_traditional_card_in_the_subgame_goes_to_the_main_game_library() {
    cr!("729.5");
    ruling!("Shahrazad", "This includes cards in the subgame’s Exile zone.");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    let lib0 = t.library_size(P0);
    let (mut s, outside) = start_subgame(&mut t);
    s.set_step(P0, Step::PrecombatMain);
    // A card in exile, a phased-out permanent and a token.
    let exiled = s.g.player(P0).hand[0];
    s.g.exile_object(exiled, None);
    let phased = s.g.player(P0).hand[0];
    s.g.move_object(phased, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0));
    let now = s.g.current(phased);
    s.g.objects[now.0 as usize].phased_out = true;
    let before = t.g.objects.len();
    end_subgame(&mut t, s, outside);
    assert_eq!(t.library_size(P0), lib0);
    // Nothing else from the subgame exists in the main game.
    assert!(t.g.battlefield.iter().all(|x| x.0 < before as u32));
    assert!(t.g.exile.is_empty());
}

#[test]
fn supplementary_decks_move_into_the_subgame_and_back() {
    cr!("729.2a", "729.5a");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow", "The Fourth Sphere", "Panopticon"]);
    add_planar_deck(&mut t, P1, &["Tazeem", "Akoum"]);
    planechase::set_starting_plane(&mut t.g);
    let face_up: Vec<ObjectId> = planechase::face_up_planar_cards(&t.g);
    assert_eq!(names(&t.g, &face_up), vec!["Goldmeadow"]);
    let (s, outside) = start_subgame(&mut t);
    // The face-down planar decks moved (and one of them provided the subgame's starting
    // plane); the face-up plane stayed in the main game.
    assert_eq!(names(&t.g, &t.g.command.clone()), vec!["Goldmeadow"]);
    let sub_planes: Vec<ObjectId> = s.g.command.clone();
    assert_eq!(sub_planes.len(), 4);
    assert_eq!(planechase::face_up_planar_cards(&s.g).len(), 1);
    end_subgame(&mut t, s, outside);
    // Back in the main-game command zone: the face-up subgame plane was turned face down
    // and put back into its deck.
    assert_eq!(planechase::planar_deck(&t.g, P0).len(), 2);
    assert_eq!(planechase::planar_deck(&t.g, P1).len(), 2);
    assert_eq!(names(&t.g, &planechase::face_up_planar_cards(&t.g)), vec!["Goldmeadow"]);
}

#[test]
fn a_vanguard_moves_into_the_subgame_and_back() {
    cr!("729.2b", "729.5b");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Vanguard,
            skip_mulligans: true,
            ..Default::default()
        },
    );
    // Titania: life modifier -5.
    t.command(P0, "Titania");
    let (s, outside) = start_subgame(&mut t);
    assert!(t.g.command.is_empty());
    let v = s.g.vanguard_of(P0).expect("P0's vanguard in the subgame");
    assert_eq!(s.g.obj(v).zone, Zone::Command);
    assert_eq!(s.g.player(P0).life, 15);
    end_subgame(&mut t, s, outside);
    let v = t.g.vanguard_of(P0).expect("P0's vanguard back in the main game");
    assert_eq!(t.g.obj(v).zone, Zone::Command);
    assert!(!t.g.obj(v).face_down);
}

#[test]
fn a_commander_in_the_command_zone_moves_into_the_subgame_and_back() {
    cr!("729.2c", "729.5c");
    ruling!(
        "Tug of War",
        "If a commander was in the command zone of the subgame, it automatically moves to the command zone of the main game."
    );
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            skip_mulligans: true,
            ..Default::default()
        },
    );
    let cmdr = t.command(P0, "Isamaru, Hound of Konda");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.players[0]
        .commander_names
        .push("Isamaru, Hound of Konda".into());
    // P1's commander is on the battlefield: it stays in the main game.
    let theirs = t.battlefield(P1, "Savannah Lions");
    t.g.objects[theirs.0 as usize].is_commander = true;
    let (s, outside) = start_subgame(&mut t);
    assert!(t.g.command.is_empty());
    let sub_cmdr = s.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda");
    assert_eq!(sub_cmdr.len(), 1);
    assert!(s.g.obj(sub_cmdr[0]).is_commander);
    assert!(t.on_battlefield(theirs));
    end_subgame(&mut t, s, outside);
    let back = t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda");
    assert_eq!(back.len(), 1);
    assert!(t.g.obj(back[0]).is_commander);
}

#[test]
fn a_subgame_within_a_subgame() {
    cr!("729.6");
    ruling!(
        "Enter the Dungeon",
        "If Enter the Dungeon is cast during a subgame, a sub-subgame begins"
    );
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    let (mut s, outside) = start_subgame(&mut t);
    assert_eq!(s.g.subgames.depth, 1);
    let sub_lib = s.library_size(P1);
    // The subgame is the main game in relation to the sub-subgame.
    let (ss, outside2) = start_subgame(&mut s);
    assert_eq!(ss.g.subgames.depth, 2);
    assert!(s.g.player(P1).library.is_empty());
    assert_eq!(ss.g.player(P1).library.len() + ss.g.player(P1).hand.len(), sub_lib);
    end_subgame(&mut s, ss, outside2);
    assert_eq!(s.library_size(P1), sub_lib);
    assert!(t.g.player(P1).library.is_empty());
    end_subgame(&mut t, s, outside);
    assert_eq!(t.library_size(P1), 30);
}

#[test]
fn restarting_a_subgame_doesnt_affect_the_main_game() {
    cr!("727.6");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    t.g.players[1].life = 9;
    let (mut s, outside) = start_subgame(&mut t);
    s.set_step(P0, Step::PrecombatMain);
    let k = s.battlefield(P0, "Karn Liberated");
    s.g.objects[k.0 as usize].counters.insert("loyalty".into(), 30);
    // "−14: Restart the game, ..."
    s.activate(P0, k, 2, &[]).unwrap();
    s.g.resolve_top();
    // The subgame restarted; the main game is unaffected.
    assert_eq!(s.g.turn.number, 1);
    assert!(s.g.result.is_none());
    assert_eq!(t.life(P1), 9);
    assert!(t.g.player(P0).library.is_empty());
    // The restarted subgame is the subgame: its winner is the subgame's winner.
    s.g.lose_game(P1);
    assert_eq!(s.g.result, Some(GameResult::Win(vec![P0])));
    let result = subgame::finish(&mut t.g, Subgame { game: s.g, outside });
    assert_eq!(subgame::winners(&result), vec![P0]);
    assert_eq!(t.library_size(P0), 30 + 1);
}

#[test]
fn a_restarted_subgame_still_returns_the_right_cards_to_the_main_game() {
    cr!("727.6", "729.4a", "729.5");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    // A main-game permanent that stays in the main game, and a main-game graveyard card
    // brought into the subgame before the subgame restarts.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    let lions = t.hand(P0, "Savannah Lions");
    let (mut s, outside) = start_subgame(&mut t);
    s.set_step(P0, Step::PrecombatMain);
    let wish = s.hand(P0, "Living Wish");
    s.lands(P0, "Forest", 2);
    let proxy = outside.iter().find(|(_, m)| *m == giant).unwrap().0;
    s.answer_choose(P0, &[Entity::Object(proxy)]);
    s.cast(P0, wish).go();
    s.resolve_all();
    assert!(s.in_hand(P0, "Hill Giant"));
    // The subgame restarts: the Giant is now a card of the restarted subgame.
    let k = s.battlefield(P0, "Karn Liberated");
    s.g.objects[k.0 as usize].counters.insert("loyalty".into(), 30);
    s.activate(P0, k, 2, &[]).unwrap();
    s.g.resolve_top();
    assert_eq!(s.g.turn.number, 1);
    assert_eq!(s.g.subgames.depth, 1);
    let count = |t: &TestGame, zone: Zone, name: &str| t.g.find_in_zone(zone, name).len();
    assert_eq!(count(&s, Zone::Library(P0), "Hill Giant") + count(&s, Zone::Hand(P0), "Hill Giant"), 1);
    end_subgame(&mut t, s, outside);
    // Main-game objects that weren't brought into the subgame stayed where they were.
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(lions), Zone::Hand(P0));
    // The Giant left the main-game graveyard and is in the main-game library, once.
    assert!(!t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(count(&t, Zone::Library(P0), "Hill Giant"), 1);
    assert_eq!(count(&t, Zone::Library(P0), "Grizzly Bears"), 0);
    assert_eq!(count(&t, Zone::Library(P0), "Savannah Lions"), 0);
}
