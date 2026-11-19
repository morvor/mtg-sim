//! Rulings batch P115 — "each upkeep" and per-player triggers (Ophiomancer's siblings
//! Verdant, Celestial and Magmatic Force; Chittering Witch, Benediction of Moons, Furious
//! Spinesplitter), looking at the top of a library at upkeep (Galvanoth, Killer
//! Instinct; Visions), Fatespinner's skipped steps and Corruption of Towashi's "do this
//! only once each turn".

use crate::r_p115_common::*;
use crate::r_s02_common::target_candidates;
use mtg_engine::events::Event;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn verdant_force_triggers_at_each_upkeep() {
    cr!("503.1a", "603.2");
    ruling!(
        "Verdant Force",
        "Verdant Force’s ability triggers at the beginning of each upkeep, not just each of your upkeeps."
    );
    supported("Verdant Force");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Verdant Force");
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Saproling"), 1);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Saproling"), 2);
}

#[test]
fn magmatic_forces_controller_chooses_the_target_in_any_upkeep() {
    cr!("603.3d", "603.3a");
    ruling!(
        "Magmatic Force",
        "The controller of Magmatic Force when its ability triggers always chooses the target, even when the ability triggers during another player’s upkeep."
    );
    supported("Magmatic Force");
    let mut t = TestGame::new(2);
    let force = t.battlefield(P0, "Magmatic Force");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // At P1's upkeep, P0 is asked and targets P1's Bears; P1's answer isn't used.
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P1, &[obj(force)]);
    let from = t.asked().len();
    to_upkeep(&mut t, P1);
    assert!(!target_candidates(&t, P0, from).is_empty());
    assert!(target_candidates(&t, P1, from).is_empty());
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(force));
    assert_eq!(t.obj_now(force).damage, 0);
}

#[test]
fn each_upkeep_forces_trigger_once_per_team_upkeep_in_two_headed_giant() {
    cr!("805.4", "805.4d", "503.1a");
    ruling!(
        "Celestial Force",
        "In a Two-Headed Giant game, this ability triggers once at the beginning of each team’s upkeep."
    );
    ruling!(
        "Magmatic Force",
        "In a Two-Headed Giant game, this ability triggers once at the beginning of each team’s upkeep."
    );
    supported("Celestial Force");
    let mut t = crate::r_p014_common::thg();
    t.battlefield(P0, "Celestial Force");
    t.battlefield(P0, "Magmatic Force");
    let life0 = t.life(P0);
    let other = t.life(P2);
    // P2 and P3's team upkeep: one of each trigger.
    t.answer_targets(P0, &[pl(P2)]);
    t.advance_to(P2, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), life0 + 3);
    assert_eq!(t.life(P2), other - 3);
    // P0 and P1's team upkeep: one of each again.
    t.answer_targets(P0, &[pl(P2)]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), life0 + 6);
    assert_eq!(t.life(P2), other - 6);
}

/// A four-player free-for-all game in which P3 has lost.
fn three_left() -> TestGame {
    let mut t = TestGame::new(4);
    t.g.players[P3.idx()].life = 0;
    t.settle();
    assert!(t.has_lost(P3));
    t
}

#[test]
fn chittering_witch_counts_only_opponents_still_in_the_game() {
    cr!("800.4", "102.1");
    ruling!(
        "Chittering Witch",
        "In a multiplayer game, opponents who have left the game aren't counted by Chittering Witch's triggered ability."
    );
    supported("Chittering Witch");
    let mut t = three_left();
    t.enter(P0, "Chittering Witch");
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Rat"), 2);
}

#[test]
fn chittering_witch_can_sacrifice_itself() {
    cr!("602.2", "701.21a");
    ruling!(
        "Chittering Witch",
        "You can sacrifice Chittering Witch to pay the cost of its last ability."
    );
    let mut t = TestGame::new(2);
    let witch = t.battlefield(P0, "Chittering Witch");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(witch)]);
    act(&mut t, P0, witch, 0, &[obj(bears)]);
    assert!(t.in_graveyard(P0, "Chittering Witch"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn benediction_of_moons_counts_players_in_the_game() {
    cr!("102.1", "800.4", "810.1", "810.9", "801.2", "801.2b", "801.11");
    ruling!(
        "Benediction of Moons",
        "\"For each player\" means for each player currently in the game, which is normally two. A Two-Headed Giant game has four players. In a Free-for-All multiplayer game, players who have been eliminated or who are outside your range of influence don't count toward the total."
    );
    supported("Benediction of Moons");
    let gain = |mut t: TestGame| {
        let before = t.life(P0);
        t.answer_targets(P0, &[]);
        cast_at(&mut t, P0, "Benediction of Moons", &[]);
        t.resolve_all();
        t.life(P0) - before
    };
    assert_eq!(gain(TestGame::new(2)), 2);
    // Two-Headed Giant: four players (the team's shared life total goes up by 4).
    assert_eq!(gain(crate::r_p014_common::thg()), 4);
    // An eliminated player doesn't count.
    assert_eq!(gain(three_left()), 3);
    // Range of influence 1 in a five-player game: P0, P1 and P4.
    let t = TestGame::with_config(
        5,
        GameConfig {
            range_of_influence: Some(1),
            ..Default::default()
        },
    );
    assert_eq!(gain(t), 3);
}

#[test]
fn furious_spinesplitter_counts_damage_dealt_before_it_arrived() {
    cr!("603.2", "603.2b");
    ruling!(
        "Furious Spinesplitter",
        "Furious Spinesplitter doesn't need to have been on the battlefield when the damage was dealt."
    );
    supported("Furious Spinesplitter");
    let mut t = TestGame::new(3);
    cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    let spine = t.battlefield(P0, "Furious Spinesplitter");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(spine, "+1/+1"), 1);
}

#[test]
fn galvanoth_casts_an_x_spell_with_x_zero() {
    cr!("107.3b", "601.2b");
    ruling!(
        "Galvanoth",
        "If the card has an X in its mana cost, you must choose 0 as its value."
    );
    supported("Galvanoth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Galvanoth");
    let geyser = t.library_top(P0, "Volcanic Geyser");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[pl(P1)]);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(geyser), Zone::Graveyard(P0));
    let cast =
        t.g.history
            .spells_cast
            .iter()
            .any(|(p, s)| *p == P0 && t.g.obj(*s).chars.name == "Volcanic Geyser");
    assert!(cast);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn killer_instinct_leaves_a_noncreature_card_on_top() {
    cr!("701.20a", "603.7");
    ruling!(
        "Killer Instinct",
        "If the top card of your library isn't a creature card, it remains on top of your library. You won't have to sacrifice it at end of turn."
    );
    supported("Killer Instinct");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Killer Instinct");
    t.set_step(P1, Step::End);
    let shock = t.library_top(P0, "Shock");
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.g.player(P0).library.last().copied(), Some(shock));
    // It's drawn in the draw step; nothing is sacrificed at the end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Shock"));
    // A creature card is put onto the battlefield and sacrificed at the end step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Killer Instinct");
    t.set_step(P1, Step::End);
    t.library_top(P0, "Hill Giant");
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn visions_with_fewer_than_five_cards() {
    cr!("121.1", "704.5b", "701.24a");
    ruling!(
        "Visions",
        "If there are less than 5 cards in the library, you look at whatever ones remain and you still get the option to shuffle."
    );
    ruling!(
        "Visions",
        "This is not a draw and will not cause a player to lose if there are less than 5 cards in the library."
    );
    supported("Visions");
    let mut t = TestGame::new(2);
    crate::r_p189_common::set_library(&mut t, P1, &["Grizzly Bears", "Hill Giant"]);
    let hand = t.hand_size(P1);
    t.answer_yes(P0, true);
    let from = t.g.turn_events.len();
    cast_at(&mut t, P0, "Visions", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 2);
    assert_eq!(t.hand_size(P1), hand);
    assert!(!t.has_lost(P1));
    assert!(t.g.turn_events[from..]
        .iter()
        .any(|e| matches!(e, Event::Shuffled { .. })));
    assert!(!t.g.turn_events[from..]
        .iter()
        .any(|e| matches!(e, Event::Drew { .. })));
}

/// P1's upkeep with `n` Fatespinners on P0's battlefield, P1 choosing `choices` (indices
/// of draw step, main phase, combat phase), then the rest of P1's turn.
fn fatespinners(n: usize, choices: &[usize]) -> TestGame {
    let mut t = TestGame::new(2);
    for _ in 0..n {
        t.battlefield(P0, "Fatespinner");
    }
    t.set_step(P0, Step::End);
    for c in choices {
        t.answer(P1, DecisionKind::Option, Answer::Index(*c));
    }
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P1, Step::End);
    t
}

#[test]
fn fatespinners_skip_only_what_is_chosen_and_may_repeat_a_choice() {
    cr!("614.10", "614.1b");
    ruling!(
        "Fatespinner",
        "That opponent skips only the phases or steps that are chosen."
    );
    ruling!(
        "Fatespinner",
        "If more than one Fatespinner affects you at the start of your upkeep, you can choose the same step or phase for each one."
    );
    // Two Fatespinners, the draw step chosen twice: only the draw step is skipped.
    let t = fatespinners(2, &[0, 0]);
    let log = &t.g.turn.step_log;
    assert!(!log.contains(&Step::Draw));
    assert!(log.iter().any(|s| s.is_main()));
    assert!(log.iter().any(|s| s.is_combat()));
    // The draw step and the combat phase chosen: the main phases still happen.
    let t = fatespinners(2, &[0, 2]);
    let log = &t.g.turn.step_log;
    assert!(!log.contains(&Step::Draw));
    assert!(!log.iter().any(|s| s.is_combat()));
    assert!(log.contains(&Step::PrecombatMain));
    assert!(log.contains(&Step::PostcombatMain));
}

fn transform(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    assert!(mtg_engine::dfc::transform(&mut t.g, id));
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

#[test]
fn corruption_of_towashi_stops_triggering_once_you_draw() {
    cr!("603.2", "603.2h");
    ruling!(
        "Corruption of Towashi",
        "Once you've chosen to draw a card because of the last ability"
    );
    supported("Corruption of Towashi");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Corruption of Towashi");
    let delvers: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Delver of Secrets // Insectile Aberration"))
        .collect();
    let hand = t.hand_size(P0);
    // First transformation: P0 declines to draw; it can trigger again.
    t.answer_yes(P0, false);
    transform(&mut t, delvers[0]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Second: P0 draws.
    t.answer_yes(P0, true);
    transform(&mut t, delvers[1]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Third: no trigger this turn.
    transform(&mut t, delvers[2]);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
}
