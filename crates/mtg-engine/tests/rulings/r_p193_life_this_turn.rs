//! Rulings batch P193 — abilities that count the life players gained or lost (or the
//! damage they were dealt) this turn: they count the total amount, not the change in life
//! total, include players who left the game, and are read when the ability resolves.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::run_effect;
use crate::r_s16_common::to_second_main;
use crate::r_s17_common::enter_transformed;
use crate::r_s24_common::pool;
use crate::r_s29_common::divide;
use crate::r_s14_common::triggers_from;
use crate::r_s33_common::two_headed_giant;
use mtg_engine::ability::{Effect, PlayerRef, Value};
use mtg_engine::mana::ManaType;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn lose(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.lose_life(p, n);
    t.g.flush_events();
    t.settle();
}

fn gain(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.gain_life(p, n);
    t.g.flush_events();
    t.settle();
}

/// `p` loses all their life and loses the game (CR 704.5a), leaving it (CR 800.4a).
fn knock_out(t: &mut TestGame, p: PlayerId) {
    let life = t.life(p) as u32;
    lose(t, p, life);
    assert!(t.has_lost(p));
}

fn x_max_asked(t: &TestGame, from: usize) -> Vec<i64> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseX { max, .. } => Some(max),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Life gained this turn: losses don't count against it.
// ---------------------------------------------------------------------------------------

#[test]
fn accomplished_alchemist_counts_life_gained_ignoring_life_lost() {
    cr!("119.3", "106.1");
    ruling!(
        "Accomplished Alchemist",
        "For example, if you lost 3 life and gained 3 life earlier in the turn, you'll add three mana."
    );
    supported("Accomplished Alchemist");
    let mut t = TestGame::new(2);
    let alchemist = t.battlefield(P0, "Accomplished Alchemist");
    lose(&mut t, P0, 3);
    gain(&mut t, P0, 3);
    assert_eq!(t.life(P0), 20);
    assert!(crate::r_s20_common::tap_for_mana(
        &mut t, P0, alchemist, "X mana"
    ));
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn ezzaroot_channeler_reduces_by_life_gained_ignoring_life_lost() {
    cr!("601.2f", "119.3");
    ruling!(
        "Ezzaroot Channeler",
        "For example, if you lost 3 life and gained 4 life earlier in the turn, creature spells you cast will cost {4} less."
    );
    supported("Ezzaroot Channeler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ezzaroot Channeler");
    lose(&mut t, P0, 3);
    gain(&mut t, P0, 4);
    // Craw Wurm {4}{G}{G} for {G}{G}.
    t.lands(P0, "Forest", 2);
    let wurm = t.hand(P0, "Craw Wurm");
    t.cast(P0, wurm).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn gumdrop_poisoner_and_celestine_use_life_gained_ignoring_life_lost() {
    cr!("119.3", "603.2");
    ruling!(
        "Gumdrop Poisoner // Tempt with Treats",
        "For example, if you lost 3 life and gained 3 life earlier in the turn, the target creature will get -3/-3."
    );
    ruling!(
        "Celestine, the Living Saint",
        "For example, if you gained 3 life this turn, X is 3, even if you also lost some amount of life in the same turn."
    );
    supported("Gumdrop Poisoner // Tempt with Treats");
    supported("Celestine, the Living Saint");
    let mut t = TestGame::new(2);
    let courser = t.battlefield(P1, "Centaur Courser");
    lose(&mut t, P0, 3);
    gain(&mut t, P0, 3);
    t.answer_targets(P0, &[Entity::Object(courser)]);
    t.enter(P0, "Gumdrop Poisoner // Tempt with Treats");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Centaur Courser"));
    // Celestine: gained 3, lost 5; X is 3, so a mana value 3 card can return.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Celestine, the Living Saint");
    let courser = t.graveyard(P0, "Centaur Courser");
    t.graveyard(P0, "Hill Giant");
    gain(&mut t, P0, 3);
    lose(&mut t, P0, 5);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(courser)]);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(courser)));
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands[0].len(), 1, "Hill Giant (mana value 4) isn't a candidate");
}

#[test]
fn moseo_doesnt_trigger_without_life_gained_before_the_end_step() {
    cr!("603.4", "513.1a");
    ruling!(
        "Moseo, Vein's New Dean",
        "It's not possible to gain life during your end step in time to have the ability trigger."
    );
    supported("Moseo, Vein's New Dean");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moseo, Vein's New Dean");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    gain(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Having gained 2 life before the end step, it triggers and returns the Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moseo, Vein's New Dean");
    let bears = t.graveyard(P0, "Grizzly Bears");
    gain(&mut t, P0, 2);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn lathiel_distribution_is_locked_in_and_lost_on_illegal_targets() {
    cr!("601.2d", "603.3d", "608.2b");
    ruling!(
        "Lathiel, the Bounteous Dawn",
        "Gaining more life in response to the triggered ability won't change how many counters will be distributed, nor will it change the distribution."
    );
    ruling!(
        "Lathiel, the Bounteous Dawn",
        "If some, but not all, of the creatures are illegal targets as the triggered ability tries to resolve, the original distribution of counters still applies and the counters that would have been put on illegal targets are lost."
    );
    ruling!(
        "Lathiel, the Bounteous Dawn",
        "If all of the creatures are illegal targets, the ability won't resolve."
    );
    supported("Lathiel, the Bounteous Dawn");
    for case in 0..3 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lathiel, the Bounteous Dawn");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Centaur Courser");
        gain(&mut t, P0, 3);
        t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
        divide(&mut t, P0, &[1, 2]);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        match case {
            0 => gain(&mut t, P0, 5),
            1 => crate::r_s05_common::move_to(&mut t, b, mtg_engine::object::Zone::Hand(P0))
                .map(|_| ())
                .unwrap(),
            _ => {
                crate::r_s05_common::move_to(&mut t, a, mtg_engine::object::Zone::Hand(P0));
                crate::r_s05_common::move_to(&mut t, b, mtg_engine::object::Zone::Hand(P0));
            }
        }
        t.resolve_all();
        match case {
            0 => {
                assert_eq!(t.counters(a, counters::PLUS1), 1);
                assert_eq!(t.counters(b, counters::PLUS1), 2);
            }
            1 => assert_eq!(t.counters(a, counters::PLUS1), 1),
            _ => assert!(t
                .g
                .permanents()
                .all(|o| o.counters.get(counters::PLUS1).copied().unwrap_or(0) == 0)),
        }
    }
}

// ---------------------------------------------------------------------------------------
// Life lost (or damage dealt) this turn: gains don't count against it.
// ---------------------------------------------------------------------------------------

#[test]
fn cryptborn_horror_counts_life_lost_not_the_change_in_life_total() {
    cr!("119.3", "614.1c");
    ruling!(
        "Cryptborn Horror",
        "For example, if an opponent loses 5 life and then gains 10 life on the turn Cryptborn Horror enters, it will enter with five +1/+1 counters on it."
    );
    ruling!(
        "Cryptborn Horror",
        "If no opponent has lost life on the turn Cryptborn Horror enters, it will enter with no +1/+1 counters on it and be put into its owner's graveyard"
    );
    supported("Cryptborn Horror");
    let mut t = TestGame::new(2);
    lose(&mut t, P1, 5);
    gain(&mut t, P1, 10);
    // P0's own loss doesn't count.
    lose(&mut t, P0, 2);
    let horror = t.enter(P0, "Cryptborn Horror");
    t.settle();
    assert_eq!(t.counters(horror, counters::PLUS1), 5);
    let mut t = TestGame::new(2);
    t.enter(P0, "Cryptborn Horror");
    t.settle();
    assert!(t.in_graveyard(P0, "Cryptborn Horror"));
}

#[test]
fn opponents_who_left_the_game_still_count_for_life_lost_this_turn() {
    cr!("800.4a", "119.3");
    ruling!(
        "Cryptborn Horror",
        "In multiplayer games, life lost the turn Cryptborn Horror enters by opponents who left the game later that turn will count toward the value of X."
    );
    ruling!(
        "Neheb, the Eternal",
        "If an opponent loses life and subsequently loses the game before your postcombat main phase, Neheb's last ability counts that loss of life."
    );
    ruling!(
        "Florian, Voldaren Scion",
        "If an opponent loses life and subsequently loses the game before your postcombat main phase, Florian's last ability counts that loss of life."
    );
    ruling!(
        "Rakdos, Lord of Riots",
        "If an opponent lost life and subsequently lost the game, Rakdos's first ability is satisfied, and its last ability counts that loss of life."
    );
    supported("Cryptborn Horror");
    supported("Neheb, the Eternal");
    supported("Florian, Voldaren Scion");
    supported("Rakdos, Lord of Riots");
    // Cryptborn Horror: P1 lost 20 and left; P2 lost 1.
    let mut t = TestGame::new(3);
    knock_out(&mut t, P1);
    lose(&mut t, P2, 1);
    let horror = t.enter(P0, "Cryptborn Horror");
    t.settle();
    assert_eq!(t.counters(horror, counters::PLUS1), 21);
    // Rakdos: castable, and creature spells cost {1} less per life lost.
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 2);
    let rakdos = t.hand(P0, "Rakdos, Lord of Riots");
    assert!(t.cast(P0, rakdos).try_go().is_err());
    lose(&mut t, P1, 3);
    knock_out(&mut t, P1);
    t.cast(P0, rakdos).go();
    t.resolve_all();
    let wurm = t.hand(P0, "Craw Wurm");
    t.lands(P0, "Forest", 2);
    t.cast(P0, wurm).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    // Neheb: R for each life lost, P1's included.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Neheb, the Eternal");
    lose(&mut t, P1, 2);
    knock_out(&mut t, P1);
    lose(&mut t, P2, 1);
    to_second_main(&mut t, P0);
    t.resolve_all();
    assert_eq!(pool(&t, P0, ManaType::R), 21);
    // Florian: looks at that many cards.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Florian, Voldaren Scion");
    lose(&mut t, P1, 2);
    knock_out(&mut t, P1);
    lose(&mut t, P2, 1);
    let from = t.asked().len();
    to_second_main(&mut t, P0);
    t.resolve_all();
    let looked = florian_looked(&t, from);
    assert_eq!(looked, Some(21));
}

/// The number of cards Florian's ability let P0 choose among since decision `from`.
fn florian_looked(t: &TestGame, from: usize) -> Option<usize> {
    asked_since(t, from)
        .into_iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates.len()),
            _ => None,
        })
}

#[test]
fn neheb_and_florian_count_life_lost_ignoring_life_gained() {
    cr!("119.3", "505.1");
    ruling!(
        "Neheb, the Eternal",
        "For example, if an opponent lost 2 life and then gained 8 life before your postcombat main phase, you'll add {R}{R}."
    );
    ruling!(
        "Florian, Voldaren Scion",
        "For example, if an opponent lost 2 life and then gained 8 life before Florian's ability resolves, you'll look at the top two cards of your library."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Neheb, the Eternal");
    lose(&mut t, P1, 2);
    gain(&mut t, P1, 8);
    to_second_main(&mut t, P0);
    t.resolve_all();
    assert_eq!(pool(&t, P0, ManaType::R), 2);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Florian, Voldaren Scion");
    lose(&mut t, P1, 2);
    to_second_main(&mut t, P0);
    gain(&mut t, P1, 8);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(florian_looked(&t, from), Some(2));
    assert_eq!(crate::r_s18_common::exiled_named(&t, "Filler").len(), 1);
}

#[test]
fn neheb_and_florian_dont_trigger_if_they_left_before_the_postcombat_main_phase() {
    cr!("603.2", "505.1");
    ruling!(
        "Neheb, the Eternal",
        "If an opponent loses life but Neheb leaves the battlefield before your postcombat main phase begins, its last ability doesn't trigger."
    );
    ruling!(
        "Florian, Voldaren Scion",
        "If an opponent loses life but Florian leaves the battlefield before your postcombat main phase begins, its last ability doesn't trigger."
    );
    for name in ["Neheb, the Eternal", "Florian, Voldaren Scion"] {
        let mut t = TestGame::new(2);
        let it = t.battlefield(P0, name);
        lose(&mut t, P1, 2);
        destroy(&mut t, it);
        to_second_main(&mut t, P0);
        assert_eq!(t.stack_len(), 0, "{name}");
    }
}

#[test]
fn neheb_uses_the_stack_and_can_be_countered() {
    cr!("605.1b", "603.3");
    ruling!(
        "Neheb, the Eternal",
        "Because it triggers at the beginning of a phase, Neheb's last ability isn't a mana ability. It uses the stack and can be countered"
    );
    supported("Stifle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Neheb, the Eternal");
    lose(&mut t, P1, 2);
    to_second_main(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool(&t, P0, ManaType::R), 0);
    let trigger = t.g.stack[0];
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(trigger).go();
    t.resolve_all();
    assert_eq!(pool(&t, P0, ManaType::R), 0);
}

#[test]
fn neheb_and_florian_trigger_in_each_postcombat_main_phase() {
    cr!("505.1a", "505.1");
    ruling!(
        "Neheb, the Eternal",
        "If you somehow have more than two main phases in a turn, each main phase after your first one is a postcombat main phase, and Neheb's last ability triggers at the beginning of each of them."
    );
    ruling!(
        "Florian, Voldaren Scion",
        "If you somehow have more than two main phases in a turn, each main phase after your first one is a postcombat main phase, and Florian's last ability triggers at the beginning of each of them."
    );
    supported("Relentless Assault");
    for name in ["Neheb, the Eternal", "Florian, Voldaren Scion"] {
        let mut t = TestGame::new(2);
        let it = t.battlefield(P0, name);
        lose(&mut t, P1, 1);
        to_second_main(&mut t, P0);
        assert_eq!(triggers_from(&t, it), 1, "{name}");
        t.resolve_all();
        // Relentless Assault: an additional combat phase followed by an additional main
        // phase.
        t.lands(P0, "Mountain", 4);
        let ra = t.hand(P0, "Relentless Assault");
        t.cast(P0, ra).go();
        t.resolve_all();
        crate::r_s21_common::go_to(&mut t, Step::EndOfCombat);
        crate::r_s21_common::go_to(&mut t, Step::PostcombatMain);
        t.settle();
        assert_eq!(triggers_from(&t, it), 1, "{name}: the additional main phase");
    }
}

#[test]
fn mortarion_x_is_capped_by_life_lost_including_a_life_total_set_lower() {
    cr!("119.5", "107.3");
    ruling!(
        "Mortarion, Daemon Primarch",
        "For example, if you lost 3 life due to combat damage, but gained 2 life later in the turn, X may be up to 3 for this ability."
    );
    ruling!(
        "Mortarion, Daemon Primarch",
        "For example, if your life total is 20 and an effect sets your life total to 15, you lost 5 life."
    );
    supported("Mortarion, Daemon Primarch");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mortarion, Daemon Primarch");
    lose(&mut t, P0, 3);
    gain(&mut t, P0, 2);
    t.lands(P0, "Swamp", 5);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(x_max_asked(&t, from), vec![3]);
    assert_eq!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Astartes").len(), 3);
    // An effect sets P0's life total from 20 to 15: 5 life lost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mortarion, Daemon Primarch");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::SetLife {
            who: PlayerRef::You,
            n: Value::c(15),
        },
        &[],
    );
    assert_eq!(t.g.history.life_lost.get(&P0).copied(), Some(5));
    t.lands(P0, "Swamp", 6);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(x_max_asked(&t, from), vec![5]);
}

#[test]
fn knollspine_dragon_counts_damage_dealt_not_the_life_total_change() {
    cr!("120.3a", "603.2");
    ruling!(
        "Knollspine Dragon",
        "For example, if an opponent was dealt 4 damage and gained 6 life during the turn, that player will have a higher life total than they started the turn with"
    );
    supported("Knollspine Dragon");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(src, Entity::Player(P1), 4, false);
    t.g.flush_events();
    t.settle();
    gain(&mut t, P1, 6);
    assert_eq!(t.life(P1), 22);
    t.hand(P0, "Island");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Knollspine Dragon");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
    assert!(t.in_graveyard(P0, "Island"));
}

#[test]
fn wound_reflections_each_double_the_life_lost_so_far() {
    cr!("603.2", "119.3");
    ruling!(
        "Wound Reflection",
        "For example, if an opponent lost 1 life, the first one's triggered ability to resolve causes them to lose another 1 life. Since they've now lost 2 life, the next triggered ability causes them to lose 2 life, and the next 4, and so on."
    );
    ruling!(
        "Wound Reflection",
        "If Wound Reflection's ability resolves, then an opponent loses life later in the same turn, that life loss is never counted by Wound Reflection."
    );
    supported("Wound Reflection");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Wound Reflection");
    }
    lose(&mut t, P1, 1);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 1 - 1 - 2 - 4);
    // Life lost after the abilities resolved isn't counted (this turn or later).
    lose(&mut t, P1, 2);
    t.advance_to(P1, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
}

#[test]
fn archfiend_of_despair_in_two_headed_giant_counts_each_player_separately() {
    cr!("810.9", "119.3");
    ruling!(
        "Archfiend of Despair",
        "When Archfiend of Despair’s ability resolves, each of those players again loses 2 life and the team’s life total goes down again by 4; they don’t each lose 4 life."
    );
    supported("Archfiend of Despair");
    let mut t = two_headed_giant();
    let src = t.battlefield(P0, "Archfiend of Despair");
    let start = t.life(P2);
    t.g.deal_damage(src, Entity::Player(P2), 2, false);
    t.g.deal_damage(src, Entity::Player(P3), 2, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P2), start - 4);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P2), start - 8);
    assert_eq!(t.life(P3), start - 8);
}

#[test]
fn discordant_spirit_triggers_once_per_end_step_in_two_headed_giant() {
    cr!("810.9", "603.2");
    ruling!(
        "Discordant Spirit",
        "In Two-Headed Giant, triggers only once per end of turn, not once for each player."
    );
    supported("Discordant Spirit");
    let mut t = two_headed_giant();
    let spirit = t.battlefield(P0, "Discordant Spirit");
    let src = t.battlefield(P2, "Grizzly Bears");
    t.set_step(P2, Step::PostcombatMain);
    t.g.deal_damage(src, Entity::Player(P0), 2, false);
    t.g.flush_events();
    t.settle();
    t.advance_to_step(Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, spirit), 1);
    t.resolve_all();
    assert_eq!(t.counters(spirit, counters::PLUS1), 2);
}

// ---------------------------------------------------------------------------------------
// Cost increases and additional costs apply before reductions (CR 601.2f).
// ---------------------------------------------------------------------------------------

#[test]
fn rakdos_reduction_applies_after_kicker() {
    cr!("601.2f", "702.33a");
    ruling!(
        "Rakdos, Lord of Riots",
        "If there are additional costs to cast a creature spell, such as a kicker cost or the commander tax, apply those increases before applying cost reductions."
    );
    supported("Kavu Titan");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rakdos, Lord of Riots");
    lose(&mut t, P1, 2);
    // Kicked Kavu Titan: {1}{G} + {2}{G} = {3}{G}{G}, minus {2} = {1}{G}{G}.
    t.lands(P0, "Forest", 3);
    let titan = t.hand(P0, "Kavu Titan");
    t.cast(P0, titan).kicked(true).go();
    t.resolve_all();
    let titan = t.named_on_battlefield("Kavu Titan")[0];
    assert_eq!(t.counters(titan, counters::PLUS1), 3);
}

#[test]
fn heliod_warped_eclipse_reduces_after_x_and_increases() {
    cr!("601.2f", "107.3a");
    ruling!(
        "Heliod, the Radiant Dawn // Heliod, the Warped Eclipse",
        "For example, if a spell’s mana cost is {X}{R} and your opponents have drawn two cards this turn, you could choose 5 as the value of X and pay {3}{R} to cast the spell."
    );
    ruling!(
        "Heliod, the Radiant Dawn // Heliod, the Warped Eclipse",
        "If there are additional costs to cast a spell, or if the cost to cast a spell is increased by an effect (such as the one created by Thalia, Guardian of Thraben’s ability), apply those increases before applying cost reductions."
    );
    supported("Heliod, the Radiant Dawn // Heliod, the Warped Eclipse");
    supported("Blaze");
    supported("Thalia, Guardian of Thraben");
    let mut t = TestGame::new(2);
    enter_transformed(&mut t, P0, "Heliod, the Radiant Dawn // Heliod, the Warped Eclipse");
    t.g.draw_cards(P1, 2);
    t.g.flush_events();
    t.settle();
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let blaze = t.hand(P0, "Blaze");
    assert!(t.cast(P0, blaze).x(5).target(Entity::Player(P1)).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    t.cast(P0, blaze).x(5).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // With Thalia: Blaze with X=1 costs {1}{R} + {1} - {2} = {R}.
    let mut t = TestGame::new(2);
    enter_transformed(&mut t, P0, "Heliod, the Radiant Dawn // Heliod, the Warped Eclipse");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.g.draw_cards(P1, 2);
    t.g.flush_events();
    t.settle();
    t.lands(P0, "Mountain", 1);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(1).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}
