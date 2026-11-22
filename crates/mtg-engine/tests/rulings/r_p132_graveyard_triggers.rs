//! Rulings batch P132 — abilities of cards in a graveyard that return them to hand:
//! simultaneity with the event (CR 603.10, 704.3), "first time each turn", multiplayer
//! (CR 800.4a), last known information (CR 608.2h), intervening "if" clauses (CR 603.4)
//! and activation restrictions (CR 602.5b, 113.6k).

use crate::r_p079_common::{cast_card, concede};
use crate::r_p132_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s04_common::cycle;
use crate::r_s06_common::attach_new;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn adewale_returns_only_from_the_graveyard_when_a_vehicle_connects() {
    cr!("603.2", "400.7", "702.122a");
    ruling!(
        "Adéwalé, Breaker of Chains",
        "Adéwalé's last ability triggers only if it's already in your graveyard as a Vehicle you control deals combat damage to a player."
    );
    ruling!(
        "Adéwalé, Breaker of Chains",
        "If Adéwalé leaves your graveyard before its last ability resolves, the ability will have no effect, even if the card returns to your graveyard before that ability resolves."
    );
    for (where_, leave) in [("hand", false), ("graveyard", false), ("graveyard", true)] {
        let mut t = TestGame::new(2);
        let ade = if where_ == "hand" {
            t.hand(P0, "Adéwalé, Breaker of Chains")
        } else {
            t.graveyard(P0, "Adéwalé, Breaker of Chains")
        };
        let copter = t.battlefield(P0, "Untethered Express");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(crate::r_s04_common::crew(&mut t, P0, copter, &[bears]));
        attack_with(&mut t, &[(copter, pl(P1))]);
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert!(t.life(P1) < 20);
        let triggered = crate::r_p006_common::triggers_from(&t, ade);
        assert_eq!(triggered, usize::from(where_ == "graveyard"), "{where_}");
        if leave {
            blink_in_graveyard(&mut t, ade);
        }
        t.resolve_all();
        let want = if where_ == "graveyard" && !leave {
            "hand"
        } else {
            where_
        };
        assert_eq!(where_is(&t, ade), want);
    }
}

#[test]
fn erebos_titan_with_lethal_damage_is_destroyed_once_it_loses_indestructible() {
    cr!("704.5g", "702.12b", "120.6");
    ruling!(
        "Erebos's Titan",
        "Damage dealt to a creature with indestructible remains marked on that creature."
    );
    let mut t = TestGame::new(2);
    let titan = t.battlefield(P0, "Erebos's Titan");
    let mountain = t.battlefield(P1, "Mountain");
    damage(&mut t, mountain, obj(titan), 6);
    assert!(t.on_battlefield(titan));
    assert!(t.obj_now(titan).has_keyword(KeywordKind::Indestructible));
    t.battlefield(P1, "Grizzly Bears");
    flush(&mut t);
    assert_eq!(where_is(&t, titan), "graveyard");
}

#[test]
fn erebos_titan_does_not_trigger_when_a_player_leaves_the_game() {
    cr!("800.4a", "603.2");
    ruling!(
        "Erebos's Titan",
        "In a multiplayer game, the last ability won't trigger when a player with a creature card in their graveyard leaves the game."
    );
    let mut t = TestGame::new(3);
    let titan = t.graveyard(P0, "Erebos's Titan");
    t.graveyard(P2, "Grizzly Bears");
    t.hand(P0, "Forest");
    concede(&mut t, P2);
    assert_eq!(t.stack_len(), 0);
    // A creature card leaving an opponent's graveyard otherwise does trigger it.
    let bears = t.graveyard(P1, "Grizzly Bears");
    move_to(&mut t, bears, Zone::Exile);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(where_is(&t, titan), "hand");
}

#[test]
fn deathless_knight_must_be_in_the_graveyard_for_the_first_life_gain_of_the_turn() {
    cr!("603.2", "510.2", "704.3", "702.15b");
    ruling!(
        "Deathless Knight",
        "Deathless Knight must be in your graveyard immediately after you gain life for its last ability to trigger."
    );
    ruling!(
        "Deathless Knight",
        "If Deathless Knight enters your graveyard after you've gained life in a turn, its last ability can't trigger during that turn."
    );
    // The Knight (4/2) is blocked by a 2/2 while a lifelinker deals combat damage: the
    // life is gained before the Knight is put into the graveyard.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Deathless Knight");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(knight, pl(P1)), (hawk, pl(P1))]);
    block_and_finish(&mut t, P1, &[(bears, knight)]);
    assert_eq!(t.life(P0), 22);
    assert_eq!(where_is(&t, knight), "graveyard");
    // Gaining life again this turn isn't the first time.
    gain(&mut t, P0, 1);
    assert_eq!(t.stack_len(), 0);
    // The next turn's first life gain returns it.
    t.advance_to(P1, Step::Upkeep);
    gain(&mut t, P0, 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(where_is(&t, knight), "hand");
}

#[test]
fn pure_intentions_returns_only_from_an_opponents_discard_effect() {
    cr!("603.7", "701.9a");
    ruling!(
        "Pure Intentions",
        "Discarding Pure Intentions to your own spell or ability doesn't trigger its abilities."
    );
    ruling!(
        "Pure Intentions",
        "The second part is a triggered ability that triggers whenever an opponent's spell or ability causes a player to discard Pure Intentions."
    );
    ruling!(
        "Pure Intentions",
        "Pure Intentions doesn't return to hand until its triggered ability has resolved."
    );
    supported("Pure Intentions");
    for opponent in [false, true] {
        let mut t = TestGame::new(2);
        let caster = if opponent { P1 } else { P0 };
        t.set_step(caster, Step::PrecombatMain);
        let pi = t.hand(P0, "Pure Intentions");
        let bears = t.hand(P0, "Grizzly Bears");
        cast_card(&mut t, caster, "Mind Rot", &[pl(P0)]).unwrap();
        t.resolve_all();
        // Both discarded at once; Pure Intentions waits for the end step.
        assert_eq!(where_is(&t, pi), "graveyard");
        assert_eq!(where_is(&t, bears), "graveyard");
        t.advance_to(caster, Step::End);
        t.settle();
        t.resolve_all();
        let want = if opponent { "hand" } else { "graveyard" };
        assert_eq!(where_is(&t, pi), want, "opponent: {opponent}");
        assert_eq!(where_is(&t, bears), "graveyard");
    }
}

#[test]
fn pure_intentions_cast_returns_cards_an_opponent_makes_you_discard() {
    cr!("608.2", "701.9a");
    ruling!(
        "Pure Intentions",
        "The first part of Pure Intentions is the effect of casting Pure Intentions."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_card(&mut t, P0, "Pure Intentions", &[]).unwrap();
    t.resolve_all();
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    cast_card(&mut t, P1, "Mind Rot", &[pl(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(where_is(&t, a), "hand");
    assert_eq!(where_is(&t, b), "hand");
}

#[test]
fn zodiac_dragon_has_no_flying_and_doesnt_trigger_when_exiled_instead() {
    cr!("614.6", "603.6c");
    ruling!(
        "Zodiac Dragon",
        "Even though Zodiac Dragon is a Dragon, it doesn’t have flying."
    );
    ruling!(
        "Zodiac Dragon",
        "Similarly, if a replacement effect has Zodiac Dragon move to a different zone instead of being put into the graveyard, the ability won’t trigger at all."
    );
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Zodiac Dragon");
    assert!(!t.obj_now(dragon).has_keyword(KeywordKind::Flying));
    t.battlefield(P1, "Rest in Peace");
    destroy(&mut t, dragon);
    assert_eq!(where_is(&t, dragon), "exile");
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn graveyard_only_activated_abilities() {
    cr!("602.5b", "113.6");
    ruling!(
        "Firewing Phoenix",
        "Firewing Phoenix's ability can be activated only if Firewing Phoenix is in your graveyard."
    );
    ruling!(
        "Undead Gladiator",
        "The ability to return to your hand can only be activated while this card is in your graveyard."
    );
    ruling!(
        "Vineweft",
        "You can activate the last ability only if Vineweft is in your graveyard."
    );
    for (name, land) in [
        ("Firewing Phoenix", "Mountain"),
        ("Undead Gladiator", "Swamp"),
        ("Vineweft", "Forest"),
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::Upkeep);
        t.lands(P0, land, 5);
        t.hand(P0, "Plains");
        let c = t.battlefield(P0, name);
        assert!(!can_activate(&mut t, P0, c), "{name} on the battlefield");
        let c = move_to(&mut t, c, Zone::Graveyard(P0));
        assert!(can_activate(&mut t, P0, c), "{name} in the graveyard");
        let c = move_to(&mut t, c, Zone::Hand(P0));
        let can = t.g.legal_actions(P0).iter().any(|a| {
            matches!(a, Action::Activate { source, .. } if *source == c)
                && !matches!(name, "Undead Gladiator")
        });
        assert!(!can, "{name} in hand");
    }
}

#[test]
fn centaur_vinecrasher_triggers_when_put_into_the_graveyard_with_a_land() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Centaur Vinecrasher",
        "If Centaur Vinecrasher and a land card are put into your graveyard at the same time, Centaur Vinecrasher's last ability will trigger."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    crate::r_s01_common::stack_library(&mut t, P0, &["Centaur Vinecrasher", "Forest"]);
    t.g.mill(P0, 2);
    flush(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Centaur Vinecrasher"));
}

#[test]
fn viviens_jaguar_cant_be_returned_when_vivien_dies_with_it() {
    cr!("602.5b", "704.3");
    ruling!(
        "Vivien's Jaguar",
        "If a Vivien planeswalker and Vivien's Jaguar are both dealt lethal damage at the same time (most likely in combat), they'll be put into their owner's graveyard at the same time."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let vivien = t.battlefield(P0, "Vivien Reid");
    let jaguar = t.battlefield(P0, "Vivien's Jaguar");
    let src = t.battlefield(P1, "Mountain");
    t.g.deal_damage_batch(vec![(src, obj(vivien), 10), (src, obj(jaguar), 10)], false);
    flush(&mut t);
    assert_eq!(where_is(&t, jaguar), "graveyard");
    assert_eq!(where_is(&t, vivien), "graveyard");
    assert!(!can_activate(&mut t, P0, jaguar));
    t.battlefield(P0, "Vivien Reid");
    assert!(can_activate(&mut t, P0, jaguar));
}

#[test]
fn the_scorpion_god_draws_once_per_creature_with_counters_including_itself() {
    cr!("603.10a", "704.3");
    ruling!(
        "The Scorpion God",
        "If a creature with a -1/-1 counter on it dies at the same time as The Scorpion God does, you'll draw a card."
    );
    ruling!(
        "The Scorpion God",
        "You'll draw only one card when a creature with more than one -1/-1 counter on it dies."
    );
    let mut t = TestGame::new(2);
    let god = t.battlefield(P0, "The Scorpion God");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(obj(giant), counters::MINUS1, 2, None);
    flush(&mut t);
    let h = t.hand_size(P0);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h + 1);
    // The God (with a counter) and another creature with a counter die together.
    let ogre = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(obj(ogre), counters::MINUS1, 1, None);
    t.g.add_counters(obj(god), counters::MINUS1, 1, None);
    flush(&mut t);
    let src = t.battlefield(P1, "Mountain");
    t.g.deal_damage_batch(vec![(src, obj(god), 10), (src, obj(ogre), 10)], false);
    flush(&mut t);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h + 3);
}

#[test]
fn unshakable_tail_investigates_once_for_a_simultaneous_mill() {
    cr!("701.17a", "603.2c");
    ruling!(
        "Unshakable Tail",
        "If an effect puts multiple cards from a library into a graveyard, all of those cards move at the same time."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unshakable Tail");
    crate::r_s01_common::stack_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Hill Giant", "Forest"],
    );
    t.g.mill(P0, 3);
    flush(&mut t);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Clue").len(), 1);
}

#[test]
fn pyrewild_shaman_triggers_per_player_but_returns_only_the_card_that_triggered() {
    cr!("603.4", "400.7", "800.4a", "510.2");
    ruling!(
        "Pyrewild Shaman",
        "If creatures you control deal combat damage to more than one player at the same time (most likely because it's a multiplayer game), Pyrewild Shaman's ability will trigger once for each of those players."
    );
    let mut t = TestGame::new(3);
    let shaman = t.graveyard(P0, "Pyrewild Shaman");
    t.lands(P0, "Mountain", 6);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P2))]);
    t.advance_to(P0, Step::CombatDamage);
        t.settle();
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "combat damage"), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(where_is(&t, shaman), "hand");
    // Discarded back into the graveyard: a new object, not the one that triggered.
    let shaman = t.g.current(shaman);
    t.g.discard(P0, shaman, None);
    flush(&mut t);
    t.resolve_all();
    assert_eq!(where_is(&t, shaman), "graveyard");
}

#[test]
fn pyrewild_shaman_dying_in_the_same_combat_damage_doesnt_trigger() {
    cr!("603.2", "704.3", "510.2");
    ruling!(
        "Pyrewild Shaman",
        "Notably, if Pyrewild Shaman is dealt lethal damage at the same time that a creature you control deals combat damage to a player, Pyrewild Shaman's ability won't trigger."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let shaman = t.battlefield(P0, "Pyrewild Shaman");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(shaman, pl(P1)), (bears, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, shaman)]),
    );
    t.advance_to(P0, Step::CombatDamage);
        t.settle();
    assert_eq!(t.life(P1), 18);
    assert_eq!(where_is(&t, shaman), "graveyard");
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn skyblades_boon_isnt_returned_after_it_changed_zones() {
    cr!("400.7", "608.2b");
    ruling!(
        "Skyblade's Boon",
        "If you activate the last ability of Skyblade's Boon while it is on the battlefield or in your graveyard and it somehow moves to the other zone before the ability resolves, it will not be returned to its owner's hand."
    );
    supported("Skyblade's Boon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let boon = attach_new(&mut t, P0, "Skyblade's Boon", obj(bears));
    t.activate(P0, boon, 0, &[]).unwrap();
    destroy(&mut t, bears);
    assert_eq!(where_is(&t, boon), "graveyard");
    t.resolve_all();
    assert_eq!(where_is(&t, boon), "graveyard");
    // Activated from the graveyard and left alone: returned.
    let boon = t.g.current(boon);
    t.activate(P0, boon, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(where_is(&t, boon), "hand");
}

#[test]
fn spellpyre_phoenix_counts_every_cycle_this_turn() {
    cr!("702.29a", "603.4");
    ruling!(
        "Spellpyre Phoenix",
        "If you somehow cycle a card, return it to your hand, and cycle it a second time in one turn, Spellpyre Phoenix's last ability counts each time you cycled it and returns to your hand."
    );
    ruling!(
        "Spellpyre Phoenix",
        "Spellpyre Phoenix's last ability checks the entire turn. Cards you cycled before Spellpyre Phoenix was put into your graveyard count."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 2);
    let phoenix = t.battlefield(P0, "Spellpyre Phoenix");
    let moor = t.hand(P0, "Barren Moor");
    cycle(&mut t, P0, moor, 0).unwrap();
    t.resolve_all();
    let moor = move_to(&mut t, moor, Zone::Hand(P0));
    cycle(&mut t, P0, moor, 0).unwrap();
    t.resolve_all();
    destroy(&mut t, phoenix);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(where_is(&t, phoenix), "hand");
}

#[test]
fn gutterbones_counts_life_lost_by_an_opponent_who_left_the_game() {
    cr!("800.4a", "602.5b");
    ruling!(
        "Gutterbones",
        "In a multiplayer game, if an opponent loses life during your turn and then leaves the game, you can activate Gutterbones's ability."
    );
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 2);
    let gb = t.graveyard(P0, "Gutterbones");
    assert!(!can_activate(&mut t, P0, gb));
    t.g.lose_life(P1, 1);
    concede(&mut t, P1);
    assert!(can_activate(&mut t, P0, gb));
}

#[test]
fn multani_is_zero_zero_off_the_battlefield_and_returns_lands_from_the_battlefield() {
    cr!("611.3a", "118.3");
    ruling!(
        "Multani, Yavimaya's Avatar",
        "Multani's ability that modifies its power and toughness applies only while it's on the battlefield. In all other zones, it's a 0/0 creature card."
    );
    ruling!(
        "Multani, Yavimaya's Avatar",
        "Land cards in your graveyard can't be returned this way."
    );
    let mut t = TestGame::new(2);
    let m = t.graveyard(P0, "Multani, Yavimaya's Avatar");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Forest");
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::G, 2);
    assert_eq!(t.pt(m), (0, 0));
    assert!(!can_activate(&mut t, P0, m), "no lands on the battlefield");
    t.lands(P0, "Forest", 2);
    assert!(can_activate(&mut t, P0, m));
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(where_is(&t, m), "hand");
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn rekindled_flame_checks_each_opponent_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Rekindled Flame",
        "Rekindled Flame's triggered ability has an \"intervening 'if' clause.\""
    );
    ruling!(
        "Rekindled Flame",
        "The opponent that meets the condition when the ability triggers and the one who meets the condition when it resolves can be different players."
    );
    // 0: every opponent has a card at the upkeep: no trigger. 1: P2's hand is empty and
    // stays so. 2: P2 gets a card before it resolves: nothing. 3: P2 gets a card but P1
    // empties theirs: still returned.
    for case in 0..4 {
        let mut t = TestGame::new(3);
        let flame = t.graveyard(P0, "Rekindled Flame");
        let p1 = t.hand(P1, "Forest");
        let p2 = t.hand(P2, "Forest");
        if case >= 1 {
            move_to(&mut t, p2, Zone::Graveyard(P2));
        }
        t.set_step(P2, Step::PostcombatMain);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        if case == 0 {
            assert_eq!(t.stack_len(), 0, "every opponent has a card");
            continue;
        }
        assert_eq!(t.stack_len(), 1);
        if case >= 2 {
            t.hand(P2, "Forest");
        }
        if case == 3 {
            move_to(&mut t, p1, Zone::Graveyard(P1));
        }
        t.resolve_all();
        let want = if case == 2 { "graveyard" } else { "hand" };
        assert_eq!(where_is(&t, flame), want, "case {case}");
    }
}

#[test]
fn relentless_dead_both_abilities_trigger_and_each_payment_is_chosen_on_resolution() {
    cr!("603.3b", "608.2", "107.3f");
    interp!("trigger-x-before-targets");
    ruling!(
        "Relentless Dead",
        "Relentless Dead’s last two abilities trigger at the same time and can be put on the stack in either order."
    );
    ruling!(
        "Relentless Dead",
        "You decide whether to pay for each triggered ability as it resolves."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let dead = t.battlefield(P0, "Relentless Dead");
    let corpse = t.graveyard(P0, "Walking Corpse");
    // X is chosen as the ability is put on the stack, before its target (CR 107.3f).
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[obj(corpse)]);
    let from = t.asked().len();
    destroy(&mut t, dead);
    assert_eq!(t.stack_len(), 2);
    let order: Vec<&str> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseX { .. } => Some("x"),
            mtg_engine::decision::Decision::ChooseTargets { .. } => Some("targets"),
            mtg_engine::decision::Decision::YesNo { .. } => Some("pay"),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec!["x", "targets"], "X before targets, no payment yet");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(where_is(&t, dead), "hand");
    assert_eq!(where_is(&t, corpse), "battlefield");
}

#[test]
fn shard_phoenix_deals_damage_with_its_last_known_colors() {
    cr!("608.2h", "702.16");
    ruling!(
        "Shard Phoenix",
        "Shard Phoenix uses Last Known Information. If, for example, it was blue when it was sacrificed, the damage will come from a blue source."
    );
    for blue in [false, true] {
        let mut t = TestGame::new(2);
        let phoenix = t.battlefield(P0, "Shard Phoenix");
        let walker = t.battlefield(P1, "Kor Firewalker");
        if blue {
            cast_card(&mut t, P0, "Cerulean Wisps", &[obj(phoenix)]).unwrap();
            t.resolve_all();
            assert!(t.obj_now(phoenix).chars.colors.contains(Color::Blue));
            assert!(!t.obj_now(phoenix).chars.colors.contains(Color::Red));
        }
        t.activate(P0, phoenix, 0, &[]).unwrap();
        t.resolve_all();
        let want = if blue { "graveyard" } else { "battlefield" };
        assert_eq!(where_is(&t, walker), want, "blue: {blue}");
    }
}

#[test]
fn shard_phoenix_returns_any_time_during_your_upkeep() {
    cr!("602.5b", "503.1");
    ruling!(
        "Shard Phoenix",
        "You can return Shard Phoenix any time during your upkeep; this is not a beginning of upkeep triggered ability."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let phoenix = t.graveyard(P0, "Shard Phoenix");
    t.set_step(P0, Step::Upkeep);
    // Something resolves during the upkeep first; it can still be activated.
    cast_card(&mut t, P0, "Lightning Bolt", &[pl(P1)]).unwrap();
    t.resolve_all();
    assert!(can_activate(&mut t, P0, phoenix));
    t.set_step(P0, Step::Draw);
    assert!(!can_activate(&mut t, P0, phoenix));
}

#[test]
fn aunties_snitch_checks_the_graveyard_on_trigger_and_resolution() {
    cr!("603.4", "400.7");
    ruling!(
        "Auntie's Snitch",
        "The last ability will trigger only if Auntie's Snitch is in your graveyard at the moment a Goblin or Rogue deals combat damage to a player."
    );
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        let snitch = t.graveyard(P0, "Auntie's Snitch");
        let gob = t.battlefield(P0, "Raging Goblin");
        attack_with(&mut t, &[(gob, pl(P1))]);
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if leave {
            blink_in_graveyard(&mut t, snitch);
        }
        t.resolve_all();
        let want = if leave { "graveyard" } else { "hand" };
        assert_eq!(where_is(&t, snitch), want);
    }
    // A non-Goblin, non-Rogue: nothing.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Auntie's Snitch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.advance_to(P0, Step::CombatDamage);
        t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn brood_of_cockroaches_loses_life_and_returns_as_one_mandatory_effect() {
    cr!("603.7", "608.2c");
    ruling!(
        "Brood of Cockroaches",
        "The life loss and the return to hand both happen at the beginning of the end step as a single effect."
    );
    ruling!("Brood of Cockroaches", "The loss of life is not optional.");
    supported("Brood of Cockroaches");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let brood = t.battlefield(P0, "Brood of Cockroaches");
    destroy(&mut t, brood);
    t.resolve_all();
    assert_eq!(where_is(&t, brood), "graveyard");
    assert_eq!(t.life(P0), 20);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(where_is(&t, brood), "hand");
}

#[test]
fn crown_of_skemfar_bonus_follows_the_number_of_elves() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Crown of Skemfar",
        "The power/toughness bonus given by Crown of Skemfar will change as the number of Elves you control changes."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Crown of Skemfar", obj(bears));
    assert_eq!(t.pt(bears), (2, 2));
    t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    let e2 = t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    destroy(&mut t, e2);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn blood_speaker_returns_however_it_got_into_the_graveyard() {
    cr!("603.6a", "113.6");
    ruling!(
        "Blood Speaker",
        "The return-to-hand triggered ability triggers while Blood Speaker is in your graveyard, no matter how it got there."
    );
    for how in [MoveCause::Discard, MoveCause::Mill, MoveCause::Destroy] {
        let mut t = TestGame::new(2);
        let bs = t.hand(P0, "Blood Speaker");
        let bs = t
            .g
            .move_object(bs, Zone::Graveyard(P0), how, Some(P0))
            .unwrap();
        flush(&mut t);
        t.enter(P0, "Abyssal Persecutor");
        flush(&mut t);
        t.resolve_all();
        assert_eq!(where_is(&t, bs), "hand", "{how:?}");
    }
}

#[test]
fn reach_of_branches_triggers_on_any_forest() {
    cr!("205.3i", "305.6");
    ruling!(
        "Reach of Branches",
        "The triggered ability will trigger whenever any land with the subtype Forest enters under your control, not just one named Forest."
    );
    let mut t = TestGame::new(2);
    let reach = t.graveyard(P0, "Reach of Branches");
    t.enter(P0, "Breeding Pool");
    flush(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(where_is(&t, reach), "hand");
}

#[test]
fn skarrgan_firebird_counts_damage_from_any_source() {
    cr!("602.5b");
    ruling!(
        "Skarrgan Firebird",
        "You can activate Skarrgan Firebird's last ability if an opponent was dealt damage by any source, even if you didn't control that source."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bird = t.graveyard(P0, "Skarrgan Firebird");
    assert!(!can_activate(&mut t, P0, bird));
    let theirs = t.battlefield(P1, "Mountain");
    damage(&mut t, theirs, pl(P1), 1);
    assert!(can_activate(&mut t, P0, bird));
}

#[test]
fn eternal_dragon_plainscycling_finds_any_plains_or_nothing() {
    cr!("702.29e", "701.23b");
    ruling!(
        "Eternal Dragon",
        "You can choose to find any card with the Plains land type, including nonbasic lands. You can also choose not to find a card, even if there is a Plains card in your library."
    );
    for find in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 2);
        let fountain = t.library_top(P0, "Hallowed Fountain");
        let dragon = t.hand(P0, "Eternal Dragon");
        if find {
            t.answer_choose(P0, &[obj(fountain)]);
        } else {
            t.answer_choose(P0, &[]);
        }
        cycle(&mut t, P0, dragon, 0).unwrap();
        t.resolve_all();
        let want = if find { "hand" } else { "library" };
        assert_eq!(where_is(&t, fountain), want);
    }
}

#[test]
fn glistening_oil_with_an_illegal_target_doesnt_enter_or_trigger() {
    cr!("608.3b", "303.4a");
    ruling!(
        "Glistening Oil",
        "If that creature is an illegal target when Glistening Oil tries to resolve, Glistening Oil won't resolve and will be put into its owner's graveyard. It won't enter and its last ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let oil = cast_card(&mut t, P0, "Glistening Oil", &[obj(bears)]).unwrap();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(where_is(&t, oil), "graveyard");
    assert_eq!(t.stack_len(), 0);
}
