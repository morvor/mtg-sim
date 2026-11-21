//! Rulings batch P039 — "whenever a creature deals combat damage to a player, draw a card"
//! abilities ("curiosity"): who is asked, how many times they trigger (several players,
//! first strike, creatures dealt lethal damage at the same time), illegal targets, and
//! the cards grouped with them (Enduring Curiosity, finality counters, Green Goblin,
//! Toski, Bident of Thassa, Driven // Despair, Drogskol Reaver, Frostcliff Siege).

use crate::r_p039_common::*;
use crate::r_p039_creatureland::activate_text;
use crate::r_s01_common::{attack_with, block_and_finish, supported, triggers_on_stack};
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s04_common::crew;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Runs a combat for the active player: `attackers` attack, P1 (or the attacked
/// players' controllers) block with `blocks`; everything resolves (scripted answers or
/// defaults) until the end of combat step.
fn combat(t: &mut TestGame, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    fight(t, attackers, blocks);
    t.resolve_all();
}

// ---------------------------------------------------------------------------
// Several players, several damage steps
// ---------------------------------------------------------------------------

#[test]
fn edric_triggers_for_an_opponent_hitting_another_opponent_and_that_player_chooses() {
    cr!("603.2", "510.2");
    ruling!(
        "Edric, Spymaster of Trest",
        "A creature controlled by an opponent that deals combat damage to another opponent will cause Edric's ability to trigger. The creature's controller chooses whether to draw a card."
    );
    supported("Edric, Spymaster of Trest");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Edric, Spymaster of Trest");
    let a = old(&mut t, P1, "Grizzly Bears");
    let b = old(&mut t, P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    yes(&mut t, P1, true);
    let from = mark(&t);
    let hands = (t.hand_size(P0), t.hand_size(P1));
    // One attacks P2 (Edric's controller's opponent), one attacks P0 (no trigger).
    combat(&mut t, &[(a, pl(P2)), (b, pl(P0))], &[]);
    let asked: Vec<PlayerId> = yes_nos(&t, from).into_iter().map(|(p, _)| p).collect();
    assert_eq!(asked, vec![P1]);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (hands.0, hands.1 + 1));
}

/// P0's creatures `attackers` attack P1 and P2 (one each, in a three-player game) and
/// deal combat damage at the same time; returns the number of triggers of the ability
/// whose text contains `text` that went on the stack in the combat damage step.
fn hit_two_players(t: &mut TestGame, a: ObjectId, b: ObjectId, text: &str) -> usize {
    attack_with(t, &[(a, pl(P1)), (b, pl(P2))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    triggers_on_stack(t, text)
}

#[test]
fn keeper_of_fables_triggers_for_each_player_and_each_damage_step() {
    cr!("510.2", "702.7b", "603.2c");
    ruling!(
        "Keeper of Fables",
        "If non-Human creatures you control deal combat damage to two or more players at the same time, Keeper of Fables's ability triggers for each of those players."
    );
    ruling!(
        "Keeper of Fables",
        "Because creatures with first strike deal combat damage before creatures without first strike, Keeper of Fables's ability can trigger twice during one combat if your creatures deal combat damage to one player at different times."
    );
    supported("Keeper of Fables");
    let mut t = TestGame::new(3);
    let k = old(&mut t, P0, "Keeper of Fables");
    let b = old(&mut t, P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    assert_eq!(hit_two_players(&mut t, k, b, "draw a card"), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // First strike and regular damage to one player: twice.
    let mut t = TestGame::new(2);
    let k = old(&mut t, P0, "Keeper of Fables");
    let r = old(&mut t, P0, "Boros Recruit");
    let hand = t.hand_size(P0);
    combat(&mut t, &[(k, pl(P1)), (r, pl(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand + 2);
    // Both in the same step: once.
    let mut t = TestGame::new(2);
    let k = old(&mut t, P0, "Keeper of Fables");
    let b = old(&mut t, P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    combat(&mut t, &[(k, pl(P1)), (b, pl(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn anowon_triggers_for_each_player_dealt_damage() {
    cr!("510.2", "603.2c");
    ruling!(
        "Anowon, the Ruin Thief",
        "If Rogues you control deal combat damage to two players at once, Anowon’s ability triggers twice."
    );
    let mut t = TestGame::new(3);
    let a = old(&mut t, P0, "Anowon, the Ruin Thief");
    let r = old(&mut t, P0, "Merfolk Looter");
    assert_eq!(hit_two_players(&mut t, a, r, "mills a card for each"), 2);
    t.resolve_all();
    // Anowon (2) mills P1 two cards, the Looter (1/1 + 1/1) mills P2 two.
    assert_eq!((t.graveyard_size(P1), t.graveyard_size(P2)), (2, 2));
}

#[test]
fn anowon_leaving_can_make_damage_on_rogues_lethal() {
    cr!("704.5g", "613.4c");
    ruling!(
        "Anowon, the Ruin Thief",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to other Rogues you control may become lethal if Anowon leaves the battlefield during that turn."
    );
    let mut t = TestGame::new(2);
    let a = old(&mut t, P0, "Anowon, the Ruin Thief");
    let r = old(&mut t, P0, "Jhessian Thief");
    assert_eq!(t.pt(r), (2, 4));
    cast_named(&mut t, P1, "Lightning Bolt", &[o(r)]);
    t.resolve_all();
    assert!(t.on_battlefield(r));
    destroy(&mut t, a);
    assert!(!t.on_battlefield(r));
}

#[test]
fn heroes_in_a_half_shell_triggers_per_player_and_counts_that_players_creatures() {
    cr!("510.2", "603.2c");
    ruling!(
        "Heroes in a Half Shell",
        "If creatures you control deal combat damage to multiple players at the same time, Heroes in a Half Shell's last ability will trigger for each of those players. When one of those abilities resolves, it will put counters only on Mutants, Ninjas, and/or Turtles that dealt combat damage to the appropriate player."
    );
    supported("Heroes in a Half Shell");
    let mut t = TestGame::new(3);
    let h = old(&mut t, P0, "Heroes in a Half Shell");
    let c = old(&mut t, P0, "Changeling Outcast");
    let c2 = old(&mut t, P0, "Changeling Outcast");
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(h, pl(P1)), (c, pl(P2)), (c2, pl(P2))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "put a +1/+1 counter on each"), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    for x in [h, c, c2] {
        assert_eq!(t.counters(x, counters::PLUS1), 1);
    }
}

#[test]
fn frostcliff_siege_jeskai_triggers_per_player() {
    cr!("510.2", "603.2c");
    ruling!(
        "Frostcliff Siege",
        "If you chose Jeskai and creatures you control deal combat damage to multiple players at the same time, Frostcliff Siege’s ability will trigger once for each player dealt combat damage this way."
    );
    supported("Frostcliff Siege");
    let mut t = TestGame::new(3);
    // Jeskai is the first choice.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    enter(&mut t, P0, "Frostcliff Siege");
    let a = old(&mut t, P0, "Grizzly Bears");
    let b = old(&mut t, P0, "Grizzly Bears");
    let c = old(&mut t, P0, "Hill Giant");
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P2)), (c, pl(P2))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn frostcliff_siege_copy_without_a_choice_has_neither_ability() {
    cr!("707.2", "607.2d");
    ruling!(
        "Frostcliff Siege",
        "If you somehow control Frostcliff Siege and no choice was made for it (perhaps because another permanent on the battlefield became a copy of it), it has neither of the two abilities."
    );
    // Impossible Man: "{2}{U}: Impossible Man becomes a copy of another target permanent
    // until end of turn, except his name is Impossible Man."
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let s = enter(&mut t, P0, "Frostcliff Siege");
    let m = old(&mut t, P0, "Impossible Man");
    let b = old(&mut t, P0, "Grizzly Bears");
    // Temur: Bears get +1/+0.
    assert_eq!(t.pt(b), (3, 2));
    mana_all(&mut t, P0, 3);
    t.answer_targets(P0, &[o(s)]);
    activate_text(&mut t, P0, m, "becomes a copy").unwrap();
    t.resolve_all();
    assert!(t.obj_now(m).is(CardType::Enchantment));
    // Only the real Siege's Temur bonus.
    assert_eq!(t.pt(b), (3, 2));
    let hand = t.hand_size(P0);
    combat(&mut t, &[(b, pl(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand);
}

// ---------------------------------------------------------------------------
// Dealt lethal damage at the same time
// ---------------------------------------------------------------------------

#[test]
fn ohran_frostfang_triggers_for_each_creature_while_dying() {
    cr!("510.2", "704.3");
    ruling!(
        "Ohran Frostfang",
        "If one or more creatures you control deal combat damage to players at the same time that lethal damage is dealt to Ohran Frostfang, its ability triggers for each of those creatures."
    );
    let mut t = TestGame::new(2);
    let o_ = old(&mut t, P0, "Ohran Frostfang");
    let a = old(&mut t, P0, "Grizzly Bears");
    let b = old(&mut t, P0, "Grizzly Bears");
    let blocker = old(&mut t, P1, "Typhoid Rats");
    let hand = t.hand_size(P0);
    combat(&mut t, &[(o_, pl(P1)), (a, pl(P1)), (b, pl(P1))], &[(blocker, o_)]);
    assert!(!t.on_battlefield(o_));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn seafloor_oracle_draws_while_dying() {
    cr!("510.2", "704.3");
    ruling!(
        "Seafloor Oracle",
        "If Seafloor Oracle is dealt lethal damage at the same time a Merfolk you control deals combat damage to a player, you'll draw a card."
    );
    let mut t = TestGame::new(2);
    let s = old(&mut t, P0, "Seafloor Oracle");
    let m = old(&mut t, P0, "Merfolk Looter");
    let g = old(&mut t, P1, "Hill Giant");
    let hand = t.hand_size(P0);
    combat(&mut t, &[(s, pl(P1)), (m, pl(P1))], &[(g, s)]);
    assert!(!t.on_battlefield(s));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn tandem_lookout_draws_though_the_pair_broke() {
    cr!("702.95a", "113.7a");
    ruling!(
        "Tandem Lookout",
        "If Tandem Lookout or the creature it’s paired with is dealt lethal damage at the same time that either deals damage to an opponent, its ability triggers. You’ll draw a card even though that creature no longer has the ability."
    );
    supported("Tandem Lookout");
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Grizzly Bears");
    yes(&mut t, P0, true);
    t.answer_choose(P0, &[o(b)]);
    let l = enter(&mut t, P0, "Tandem Lookout");
    t.resolve_all();
    assert_eq!(t.obj_now(l).paired_with, Some(b));
    t.g.objects[l.0 as usize].summoning_sick = false;
    let g = old(&mut t, P1, "Hill Giant");
    let hand = t.hand_size(P0);
    // The Bears hit P1 while the Lookout dies blocked by the Giant.
    combat(&mut t, &[(l, pl(P1)), (b, pl(P1))], &[(g, l)]);
    assert!(!t.on_battlefield(l));
    assert_eq!(t.obj_now(b).paired_with, None);
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------
// Targets and costs on resolution
// ---------------------------------------------------------------------------

#[test]
fn sword_of_fire_and_ice_with_an_illegal_target_doesnt_draw() {
    cr!("608.2b");
    ruling!(
        "Sword of Fire and Ice",
        "If the chosen target is an illegal target by the time the triggered ability tries to resolve, the ability won't resolve. You won't draw a card."
    );
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Grizzly Bears");
    let s = t.battlefield(P0, "Sword of Fire and Ice");
    t.g.attach(s, o(b));
    let elf = old(&mut t, P1, "Llanowar Elves");
    t.answer_targets(P0, &[o(elf)]);
    attack_with(&mut t, &[(b, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    let hand = t.hand_size(P0);
    destroy(&mut t, elf);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn killians_confidence_with_an_illegal_target_doesnt_draw() {
    cr!("608.2b");
    ruling!(
        "Killian's Confidence",
        "If the target creature is an illegal target as Killian's Confidence tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_named(&mut t, P0, "Killian's Confidence", &[o(b)]);
    destroy(&mut t, b);
    t.resolve_all();
    // (Killian's Confidence came from hand and went to the graveyard: no card drawn.)
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Killian's Confidence"));
}

#[test]
fn subira_checks_power_only_on_resolution() {
    cr!("608.2b", "611.2c");
    ruling!(
        "Subira, Tulzidi Caravanner",
        "If the target creature's power is greater than 2 as the first activated ability tries to resolve, the ability doesn't resolve. However, if instead the creature's power is raised above 2 after the ability resolves, it still can't be blocked that turn."
    );
    supported("Subira, Tulzidi Caravanner");
    for pump_first in [true, false] {
        let mut t = TestGame::new(2);
        let s = old(&mut t, P0, "Subira, Tulzidi Caravanner");
        let b = old(&mut t, P0, "Grizzly Bears");
        let g = old(&mut t, P1, "Hill Giant");
        mana(&mut t, P0, ManaType::C, 1);
        activate_text(&mut t, P0, s, "can't be blocked").unwrap();
        if pump_first {
            // In response: Giant Growth.
            cast_named(&mut t, P0, "Giant Growth", &[o(b)]);
            t.resolve();
        }
        t.resolve_all();
        if !pump_first {
            cast_named(&mut t, P0, "Giant Growth", &[o(b)]);
            t.resolve_all();
        }
        assert_eq!(t.pt(b), (5, 5));
        combat(&mut t, &[(b, pl(P1))], &[(g, b)]);
        // Blocked (and the Giant dies) only when the ability didn't resolve.
        assert_eq!(t.life(P1), if pump_first { 20 } else { 15 });
        assert_eq!(t.on_battlefield(g), !pump_first);
    }
}

#[test]
fn familys_favor_cant_remove_a_shield_counter_already_gone() {
    cr!("122.1c", "510.2");
    ruling!(
        "Family's Favor",
        "If the attacking creature loses its last shield counter at the same time it deals combat damage to a player (for example, if the attacking creature has trample and would be dealt damage by a blocker), then you will not be able to pay the cost of removing a counter when the triggered ability resolves."
    );
    supported("Family's Favor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Family's Favor");
    let d = old(&mut t, P0, "Colossal Dreadmaw");
    let elf = old(&mut t, P1, "Llanowar Elves");
    t.answer_targets(P0, &[o(d)]);
    let hand = t.hand_size(P0);
    combat(&mut t, &[(d, pl(P1))], &[(elf, d)]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.counters(d, counters::SHIELD), 0);
    assert_eq!(t.obj_now(d).damage, 0);
    assert_eq!(t.hand_size(P0), hand);
    // Unblocked, the counter is still there: it's removed and P0 draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Family's Favor");
    let d = old(&mut t, P0, "Colossal Dreadmaw");
    t.answer_targets(P0, &[o(d)]);
    let hand = t.hand_size(P0);
    combat(&mut t, &[(d, pl(P1))], &[]);
    assert_eq!(t.counters(d, counters::SHIELD), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn urgoros_one_card_is_discarded_at_random_and_no_draw() {
    cr!("701.9b");
    ruling!(
        "Urgoros, the Empty One",
        "If that player has one card in hand, it's discarded at random (even though that's not very random). You won't draw a card."
    );
    supported("Urgoros, the Empty One");
    for cards in [1usize, 0] {
        let mut t = TestGame::new(2);
        let u = old(&mut t, P0, "Urgoros, the Empty One");
        if cards == 1 {
            t.hand(P1, "Shock");
        }
        let hand = t.hand_size(P0);
        combat(&mut t, &[(u, pl(P1))], &[]);
        assert_eq!(t.hand_size(P1), 0);
        assert_eq!(t.in_graveyard(P1, "Shock"), cards == 1);
        assert_eq!(t.hand_size(P0), hand + (cards == 0) as usize);
    }
}

#[test]
fn jackdaw_may_discard_an_empty_hand() {
    cr!("701.9a");
    ruling!(
        "Jackdaw",
        "If you have no cards in hand as Jackdaw’s first ability resolves, you may still choose to discard your hand."
    );
    supported("Jackdaw");
    let mut t = TestGame::new(2);
    let j = old(&mut t, P0, "Jackdaw");
    t.battlefield(P0, "Ornithopter");
    let g = old(&mut t, P0, "Hill Giant");
    assert_eq!(t.hand_size(P0), 0);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crew(&mut t, P0, j, &[g]));
    t.resolve_all();
    yes(&mut t, P0, true);
    combat(&mut t, &[(j, pl(P1))], &[]);
    // Jackdaw and Ornithopter.
    assert_eq!(t.hand_size(P0), 2);
}

// ---------------------------------------------------------------------------
// Blocks and attack requirements
// ---------------------------------------------------------------------------

#[test]
fn deepfathom_skulker_after_blocks_doesnt_undo_the_block() {
    cr!("509.1h");
    ruling!(
        "Deepfathom Skulker",
        "Activating Deepfathom Skulker’s last ability after it’s been legally blocked won’t change or undo that block."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Deepfathom Skulker");
    let b = old(&mut t, P0, "Grizzly Bears");
    let g = old(&mut t, P1, "Hill Giant");
    attack_with(&mut t, &[(b, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(g, b)]));
    t.advance_to(P0, Step::DeclareBlockers);
    mana(&mut t, P0, ManaType::C, 4);
    t.answer_targets(P0, &[o(b)]);
    activate_text(&mut t, P0, s, "can't be blocked").unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(b));
}

#[test]
fn toski_attacks_if_able_but_not_when_tapped_or_for_a_cost() {
    cr!("508.1d");
    ruling!(
        "Toski, Bearer of Secrets",
        "If Toski can't attack for some reason (such as being tapped or having come under that player's control that turn), then it doesn't attack. If there's a cost associated with having it attack, you aren't forced to pay that cost"
    );
    // Required: the empty declaration isn't accepted.
    let mut t = TestGame::new(2);
    let toski = old(&mut t, P0, "Toski, Bearer of Secrets");
    attack_with(&mut t, &[]);
    assert!(t.g.is_attacking(toski));
    // Tapped: it doesn't attack.
    let mut t = TestGame::new(2);
    let toski = old(&mut t, P0, "Toski, Bearer of Secrets");
    t.g.objects[toski.0 as usize].tapped = true;
    attack_with(&mut t, &[]);
    assert!(!t.g.is_attacking(toski));
    // Ghostly Prison: a cost to attack; it doesn't have to.
    let mut t = TestGame::new(2);
    let toski = old(&mut t, P0, "Toski, Bearer of Secrets");
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Plains", 2);
    attack_with(&mut t, &[]);
    assert!(!t.g.is_attacking(toski));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
}

#[test]
fn bident_of_thassa_creatures_attack_if_able_but_not_for_a_cost() {
    cr!("508.1d");
    ruling!(
        "Bident of Thassa",
        "If a creature can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack. If there's a cost associated with having a creature attack, the player isn't forced to pay that cost"
    );
    supported("Bident of Thassa");
    for prison in [false, true] {
        let mut t = TestGame::new(2);
        let bident = old(&mut t, P0, "Bident of Thassa");
        let a = old(&mut t, P1, "Grizzly Bears");
        let tapped = old(&mut t, P1, "Hill Giant");
        t.g.objects[tapped.0 as usize].tapped = true;
        if prison {
            t.battlefield(P0, "Ghostly Prison");
            t.lands(P1, "Plains", 2);
        }
        t.set_step(P1, Step::Upkeep);
        mana_all(&mut t, P0, 1);
        activate_text(&mut t, P0, bident, "attack this turn if able").unwrap();
        t.resolve_all();
        t.set_step(P1, Step::BeginningOfCombat);
        attack_with(&mut t, &[]);
        assert_eq!(t.g.is_attacking(a), !prison, "prison={prison}");
        assert!(!t.g.is_attacking(tapped));
    }
}

#[test]
fn illvoi_infiltrator_counts_itself_and_countered_spells() {
    cr!("601.2i");
    ruling!(
        "Illvoi Infiltrator",
        "Illvoi Infiltrator’s first ability will count any spells you’ve cast this turn, which may include Illvoi Infiltrator itself. It doesn’t matter if the other spells resolved, didn’t resolve, were countered, or are still on the stack."
    );
    supported("Illvoi Infiltrator");
    for second in [false, true] {
        let mut t = TestGame::new(2);
        let i = cast_named(&mut t, P0, "Illvoi Infiltrator", &[]);
        t.resolve_all();
        let g = old(&mut t, P1, "Hill Giant");
        let i = t.g.current(i);
        t.g.objects[i.0 as usize].summoning_sick = false;
        if second {
            // A second spell, countered.
            let s = cast_named(&mut t, P0, "Shock", &[pl(P1)]);
            cast_named(&mut t, P1, "Counterspell", &[o(s)]);
            t.resolve_all();
            assert!(t.in_graveyard(P0, "Shock"));
        }
        combat(&mut t, &[(i, pl(P1))], &[(g, i)]);
        // Unblockable with two spells cast (itself and the countered one).
        assert_eq!(t.life(P1), if second { 19 } else { 20 }, "second={second}");
        assert_eq!(t.on_battlefield(i), second);
    }
}

// ---------------------------------------------------------------------------
// Granted triggered abilities
// ---------------------------------------------------------------------------

#[test]
fn nimble_trapfinder_grant_lasts_after_the_party_breaks() {
    cr!("611.2a", "611.2c");
    ruling!(
        "Nimble Trapfinder",
        "After Nimble Trapfinder’s second ability has resolved, creatures you control keep the triggered ability for the turn even if you no longer have a full party later in the turn."
    );
    let mut t = TestGame::new(2);
    old(&mut t, P0, "Nimble Trapfinder");
    old(&mut t, P0, "Kor Celebrant");
    let wiz = old(&mut t, P0, "Prodigal Sorcerer");
    let w = old(&mut t, P0, "Elvish Warrior");
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    destroy(&mut t, wiz);
    let hand = t.hand_size(P0);
    combat(&mut t, &[(w, pl(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn driven_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Driven // Despair",
        "Driven and Despair each affect only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't gain the keyword ability or the triggered ability."
    );
    supported("Driven // Despair");
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Grizzly Bears");
    let c = t.hand(P0, "Driven // Despair");
    mana_all(&mut t, P0, 2);
    let m = cast_methods(&mut t, P0, c);
    assert_eq!(m.len(), 1);
    t.cast(P0, c).method(m[0].clone()).go();
    t.resolve_all();
    let later = old(&mut t, P0, "Hill Giant");
    assert!(t.obj_now(b).has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    assert!(!t.obj_now(later).has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    let hand = t.hand_size(P0);
    combat(&mut t, &[(b, pl(P1)), (later, pl(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn despair_twice_gives_two_instances() {
    cr!("113.2c", "702.127a");
    ruling!(
        "Driven // Despair",
        "If you resolve multiples of Driven or of Despair in one turn, your creatures have that many instances of the appropriate triggered ability. Each instance triggers separately."
    );
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Grizzly Bears");
    for _ in 0..2 {
        let c = t.graveyard(P0, "Driven // Despair");
        mana_all(&mut t, P0, 2);
        let m = cast_methods(&mut t, P0, c);
        assert_eq!(m.len(), 1);
        t.cast(P0, c).method(m[0].clone()).go();
        t.resolve_all();
    }
    for _ in 0..3 {
        t.hand(P1, "Shock");
    }
    combat(&mut t, &[(b, pl(P1))], &[]);
    assert_eq!(t.hand_size(P1), 1);
}

// ---------------------------------------------------------------------------
// Drogskol Reaver: one trigger per life-gaining event
// ---------------------------------------------------------------------------

#[test]
fn drogskol_reaver_triggers_once_per_life_gain_event() {
    cr!("603.2c");
    ruling!(
        "Drogskol Reaver",
        "Drogskol Reaver's last ability triggers just once for each life-gaining event"
    );
    ruling!(
        "Drogskol Reaver",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and the last ability of Drogskol Reaver triggers only once."
    );
    // Healing Salve: 3 life at once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drogskol Reaver");
    let hand = t.hand_size(P0);
    cast_named(&mut t, P0, "Healing Salve", &[pl(P0)]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.hand_size(P0)), (23, hand + 1));
    // Blunt the Assault: 1 life for each of four creatures.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drogskol Reaver");
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    let hand = t.hand_size(P0);
    cast_named(&mut t, P0, "Blunt the Assault", &[]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.hand_size(P0)), (24, hand + 1));
}

#[test]
fn drogskol_reaver_killing_its_blocker_with_first_strike_triggers_once() {
    cr!("702.4b", "702.19b");
    ruling!(
        "Drogskol Reaver",
        "If Drogskol Reaver deals enough first-strike damage to destroy each creature it's blocking or was blocked by, it won't deal any damage during the regular combat damage step and its last ability won't trigger a second time that combat. This is different if Drogskol Reaver is attacking and somehow gains trample."
    );
    for trample in [false, true] {
        let mut t = TestGame::new(2);
        let r = old(&mut t, P0, "Drogskol Reaver");
        let g = old(&mut t, P1, "Wind Drake");
        if trample {
            // Rancor: +2/+0 and trample.
            cast_named(&mut t, P0, "Rancor", &[o(r)]);
            t.resolve_all();
        }
        let hand = t.hand_size(P0);
        combat(&mut t, &[(r, pl(P1))], &[(g, r)]);
        assert!(!t.on_battlefield(g));
        if trample {
            // 2 to the Drake and 3 to P1, then 5 to P1.
            assert_eq!(t.life(P1), 12);
            assert_eq!((t.life(P0), t.hand_size(P0)), (30, hand + 2));
        } else {
            assert_eq!(t.life(P1), 20);
            assert_eq!((t.life(P0), t.hand_size(P0)), (23, hand + 1));
        }
    }
}

#[test]
fn tovolar_triggers_once_for_a_wolf_werewolf() {
    cr!("603.2c", "702.73a");
    ruling!(
        "Tovolar, Dire Overlord // Tovolar, the Midnight Scourge",
        "If a creature you control that's a Wolf and a Werewolf deals combat damage to a player, the first ability triggers only once."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tovolar, Dire Overlord // Tovolar, the Midnight Scourge");
    // Changeling Outcast is every creature type.
    let c = old(&mut t, P0, "Changeling Outcast");
    let hand = t.hand_size(P0);
    combat(&mut t, &[(c, pl(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn piper_wright_sees_a_nontoken_clue_sacrificed() {
    cr!("205.3g", "701.21a");
    ruling!(
        "Piper Wright, Publick Reporter",
        "If an effect refers to a Clue, it means any Clue artifact, not just a Clue artifact token."
    );
    supported("Piper Wright, Publick Reporter");
    let mut t = TestGame::new(2);
    let p = old(&mut t, P0, "Piper Wright, Publick Reporter");
    let c = old(&mut t, P0, "Candy Trail");
    mana(&mut t, P0, ManaType::C, 2);
    t.answer_targets(P0, &[o(p)]);
    activate_text(&mut t, P0, c, "Sacrifice").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Candy Trail"));
    assert_eq!(t.counters(p, counters::PLUS1), 1);
}

// ---------------------------------------------------------------------------
// Toski and counterspells
// ---------------------------------------------------------------------------

#[test]
fn toski_cant_be_countered_but_the_counterspell_still_draws() {
    cr!("101.2", "608.2c");
    ruling!(
        "Toski, Bearer of Secrets",
        "A spell or ability that counters spells can still target Toski. When that spell or ability resolves, Toski won't be countered, but any additional effects of that spell or ability will still happen."
    );
    let mut t = TestGame::new(2);
    let toski = cast_named(&mut t, P0, "Toski, Bearer of Secrets", &[]);
    let hand = t.hand_size(P1);
    // Exclude: "Counter target creature spell. Draw a card."
    cast_named(&mut t, P1, "Exclude", &[o(toski)]);
    t.resolve_all();
    assert!(t.on_battlefield(toski));
    assert_eq!(t.hand_size(P1), hand + 1);
}

// ---------------------------------------------------------------------------
// Enduring Curiosity
// ---------------------------------------------------------------------------

#[test]
fn enduring_curiosity_returns_as_a_noncreature_enchantment() {
    cr!("205.3d", "400.7");
    ruling!(
        "Enduring Curiosity",
        "Cat and Glimmer are both creature types. Enduring Curiosity won't have those creature types when its last ability returns it to the battlefield because it won't be a creature."
    );
    let mut t = TestGame::new(2);
    let e = old(&mut t, P0, "Enduring Curiosity");
    destroy(&mut t, e);
    t.resolve_all();
    assert!(t.on_battlefield(e));
    let now = t.obj_now(e);
    assert!(now.is(CardType::Enchantment) && !now.is(CardType::Creature));
    assert!(!now.chars.has_subtype("Cat") && !now.chars.has_subtype("Glimmer"));
}

#[test]
fn enduring_curiosity_token_copy_doesnt_return() {
    cr!("111.8");
    ruling!(
        "Enduring Curiosity",
        "A token that's a copy of Enduring Curiosity won't return to the battlefield when its last ability resolves."
    );
    let mut t = TestGame::new(2);
    let e = old(&mut t, P0, "Enduring Curiosity");
    cast_named(&mut t, P0, "Cackling Counterpart", &[o(e)]);
    t.resolve_all();
    let tok = named(&t, P0, "Enduring Curiosity")
        .into_iter()
        .find(|x| t.obj_now(*x).is_token())
        .unwrap();
    destroy(&mut t, tok);
    t.resolve_all();
    assert_eq!(named(&t, P0, "Enduring Curiosity"), vec![e]);
}

#[test]
fn enduring_curiosity_nontoken_copy_returns_as_only_an_enchantment() {
    cr!("707.2", "205.1a");
    ruling!(
        "Enduring Curiosity",
        "If a nontoken permanent that's a copy of Enduring Curiosity dies while it's a creature, it will return to the battlefield as an enchantment when its last ability resolves. It won't have any card types other than enchantment."
    );
    let mut t = TestGame::new(2);
    let e = old(&mut t, P0, "Enduring Curiosity");
    let c = crate::r_p017_common::enter_copying(&mut t, P0, "Clone", Some(e));
    assert!(creature_now(&t, c));
    destroy(&mut t, c);
    // As the Clone returns, it may copy Enduring Curiosity again.
    t.answer_choose(P0, &[o(e)]);
    t.resolve_all();
    assert!(t.on_battlefield(c));
    let now = t.obj_now(c);
    assert_eq!(now.chars.name, "Enduring Curiosity");
    assert!(now.is(CardType::Enchantment) && !now.is(CardType::Creature));
}

// ---------------------------------------------------------------------------
// Finality counters
// ---------------------------------------------------------------------------

#[test]
fn finality_counter_works_on_a_permanent_without_abilities() {
    cr!("122.1h", "614.1a");
    ruling!(
        "Charnel Serenade",
        "Finality counters aren’t keyword counters, and a finality counter doesn’t give any abilities to the permanent it’s on. If that permanent loses its abilities and then would go to a graveyard, it will still be exiled instead."
    );
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Grizzly Bears");
    t.g.add_counters(o(b), counters::FINALITY, 1, None);
    // Turn to Frog: it loses all abilities.
    cast_named(&mut t, P0, "Turn to Frog", &[o(b)]);
    t.resolve_all();
    assert!(t.obj_now(b).chars.abilities.is_empty());
    destroy(&mut t, b);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn finality_counter_doesnt_stop_other_zone_changes() {
    cr!("122.1h");
    ruling!(
        "Charnel Serenade",
        "Finality counters don’t stop permanents from going to zones other than the graveyard from the battlefield."
    );
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Grizzly Bears");
    t.g.add_counters(o(b), counters::FINALITY, 1, None);
    cast_named(&mut t, P0, "Unsummon", &[o(b)]);
    t.resolve_all();
    assert_eq!(t.zone(b), Zone::Hand(P0));
}

// ---------------------------------------------------------------------------
// Green Goblin
// ---------------------------------------------------------------------------

#[test]
fn green_goblin_reduction_doesnt_itself_allow_graveyard_casting() {
    cr!("601.2f", "601.3");
    ruling!(
        "Norman Osborn // Green Goblin",
        "Green Goblin's cost-reduction ability doesn't itself allow you to cast any spells from your graveyard."
    );
    let mut t = TestGame::new(2);
    crate::r_s17_common::enter_transformed(&mut t, P0, "Norman Osborn // Green Goblin");
    // Not discarded this turn: no mayhem, so it can't be cast.
    let g = t.graveyard(P0, "Grizzly Bears");
    mana_all(&mut t, P0, 3);
    assert!(!can_cast(&mut t, P0, g, CastMethod::Normal));
    let any = t.g.legal_actions(P0).iter().any(|a| {
        matches!(a, mtg_engine::decision::Action::Cast { card, .. } if *card == g)
    });
    assert!(!any);
}

#[test]
fn green_goblin_reduces_only_generic_mana() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Norman Osborn // Green Goblin",
        "Green Goblin's cost-reduction ability reduces only the generic mana component of a spell's cost. For example, if you cast a spell from your graveyard that would normally cost {1}{B}, you will have to pay {B} to cast it."
    );
    let mut t = TestGame::new(2);
    crate::r_s17_common::enter_transformed(&mut t, P0, "Norman Osborn // Green Goblin");
    // Discarded this turn: it has mayhem {1}{G} (its mana cost).
    let c = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, c, None);
    t.g.flush_events();
    let c = t.g.current(c);
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
    // Only {G} available: enough.
    mana(&mut t, P0, ManaType::G, 1);
    let actions = t.g.legal_actions(P0);
    let method = actions
        .iter()
        .find_map(|a| match a {
            mtg_engine::decision::Action::Cast { card, method } if *card == c => {
                Some(method.clone())
            }
            _ => None,
        })
        .expect("castable with mayhem");
    t.cast(P0, c).method(method).go();
    t.resolve_all();
    assert!(t.on_battlefield(c));
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    // A spell with no generic mana ({G}{G} ... here: Llanowar Elves {G}) still costs {G}.
    let e = t.hand(P0, "Llanowar Elves");
    t.g.discard(P0, e, None);
    t.g.flush_events();
    let e = t.g.current(e);
    let methods: Vec<_> = t
        .g
        .legal_actions(P0)
        .into_iter()
        .filter(|a| matches!(a, mtg_engine::decision::Action::Cast { card, .. } if *card == e))
        .collect();
    assert!(methods.is_empty(), "{methods:?}");
}

#[test]
fn the_ravens_warning_puts_a_sideboard_card_on_top() {
    cr!("400.11a", "714.2b");
    ruling!(
        "The Raven's Warning",
        "a card from outside the game comes from your sideboard"
    );
    let mut t = TestGame::new(2);
    let side = t.custom(P0, (*mtg_engine::card::card("Serra Angel")).clone(), Zone::Outside(P0));
    let saga = old(&mut t, P0, "The Raven's Warning");
    t.g.add_counters(o(saga), counters::LORE, 2, None);
    yes(&mut t, P0, true);
    t.answer_choose(P0, &[o(side)]);
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let top = t.g.library_top(P0).unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Serra Angel");
}

#[test]
fn invasion_of_kamigawa_can_target_a_tapped_creature() {
    cr!("115.1");
    ruling!(
        "Invasion of Kamigawa // Rooftop Saboteurs",
        "Invasion of Kamigawa's enters-the-battlefield ability can target an artifact or creature that's already tapped."
    );
    supported("Invasion of Kamigawa // Rooftop Saboteurs");
    let mut t = TestGame::new(2);
    let g = old(&mut t, P1, "Hill Giant");
    t.g.objects[g.0 as usize].tapped = true;
    let id = new_card(&mut t, P0, "Invasion of Kamigawa // Rooftop Saboteurs");
    let inv = enter_quiet(&mut t, P0, id);
    let _ = inv;
    t.answer_targets(P0, &[o(g)]);
    let from = mark(&t);
    t.settle();
    let c = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(c.iter().any(|c| c.contains(&o(g))), "{c:?}");
    t.resolve_all();
    assert_eq!(t.counters(g, counters::STUN), 1);
}

#[test]
fn firkraag_ignores_creatures_whose_controller_had_to_attack() {
    cr!("506.7", "508.1d", "701.15a");
    ruling!(
        "Firkraag, Cunning Instigator",
        "If an ability required that creature's controller to attack rather than the creature itself (such as that of Trove of Temptation), that creature dealing combat damage won't cause Firkraag's last ability to trigger"
    );
    ruling!(
        "Firkraag, Cunning Instigator",
        "An ability that looks for whether a creature \"had to attack\" checks to see if there were any requirements to attack that were applying to that particular creature at the time it was declared as an attacker."
    );
    supported("Firkraag, Cunning Instigator");
    for goaded in [false, true] {
        let mut t = TestGame::new(3);
        let f = t.battlefield(P0, "Firkraag, Cunning Instigator");
        let b = old(&mut t, P2, "Grizzly Bears");
        if goaded {
            // Disrupt Decorum: "Goad all creatures you don't control."
            cast_named(&mut t, P0, "Disrupt Decorum", &[]);
            t.resolve_all();
        } else {
            // Trove of Temptation (P1's): "Each opponent must attack you or a
            // planeswalker you control with at least one creature each combat if able."
            t.battlefield(P1, "Trove of Temptation");
        }
        t.set_step(P2, Step::BeginningOfCombat);
        let hand = t.hand_size(P0);
        // The Bears attack P1 (one of P0's opponents) in either case.
        attack_with(&mut t, &[(b, pl(P1))]);
        assert!(t.g.is_attacking(b));
        block_and_finish(&mut t, P1, &[]);
        t.resolve_all();
        assert_eq!(t.life(P1), 18);
        assert_eq!(t.hand_size(P0), hand + goaded as usize, "goaded={goaded}");
        assert_eq!(t.counters(f, counters::PLUS1), goaded as u32);
    }
}
