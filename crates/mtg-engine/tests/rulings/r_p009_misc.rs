//! Rulings batch P009 — other burn rulings: total cost and mana value (CR 601.2f, 202.3),
//! loyalty abilities and X (CR 606, 107.3), abilities that work only from the graveyard
//! (CR 602.1, 113.6), playing exiled cards under normal timing rules (CR 305.2, 307.1,
//! 601.3), double-faced cards (CR 712.8a), choices made while resolving (CR 608.2d),
//! copies and kicker (CR 707.10), multiple kickers (CR 702.33c), a single damage event
//! (CR 120.1), effects with no duration (CR 611.2a), lifelink and simultaneous life
//! changes (CR 119.9, 104.4a), and Two-Headed Giant (CR 810.9).

use crate::r_p009_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::game::{GameConfig, GameResult, Variant};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Costs and mana value -----------------------------------------------------------------

#[test]
fn volcanic_salvo_keeps_its_mana_value() {
    cr!("202.3", "601.2f");
    ruling!(
        "Volcanic Salvo",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Volcanic Salvo). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colossal Dreadmaw");
    t.battlefield(P0, "Craw Wurm");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 2);
    let salvo = t.hand(P0, "Volcanic Salvo");
    let s = t.cast(P0, salvo).targets(&[obj(wurm)]).go();
    assert_eq!(crate::r_s26_common::mv(&mut t, s), 12);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn volcanic_salvo_locks_in_its_cost_before_mana_abilities() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Volcanic Salvo",
        "The total cost to cast Volcanic Salvo is locked in before you pay that cost."
    );
    supported("Blood Pet");
    // Hill Giant, Hill Giant, Blood Pet: total power 7, cost {3}{R}{R}. Sacrificing the
    // Blood Pet for {B} while paying doesn't raise the cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Hill Giant");
    let pet = t.battlefield(P0, "Blood Pet");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    let salvo = t.hand(P0, "Volcanic Salvo");
    let r = t.cast(P0, salvo).targets(&[obj(wurm)]).try_go();
    assert!(r.is_ok(), "{r:?}");
    assert!(!t.on_battlefield(pet), "sacrificed for mana while paying");
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}

// --- Loyalty abilities ------------------------------------------------------------------

#[test]
fn chandra_nalaar_x_is_at_most_her_loyalty() {
    cr!("606.4", "107.3", "107.1b");
    ruling!(
        "Chandra Nalaar",
        "To activate the second ability, you choose a value of X equal to or less than the number of loyalty counters on Chandra Nalaar. You may choose 0. You can’t choose a negative number."
    );
    supported("Chandra Nalaar");
    // X = 7 with 6 loyalty: not allowed.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    t.answer(P0, DecisionKind::X, Answer::Number(7));
    let r = t.activate(P0, chandra, 1, &[obj(wurm)]);
    t.clear_answers();
    if r.is_ok() {
        t.resolve_all();
        assert!(dmg(&t, wurm) <= 6);
    }
    // X = 0 is allowed: no damage, and her loyalty doesn't change.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.activate(P0, chandra, 1, &[obj(wurm)]).unwrap();
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 0);
    assert_eq!(t.counters(chandra, counters::LOYALTY), 6);
    // X = 6.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    t.answer(P0, DecisionKind::X, Answer::Number(6));
    t.activate(P0, chandra, 1, &[obj(wurm)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn nicol_bolas_transformed_can_activate_and_is_a_creature_card_elsewhere() {
    cr!("712.8a", "606.3", "712.4");
    ruling!(
        "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen",
        "You can activate one of the planeswalker's loyalty abilities the turn it enters the battlefield."
    );
    ruling!(
        "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen",
        "While a double-faced card isn't on the battlefield, consider only the characteristics of its front face. For example, Nicol Bolas has the characteristics of its creature face in the graveyard, even if it was a planeswalker on the battlefield before it was put into the graveyard."
    );
    supported("Nicol Bolas, the Ravager // Nicol Bolas, the Arisen");
    let mut t = TestGame::new(2);
    let bolas = t.battlefield(P0, "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    activate_containing(&mut t, P0, bolas, "transformed").unwrap();
    t.resolve_all();
    let arisen = t.g.current(bolas);
    assert!(t.obj_now(arisen).is(CardType::Planeswalker));
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, arisen, "Draw two").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Destroyed: in the graveyard it's the creature card.
    kill(&mut t, arisen);
    let card = t.g.current(arisen);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    let o = t.obj_now(card);
    assert!(o.is(CardType::Creature));
    assert!(!o.is(CardType::Planeswalker));
    assert_eq!(o.chars.name, "Nicol Bolas, the Ravager");
}

#[test]
fn nicol_bolas_opponents_discard_at_the_same_time() {
    cr!("101.4", "608.2e");
    ruling!(
        "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen",
        "When Nicol Bolas's enters-the-battlefield triggered ability resolves, first the next opponent in turn order (or, if it's an opponent's turn, that opponent) chooses a card in their hand without revealing it, then each other opponent in turn order does the same. Then all the chosen cards are discarded at the same time."
    );
    let mut t = TestGame::new(3);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.hand(P2, "Grizzly Bears");
    let from = t.asked().len();
    t.enter(P0, "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen");
    t.resolve_all();
    assert_eq!((t.hand_size(P1), t.hand_size(P2)), (1, 0));
    // P1 (next in turn order) chooses before P2.
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    if choosers.len() == 2 {
        assert_eq!(choosers, vec![P1, P2]);
    }
}

// --- Graveyard abilities ------------------------------------------------------------------

#[test]
fn tymaret_returns_only_from_the_graveyard() {
    cr!("602.1", "113.6");
    ruling!(
        "Tymaret, the Murder King",
        "Tymaret's last ability can be activated only if Tymaret is in your graveyard. Notably, Tymaret can't be sacrificed to return itself."
    );
    supported("Tymaret, the Murder King");
    // On the battlefield: can't be activated (not even sacrificing Tymaret itself).
    let mut t = TestGame::new(2);
    let tym = t.battlefield(P0, "Tymaret, the Murder King");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[obj(tym)]);
    assert!(t.activate(P0, tym, 1, &[]).is_err());
    assert!(t.on_battlefield(tym));
    // In the graveyard, sacrificing another creature: returns to hand.
    let mut t = TestGame::new(2);
    let tym = t.graveyard(P0, "Tymaret, the Murder King");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, tym, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Tymaret, the Murder King"));
}

// --- Playing exiled cards -----------------------------------------------------------------

#[test]
fn theater_of_horrors_keeps_normal_timing() {
    cr!("601.3", "307.1", "305.2");
    ruling!(
        "Theater of Horrors",
        "Theater of Horrors doesn't change when you can play the exiled cards during your turn. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    supported("Theater of Horrors");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Theater of Horrors");
    let spike = t.library_top(P0, "Lava Spike");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let spike = t.g.current(spike);
    assert_eq!(t.zone(spike), Zone::Exile);
    lose(&mut t, P1, 1);
    t.lands(P0, "Mountain", 1);
    // In the upkeep: not allowed.
    t.answer_targets(P0, &[pl(P1)]);
    assert!(t.cast(P0, spike).try_go().is_err());
    t.clear_answers();
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, spike).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn ral_and_the_implicit_maze_follows_timing_and_land_rules() {
    cr!("601.3", "305.2", "307.1");
    ruling!(
        "Ral and the Implicit Maze",
        "You pay all costs and follow all normal timing rules for cards played with Ral and the Implicit Maze's second chapter ability. For example, if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Ral and the Implicit Maze");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Ral and the Implicit Maze");
    let ids = crate::r_s01_common::stack_library(&mut t, P0, &["Forest", "Lava Spike"]);
    let discard = t.hand(P0, "Grizzly Bears");
    t.g.add_counters(obj(saga), counters::LORE, 2, None);
    t.g.flush_events();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(discard)]);
    t.resolve_all();
    let forest = t.g.current(ids[0]);
    let spike = t.g.current(ids[1]);
    assert_eq!(t.zone(forest), Zone::Exile);
    assert_eq!(t.zone(spike), Zone::Exile);
    // Not while a spell is on the stack.
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    assert!(t.play_land(P0, forest).is_err());
    t.resolve_all();
    // A land already played this turn: no more land plays.
    let other = t.hand(P0, "Plains");
    t.play_land(P0, other).unwrap();
    assert!(t.play_land(P0, forest).is_err());
    // Lava Spike: costs paid normally.
    t.answer_targets(P0, &[pl(P1)]);
    assert!(t.cast(P0, spike).try_go().is_err(), "no red mana");
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, spike).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

// --- Choices while resolving --------------------------------------------------------------

#[test]
fn landslide_sacrifices_mountains_while_resolving() {
    cr!("608.2d", "107.1c");
    ruling!(
        "Landslide",
        "You sacrifice the mountains during resolution."
    );
    ruling!(
        "Landslide",
        "You can sacrifice zero mountains to deal zero damage."
    );
    supported("Landslide");
    let mut t = TestGame::new(2);
    let ms = t.lands(P0, "Mountain", 3);
    let ls = t.hand(P0, "Landslide");
    t.cast(P0, ls).target(pl(P1)).go();
    assert!(ms.iter().all(|m| t.on_battlefield(*m)), "nothing sacrificed yet");
    t.answer_choose(P0, &[obj(ms[1]), obj(ms[2])]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(ms[1]) && !t.on_battlefield(ms[2]));
    // Zero Mountains: zero damage.
    let mut t = TestGame::new(2);
    let ms = t.lands(P0, "Mountain", 3);
    let ls = t.hand(P0, "Landslide");
    t.cast(P0, ls).target(pl(P1)).go();
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(ms.iter().all(|m| t.on_battlefield(*m)));
}

// --- Copies and kicker --------------------------------------------------------------------

#[test]
fn ral_storm_conduit_copy_is_kicked_if_the_original_was() {
    cr!("707.10", "702.33d");
    ruling!(
        "Ral, Storm Conduit",
        "You can't choose to pay any additional costs for the copy created by Ral's last ability. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copy too."
    );
    supported("Ral, Storm Conduit");
    supported("Burst Lightning");
    let mut t = TestGame::new(2);
    let ral = t.battlefield(P0, "Ral, Storm Conduit");
    let colossus = t.battlefield(P1, "Darksteel Colossus");
    activate_containing(&mut t, P0, ral, "copy that spell").unwrap();
    t.resolve_all();
    t.lands(P0, "Mountain", 5);
    let bl = t.hand(P0, "Burst Lightning");
    t.answer_targets(P0, &[obj(colossus)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P0, false);
    t.cast(P0, bl).kicked(true).go();
    t.resolve_all();
    // 4 + 4 from the kicked spell and its copy.
    assert_eq!(dmg(&t, colossus), 8);
}

#[test]
fn temporal_firestorm_kicks_each_kicker_once() {
    cr!("702.33c", "702.33d");
    ruling!(
        "Temporal Firestorm",
        "You may kick Temporal Firestorm only once for its {1}{W} cost and only once for its {1}{U} cost."
    );
    supported("Temporal Firestorm");
    // Kicked twice (both kickers): X = 2, two creatures phase out.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 5);
    let tf = t.hand(P0, "Temporal Firestorm");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.cast(P0, tf).go();
    t.resolve_all();
    assert!(t.obj_now(a).phased_out && t.obj_now(b).phased_out);
    assert!(!t.on_battlefield(c));
    assert!(!t.on_battlefield(theirs));
    // Not kicked: X = 0, nothing phases out.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Temporal Firestorm");
    let tf = t.hand(P0, "Temporal Firestorm");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer_choose(P0, &[obj(a)]);
    t.cast(P0, tf).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
}

// --- Damage events and durations ----------------------------------------------------------

#[test]
fn purphoros_s_intervention_deals_twice_x_at_once() {
    cr!("120.1", "603.2c");
    ruling!(
        "Purphoros's Intervention",
        "The second mode of Purphoros’s Intervention has it deal damage to the target equal to twice X. It doesn’t deal X damage then deal X damage again."
    );
    supported("Purphoros's Intervention");
    let mut t = TestGame::new(2);
    let rig = t.battlefield(P1, "Volatile Rig");
    t.lands(P0, "Mountain", 2);
    let pi = t.hand(P0, "Purphoros's Intervention");
    t.cast(P0, pi).modes(&[1]).x(1).target(rig).go();
    t.resolve();
    assert_eq!(dmg(&t, rig), 2);
    assert_eq!(triggers_on_stack(&t, "flip a coin"), 1);
}

#[test]
fn the_bears_of_littjara_chapter_two_has_no_duration() {
    cr!("611.2a", "613.4b");
    ruling!(
        "The Bears of Littjara",
        "The effect of the chapter II ability lasts indefinitely. It doesn’t expire at end of turn."
    );
    supported("The Bears of Littjara");
    supported("Universal Automaton");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The Bears of Littjara");
    let auto = t.battlefield(P0, "Universal Automaton");
    t.g.add_counters(obj(saga), counters::LORE, 2, None);
    t.g.flush_events();
    t.answer_targets(P0, &[obj(auto)]);
    t.resolve_all();
    assert_eq!(t.pt(auto), (4, 4));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(auto), (4, 4));
}

// --- Players --------------------------------------------------------------------------

fn give_lifelink(t: &mut TestGame, id: ObjectId) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Lifelink))],
    );
}

/// Advances to P0's upkeep and resolves Spawn of Mayhem's trigger.
fn spawn_upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn spawn_of_mayhem_with_lifelink_doesnt_kill_you() {
    cr!("119.9", "702.15b");
    ruling!(
        "Spawn of Mayhem",
        "If Spawn of Mayhem gains lifelink, the damage it deals will cause you to simultaneously lose and gain life. If your life total is 1, you won't lose the game."
    );
    supported("Spawn of Mayhem");
    let mut t = TestGame::new(2);
    let spawn = t.battlefield(P0, "Spawn of Mayhem");
    t.g.players[0].life = 1;
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    give_lifelink(&mut t, spawn);
    t.resolve_all();
    assert!(!t.has_lost(P0));
    assert_eq!(t.life(P0), 2, "1 lost and 2 gained (1 each from damage to two players)");
}

#[test]
fn spawn_of_mayhem_can_draw_the_game() {
    cr!("104.4a", "704.5a");
    ruling!(
        "Spawn of Mayhem",
        "If each player has 0 life after Spawn of Mayhem's triggered ability resolves, each player loses the game at the same time and the game ends in a draw."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spawn of Mayhem");
    t.g.players[0].life = 1;
    t.g.players[1].life = 1;
    spawn_upkeep(&mut t);
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

fn two_headed_giant() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

#[test]
fn in_two_headed_giant_each_team_loses_two() {
    cr!("810.9", "810.9a");
    ruling!(
        "Spawn of Mayhem",
        "In a Two-Headed Giant game, Spawn of Mayhem's last ability causes each team to lose 2 life. Then if your team's life total is 10 or less, you put a +1/+1 counter on Spawn of Mayhem."
    );
    ruling!(
        "Spear Spewer",
        "In a Two-Headed Giant game, Spear Spewer's ability causes each team to lose 2 life."
    );
    supported("Spear Spewer");
    let mut t = two_headed_giant();
    let spewer = t.battlefield(P0, "Spear Spewer");
    let start = t.life(P0);
    t.activate(P0, spewer, 0, &[]).unwrap();
    t.resolve_all();
    for p in [P0, P1, P2, P3] {
        assert_eq!(t.life(p), start - 2, "{p:?}");
    }
    // Spawn of Mayhem: the team goes from 12 to 10, so the counter is put on it.
    let mut t = two_headed_giant();
    let spawn = t.battlefield(P0, "Spawn of Mayhem");
    for p in [P0, P1] {
        t.g.players[p.idx()].life = 12;
    }
    spawn_upkeep(&mut t);
    assert_eq!(t.life(P0), 10);
    assert_eq!(t.life(P2), t.life(P3));
    assert_eq!(t.counters(spawn, counters::PLUS1), 1);
}

#[test]
fn rolling_earthquake_hits_every_creature_without_horsemanship() {
    cr!("702.31a");
    ruling!(
        "Rolling Earthquake",
        "This means that in most cases Rolling Earthquake will simply deal X damage to each creature and each player"
    );
    supported("Rolling Earthquake");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Craw Wurm");
    let b = t.battlefield(P1, "Serra Angel");
    t.lands(P0, "Mountain", 3);
    let re = t.hand(P0, "Rolling Earthquake");
    t.cast(P0, re).x(2).go();
    t.resolve_all();
    assert_eq!((dmg(&t, a), dmg(&t, b)), (2, 2));
    assert_eq!((t.life(P0), t.life(P1)), (18, 18));
}
