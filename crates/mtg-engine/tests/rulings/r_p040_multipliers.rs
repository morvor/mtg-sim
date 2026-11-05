//! Rulings batch P040 — damage multipliers ("deals double/triple that damage instead",
//! CR 701.10g, 614.1a): any damage from any source, trample and divided damage assigned
//! unmodified, several multipliers stacking, the affected player ordering multipliers
//! and prevention (CR 616.1), the source staying the same, Isengard Unleashed's "damage
//! can't be prevented" (CR 615.12), and battles' protectors.

use crate::r_p040_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s03_common::respond;
use crate::r_s04_common::spell_targets;
use crate::r_s06_common::attach_new;
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s29_common::replacement_choosers;
use crate::r_s30_common::{damage_events, pick_replacement};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Damage dealt this turn by `source` to `to`.
fn dealt(t: &TestGame, source: ObjectId, to: Entity) -> u32 {
    damage_events(t)
        .iter()
        .filter(|(s, r, _, _)| *s == source && *r == to)
        .map(|(_, _, n, _)| *n)
        .sum()
}

/// `p` casts Healing Salve's second mode ("Prevent the next 3 damage that would be dealt
/// to any target this turn") on `to`.
fn salve(t: &mut TestGame, p: PlayerId, to: Entity) {
    lands_for_cost(t, p, "Healing Salve");
    let c = t.hand(p, "Healing Salve");
    t.cast(p, c).modes(&[1]).target(to).go();
    t.resolve_all();
}

fn not_prevention(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let _ = g;
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| !o.contains("Healing Salve") && !o.contains("Mending Hands"))
            .map(Answer::Index),
        _ => None,
    }
}

fn prevention_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "Healing Salve").or_else(|| pick_replacement(g, d, "Mending Hands"))
}

// ---------------------------------------------------------------------------
// What gets multiplied
// ---------------------------------------------------------------------------

#[test]
fn dictate_doubles_any_damage_whoever_controls_the_source() {
    cr!("701.10g", "614.1a");
    ruling!(
        "Dictate of the Twin Gods",
        "Dictate of the Twin Gods applies to any damage, not just combat damage. It also doesn't matter who controls the source of the damage that's being dealt."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dictate of the Twin Gods");
    // Noncombat damage from P0's and from P1's spells.
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    // Combat damage from P1's creature to P0, and to a creature.
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P0, "Wall of Stone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, pl(P0)), (bears, pl(P0))]);
    block_and_finish(&mut t, P0, &[(wall, bears)]);
    assert_eq!(t.life(P0), 10);
    assert_eq!(damage(&t, wall), 4);
}

#[test]
fn trample_assigns_the_unmodified_damage() {
    cr!("702.19b", "510.1c", "701.10g");
    ruling!(
        "Angrath's Marauders",
        "If a creature with trample you control would deal combat damage to a blocking creature while you control Angrath's Marauders, you must assign its unmodified damage. For example, a 3/3 creature with trample blocked by a 2/2 creature can have at most 1 damage assigned to the defending player. It will then deal 4 damage to the blocking creature and 2 damage to the defending player."
    );
    ruling!(
        "Fiery Emancipation",
        "If a creature with trample you control would deal combat damage to a blocking creature while you control Fiery Emancipation, you must assign its unmodified damage. For example, a 3/3 creature with trample blocked by a 2/2 creature can have 2 damage assigned to the blocking creature and 1 damage assigned to the defending player. It will then deal 6 damage to the blocking creature and 3 damage to the defending player."
    );
    // (card, factor)
    for (card, k) in [("Angrath's Marauders", 2u32), ("Fiery Emancipation", 3)] {
        supported(card);
        // A legal assignment: 2 to the Bears, 1 to P1.
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        let mammoth = old(&mut t, P0, "War Mammoth");
        let bears = t.battlefield(P1, "Grizzly Bears");
        attack_with(&mut t, &[(mammoth, pl(P1))]);
        let from = mark(&t);
        t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 1]));
        block_and_finish(&mut t, P1, &[(bears, mammoth)]);
        assert_eq!(lethal_offers(&t, from), vec![vec![2]], "{card}");
        assert_eq!(dealt(&t, mammoth, o(bears)), 2 * k, "{card}");
        assert_eq!(dealt(&t, mammoth, pl(P1)), k, "{card}");
        assert_eq!(t.life(P1), 20 - k as i32, "{card}");
        // An illegal one (only 1 to the Bears, as if its damage were already multiplied):
        // not accepted; the default (lethal damage to the Bears, the rest to P1) is used,
        // so still only 1 is assigned to P1.
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        let mammoth = old(&mut t, P0, "War Mammoth");
        let bears = t.battlefield(P1, "Grizzly Bears");
        attack_with(&mut t, &[(mammoth, pl(P1))]);
        t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 2]));
        block_and_finish(&mut t, P1, &[(bears, mammoth)]);
        assert_eq!(dealt(&t, mammoth, pl(P1)), k, "{card}");
        assert_eq!(dealt(&t, mammoth, o(bears)), 2 * k, "{card}");
    }
}

/// P0 casts Forked Bolt dividing 1 damage to P1 and 1 to P1's Hill Giant. Returns the
/// total offered to divide, and the damage dealt to P1 and to the Giant.
fn forked_bolt_1_and_1(t: &mut TestGame) -> (Vec<u32>, u32, u32) {
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for_cost(t, P0, "Forked Bolt");
    let card = t.hand(P0, "Forked Bolt");
    t.answer_targets(P0, &[o(giant), pl(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    let from = mark(t);
    let spell = t.cast_with(P0, card, &[]).unwrap();
    t.resolve_all();
    let totals = crate::r_s01_common::asked_since(t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide { total, .. } => Some(total),
            _ => None,
        })
        .collect();
    (totals, dealt(t, spell, pl(P1)), dealt(t, spell, o(giant)))
}

#[test]
fn divided_damage_is_divided_before_it_is_multiplied() {
    cr!("601.2d", "701.10g");
    ruling!(
        "Furnace of Rath",
        "If a spell or ability damages multiple things, divide up the damage before applying this effect. This means you can’t normally end up with an odd amount of damage on something."
    );
    ruling!(
        "Fiery Emancipation",
        "If an effect asks you to divide damage among targets, you must divide the unmodified damage before tripling it."
    );
    for (card, k) in [("Furnace of Rath", 2u32), ("Fiery Emancipation", 3)] {
        supported(card);
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        assert_eq!(forked_bolt_1_and_1(&mut t), (vec![2], k, k), "{card}");
    }
}

#[test]
fn multipliers_stack() {
    cr!("701.10g", "616.1");
    ruling!(
        "Angrath's Marauders",
        "If you control a second Angrath's Marauders, damage dealt by sources you control will be multiplied by 4. If you control a third, it will be multiplied by 8, and so on."
    );
    ruling!(
        "Furnace of Rath",
        "If you have two of these on the battlefield, the damage is multiplied by 4."
    );
    // (card, number of them, factor)
    for (card, n, k) in [
        ("Angrath's Marauders", 1, 2),
        ("Angrath's Marauders", 2, 4),
        ("Angrath's Marauders", 3, 8),
        ("Furnace of Rath", 2, 4),
    ] {
        let mut t = TestGame::new(2);
        t.g.players[P1.idx()].life = 100;
        for _ in 0..n {
            t.battlefield(P0, card);
        }
        cast_new(&mut t, P0, "Shock", &[pl(P1)]);
        t.resolve_all();
        assert_eq!(t.life(P1), 100 - 2 * k, "{card} x{n}");
    }
}

#[test]
fn isengard_unleashed_twice_or_three_times_multiplies_by_9_or_27() {
    cr!("701.10g", "616.1", "611.2a");
    ruling!(
        "Isengard Unleashed",
        "If you resolve two copies of Isengard Unleashed in one turn, damage dealt by sources you control will be multiplied by 9. If you resolve three, it will be multiplied by 27, and so on."
    );
    for (n, k) in [(1, 3), (2, 9), (3, 27)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.g.players[P1.idx()].life = 100;
        for _ in 0..n {
            cast_new(&mut t, P0, "Isengard Unleashed", &[]);
            t.resolve_all();
        }
        cast_new(&mut t, P0, "Shock", &[pl(P1)]);
        t.resolve_all();
        assert_eq!(t.life(P1), 100 - 2 * k, "x{n}");
    }
}

// ---------------------------------------------------------------------------
// The source of the damage
// ---------------------------------------------------------------------------

#[test]
fn multiplied_damage_comes_from_the_original_source() {
    cr!("701.10g", "120.3f", "702.15b");
    ruling!(
        "Furnace of Rath",
        "The multiplied damage counts in all ways as if it came from the original source. Furnace of Rath is not the source."
    );
    ruling!(
        "Dictate of the Twin Gods",
        "The source of the damage doesn't change. A spell that deals damage will specify the source of the damage, often the spell itself. An ability that deals damage will also specify the source of the damage, although the ability itself will never be that source. Often the source of the ability is also the source of the damage."
    );
    ruling!(
        "Curse of Bloodletting",
        "The source of the damage doesn’t change. A spell that deals damage will specify the source of the damage, often the spell itself. An ability that deals damage will also specify the source of the damage, although the ability itself will never be that source. Often the source of the ability is also the source of the damage."
    );
    supported("Curse of Bloodletting");
    for card in [
        "Furnace of Rath",
        "Dictate of the Twin Gods",
        "Curse of Bloodletting",
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let doubler = if card == "Curse of Bloodletting" {
            attach_new(&mut t, P0, card, pl(P1))
        } else {
            t.battlefield(P0, card)
        };
        // A spell: Shock is the source of all 4.
        let shock = cast_new(&mut t, P0, "Shock", &[pl(P1)]);
        t.resolve_all();
        assert_eq!(dealt(&t, shock, pl(P1)), 4, "{card}");
        // An ability: Prodigal Pyromancer (with Spirit Link: "Whenever enchanted creature
        // deals damage, you gain that much life") is the source of all 2, not the ability.
        let pyro = old(&mut t, P0, "Prodigal Pyromancer");
        attach_new(&mut t, P0, "Spirit Link", o(pyro));
        let life = t.life(P0);
        t.answer_targets(P0, &[pl(P1)]);
        crate::r_p039_creatureland::activate_text(&mut t, P0, pyro, "deals 1 damage")
            .expect("Pyromancer");
        t.resolve_all();
        assert_eq!(dealt(&t, pyro, pl(P1)), 2, "{card}");
        assert_eq!(t.life(P0), life + 2, "{card}");
        // Combat damage with lifelink: Vampire Nighthawk deals 4 and P0 gains 4.
        let hawk = old(&mut t, P0, "Vampire Nighthawk");
        let life = t.life(P0);
        to_combat(&mut t, P0);
        attack_with(&mut t, &[(hawk, pl(P1))]);
        block_and_finish(&mut t, P1, &[]);
        assert_eq!(dealt(&t, hawk, pl(P1)), 4, "{card}");
        assert_eq!(t.life(P0), life + 4, "{card}");
        // The doubler itself dealt nothing.
        assert!(
            damage_events(&t).iter().all(|(s, _, _, _)| *s != doubler),
            "{card}"
        );
        assert_eq!(t.life(P1), 20 - 4 - 2 - 4, "{card}");
    }
}

// ---------------------------------------------------------------------------
// Ordering with prevention (CR 616.1)
// ---------------------------------------------------------------------------

/// A board with a damage multiplier for damage P0's Shock deals to P1.
struct Order {
    card: &'static str,
    /// Sets up the multiplier; returns the factor.
    setup: fn(&mut TestGame) -> u32,
}

fn order_cases() -> Vec<Order> {
    vec![
        Order {
            card: "Sawhorn Nemesis",
            setup: |t| {
                t.answer_choose(P0, &[pl(P1)]);
                t.enter(P0, "Sawhorn Nemesis");
                t.resolve_all();
                2
            },
        },
        Order {
            card: "Lightning, Army of One",
            setup: |t| {
                // Stagger: after it deals combat damage to P1, damage to P1 is doubled.
                let l = old(t, P0, "Lightning, Army of One");
                to_combat(t, P0);
                attack_with(t, &[(l, pl(P1))]);
                block_and_finish(t, P1, &[]);
                t.resolve_all();
                t.set_step(P0, Step::PostcombatMain);
                t.g.players[P1.idx()].life = 20;
                t.g.turn_events.clear();
                2
            },
        },
        Order {
            card: "Twinflame Tyrant",
            setup: |t| {
                t.battlefield(P0, "Twinflame Tyrant");
                2
            },
        },
        Order {
            card: "Fiery Emancipation",
            setup: |t| {
                t.battlefield(P0, "Fiery Emancipation");
                3
            },
        },
    ]
}

#[test]
fn the_player_dealt_damage_orders_multipliers_and_prevention() {
    cr!("616.1", "615.1a", "701.10g", "120.8");
    ruling!(
        "Sawhorn Nemesis",
        "If another effect (or effects) modifies how much damage a source would deal to the chosen player or a permanent they control (for example, by preventing some of that damage), the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects, including the effect of Sawhorn Nemesis, apply. If all of the damage is prevented before Sawhorn Nemesis's effect would apply, its effect no longer applies."
    );
    ruling!(
        "Lightning, Army of One",
        "If another effect (or effects) modifies how much damage a source you control would deal—by preventing some of it, for example—the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects (including Lightning's) apply. If all of the damage is prevented, Lightning's effect no longer applies."
    );
    ruling!(
        "Twinflame Tyrant",
        "If another effect (or effects) modifies how much damage a source you control would deal—by preventing some of it, for example—the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects (including Twinflame Tyrant's) apply. If all of the damage is prevented, Twinflame Tyrant's effect no longer applies."
    );
    ruling!(
        "Fiery Emancipation",
        "If multiple replacement or prevention effects try to modify damage that would be dealt to a player or permanent, the player or the controller of the permanent chooses the order in which they apply."
    );
    for c in order_cases() {
        supported(c.card);
        for prevent_first in [true, false] {
            // Some damage prevented: Lava Axe (5) and Mending Hands (prevent 4) on P1.
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let k = (c.setup)(&mut t);
            cast_new(&mut t, P1, "Mending Hands", &[pl(P1)]);
            t.resolve_all();
            respond(
                &mut t,
                P1,
                if prevent_first {
                    prevention_first
                } else {
                    not_prevention
                },
            );
            let from = mark(&t);
            cast_new(&mut t, P0, "Lava Axe", &[pl(P1)]);
            t.resolve_all();
            assert_eq!(replacement_choosers(&t, from), vec![P1], "{}", c.card);
            let want = if prevent_first { k } else { 5 * k - 4 };
            assert_eq!(20 - t.life(P1), want as i32, "{} {prevent_first}", c.card);
            // All of it prevented first: Shock (2) and Healing Salve (prevent 3) on P1. The
            // multiplier no longer applies; applied first, it leaves damage over.
            let mut t = TestGame::new(2);
            t.set_step(P0, Step::PrecombatMain);
            let k = (c.setup)(&mut t);
            salve(&mut t, P1, pl(P1));
            respond(
                &mut t,
                P1,
                if prevent_first {
                    prevention_first
                } else {
                    not_prevention
                },
            );
            let from = mark(&t);
            let shock = cast_new(&mut t, P0, "Shock", &[pl(P1)]);
            t.resolve_all();
            assert_eq!(replacement_choosers(&t, from), vec![P1], "{}", c.card);
            let want = if prevent_first { 0 } else { 2 * k - 3 };
            assert_eq!(dealt(&t, shock, pl(P1)), want, "{} {prevent_first}", c.card);
            assert_eq!(20 - t.life(P1), want as i32, "{} {prevent_first}", c.card);
        }
    }
}

#[test]
fn calamity_bearer_damage_order_is_chosen_by_the_player_dealt_damage() {
    cr!("616.1", "615.1a", "701.10g");
    ruling!(
        "Calamity Bearer",
        "If another effect (or effects) modifies how much damage your Giant source would deal—by preventing some of it, for example—the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects (including Calamity Bearer’s) apply. If all of the damage is prevented, Calamity Bearer’s effect no longer applies."
    );
    supported("Calamity Bearer");
    // P0's Hill Giant (a Giant, 3 power) hits P1, who has Healing Salve's 3-point shield.
    for prevent_first in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Calamity Bearer");
        let giant = old(&mut t, P0, "Hill Giant");
        salve(&mut t, P1, pl(P1));
        respond(
            &mut t,
            P1,
            if prevent_first {
                prevention_first
            } else {
                not_prevention
            },
        );
        let from = mark(&t);
        to_combat(&mut t, P0);
        attack_with(&mut t, &[(giant, pl(P1))]);
        block_and_finish(&mut t, P1, &[]);
        assert_eq!(replacement_choosers(&t, from), vec![P1]);
        assert_eq!(t.life(P1), if prevent_first { 20 } else { 17 });
    }
    // Control: a non-Giant isn't doubled (and nothing is asked).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Calamity Bearer");
    let bears = old(&mut t, P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn goldnight_castigators_controller_orders_damage_to_them_and_it() {
    cr!("616.1", "615.1a", "701.10g");
    ruling!(
        "Goldnight Castigator",
        "If multiple replacement effects could apply to damage being dealt to you or Goldnight Castigator, you choose the order to apply those effects."
    );
    // To P0, then to the Castigator: P1's Shock, P0's Healing Salve shield.
    for to_castigator in [false, true] {
        for prevent_first in [true, false] {
            let mut t = TestGame::new(2);
            t.set_step(P1, Step::PrecombatMain);
            let c = t.battlefield(P0, "Goldnight Castigator");
            let target = if to_castigator { o(c) } else { pl(P0) };
            salve(&mut t, P0, target);
            respond(
                &mut t,
                P0,
                if prevent_first {
                    prevention_first
                } else {
                    not_prevention
                },
            );
            let from = mark(&t);
            let shock = cast_new(&mut t, P1, "Shock", &[target]);
            t.resolve_all();
            assert_eq!(replacement_choosers(&t, from), vec![P0]);
            assert_eq!(dealt(&t, shock, target), if prevent_first { 0 } else { 1 });
        }
    }
}

// ---------------------------------------------------------------------------
// Isengard Unleashed: damage can't be prevented
// ---------------------------------------------------------------------------

#[test]
fn isengard_unleashed_stops_only_prevention() {
    cr!("615.12", "614.1a", "616.1");
    ruling!(
        "Isengard Unleashed",
        "Isengard Unleashed only stops damage from being prevented by effects that use the word \"prevent\" or keyword abilities like protection that prevent damage."
    );
    // "Prevent": Healing Salve's shield doesn't work.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Isengard Unleashed", &[]);
    t.resolve_all();
    salve(&mut t, P1, pl(P1));
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // Not prevention: Benevolent Unicorn ("If a spell would deal damage to a permanent
    // or player, it deals that much damage minus 1 to that permanent or player instead")
    // still applies, in the order P1 chooses.
    for unicorn_first in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P1, "Benevolent Unicorn");
        cast_new(&mut t, P0, "Isengard Unleashed", &[]);
        t.resolve_all();
        if unicorn_first {
            respond(&mut t, P1, |g, d| {
                pick_replacement(g, d, "Benevolent Unicorn")
            });
        } else {
            respond(&mut t, P1, |g, d| pick_replacement(g, d, "Isengard"));
        }
        let from = mark(&t);
        cast_new(&mut t, P0, "Shock", &[pl(P1)]);
        t.resolve_all();
        assert_eq!(replacement_choosers(&t, from), vec![P1]);
        // (2 - 1) x 3 = 3, or 2 x 3 - 1 = 5.
        assert_eq!(t.life(P1), if unicorn_first { 17 } else { 15 });
    }
}

#[test]
fn isengard_unleashed_beats_protection_damage_but_not_its_targeting() {
    cr!("615.12", "702.16b", "702.16e");
    ruling!(
        "Isengard Unleashed",
        "Protection prevents damage, so protection will be unable to prevent damage after Isengard Unleashed has resolved. However, this won't allow a spell or ability to target illegally, even if that spell or ability would cause damage to be dealt."
    );
    // Paladin en-Vec: protection from black and from red.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let paladin = t.battlefield(P1, "Paladin en-Vec");
    // Control: before Isengard, Pyroclasm's damage to it is prevented.
    cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert_eq!(damage(&t, paladin), 0);
    cast_new(&mut t, P0, "Isengard Unleashed", &[]);
    t.resolve_all();
    // Shock still can't target it.
    assert!(!spell_targets(&mut t, P0, "Shock").contains(&o(paladin)));
    assert!(spell_targets(&mut t, P0, "Shock").contains(&pl(P1)));
    // Pyroclasm's damage (tripled) isn't prevented.
    let spell = cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert_eq!(dealt(&t, spell, o(paladin)), 6);
    assert!(t.in_graveyard(P1, "Paladin en-Vec"));
}

// ---------------------------------------------------------------------------
// Battles: protector vs controller
// ---------------------------------------------------------------------------

#[test]
fn sawhorn_nemesis_doubles_damage_to_battles_the_chosen_player_controls_not_protects() {
    cr!("310.9a", "310.6", "120.3h");
    ruling!(
        "Sawhorn Nemesis",
        "The protector of a battle with the Siege subtype is not that battle's controller."
    );
    // P1 controls an Invasion of Arcavios (defense 7) protected by P0. (chosen player, defense lost to
    // P0's Shock)
    for (chosen, lost) in [(P1, 4), (P0, 2)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.answer_choose(P1, &[pl(P0)]);
        let siege = t.enter(P1, "Invasion of Arcavios // Invocation of the Founders");
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(mtg_engine::battle::protector(&t.g, siege), Some(P0));
        assert_eq!(t.obj_now(siege).controller, P1);
        t.answer_choose(P0, &[pl(chosen)]);
        t.enter(P0, "Sawhorn Nemesis");
        t.g.flush_events();
        t.resolve_all();
        let before = t.counters(siege, counters::DEFENSE);
        assert!(before > 4);
        cast_new(&mut t, P0, "Shock", &[o(siege)]);
        t.resolve_all();
        assert_eq!(
            before - t.counters(siege, counters::DEFENSE),
            lost,
            "{chosen:?}"
        );
    }
}
