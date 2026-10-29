//! Rulings batch P183 — damage and power: Aegar's excess damage (CR 120.10),
//! Valley Flamecaller's damage replacement (CR 614.1a, 616.1, 510.1c), Jason Bright's
//! base power (CR 208.4b), Rakdos's coins (CR 608.2c), and Righteous Valkyrie's life
//! gain read on resolution or from last known information (CR 608.2h).

use crate::r_p009_common::{kill, obj, pl};
use crate::r_p185_common::pump;
use crate::r_p189_common::bounce;
use crate::r_p194_common::{damage, damage_events, shock};
use crate::r_s01_common::{attack_with, supported};
use crate::r_s25_common::cast_new;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn aegar_excess_damage_takes_marked_damage_into_account() {
    cr!("120.10");
    ruling!(
        "Aegar, the Freezing Flame",
        "A creature has been dealt excess damage if one or more sources deal more damage to it than the minimum amount of damage required to be lethal damage. In most cases, this means damage greater than its toughness, but consider the damage already dealt to it that turn."
    );
    supported("Aegar, the Freezing Flame");
    for (earlier, draws) in [(0, 0), (2, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Aegar, the Freezing Flame");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        if earlier > 0 {
            damage(&mut t, bears, obj(giant), earlier);
        }
        let hand = t.hand_size(P0);
        shock(&mut t, P0, obj(giant));
        assert_eq!(t.hand_size(P0), hand + draws, "{earlier} damage earlier");
    }
}

#[test]
fn aegar_uses_the_smallest_threshold_of_a_creature_planeswalker() {
    cr!("120.10");
    ruling!(
        "Aegar, the Freezing Flame",
        "If a permanent is both a creature and a planeswalker, the minimum amount of damage to be considered lethal damage is used to determine if excess damage has been dealt."
    );
    // P1's Jace Beleren (3 loyalty) made a 5/5 creature, dealt 4 damage by Aegar.
    for (creature_only, draws) in [(false, 1), (true, 0)] {
        let mut t = TestGame::new(2);
        let aegar = t.battlefield(P0, "Aegar, the Freezing Flame");
        let target = if creature_only {
            t.battlefield(P1, "Grizzly Bears")
        } else {
            t.battlefield(P1, "Jace Beleren")
        };
        modify_until_eot(
            &mut t,
            target,
            vec![
                Modification::AddTypes(vec![CardType::Creature]),
                Modification::SetPT(Some(Value::c(5)), Some(Value::c(5))),
            ],
        );
        assert_eq!(t.pt(target), (5, 5));
        let hand = t.hand_size(P0);
        damage(&mut t, aegar, obj(target), 4);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws, "creature only: {creature_only}");
    }
}

#[test]
fn aegar_doesnt_care_who_dealt_the_excess_damage() {
    cr!("120.10", "603.4");
    ruling!(
        "Aegar, the Freezing Flame",
        "It doesn't matter whether a Giant, Wizard, or spell you control deals the excess damage, only that excess damage was dealt and that one of those three things dealt damage to the creature or planeswalker at some point during the turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aegar, the Freezing Flame");
    let angel = t.battlefield(P1, "Serra Angel");
    let hand = t.hand_size(P0);
    shock(&mut t, P0, obj(angel));
    assert_eq!(t.hand_size(P0), hand);
    // P1's own Lightning Bolt deals the excess.
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(angel)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Serra Angel"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn aegar_doesnt_trigger_for_a_battle() {
    cr!("120.10");
    ruling!(
        "Aegar, the Freezing Flame",
        "Aegar's ability won't trigger when a battle is dealt excess damage."
    );
    let mut t = TestGame::new(2);
    let aegar = t.battlefield(P0, "Aegar, the Freezing Flame");
    // P1's battle, protected by P0.
    t.answer_choose(P1, &[pl(P0)]);
    let battle = t.enter(P1, "Invasion of Segovia");
    t.resolve_all();
    assert_eq!(t.obj_now(battle).controller, P1);
    let hand = t.hand_size(P0);
    damage(&mut t, aegar, obj(battle), 10);
    t.resolve_all();
    // 10 damage is more than its defense.
    assert!(damage_events(&t).contains(&(aegar, obj(battle), 10)));
    assert_eq!(t.hand_size(P0), hand);
}

/// The options of the replacement-order choices asked of `p` since `from`.
fn order_choices(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseReplacement { options } if *q == p => Some(options.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn valley_flamecaller_order_is_chosen_by_the_affected_player_and_prevention_can_preempt_it() {
    cr!("616.1", "615.1a");
    ruling!(
        "Valley Flamecaller",
        "If another effect modifies how much damage your Lizard, Mouse, Otter, or Raccoon would deal, including preventing some of it, the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Valley Flamecaller’s effect no longer applies."
    );
    supported("Valley Flamecaller");
    supported("Furnace of Rath");
    // P0's Lagac Lizard deals 2 to P1 or P1's Craw Wurm with P0's Furnace of Rath: P1
    // orders the two effects: +1 then double (6) or double then +1 (5).
    for to_creature in [false, true] {
        let mut results = vec![];
        for first in [0, 1] {
            let mut t = TestGame::new(2);
            t.battlefield(P0, "Valley Flamecaller");
            t.battlefield(P0, "Furnace of Rath");
            let lizard = t.battlefield(P0, "Lagac Lizard");
            let wurm = t.battlefield(P1, "Craw Wurm");
            let from = t.asked().len();
            t.answer(P1, DecisionKind::Replacement, Answer::Index(first));
            let to = if to_creature { obj(wurm) } else { pl(P1) };
            damage(&mut t, lizard, to, 2);
            assert_eq!(order_choices(&t, P1, from).len(), 1);
            assert!(order_choices(&t, P0, from).is_empty());
            let opts = &order_choices(&t, P1, from)[0];
            let doubled_first = opts[first].contains("Furnace");
            let dealt: u32 = damage_events(&t)
                .iter()
                .filter(|(s, r, _)| *s == lizard && *r == to)
                .map(|(_, _, n)| n)
                .sum();
            assert_eq!(dealt, if doubled_first { 5 } else { 6 }, "{opts:?}");
            results.push(dealt);
        }
        results.sort();
        assert_eq!(results, vec![5, 6]);
    }
    // All of it prevented (P1's Safe Passage): nothing is dealt, whichever order.
    for first in [0, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Valley Flamecaller");
        let lizard = t.battlefield(P0, "Lagac Lizard");
        cast_new(&mut t, P1, "Safe Passage", &[]);
        t.resolve_all();
        t.answer(P1, DecisionKind::Replacement, Answer::Index(first));
        damage(&mut t, lizard, pl(P1), 2);
        assert_eq!(t.life(P1), 20);
    }
}

#[test]
fn valley_flamecaller_adds_1_to_each_share_of_divided_damage_and_doesnt_deal_damage_itself() {
    cr!("510.1c", "510.2", "614.1a");
    ruling!(
        "Valley Flamecaller",
        "If damage dealt by a source you control is being divided or assigned among multiple permanents an opponent controls or among an opponent and one or more permanents they control simultaneously, divide the original amount before adding 1."
    );
    ruling!(
        "Valley Flamecaller",
        "Valley Flamecaller’s ability doesn’t cause it to deal damage; it affects the amount of damage dealt by the original source."
    );
    let mut t = TestGame::new(2);
    let flamecaller = t.battlefield(P0, "Valley Flamecaller");
    let lizard = t.battlefield(P0, "Lagac Lizard");
    let wall = t.battlefield(P1, "Wall of Wood");
    let wurm = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(lizard, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, lizard), (wurm, lizard)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    // Lagac Lizard (3 power) assigns 2 and 1 among its blockers.
    t.answer(
        P0,
        DecisionKind::Damage,
        Answer::Numbers(vec![2, 1]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    let recipients: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::AssignCombatDamage { recipients, .. } => Some(recipients),
            _ => None,
        })
        .collect();
    assert_eq!(recipients, vec![vec![obj(wall), obj(wurm)]]);
    let dealt: Vec<(ObjectId, Entity, u32)> = damage_events(&t)
        .into_iter()
        .filter(|(s, _, _)| *s == lizard)
        .collect();
    assert!(dealt.contains(&(lizard, obj(wall), 3)), "{dealt:?}");
    assert!(dealt.contains(&(lizard, obj(wurm), 2)), "{dealt:?}");
    // Valley Flamecaller dealt nothing.
    assert!(damage_events(&t).iter().all(|(s, _, _)| *s != flamecaller));
}

#[test]
fn jason_bright_compares_power_with_base_power() {
    cr!("208.4b", "613.4b");
    ruling!(
        "Jason Bright, Glowing Prophet",
        "Normally, a creature’s base power is the power printed on the card, or, for a token, the power set by the effect that created it. If another effect sets a creature’s power to a specific number or value, that becomes its base power. If an effect modifies a creature’s power without setting it, that is not included when determining its base power."
    );
    ruling!(
        "Jason Bright, Glowing Prophet",
        "Some creatures have base power 0 and an ability that gives them a bonus based on some criteria. Those are not characteristic-defining abilities, and that ability doesn’t change its base power. Such a creature will have power different from its base power if that ability causes its power to be anything other than 0."
    );
    supported("Jason Bright, Glowing Prophet");
    supported("Nim Shrieker");
    supported("Square Up");
    // (the dying Zombie, what happens to it first, whether Jason Bright draws)
    #[derive(Clone, Copy, Debug)]
    enum Setup {
        Nothing,
        Pumped,
        SetBase,
        SetBaseAndPumped,
        TwoArtifacts,
    }
    let cases = [
        ("Walking Corpse", Setup::Nothing, false),
        ("Walking Corpse", Setup::Pumped, true),
        ("Walking Corpse", Setup::SetBase, false),
        ("Walking Corpse", Setup::SetBaseAndPumped, true),
        ("Nim Shrieker", Setup::Nothing, false),
        ("Nim Shrieker", Setup::TwoArtifacts, true),
    ];
    for (zombie, setup, draws) in cases {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Jason Bright, Glowing Prophet");
        let z = t.battlefield(P0, zombie);
        match setup {
            Setup::Nothing => {}
            Setup::Pumped => pump(&mut t, z, 1, 0),
            Setup::SetBase | Setup::SetBaseAndPumped => {
                cast_new(&mut t, P0, "Square Up", &[obj(z)]);
                t.resolve_all();
                assert_eq!(t.pt(z), (4, 4));
                if matches!(setup, Setup::SetBaseAndPumped) {
                    pump(&mut t, z, 1, 0);
                }
            }
            Setup::TwoArtifacts => {
                t.battlefield(P0, "Mind Stone");
                t.battlefield(P0, "Mind Stone");
                assert_eq!(t.pt(z).0, 2);
            }
        }
        let hand = t.hand_size(P0);
        kill(&mut t, z);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws as usize, "{zombie} {setup:?}");
    }
}

#[test]
fn rakdos_flips_and_destroys_without_any_player_acting_in_between() {
    cr!("608.2c");
    ruling!(
        "Rakdos, the Showstopper",
        "Players can't take any action between the time that coins are flipped and the time creatures are destroyed."
    );
    supported("Rakdos, the Showstopper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Serra Angel");
    t.battlefield(P1, "Demon Wall");
    let rakdos = t.enter(P0, "Rakdos, the Showstopper");
    t.g.flush_events();
    t.settle();
    let events = t.g.turn_events.len();
    let asked = t.asked().len();
    t.g.resolve_top();
    let during: Vec<Event> = t.g.turn_events[events..].to_vec();
    // No decision at all (in particular no priority) during the resolution.
    assert_eq!(t.asked().len(), asked);
    let flips = during
        .iter()
        .filter(|e| matches!(e, Event::CoinFlipped { .. }))
        .count();
    // Four creatures that aren't Demons, Devils, or Imps (not Rakdos or Demon Wall).
    assert_eq!(flips, 4);
    let tails = during
        .iter()
        .filter(|e| matches!(e, Event::CoinFlipped { heads: false, .. }))
        .count();
    t.settle();
    let left = ["Grizzly Bears", "Hill Giant", "Craw Wurm", "Serra Angel"]
        .iter()
        .filter(|n| !t.named_on_battlefield(n).is_empty())
        .count();
    assert_eq!(left, 4 - tails);
    assert!(t.on_battlefield(rakdos));
    assert_eq!(t.named_on_battlefield("Demon Wall").len(), 1);
}

#[test]
fn righteous_valkyrie_gains_the_toughness_on_resolution_or_last_known() {
    cr!("608.2h");
    ruling!(
        "Righteous Valkyrie",
        "The amount of life you gain is equal to the Angel or Cleric's toughness as the triggered ability resolves. If the Angel or Cleric is no longer on the battlefield at that time, use its toughness from when it was last on the battlefield."
    );
    supported("Righteous Valkyrie");
    // Giant Growth in response: 4 + 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Righteous Valkyrie");
    let angel = t.enter(P0, "Serra Angel");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    cast_new(&mut t, P0, "Giant Growth", &[obj(angel)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 27);
    // Pumped, then returned to hand in response: its last toughness, 5.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Righteous Valkyrie");
    let angel = t.enter(P0, "Serra Angel");
    t.g.flush_events();
    t.settle();
    pump(&mut t, angel, 0, 1);
    bounce(&mut t, angel);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}
