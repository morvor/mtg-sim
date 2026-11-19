//! Rulings batch P115 — Word of Command: controlling another player while they play the
//! chosen card (CR 723), which cards can be chosen, and what "play" means (CR 305).

use crate::r_p115_common::*;
use crate::r_s01_common::watch;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn next_seq() -> usize {
    SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

/// P0 casts Word of Command targeting P1 (choosing `chosen`, if any, as it resolves);
/// both players pass, so it resolves through the priority loop.
fn word_of_command(t: &mut TestGame, chosen: Option<ObjectId>) -> ObjectId {
    t.lands(P0, "Swamp", 2);
    let woc = t.hand(P0, "Word of Command");
    if let Some(c) = chosen {
        t.answer_choose(P0, &[obj(c)]);
    }
    let spell = t.cast(P0, woc).target(pl(P1)).go();
    t.g.turn.passes = 0;
    t.g.pass_priority(P0);
    t.g.pass_priority(P1);
    spell
}

#[test]
fn word_of_command_can_be_responded_to_and_chooses_on_resolution() {
    cr!("608.2c", "117.3c");
    ruling!(
        "Word of Command",
        "Since this spell is an instant, your opponent gets a chance to respond to it as normal. Once this spell resolves, you look at your opponent's hand and choose a card."
    );
    supported("Word of Command");
    // P1 responds by casting the Shock in their hand; Word of Command then finds only
    // Grizzly Bears in it.
    let mut t = TestGame::new(2);
    t.hand(P1, "Shock");
    t.hand(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.lands(P1, "Forest", 2);
    let woc = t.hand(P0, "Word of Command");
    t.cast(P0, woc).target(pl(P1)).go();
    let shock = t.g.player(P1).hand[0];
    lots(&mut t, P1);
    t.answer_targets(P1, &[pl(P0)]);
    t.g.turn.priority = Some(P1);
    t.g.cast_spell(P1, shock, mtg_engine::object::CastMethod::Normal)
        .expect("respond");
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    let offered: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if p == P0 && prompt.contains("that player's hand") => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].len(), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn word_of_command_chooses_only_from_the_hand() {
    cr!("723.5");
    ruling!(
        "Word of Command",
        "Word of Command can't be used to force a player to play a card that isn't in their hand."
    );
    // Think Twice in P1's graveyard (flashback) isn't offered; only the hand's cards.
    let mut t = TestGame::new(2);
    let twice = t.graveyard(P1, "Think Twice");
    let bears = t.hand(P1, "Grizzly Bears");
    let from = t.asked().len();
    word_of_command(&mut t, None);
    let offered: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("that player's hand") => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![vec![obj(bears)]]);
    assert!(!offered[0].contains(&obj(twice)));
}

#[test]
fn word_of_command_can_make_the_player_play_a_land() {
    cr!("305.2b", "305.3", "723.7");
    ruling!(
        "Word of Command",
        "To \"play a card\" is to either cast a spell or to put a land onto the battlefield using the main phase special action."
    );
    ruling!(
        "Word of Command",
        "You may Command your opponent to play a land (if they have not already done so this turn)."
    );
    // On P1's turn (Word of Command cast at instant speed), P1 plays the chosen land.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let forest = t.hand(P1, "Forest");
    word_of_command(&mut t, Some(forest));
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert_eq!(t.g.player(P1).lands_played_this_turn, 1);
    // If P1 already played a land this turn, it stays in their hand.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    t.g.players[P1.idx()].lands_played_this_turn = 1;
    let forest = t.hand(P1, "Forest");
    word_of_command(&mut t, Some(forest));
    assert_eq!(t.zone(forest), Zone::Hand(P1));
    // On P0's turn, P1 can't play a land at all.
    let mut t = TestGame::new(2);
    let forest = t.hand(P1, "Forest");
    word_of_command(&mut t, Some(forest));
    assert_eq!(t.zone(forest), Zone::Hand(P1));
}

#[test]
fn word_of_command_lets_the_player_counter_the_forced_spell() {
    cr!("723.2", "117.3b");
    ruling!(
        "Word of Command",
        "Your opponent can't counter the Word of Command after letting you look at their hand, but they can attempt to counter the spell you force them to cast."
    );
    let mut t = TestGame::new(2);
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    // P1's priority decisions while Word of Command is on the stack, and P0's choice of
    // the card, in the order they happened.
    let prio = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        |g| (next_seq(), g.stack.iter().any(|s| g.obj(*s).chars.name == "Word of Command")),
    );
    let choice = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("that player's hand")),
        |_| next_seq(),
    );
    t.lands(P0, "Swamp", 2);
    let woc = t.hand(P0, "Word of Command");
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, woc).target(pl(P1)).go();
    // Run the priority loop until Word of Command has resolved.
    assert!(t.g.run_until(1000, |g| g
        .stack
        .iter()
        .all(|s| g.obj(*s).chars.name != "Word of Command")));
    assert!(t.in_graveyard(P0, "Word of Command"));
    assert_eq!(t.stack_len(), 1);
    let chose = choice.lock().unwrap()[0];
    let prio = prio.lock().unwrap().clone();
    assert!(prio.iter().any(|(_, woc)| *woc), "P1 could respond before");
    assert!(prio.iter().all(|(seq, woc)| !woc || *seq < chose), "{prio:?}");
    // P1 counters their own creature spell.
    let spell = t.g.stack[0];
    cast_at(&mut t, P1, "Counterspell", &[obj(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn word_of_command_controller_makes_the_forced_spells_search() {
    cr!("723.5", "723.4", "701.23a");
    ruling!(
        "Word of Command",
        "If that player is required to search their library as part of playing the card or as part of its resolution if it's cast as a spell, then you can see the cards in that player's library as well."
    );
    // Demonic Tutor forced: P0 makes P1's search choice (Hill Giant, not the Lions P1
    // would pick), seeing P1's library.
    let mut t = TestGame::new(2);
    let tutor = t.hand(P1, "Demonic Tutor");
    t.lands(P1, "Swamp", 2);
    let giant = t.library_top(P1, "Hill Giant");
    let lions = t.library_top(P1, "Savannah Lions");
    t.answer_choose(P1, &[obj(lions)]);
    // P0's choices: first the card from P1's hand, then the card found (the watch
    // records the decisions P0's agent gets other than the first).
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if !prompt.contains("that player's hand")),
        |_| (),
    );
    t.answer_choose(P0, &[obj(tutor)]);
    t.answer_choose(P0, &[obj(giant)]);
    word_of_command(&mut t, None);
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(!t.in_hand(P1, "Savannah Lions"));
    // P0's agent was asked the search, offered the cards in P1's library.
    assert_eq!(seen.lock().unwrap().len(), 1);
    let offered = t.asked().into_iter().any(|(p, d)| {
        p == P1
            && matches!(&d, Decision::ChooseEntities { candidates, .. }
                if candidates.contains(&obj(giant)) && candidates.contains(&obj(lions)))
    });
    assert!(offered);
}

#[test]
fn word_of_command_can_require_a_pitch_spells_alternative_cost() {
    cr!("723.5", "118.9");
    ruling!(
        "Word of Command",
        "If there is a non-mana way to cast a spell, as with Pitch Spells, you may require that way to be used if it is possible to do so."
    );
    supported("Fireblast");
    // P1 has six Mountains: Fireblast could be cast for its mana cost; P0 chooses the
    // alternative cost, sacrificing two Mountains.
    let mut t = TestGame::new(2);
    let blast = t.hand(P1, "Fireblast");
    t.lands(P1, "Mountain", 6);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[pl(P1)]);
    word_of_command(&mut t, Some(blast));
    t.resolve_all();
    let mountains = t
        .g
        .permanents()
        .filter(|o| o.controller == P1 && o.chars.name == "Mountain")
        .count();
    assert_eq!(mountains, 4);
    assert_eq!(t.life(P1), 16);
}
