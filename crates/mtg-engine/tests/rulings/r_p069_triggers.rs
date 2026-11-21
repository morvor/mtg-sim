//! Rulings batch P069 — triggered abilities of cards that cast from exile: intervening
//! "if" clauses checked as the ability would trigger and again on resolution (CR 603.4),
//! triggers that resolve before the spell that caused them (CR 603.3, 608.1), delayed
//! triggers (CR 603.7), triggers of sources that left (CR 603.10, 113.7a), and targets that
//! became illegal (CR 608.2b).

use crate::r_p069_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported, watch};
use crate::r_s04_common::ability_targets;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn prepared(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).prepared.is_some()
}

/// Moves every card in `p`'s hand to their graveyard.
fn empty_hand(t: &mut TestGame, p: PlayerId) {
    for c in t.g.player(p).hand.clone() {
        t.g.move_object(c, Zone::Graveyard(p), mtg_engine::events::MoveCause::Effect, None)
            .unwrap();
    }
}

/// An intervening-"if" trigger of P0's `name` at the beginning of P0's `step` that makes
/// it prepared: it doesn't trigger if the condition is false as the step begins (making it
/// true during the step is too late), and it does nothing if the condition is false again
/// as it resolves.
fn checks_twice(
    name: &str,
    step: Step,
    make_true: fn(&mut TestGame),
    make_false: fn(&mut TestGame),
    rechecks: bool,
) {
    supported(name);
    // True as the step begins: it triggers and the permanent becomes prepared.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    make_true(&mut t);
    t.advance_to(P0, step);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{name}: {}", t.dump_log());
    t.resolve_all();
    assert!(prepared(&t, c), "{name}: {}", t.dump_log());
    // False as the step begins, made true during the step: no trigger.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    make_false(&mut t);
    t.advance_to(P0, step);
    t.settle();
    assert_eq!(t.stack_len(), 0, "{name}");
    make_true(&mut t);
    t.settle();
    assert_eq!(t.stack_len(), 0, "{name}");
    t.resolve_all();
    assert!(!prepared(&t, c), "{name}");
    if rechecks {
        // True as it triggers, false as it resolves: nothing happens.
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        make_true(&mut t);
        t.advance_to(P0, step);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{name}");
        make_false(&mut t);
        t.resolve_all();
        assert!(!prepared(&t, c), "{name}");
    }
}

#[test]
fn joined_researchers_checks_hands_as_the_end_step_starts_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Joined Researchers // Secret Rendezvous",
        "Joined Researchers's last ability will check as each end step starts to see if an opponent has more cards in hand than you. If none do, the ability won't trigger at all. It's not possible to discard cards or cause an opponent to draw cards during an end step in time to have the ability trigger. If it does trigger, the ability will check again as it tries to resolve. If no opponent has more cards in hand than you at that time, the ability won't resolve and none of its effects will happen."
    );
    checks_twice(
        "Joined Researchers // Secret Rendezvous",
        Step::End,
        |t| {
            empty_hand(t, P0);
            t.hand(P1, "Island");
        },
        |t| {
            empty_hand(t, P1);
            t.hand(P0, "Island");
        },
        true,
    );
}

#[test]
fn lorehold_archivist_checks_the_graveyard_as_it_triggers_and_on_resolution() {
    cr!("603.4", "503.1a");
    ruling!(
        "Lorehold Archivist // Restore Relic",
        "Lorehold Archivist's last ability checks at the moment it would trigger to see if there are three or more artifact and/or creature cards in your graveyard. If there aren't, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you don't have three or more artifact and/or creature cards in your graveyard at that time, the ability won't resolve and none of its effects will happen."
    );
    checks_twice(
        "Lorehold Archivist // Restore Relic",
        Step::Upkeep,
        |t| {
            t.graveyard(P0, "Grizzly Bears");
            t.graveyard(P0, "Mind Stone");
            t.graveyard(P0, "Ornithopter");
        },
        |t| {
            for c in t.g.player(P0).graveyard.clone() {
                t.g.move_object(c, Zone::Exile, mtg_engine::events::MoveCause::Effect, None)
                    .unwrap();
            }
            // Two creature cards and a sorcery: not enough.
            t.graveyard(P0, "Grizzly Bears");
            t.graveyard(P0, "Hill Giant");
            t.graveyard(P0, "Divination");
        },
        true,
    );
}

#[test]
fn naktamun_lorespinner_checks_hand_sizes_as_it_triggers_and_on_resolution() {
    cr!("603.4", "503.1a");
    ruling!(
        "Naktamun Lorespinner // Wheel of Fortune",
        "Naktamun Lorespinner's ability checks at the moment it would trigger to see if there are any players with one or fewer cards in hand. If there aren't, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If there are no players with one or fewer cards in hand at that time, the ability won't resolve and none of its effects will happen."
    );
    checks_twice(
        "Naktamun Lorespinner // Wheel of Fortune",
        Step::Upkeep,
        |t| empty_hand(t, P0),
        |t| {
            for p in [P0, P1] {
                while t.hand_size(p) < 2 {
                    t.hand(p, "Island");
                }
            }
        },
        true,
    );
}

#[test]
fn scheming_silvertongue_checks_life_gained_as_the_second_main_phase_starts() {
    cr!("603.4", "505.1b");
    ruling!(
        "Scheming Silvertongue // Sign in Blood",
        "Scheming Silvertongue's last ability will check as your second main phase starts to see if you gained 2 or more life this turn. If you didn't, the ability won't trigger at all. It's not possible to gain life during your second main phase in time to have the ability trigger."
    );
    checks_twice(
        "Scheming Silvertongue // Sign in Blood",
        Step::PostcombatMain,
        |t| {
            t.g.gain_life(P0, 2);
        },
        |t| {
            t.g.gain_life(P0, 1);
        },
        false,
    );
}

#[test]
fn jace_architect_of_thought_affects_creatures_attacking_anyone() {
    cr!("603.7a", "508.1");
    ruling!(
        "Jace, Architect of Thought",
        "Jace's first ability creates a delayed triggered ability that triggers whenever a creature an opponent controls attacks. It doesn't matter which player or planeswalker that creature is attacking."
    );
    supported("Jace, Architect of Thought");
    let mut t = TestGame::new(3);
    let jace = t.battlefield(P0, "Jace, Architect of Thought");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, jace, 0, &[]).unwrap();
    t.resolve_all();
    // P1's Grizzly Bears attacks P2, not P0 or Jace.
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P2))], &[]);
    assert_eq!(t.pt(bears), (1, 2), "{}", t.dump_log());
    assert_eq!(t.life(P2), 19);
}

#[test]
fn living_lore_triggers_on_any_combat_damage_and_needs_to_be_sacrificed() {
    cr!("510.2", "603.2", "701.21a");
    ruling!(
        "Living Lore",
        "Living Lore's last ability will trigger whenever it deals combat damage, no matter if that damage is dealt to an opponent, an opposing planeswalker, or a creature it's blocking or being blocked by. You choose whether to sacrifice Living Lore as that ability resolves. If Living Lore has already left the battlefield, perhaps because it was dealt lethal damage at the same time it dealt damage, you can't sacrifice it."
    );
    supported("Living Lore");
    // Living Lore exiles Divination (mana value 3): a 3/3. It deals combat damage to
    // `whom`; P0 sacrifices it and casts Divination (drawing two) when it can.
    let run = |whom: &str| {
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Divination");
        let ll = t.enter(P0, "Living Lore");
        t.resolve_all();
        assert_eq!(t.pt(ll), (3, 3));
        t.g.obj_mut(ll).summoning_sick = false;
        let target = match whom {
            "planeswalker" => Entity::Object(t.battlefield(P1, "Jace, Architect of Thought")),
            _ => Entity::Player(P1),
        };
        let blocker = match whom {
            "bears" => Some(t.battlefield(P1, "Grizzly Bears")),
            "giant" => Some(t.battlefield(P1, "Hill Giant")),
            _ => None,
        };
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        attack_with(&mut t, &[(ll, target)]);
        let blocks: Vec<_> = blocker.iter().map(|b| (*b, ll)).collect();
        block_and_finish(&mut t, P1, &blocks);
        t.resolve_all();
        (t, ll, hand)
    };
    for whom in ["player", "planeswalker", "bears"] {
        let (t, ll, hand) = run(whom);
        assert!(!t.on_battlefield(ll), "{whom}");
        assert!(t.in_graveyard(P0, "Living Lore"), "{whom}");
        assert_eq!(t.hand_size(P0), hand + 2, "{whom}: {}", t.dump_log());
    }
    // Blocked by Hill Giant (3/3): Living Lore dies at the same time, so it can't be
    // sacrificed and Divination isn't cast.
    let (t, _, hand) = run("giant");
    assert!(t.in_graveyard(P0, "Living Lore"));
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_exile("Divination"));
}

#[test]
fn long_range_sensor_triggers_once_per_player_attacked() {
    cr!("508.1", "603.2c");
    ruling!(
        "Long-Range Sensor",
        "Long-Range Sensor’s first ability will trigger once for each player you attack."
    );
    // Two attackers: at P1 and P2, then both at P1.
    for (targets, n) in [([P1, P2], 2), ([P1, P1], 1)] {
        let mut t = TestGame::new(3);
        let s = t.battlefield(P0, "Long-Range Sensor");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        t.attack(
            &[(a, Entity::Player(targets[0])), (b, Entity::Player(targets[1]))],
            &[],
        );
        assert_eq!(t.counters(s, counters::CHARGE), n, "{}", t.dump_log());
    }
}

#[test]
fn etali_primal_sickness_poison_doesnt_need_etali_to_stay() {
    cr!("704.5c", "113.7a");
    ruling!(
        "Etali, Primal Conqueror // Etali, Primal Sickness",
        "Losing the game because a player (preferably an opponent) has ten or more poison counters is a rule of the game. Etali, Primal Sickness doesn’t have to still be on the battlefield when someone (preferably an opponent) gets their tenth poison counter."
    );
    let mut t = TestGame::new(3);
    let e = t.battlefield(P0, "Etali, Primal Conqueror // Etali, Primal Sickness");
    assert!(mtg_engine::dfc::transform(&mut t.g, e));
    t.g.recompute();
    assert_eq!(t.obj_now(e).chars.name, "Etali, Primal Sickness");
    t.g.players[1].counters.insert(counters::POISON.into(), 3);
    attack_with(&mut t, &[(e, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    // Etali leaves with its trigger on the stack.
    t.g.move_object(e, Zone::Exile, mtg_engine::events::MoveCause::Effect, None)
        .unwrap();
    t.resolve_all();
    // Etali, Primal Sickness is an 11/11: P1 gets 11 poison counters.
    assert_eq!(t.g.player(P1).counters.get(counters::POISON).copied(), Some(14));
    assert!(t.has_lost(P1));
}

#[test]
fn nassari_triggers_on_any_spell_cast_from_exile() {
    cr!("603.2", "601.2i");
    ruling!(
        "Uvilda, Dean of Perfection // Nassari, Dean of Expression",
        "Nassari’s last ability triggers when you cast any spell from exile, not just the cards that were exiled with its first ability."
    );
    let mut t = TestGame::new(2);
    let u = t.hand(P0, "Uvilda, Dean of Perfection // Nassari, Dean of Expression");
    mana(&mut t, P0, ManaType::R, 5);
    let n = t.cast(P0, u).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(n).chars.name, "Nassari, Dean of Expression");
    // A Lightning Bolt exiled by Ornate Kanzashi.
    let k = t.battlefield(P0, "Ornate Kanzashi");
    let bolt = t.library_top(P1, "Lightning Bolt");
    mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, k, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, bolt));
    mana(&mut t, P0, ManaType::R, 1);
    cast_now(&mut t, P0, bolt, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.counters(n, counters::PLUS1), 1, "{}", t.dump_log());
}

/// P0's Nita exiles P1's Lightning Bolt from P1's graveyard (sacrificing a Grizzly Bears)
/// and P0 casts it at P1, paying with black mana. Returns (Nita, Hill Giant, Bolt).
fn nita_casts_bolt(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    supported("Nita, Forum Conciliator");
    let nita = t.battlefield(P0, "Nita, Forum Conciliator");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let bolt = t.graveyard(P1, "Lightning Bolt");
    mana(t, P0, ManaType::C, 2);
    t.answer_choose(P0, &[o(bears)]);
    t.activate(P0, nita, 0, &[o(bolt)]).unwrap();
    t.resolve_all();
    assert!(exiled(t, bolt), "{}", t.dump_log());
    mana(t, P0, ManaType::B, 1);
    cast_now(t, P0, bolt, &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    (nita, giant, bolt)
}

#[test]
fn nita_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Nita, Forum Conciliator",
        "Nita's first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    let mut t = TestGame::new(2);
    let (nita, giant, bolt) = nita_casts_bolt(&mut t);
    t.resolve();
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
    assert_eq!(t.counters(nita, counters::PLUS1), 1);
    assert_eq!(t.zone(bolt), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // P1 counters the Bolt above the trigger: the trigger still resolves.
    let mut t = TestGame::new(2);
    let (_, giant, bolt) = nita_casts_bolt(&mut t);
    let cs = t.hand(P1, "Counterspell");
    mana(&mut t, P1, ManaType::U, 2);
    t.g.turn.priority = Some(P1);
    let b = t.g.current(bolt);
    t.cast(P1, cs).target(b).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(giant, counters::PLUS1), 1, "{}", t.dump_log());
}

#[test]
fn norin_exiles_a_blocked_creature_before_combat_damage() {
    cr!("509.1h", "510.1");
    ruling!(
        "Norin, Swift Survivalist",
        "Norin's last ability triggers before combat damage is dealt. If you choose to exile the creature, it won't deal or receive combat damage."
    );
    supported("Norin, Swift Survivalist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Norin, Swift Survivalist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    t.resolve_all();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    assert_eq!(t.obj_now(giant).damage, 0);
    assert!(t.on_battlefield(giant));
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::G, 2);
    assert!(castable(&mut t, P0, bears));
}

#[test]
fn eiganjo_dynastorian_becomes_prepared_whatever_happens_to_the_attackers() {
    cr!("603.2", "722.3a");
    ruling!(
        "Eiganjo Dynastorian // Replenish",
        "Once Eiganjo Dynastorian's last ability triggers, it doesn't matter what happens to the attacking creatures in response."
    );
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Eiganjo Dynastorian // Replenish");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(e, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(prepared(&t, e), "{}", t.dump_log());
}

#[test]
fn rona_untaps_for_legendary_spells_not_lands_and_resolves_first() {
    cr!("305.1", "603.2", "603.3");
    ruling!(
        "Rona, Herald of Invasion // Rona, Tolarian Obliterator",
        "Playing a legendary land won’t cause the first ability to trigger."
    );
    ruling!(
        "Rona, Herald of Invasion // Rona, Tolarian Obliterator",
        "The first ability of Rona, Herald of Invasion will resolve before the spell that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    let rona = t.battlefield(P0, "Rona, Herald of Invasion // Rona, Tolarian Obliterator");
    t.g.obj_mut(rona).tapped = true;
    let cradle = t.hand(P0, "Gaea's Cradle");
    t.play_land(P0, cradle).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.obj_now(rona).tapped);
    let isamaru = t.hand(P0, "Isamaru, Hound of Konda");
    mana(&mut t, P0, ManaType::W, 1);
    t.cast(P0, isamaru).go();
    t.resolve();
    assert!(!t.obj_now(rona).tapped);
    assert_eq!(t.zone(isamaru), Zone::Stack);
}

#[test]
fn rashmi_and_ragavan_must_be_on_the_battlefield_as_the_first_spell_is_cast() {
    cr!("603.2", "601.2i", "601.2h");
    ruling!(
        "Rashmi and Ragavan",
        "Rashmi and Ragavan has to be on the battlefield at the moment you cast your first spell. Notably, on the turn you cast Rashmi and Ragavan, the ability won’t trigger for you that turn. If a spell causes Rashmi and Ragavan to leave the battlefield as an additional cost to cast it, the ability won’t trigger."
    );
    supported("Rashmi and Ragavan");
    // Cast this turn: the Bolt, the second spell, doesn't make it trigger.
    let mut t = TestGame::new(2);
    let rr = t.hand(P0, "Rashmi and Ragavan");
    for ty in [ManaType::G, ManaType::U, ManaType::R, ManaType::R] {
        mana(&mut t, P0, ty, 1);
    }
    t.cast(P0, rr).go();
    t.resolve_all();
    assert!(t.on_battlefield(rr));
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Sacrificed to cast Bone Splinters, the first spell: no trigger.
    let mut t = TestGame::new(2);
    let rr = t.battlefield(P0, "Rashmi and Ragavan");
    let giant = t.battlefield(P1, "Hill Giant");
    let bs = t.hand(P0, "Bone Splinters");
    mana(&mut t, P0, ManaType::B, 1);
    t.answer_choose(P0, &[o(rr)]);
    t.cast(P0, bs).target(giant).go();
    assert!(!t.on_battlefield(rr));
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    // On the battlefield as the first spell is cast: it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi and Ragavan");
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, ManaType::R, 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
}

/// P0 activates Codie (with Thalia, Guardian of Thraben making noncreature spells cost
/// {1} more when `thalia`) and casts Divination (mana value 3); P0's library is `lib`
/// (top first). Returns (library cards, Divination).
fn codie(t: &mut TestGame, lib: &[&str], thalia: bool) -> (Vec<ObjectId>, ObjectId) {
    supported("Codie, Vociferous Codex");
    let codie = t.battlefield(P0, "Codie, Vociferous Codex");
    if thalia {
        t.battlefield(P1, "Thalia, Guardian of Thraben");
    }
    t.g.players[0].library.clear();
    let cards = stack_library(t, P0, lib);
    mana(t, P0, ManaType::C, 4);
    t.activate(P0, codie, 0, &[]).unwrap();
    t.resolve_all();
    if thalia {
        mana(t, P0, ManaType::C, 1);
    }
    let div = t.hand(P0, "Divination");
    let div = t.cast(P0, div).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    (cards, div)
}

#[test]
fn codie_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.7a", "603.3");
    ruling!(
        "Codie, Vociferous Codex",
        "The delayed triggered ability created by Codie’s second ability will resolve before the spell that caused it to trigger. That ability will resolve even if the spell is countered or otherwise doesn’t resolve."
    );
    // The trigger resolves first: the Bolt is exiled while Divination is on the stack.
    let mut t = TestGame::new(2);
    let (cards, div) = codie(&mut t, &["Hill Giant", "Lightning Bolt"], false);
    t.resolve();
    assert!(exiled(&t, cards[1]), "{}", t.dump_log());
    assert_eq!(t.zone(div), Zone::Stack);
    // Countered in response, it still happens.
    let mut t = TestGame::new(2);
    let (cards, div) = codie(&mut t, &["Hill Giant", "Lightning Bolt"], false);
    let cs = t.hand(P1, "Counterspell");
    mana(&mut t, P1, ManaType::U, 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(div).go();
    // Counterspell resolves, then the trigger: the Bolt is exiled, castable this turn.
    t.resolve();
    assert_eq!(t.zone(div), Zone::Graveyard(P0));
    t.resolve();
    assert!(exiled(&t, cards[1]), "{}", t.dump_log());
    assert!(castable(&mut t, P0, cards[1]));
}

#[test]
fn codie_uses_the_mana_value_not_the_total_cost() {
    cr!("202.3", "601.2f");
    ruling!(
        "Codie, Vociferous Codex",
        "The mana value of the spell that caused the ability to trigger is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    // Divination costs {3}{U} with Thalia, but its mana value is 3: Cancel (3) isn't
    // lesser, Counterspell (2) is.
    let mut t = TestGame::new(2);
    let (cards, _) = codie(&mut t, &["Cancel", "Counterspell", "Lightning Bolt"], true);
    t.resolve();
    assert!(exiled(&t, cards[1]), "{}", t.dump_log());
    assert_eq!(t.zone(cards[0]), Zone::Library(P0));
    assert_eq!(t.zone(cards[2]), Zone::Library(P0));
}

#[test]
fn jodah_includes_x_in_the_mana_value() {
    cr!("202.3e", "107.3");
    ruling!(
        "Jodah, the Unifier",
        "The mana value of the spell that caused the ability to trigger is determined only by its mana cost, no matter what the total cost to cast the spell was. If that spell had an {X} in its cost, include the chosen value for X when determining its mana value."
    );
    supported("Jodah, the Unifier");
    // Big Bertha ({X}{G}) with X = 3 has mana value 4, though Helm of Awakening ("Spells
    // cost {1} less to cast.") makes its total cost 3: Jace, Architect of Thought (4) isn't
    // lesser; Oketra's Monument (3) is.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jodah, the Unifier");
    t.battlefield(P0, "Helm of Awakening");
    let mon = t.library_top(P0, "Oketra's Monument");
    let jace = t.library_top(P0, "Jace, Architect of Thought");
    let bb = t.hand(P0, "Big Bertha");
    mana(&mut t, P0, ManaType::G, 3);
    t.answer_yes(P0, true);
    t.cast(P0, bb).x(3).go();
    assert!(t.g.player(P0).mana_pool.is_empty());
    t.resolve_all();
    assert!(t.on_battlefield(mon), "{}", t.dump_log());
    assert!(!t.on_battlefield(jace));
}

#[test]
fn hidetsugu_and_kairi_does_nothing_if_the_opponent_becomes_an_illegal_target() {
    cr!("608.2b", "702.11c");
    ruling!(
        "Hidetsugu and Kairi",
        "The last triggered ability targets the opponent. If that player is an illegal target as the ability tries to resolve, the ability won’t resolve and none of its effects will happen. You won’t exile a card."
    );
    let mut t = TestGame::new(2);
    let hk = t.battlefield(P0, "Hidetsugu and Kairi");
    let top = t.library_top(P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.destroy(hk, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // P1 gains hexproof in response.
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Library(P0));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn smirking_spelljacker_exiles_a_spell_that_cant_be_countered() {
    cr!("701.6a");
    ruling!(
        "Smirking Spelljacker",
        "Spells that can’t be countered can still be exiled by Smirking Spelljacker’s triggered ability. They won’t resolve."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let decay = t.hand(P1, "Abrupt Decay");
    mana(&mut t, P1, ManaType::B, 1);
    mana(&mut t, P1, ManaType::G, 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, decay).target(bears).go();
    t.answer_targets(P0, &[o(decay)]);
    t.enter(P0, "Smirking Spelljacker");
    t.resolve_all();
    assert!(exiled(&t, decay), "{}", t.dump_log());
    assert!(t.on_battlefield(bears));
}

#[test]
fn shell_of_the_last_kappa_targets_only_spells_that_target_you() {
    cr!("115.9b", "115.2");
    ruling!(
        "Shell of the Last Kappa",
        "Shell of the Last Kappa’s first ability can only target a spell that targets you."
    );
    let mut t = TestGame::new(2);
    let shell = t.battlefield(P0, "Shell of the Last Kappa");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, ManaType::R, 2);
    t.g.turn.priority = Some(P1);
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.g.turn.priority = Some(P1);
    let shock = t.cast(P1, shock).target(bears).go();
    let targets = ability_targets(&mut t, shell, 0);
    assert!(targets.contains(&o(bolt)), "{targets:?}");
    assert!(!targets.contains(&o(shock)), "{targets:?}");
}

#[test]
fn jeleva_casts_the_card_before_blockers_are_declared() {
    cr!("508.1", "509.1");
    ruling!(
        "Jeleva, Nephalia's Scourge",
        "The card is cast before blocking creatures are declared."
    );
    supported("Jeleva, Nephalia's Scourge");
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P1, "Lightning Bolt");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jeleva = t.hand(P0, "Jeleva, Nephalia's Scourge");
    for ty in [ManaType::U, ManaType::B, ManaType::R, ManaType::C] {
        mana(&mut t, P0, ty, 1);
    }
    t.cast(P0, jeleva).go();
    t.resolve_all();
    assert!(exiled(&t, bolt), "{}", t.dump_log());
    t.g.obj_mut(jeleva).summoning_sick = false;
    let j = t.g.current(jeleva);
    t.g.obj_mut(j).summoning_sick = false;
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[o(bolt)]);
    t.answer_targets(P0, &[o(bears)]);
    attack_with(&mut t, &[(j, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
}

#[test]
fn kaya_ghostform_counter_is_only_a_reminder() {
    cr!("122.1", "613.1f");
    ruling!(
        "Kaya the Inexorable",
        "The ghostform counter added by the first ability doesn’t have any functional importance. It’s there to remind you of the ability the creature gains. Even if that counter is removed from the creature, the triggered ability it gains will still work."
    );
    supported("Kaya the Inexorable");
    let mut t = TestGame::new(2);
    let kaya = t.battlefield(P0, "Kaya the Inexorable");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, kaya, 0, &[o(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "ghostform"), 1);
    t.g.remove_counters(Entity::Object(bears), "ghostform", 1);
    assert_eq!(t.counters(bears, "ghostform"), 0);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(spirits(&t), 1);
}

fn spirits(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Spirit"))
        .count()
}

#[test]
fn the_legend_of_yangchen_exiles_the_chosen_permanents_simultaneously() {
    cr!("714.2b", "101.4");
    ruling!(
        "The Legend of Yangchen // Avatar Yangchen",
        "Permanents chosen by players using The Legend of Yangchen's first ability will be exiled simultaneously once all players have chosen permanents."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_choose(P0, &[o(giant)]);
    t.answer_choose(P1, &[o(angel)]);
    // As P1 chooses, P0's choice is still on the battlefield.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            g.battlefield
                .iter()
                .filter(|id| g.obj(**id).chars.name == "Hill Giant")
                .count()
        },
    );
    t.enter(P0, "The Legend of Yangchen // Avatar Yangchen");
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![1]);
    assert!(exiled(&t, giant) && exiled(&t, angel), "{}", t.dump_log());
}
