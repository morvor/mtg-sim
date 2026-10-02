//! Rulings batch P223 — when and how much you scry (CR 701.22, 608.2c): X values checked
//! on resolution, scry 0 being no event, separate scries, and the order of a spell's or
//! ability's instructions.

use crate::r_p223_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s03_common::{in_hand_with_mana, priority_asked_since};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Fills the top of `p`'s library with `n` Hill Giants and returns them, top first.
fn giants(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    let mut v: Vec<ObjectId> = (0..n).map(|_| t.library_top(p, "Hill Giant")).collect();
    v.reverse();
    v
}

/// Arwen Undómiel's "Whenever you scry" triggers on the stack.
fn arwen_triggers(t: &TestGame) -> usize {
    triggers_on_stack(t, "Whenever you scry")
}

#[test]
fn cascade_seer_with_no_party_doesnt_scry_and_scry_triggers_dont_trigger() {
    cr!("701.22b", "700.8", "608.2h");
    ruling!("Cascade Seer", "If there are no creatures in your party as Cascade Seer's ability resolves, you don't scry at all. Abilities that trigger whenever you scry don't trigger.");
    supported("Cascade Seer");
    supported("Arwen Undómiel");
    // Arwen (an Elf Noble) isn't in a party. Cascade Seer (a Merfolk Wizard) is, until it
    // leaves the battlefield.
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 3);
        t.battlefield(P0, "Arwen Undómiel");
        let from = t.asked().len();
        let seer = t.enter(P0, "Cascade Seer");
        t.settle();
        if leaves {
            destroy(&mut t, seer);
        }
        t.resolve();
        if leaves {
            assert!(scries_since(&t, from).is_empty());
            assert_eq!(scry_events(&t, P0), 0);
            assert_eq!(arwen_triggers(&t), 0);
        } else {
            assert_eq!(scry_sizes(&t, P0, from), vec![1]);
            assert_eq!(arwen_triggers(&t), 1);
        }
    }
}

#[test]
fn oath_of_jace_with_no_planeswalkers_doesnt_scry() {
    cr!("701.22b");
    ruling!("Oath of Jace", "If you control no planeswalkers as the last ability resolves, you won’t scry at all. Abilities that trigger whenever you scry won’t trigger.");
    supported("Oath of Jace");
    for walkers in [0usize, 2] {
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 3);
        t.battlefield(P0, "Oath of Jace");
        t.battlefield(P0, "Arwen Undómiel");
        for name in ["Samut, Tyrant Smasher", "Jace Beleren"].iter().take(walkers) {
            t.battlefield(P0, name);
        }
        t.set_step(P0, Step::End);
        t.advance_to(P0, Step::Upkeep);
        let from = t.asked().len();
        t.resolve();
        if walkers == 0 {
            assert!(scries_since(&t, from).is_empty());
            assert_eq!(arwen_triggers(&t), 0);
        } else {
            assert_eq!(scry_sizes(&t, P0, from), vec![2]);
            assert_eq!(arwen_triggers(&t), 1);
        }
    }
}

#[test]
fn alibou_with_no_tapped_artifacts_deals_no_damage_and_doesnt_scry() {
    cr!("701.22b", "608.2h");
    ruling!("Alibou, Ancient Witness", "If you control no tapped artifacts when the triggered ability resolves (perhaps because they were destroyed or had vigilance), no damage will be dealt and you won't scry.");
    supported("Alibou, Ancient Witness");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let alibou = t.battlefield(P0, "Alibou, Ancient Witness");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(
        &mut t,
        &[(alibou, Entity::Player(P1)), (thopter, Entity::Player(P1))],
    );
    // Untapped before the trigger resolves (as if they had vigilance).
    t.g.untap(alibou);
    t.g.untap(thopter);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(scries_since(&t, from).is_empty());
}

#[test]
fn watchful_automaton_activated_twice_scries_one_twice() {
    cr!("701.22a", "602.2");
    ruling!("Watchful Automaton", "If you activate Watchful Automaton’s ability more than once, you’ll scry 1 each time. You won’t be able to look at multiple cards at once.");
    supported("Watchful Automaton");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let bot = t.battlefield(P0, "Watchful Automaton");
    t.lands(P0, "Island", 6);
    let from = t.asked().len();
    t.activate(P0, bot, 0, &[]).unwrap();
    t.activate(P0, bot, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![1, 1]);
}

#[test]
fn reaper_of_the_wilds_triggers_for_each_creature() {
    cr!("603.2c", "701.22a");
    ruling!("Reaper of the Wilds", "Reaper of the Wilds's first ability triggers separately for each creature. For example, if five creatures die at the same time, you'll scry 1 five times. You won't scry 5.");
    supported("Reaper of the Wilds");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 5);
    t.battlefield(P0, "Reaper of the Wilds");
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    // Pyroclasm: 2 damage to each creature; the 4/5 Reaper survives.
    let clasm = in_hand_with_mana(&mut t, P0, "Pyroclasm");
    let from = t.asked().len();
    t.cast(P0, clasm).go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(scry_sizes(&t, P0, from), vec![1, 1, 1]);
}

#[test]
fn ugins_insight_uses_the_greatest_mana_value_as_it_resolves() {
    cr!("608.2h", "701.22a");
    ruling!("Ugin's Insight", "Use the highest mana value among permanents you control as Ugin's Insight resolves to determine how many cards you scry.");
    supported("Ugin's Insight");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 8);
    let giant = t.battlefield(P0, "Hill Giant");
    let insight = in_hand_with_mana(&mut t, P0, "Ugin's Insight");
    t.cast(P0, insight).go();
    // In response, the mana value 4 permanent leaves and a mana value 2 one arrives.
    destroy(&mut t, giant);
    t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![2]);
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn cryptic_annelid_scries_one_two_three_in_order() {
    cr!("608.2c", "701.22a");
    ruling!("Cryptic Annelid", "You do the three different scry actions in sequence, in the order they're printed on the card.");
    supported("Cryptic Annelid");
    let mut t = TestGame::new(2);
    let g = giants(&mut t, P0, 6);
    // The first scry puts the top card on the bottom; the second sees the next two.
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![], vec![g[0]]));
    let from = t.asked().len();
    t.enter(P0, "Cryptic Annelid");
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![1, 2, 3]);
    let seen: Vec<Vec<ObjectId>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Scry { cards } => Some(cards.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(seen[1], vec![g[1], g[2]]);
}

#[test]
fn overwhelmed_apprentice_scries_once_after_every_opponent_mills() {
    cr!("608.2c", "701.22a", "701.17a");
    ruling!("Overwhelmed Apprentice", "You scry 2 once after each opponent has moved the top cards of their library. You don't scry 2 for each opponent.");
    supported("Overwhelmed Apprentice");
    let mut t = TestGame::new(3);
    giants(&mut t, P0, 3);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Scry { .. }),
        |g| (g.player(P1).graveyard.len(), g.player(P2).graveyard.len()),
    );
    let from = t.asked().len();
    t.enter(P0, "Overwhelmed Apprentice");
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![2]);
    assert_eq!(*seen.lock().unwrap(), vec![(2, 2)]);
}

#[test]
fn mischievous_chimera_scries_once_however_many_opponents_are_dealt_damage() {
    cr!("608.2c", "701.22a");
    ruling!("Mischievous Chimera", "You scry 1 just once, no matter how many opponents are dealt damage.");
    supported("Mischievous Chimera");
    let mut t = TestGame::new(3);
    giants(&mut t, P0, 3);
    t.battlefield(P0, "Mischievous Chimera");
    t.set_step(P1, Step::PrecombatMain);
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    let from = t.asked().len();
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P2), 19);
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
}

fn two_headed() -> TestGame {
    use mtg_engine::game::{GameConfig, Variant};
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
fn two_headed_giant_each_opponent_loses_life_but_you_scry_once() {
    cr!("810.9", "701.22a");
    ruling!("Mischievous Chimera", "In a Two-Headed Giant game, each opposing team loses 2 life and you scry 1.");
    supported("Mischievous Chimera");
    let mut t = two_headed();
    giants(&mut t, P0, 3);
    let team_life = t.life(P2);
    t.battlefield(P0, "Mischievous Chimera");
    t.set_step(P2, Step::PrecombatMain);
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P0).go();
    let from = t.asked().len();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P2), team_life - 2);
    assert_eq!(t.life(P3), team_life - 2);
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
}

#[test]
fn two_headed_giant_the_scarab_god_scries_only_the_number_of_zombies() {
    cr!("810.9", "701.22a");
    ruling!("The Scarab God", "In a Two-Headed Giant game, The Scarab God's first ability causes the opposing team to lose life equal to twice the number of Zombies you control, although you scry only equal to the number of Zombies you control.");
    supported("The Scarab God");
    let mut t = two_headed();
    giants(&mut t, P0, 5);
    let team_life = t.life(P2);
    t.battlefield(P0, "The Scarab God");
    t.battlefield(P0, "Gravecrawler");
    t.battlefield(P0, "Gravecrawler");
    t.set_step(P3, Step::End);
    t.advance_to(P0, Step::Upkeep);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.life(P2), team_life - 4);
    assert_eq!(scry_sizes(&t, P0, from), vec![2]);
}

#[test]
fn sphinx_of_foresight_revealed_twice_scries_three_twice() {
    cr!("103.6", "701.22a");
    ruling!("Sphinx of Foresight", "If you reveal two Sphinxes of Foresight from your opening hand, you'll scry 3 twice; you won't scry 6. Any cards you put on top of your library the first time you scry 3 will be part of the second time you scry 3.");
    supported("Sphinx of Foresight");
    let mut t = TestGame::new(2);
    let g = giants(&mut t, P0, 6);
    t.hand(P0, "Sphinx of Foresight");
    t.hand(P0, "Sphinx of Foresight");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    mtg_engine::opening_hand::opening_hand_actions(&mut t.g);
    // The first scry keeps the second card on top and bottoms the others.
    t.answer(
        P0,
        DecisionKind::Scry,
        Answer::Split(vec![g[1]], vec![g[0], g[2]]),
    );
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    let from = t.asked().len();
    t.resolve_all();
    let seen: Vec<Vec<ObjectId>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Scry { cards } => Some(cards.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(seen.len(), 2);
    assert!(seen.iter().all(|c| c.len() == 3));
    assert_eq!(seen[1][0], g[1]);
}

#[test]
fn netherese_puzzle_ward_scries_before_drawing_for_a_natural_four() {
    cr!("603.3", "706.2", "701.22a");
    ruling!("Netherese Puzzle-Ward", "If you roll a natural 4 while resolving the Focus Beam ability, you will scry before you draw a card for the Perfect Illumination ability.");
    supported("Netherese Puzzle-Ward");
    let mut t = TestGame::new(2);
    let g = giants(&mut t, P0, 4);
    t.battlefield(P0, "Netherese Puzzle-Ward");
    t.g.dice.loaded.extend([4]);
    // The top card goes to the bottom: the card drawn is the second one.
    t.answer(
        P0,
        DecisionKind::Scry,
        Answer::Split(vec![g[1], g[2], g[3]], vec![g[0]]),
    );
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(g[1]), Zone::Hand(P0));
    assert_eq!(t.zone(g[0]), Zone::Library(P0));
}

#[test]
fn lifecrafters_bestiary_scries_before_the_draw() {
    cr!("503.1", "504.1", "701.22a");
    ruling!("Lifecrafter's Bestiary", "The draw step is after the upkeep step, so you'll scry 1 before you draw for the turn.");
    supported("Lifecrafter's Bestiary");
    let mut t = TestGame::new(2);
    let g = giants(&mut t, P0, 2);
    t.battlefield(P0, "Lifecrafter's Bestiary");
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![], vec![g[0]]));
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(g[1]), Zone::Hand(P0));
    assert_eq!(t.zone(g[0]), Zone::Library(P0));
}

#[test]
fn trelasarra_dies_but_you_still_scry() {
    cr!("608.2c", "603.10a", "510.2");
    ruling!("Trelasarra, Moon Dancer", "If you gain life at the same time Trelasarra, Moon Dancer is dealt lethal damage, it dies before its triggered ability can put a +1/+1 counter on it, but you will still scry 1.");
    supported("Trelasarra, Moon Dancer");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let trel = t.battlefield(P0, "Trelasarra, Moon Dancer");
    let lifelinker = t.battlefield(P0, "Vampire Nighthawk");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    let from = t.asked().len();
    t.attack(
        &[(giant, Entity::Player(P0)), (bears, Entity::Player(P0))],
        &[(trel, giant), (lifelinker, bears)],
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Trelasarra, Moon Dancer"));
    assert_eq!(t.life(P0), 22);
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
}

#[test]
fn tenth_district_legionnaire_scries_even_without_the_counter() {
    cr!("608.2c", "603.2");
    ruling!("Tenth District Legionnaire", "You scry 1 even if you can't put a +1/+1 counter on Tenth District Legionnaire, most likely because it has left the battlefield.");
    supported("Tenth District Legionnaire");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let leg = t.battlefield(P0, "Tenth District Legionnaire");
    let growth = in_hand_with_mana(&mut t, P0, "Giant Growth");
    t.cast(P0, growth).target(leg).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    destroy(&mut t, leg);
    let from = t.asked().len();
    t.resolve();
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
}

#[test]
fn harsh_scrutiny_scries_even_with_no_creature_card_to_discard() {
    cr!("608.2c", "701.9a");
    ruling!("Harsh Scrutiny", "You scry 1 even if that player has no creature card to discard.");
    supported("Harsh Scrutiny");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    t.hand(P1, "Lightning Bolt");
    let hs = in_hand_with_mana(&mut t, P0, "Harsh Scrutiny");
    let from = t.asked().len();
    t.cast(P0, hs).target(P1).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Lightning Bolt"));
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
}

#[test]
fn undercity_scavenger_sacrifices_one_creature_on_resolution_with_no_window() {
    cr!("608.2c", "117.3b", "701.21a");
    ruling!("Undercity Scavenger", "You can’t sacrifice multiple creatures to put more +1/+1 counters on Undercity Scavenger or to scry more.");
    ruling!("Undercity Scavenger", "You choose whether to sacrifice a creature (and which one to sacrifice) while Undercity Scavenger’s ability is resolving. No player may take actions between the time you choose which creature to sacrifice, the time Undercity Scavenger has +1/+1 counters on it, and the time you scry 2.");
    supported("Undercity Scavenger");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let scav = t.enter(P0, "Undercity Scavenger");
    t.settle();
    // Nothing is chosen until the ability resolves.
    assert_eq!(t.stack_len(), 1);
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    let from = t.asked().len();
    t.g.resolve_top();
    let after = t.asked().len();
    t.settle();
    // Exactly one creature was sacrificed.
    assert_eq!(
        [a, b].iter().filter(|x| t.on_battlefield(**x)).count(),
        1,
        "one sacrifice"
    );
    assert_eq!(t.counters(scav, counters::PLUS1), 2);
    assert_eq!(scry_sizes(&t, P0, from), vec![2]);
    // No priority during the resolution.
    assert!(t.asked()[from..after]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
}

#[test]
fn sifter_wurm_scries_and_reveals_with_no_window_between() {
    cr!("608.2c", "117.3b");
    ruling!("Sifter Wurm", "Once Sifter Wurm's triggered ability begins to resolve, no player may take other actions until it's done. Notably, opponents can't try to change your library after you scry but before you reveal the top card of your library.");
    supported("Sifter Wurm");
    let mut t = TestGame::new(2);
    let g = giants(&mut t, P0, 2);
    t.library_top(P0, "Grizzly Bears");
    // Library: Bears, Giant, Giant. Keep the Giant (mana value 4) on top.
    let bears = library_top(&t);
    t.answer(
        P0,
        DecisionKind::Scry,
        Answer::Split(vec![g[0], g[1]], vec![bears]),
    );
    t.enter(P0, "Sifter Wurm");
    t.settle();
    let from = t.asked().len();
    t.g.resolve_top();
    let after = t.asked().len();
    assert_eq!(t.life(P0), 24);
    assert_eq!(scry_sizes(&t, P0, from), vec![3]);
    assert!(priority_asked_since(&t, from).len() == priority_asked_since(&t, after).len());
}

fn library_top(t: &TestGame) -> ObjectId {
    *t.g.player(P0).library.last().unwrap()
}

#[test]
fn arwen_finishes_scrying_before_choosing_a_target() {
    cr!("701.22d", "603.3d");
    ruling!("Arwen Undómiel", "You finish scrying before choosing a target for Arwen's first ability.");
    supported("Arwen Undómiel");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let arwen = t.battlefield(P0, "Arwen Undómiel");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 4);
    let from = t.asked().len();
    t.activate(P0, arwen, 0, &[]).unwrap();
    t.answer_targets(P0, &[Entity::Object(arwen)]);
    t.resolve_all();
    let scry = first_scry(&t, from).expect("scried");
    let target = t
        .asked()
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, (_, d))| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(i, _)| i)
        .expect("target chosen");
    assert!(scry < target);
    assert_eq!(t.counters(arwen, counters::PLUS1), 1);
}

#[test]
fn spite_of_mogis_doesnt_count_itself() {
    cr!("608.2c", "608.2n");
    ruling!("Spite of Mogis", "Spite of Mogis isn't counted among the number of instant and sorcery cards in your graveyard. It isn't put there until after it deals damage and you scry 1.");
    supported("Spite of Mogis");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    t.graveyard(P0, "Lightning Bolt");
    let giant = t.battlefield(P1, "Hill Giant");
    let spite = in_hand_with_mana(&mut t, P0, "Spite of Mogis");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Scry { .. }),
        |g| g.player(P0).graveyard.len(),
    );
    t.cast(P0, spite).target(giant).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 1);
    assert_eq!(*seen.lock().unwrap(), vec![1]);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn artisans_sorrow_destroys_before_you_scry() {
    cr!("608.2c");
    ruling!("Artisan's Sorrow", "The artifact or enchantment won't be on the battlefield when you scry, unless it regenerated or had indestructible.");
    supported("Artisan's Sorrow");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let relic = t.battlefield(P1, "Ornithopter");
    let sorrow = in_hand_with_mana(&mut t, P0, "Artisan's Sorrow");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Scry { .. }),
        |g| g.player(P1).graveyard.len(),
    );
    t.cast(P0, sorrow).target(relic).go();
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![1]);
    // Indestructible: still there when you scry.
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 3);
    let citadel = t.battlefield(P1, "Darksteel Citadel");
    let sorrow = in_hand_with_mana(&mut t, P0, "Artisan's Sorrow");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Scry { .. }),
        |g| g.battlefield.len(),
    );
    let before = t.g.battlefield.len();
    t.cast(P0, sorrow).target(citadel).go();
    t.resolve_all();
    assert!(t.on_battlefield(citadel));
    // The lands paid for the spell are still there; nothing left the battlefield.
    assert_eq!(*seen.lock().unwrap(), vec![before]);
}

#[test]
fn aangs_iceberg_must_still_be_there_to_be_sacrificed_for_the_scry() {
    cr!("608.2c", "701.21a");
    ruling!("Aang's Iceberg", "If Aang's Iceberg leaves the battlefield before the activated ability resolves, you will be unable to sacrifice Aang's Iceberg and will not scry, even if Aang's Iceberg was sacrificed via a different ability while its activated ability was on the stack.");
    supported("Aang's Iceberg");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        giants(&mut t, P0, 3);
        let berg = t.battlefield(P0, "Aang's Iceberg");
        t.lands(P0, "Wastes", 3);
        t.activate(P0, berg, 0, &[]).expect("waterbend");
        assert!(t.on_battlefield(berg), "sacrificed on resolution, not as a cost");
        if leaves {
            destroy(&mut t, berg);
        }
        let from = t.asked().len();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Aang's Iceberg"));
        assert_eq!(
            scry_sizes(&t, P0, from),
            if leaves { vec![] } else { vec![2] }
        );
    }
}

#[test]
fn serum_sovereigns_oil_counter_is_removed_as_a_cost() {
    cr!("602.2b", "601.2h", "118.3");
    ruling!("Serum Sovereign", "Removing an oil counter is part of the cost to activate Serum Sovereign's last ability. Players can't respond with a spell or ability that removes counters to prevent you from drawing a card and scrying 2.");
    supported("Serum Sovereign");
    let mut t = TestGame::new(2);
    giants(&mut t, P0, 4);
    let sov = t.battlefield(P0, "Serum Sovereign");
    t.g.objects[sov.0 as usize]
        .counters
        .insert(counters::OIL.into(), 1);
    t.lands(P0, "Island", 1);
    t.activate(P0, sov, 0, &[]).expect("activate");
    // Paid: the counter is gone while the ability waits on the stack.
    assert_eq!(t.counters(sov, counters::OIL), 0);
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(scry_sizes(&t, P0, from), vec![2]);
    // With no oil counter, it can't be activated.
    t.lands(P0, "Island", 1);
    assert!(t.activate(P0, sov, 0, &[]).is_err());
}

#[test]
fn a_shard_token_is_a_colorless_enchantment_that_scries_then_draws() {
    cr!("111.10e", "701.22a");
    ruling!("Niko Aris", "Shard is a new predefined token, similar to Food and Treasure. A Shard token is a colorless enchantment with \"{2}, Sacrifice this enchantment: Scry 1, then draw a card.\" Shard is a new enchantment subtype.");
    let mut t = TestGame::new(2);
    let g = giants(&mut t, P0, 2);
    let shard = create_token(&mut t, P0, "Shard");
    let o = t.obj_now(shard);
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert!(o.chars.is(CardType::Enchantment));
    assert!(!o.chars.is(CardType::Creature) && !o.chars.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Shard"));
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![], vec![g[0]]));
    let from = t.asked().len();
    t.activate(P0, shard, 0, &[]).expect("Shard");
    assert!(!t.on_battlefield(shard), "sacrificed as a cost");
    t.resolve_all();
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
    // It scried first: the card drawn is the one that was second.
    assert_eq!(t.zone(g[1]), Zone::Hand(P0));
}

/// P0 has `others` on the battlefield; Littjara Kinseekers (a changeling) enters. Returns
/// the Kinseekers after its ability is put on the stack (if it triggered).
fn kinseekers(t: &mut TestGame, others: &[&str]) -> ObjectId {
    supported("Littjara Kinseekers");
    giants(t, P0, 3);
    for o in others {
        t.battlefield(P0, o);
    }
    let k = t.enter(P0, "Littjara Kinseekers");
    t.settle();
    k
}

#[test]
fn littjara_kinseekers_checks_for_three_sharing_a_type_twice() {
    cr!("603.4", "702.73a", "608.2h");
    ruling!("Littjara Kinseekers", "If you don’t control three or more creatures that share a creature type immediately after Littjara Kinseekers enters the battlefield, its ability doesn’t trigger. If you don’t control three or more as the ability resolves, you won’t put a +1/+1 counter on Littjara Kinseekers or scry 1. The three shared-type creatures you control when the ability resolves don’t have to be the same three you controlled when the ability triggered.");
    // A Bear and a Giant: with the changeling, only two share any type.
    let mut t = TestGame::new(2);
    kinseekers(&mut t, &["Grizzly Bears", "Hill Giant"]);
    assert_eq!(t.stack_len(), 0);
    // Two Bears: it triggers; one leaves before it resolves: nothing happens.
    let mut t = TestGame::new(2);
    let k = kinseekers(&mut t, &["Grizzly Bears", "Grizzly Bears"]);
    assert_eq!(t.stack_len(), 1);
    let bear = with_subtype(&t, P0, "Bear")
        .into_iter()
        .find(|b| *b != k)
        .unwrap();
    destroy(&mut t, bear);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.counters(k, counters::PLUS1), 0);
    assert!(scries_since(&t, from).is_empty());
    // Two Bears when it triggers, two Giants (and no Bears) when it resolves: a
    // different three share a type.
    let mut t = TestGame::new(2);
    let k = kinseekers(&mut t, &["Grizzly Bears", "Grizzly Bears"]);
    for b in with_subtype(&t, P0, "Bear").into_iter().filter(|b| *b != k) {
        destroy(&mut t, b);
    }
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Hill Giant");
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.counters(k, counters::PLUS1), 1);
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
}

#[test]
fn littjara_kinseekers_gets_one_counter_however_many_trios() {
    cr!("603.4", "608.2c");
    ruling!("Littjara Kinseekers", "You put just one +1/+1 counter on Littjara Kinseekers and scry 1, no matter how many extra trios of creatures that share a creature type you control.");
    let mut t = TestGame::new(2);
    let k = kinseekers(
        &mut t,
        &[
            "Grizzly Bears",
            "Grizzly Bears",
            "Grizzly Bears",
            "Hill Giant",
            "Hill Giant",
            "Hill Giant",
        ],
    );
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.counters(k, counters::PLUS1), 1);
    assert_eq!(scry_sizes(&t, P0, from), vec![1]);
}

#[test]
fn synchronized_eviction_costs_less_with_two_creatures_sharing_a_type() {
    cr!("601.2f");
    supported("Synchronized Eviction");
    // "This spell costs {2} less to cast if you control at least two creatures that share
    // a creature type." Three lands: castable only with two Bears.
    for (others, castable) in [
        (vec!["Grizzly Bears", "Grizzly Bears"], true),
        (vec!["Grizzly Bears", "Hill Giant"], false),
    ] {
        let mut t = TestGame::new(2);
        for o in &others {
            t.battlefield(P0, o);
        }
        let target = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Island", 3);
        let se = t.hand(P0, "Synchronized Eviction");
        let r = t.cast(P0, se).target(target).try_go();
        assert_eq!(r.is_ok(), castable, "{others:?}");
    }
}
