//! Rulings batch P106 — mana artifacts' other abilities: becoming (or stopping being)
//! creatures, Copy Land's copiable values, counter thresholds checked on resolution,
//! "choose a color/type" with nothing chosen, commander color identity, multiplayer
//! triggers, and the city's blessing (CR 506.4, 613, 707.2, 603.4, 702.131, 903.4).

use crate::r_p106_common::*;
use crate::r_s01_common::attack_with;
use crate::r_s02_common::{can_attack, can_cast};
use crate::r_s06_common::{attach_new, give_control};
use crate::r_s10_common::attacking;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const TRIAD: &str = "The Warring Triad";

/// Puts `n` Islands into `p`'s graveyard.
fn fill_graveyard(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.graveyard(p, "Island");
    }
    t.g.recompute();
}

#[test]
fn the_warring_triad_as_a_noncreature_artifact() {
    cr!("205.3d", "122.1", "113.6");
    ruling!(
        "The Warring Triad",
        "If The Warring Triad stops being a creature, it loses the type creature and the creature type God. It continues to be a legendary artifact."
    );
    ruling!(
        "The Warring Triad",
        "Counters put on The Warring Triad remain on it while it's not a creature, even if they have no effect."
    );
    ruling!(
        "The Warring Triad",
        "The Warring Triad's abilities function while it's on the battlefield, regardless of whether it's a creature."
    );
    let mut t = TestGame::new(2);
    let triad = t.battlefield(P0, TRIAD);
    t.g.recompute();
    let o = t.obj_now(triad);
    assert!(!o.is(CardType::Creature));
    assert!(!o.chars.has_subtype("God"));
    assert!(o.is(CardType::Artifact));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    // Its mana ability works while it isn't a creature.
    act(&mut t, P0, triad, "Mill a card", &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 1);
    assert_eq!(t.graveyard_size(P0), 1);
    // +1/+1 counters stay on it, and count once it's a creature.
    put_counters(&mut t, triad, counters::PLUS1, 2);
    assert_eq!(t.counters(triad, counters::PLUS1), 2);
    fill_graveyard(&mut t, P0, 7);
    assert!(is_creature(&t, triad));
    assert!(t.obj_now(triad).chars.has_subtype("God"));
    assert_eq!(t.pt(triad), (7, 7));
}

#[test]
fn the_warring_triad_leaves_combat_when_it_stops_being_a_creature() {
    cr!("506.4");
    ruling!(
        "The Warring Triad",
        "If The Warring Triad is attacking or blocking and it stops being a creature, it will be removed from combat. It won't rejoin combat if it resumes being a creature later during that combat."
    );
    let mut t = TestGame::new(2);
    let triad = t.battlefield(P0, TRIAD);
    fill_graveyard(&mut t, P0, 8);
    assert!(is_creature(&t, triad));
    attack_with(&mut t, &[(triad, Entity::Player(P1))]);
    assert!(attacking(&t, triad));
    // A card leaves the graveyard: seven cards, no longer a creature.
    let card = t.g.player(P0).graveyard[0];
    t.g.move_object(card, Zone::Exile, MoveCause::Effect, None);
    t.g.recompute();
    t.settle();
    assert!(!is_creature(&t, triad));
    assert!(!attacking(&t, triad));
    // Eight again: a creature, but not attacking.
    fill_graveyard(&mut t, P0, 1);
    t.settle();
    assert!(is_creature(&t, triad));
    assert!(!attacking(&t, triad));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn the_warring_triad_losing_all_abilities_still_isnt_a_creature() {
    cr!("613.1d", "613.1f", "613.6");
    ruling!(
        "The Warring Triad",
        "If an effect causes The Warring Triad to lose all abilities, its ability that causes it to stop being a creature still applies if appropriate."
    );
    let mut t = TestGame::new(2);
    let triad = t.battlefield(P0, TRIAD);
    crate::r_p124_common::lose_abilities(&mut t, triad);
    assert!(!is_creature(&t, triad));
    assert!(!t
        .obj_now(triad)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("Mill a card")));
}

#[test]
fn izzet_keyrune_animated_another_way_still_loots() {
    cr!("603.2", "510.2");
    ruling!(
        "Izzet Keyrune",
        "If Izzet Keyrune becomes a creature in a way other than its own activated ability, its last ability will still work."
    );
    // March of the Machines makes it a 3/3 artifact creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "March of the Machines");
    let k = t.battlefield(P0, "Izzet Keyrune");
    assert!(is_creature(&t, k));
    t.library_top(P0, "Island");
    t.answer_yes(P0, true);
    t.attack(&[(k, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.graveyard_size(P0), 1, "drew, then discarded");
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn orzhov_keyrune_activated_twice_has_lifelink_once() {
    cr!("702.15f", "702.15b");
    ruling!(
        "Orzhov Keyrune",
        "Multiple instances of lifelink on the same creature are redundant. Activating the last ability multiple times won't cause you to gain more life."
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Orzhov Keyrune");
    for _ in 0..2 {
        mana(&mut t, P0, ManaType::W, 1);
        mana(&mut t, P0, ManaType::B, 1);
        act(&mut t, P0, k, "Thrull", &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.pt(k), (1, 4));
    t.attack(&[(k, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn firdoch_core_overwrites_earlier_pt_setting_and_is_overwritten_later() {
    cr!("613.4b", "613.7");
    ruling!(
        "Firdoch Core",
        "Firdoch Core's last ability overwrites all previous effects that set its base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after the ability resolves will overwrite this effect."
    );
    // March of the Machines makes it a 3/3 (its mana value); its ability then makes it a
    // 4/4; Turn to Frog afterwards makes it a 1/1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "March of the Machines");
    let core = t.battlefield(P0, "Firdoch Core");
    assert_eq!(t.pt(core), (3, 3));
    mana(&mut t, P0, ManaType::C, 4);
    act(&mut t, P0, core, "4/4", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(core), (4, 4));
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 1);
    cast_from_hand(&mut t, P0, "Turn to Frog", &[obj(core)]);
    t.resolve_all();
    assert_eq!(t.pt(core), (1, 1));
}

#[test]
fn foriysian_totem_that_cant_block_blocks_nothing() {
    cr!("509.1a", "509.1b");
    ruling!(
        "Foriysian Totem",
        "If an effect says Foriysian Totem can't block, then it can't block any creatures."
    );
    let mut t = TestGame::new(2);
    let totem = t.battlefield(P0, "Foriysian Totem");
    let atk: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    t.advance_to(P1, Step::Upkeep);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 4);
    act(&mut t, P0, totem, "4/4", &[]).unwrap();
    t.resolve_all();
    assert!(is_creature(&t, totem));
    attack_with(
        &mut t,
        &atk.iter()
            .map(|a| (*a, Entity::Player(P0)))
            .collect::<Vec<_>>(),
    );
    use crate::r_s21_common::legal_blocks;
    // It could block both attackers.
    assert!(legal_blocks(
        &mut t,
        P0,
        &[(totem, atk[0]), (totem, atk[1])]
    ));
    // Panic ("Cast this spell only during combat before blockers are declared. Target
    // creature can't block this turn."): now it can block neither.
    mana(&mut t, P1, ManaType::R, 1);
    let panic = t.hand(P1, "Panic");
    t.cast_with(P1, panic, &[obj(totem)]).unwrap();
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P0, &[(totem, atk[0])]));
    assert!(!legal_blocks(&mut t, P0, &[(totem, atk[1])]));
    assert!(!legal_blocks(
        &mut t,
        P0,
        &[(totem, atk[0]), (totem, atk[1])]
    ));
    assert!(legal_blocks(&mut t, P0, &[]));
}

#[test]
fn fountain_of_ichor_animated_while_summoning_sick() {
    cr!("302.6");
    ruling!(
        "Fountain of Ichor",
        "If you turn Fountain of Ichor into a creature but haven't controlled it continuously since your most recent turn began, you won't be able to activate its mana ability or attack with it."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let f = crate::r_s20_common::entered_this_turn(&mut t, P0, "Fountain of Ichor");
    mana(&mut t, P0, ManaType::C, 3);
    act(&mut t, P0, f, "3/3", &[]).unwrap();
    t.resolve_all();
    assert!(is_creature(&t, f));
    assert!(!tap_for_mana(&mut t, P0, f, "Add one mana"));
    t.set_step(P0, Step::DeclareAttackers);
    assert!(can_attack(&mut t, bears));
    assert!(!can_attack(&mut t, f));
}

/// P0's Copy Land entering as a copy of `of` (or of nothing).
fn copy_land(t: &mut TestGame, of: Option<ObjectId>) -> ObjectId {
    match of {
        Some(o) => {
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[obj(o)]);
        }
        None => {
            t.answer_yes(P0, false);
            t.answer_choose(P0, &[]);
        }
    }
    let cl = t.enter(P0, "Copy Land");
    t.settle();
    cl
}

#[test]
fn copy_land_copies_only_the_printed_land() {
    cr!("707.2", "707.9b", "122.1");
    ruling!(
        "Copy Land",
        "Copy Land copies exactly what was printed on the original permanent and nothing more, with the listed exception (unless that permanent is itself copying something or is a token; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    // A tapped Forest with counters and an Aura.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    t.g.tap(forest);
    put_counters(&mut t, forest, counters::PLUS1, 2);
    attach_new(&mut t, P1, "Wild Growth", forest);
    let cl = copy_land(&mut t, Some(forest));
    let o = t.obj_now(cl);
    assert_eq!(o.chars.name, "Forest");
    assert!(o.chars.has_subtype("Forest"));
    assert!(o.is(CardType::Land) && o.is(CardType::Enchantment));
    assert!(!o.tapped);
    assert_eq!(t.counters(cl, counters::PLUS1), 0);
    assert!(!t
        .g
        .permanents()
        .any(|p| p.attached_to == Some(obj(t.g.current(cl)))));
    // An animated Mutavault (a non-copy effect): the copy isn't a creature.
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P1, "Mutavault");
    mana(&mut t, P1, ManaType::C, 1);
    act(&mut t, P1, vault, "2/2", &[]).unwrap();
    t.resolve_all();
    assert!(is_creature(&t, vault));
    let cl = copy_land(&mut t, Some(vault));
    assert_eq!(t.obj_now(cl).chars.name, "Mutavault");
    assert!(!is_creature(&t, cl));
    assert!(t.obj_now(cl).is(CardType::Enchantment));
}

#[test]
fn copy_land_cant_copy_a_land_entering_with_it() {
    cr!("707.5", "614.12");
    ruling!(
        "Copy Land",
        "If Copy Land somehow enters the battlefield at the same time as another permanent, Copy Land can't become a copy of that permanent. You may choose only a permanent that's already on the battlefield."
    );
    let mut t = TestGame::new(2);
    let island = t.battlefield(P1, "Island");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(island)]);
    let from = t.asked().len();
    let ids = enter_together(&mut t, &[(P0, "Copy Land"), (P0, "Forest")]);
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&obj(island)));
    for id in &ids {
        assert!(!offered.contains(&obj(*id)));
    }
    assert_eq!(t.named_on_battlefield("Island").len(), 2);
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
}

#[test]
fn copy_land_copying_a_token_or_a_copy() {
    cr!("707.2", "707.3", "111.3");
    ruling!(
        "Copy Land",
        "If the chosen permanent is a token, Copy Land copies the original characteristics of that token as stated by the effect that created the token, with the listed exception. Copy Land does not become a token."
    );
    ruling!(
        "Copy Land",
        "If the chosen permanent is copying something else (for example, if the chosen permanent is another Copy Land), then your Copy Land enters the battlefield as whatever the chosen permanent copied, with the listed exception."
    );
    // A token copy of a Forest, tapped: Copy Land is an untapped Forest enchantment, not a
    // token.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let tok = crate::r_s17_common::token_copy(&mut t, P1, forest)[0];
    t.g.tap(tok);
    let cl = copy_land(&mut t, Some(tok));
    let o = t.obj_now(cl);
    assert_eq!(o.chars.name, "Forest");
    assert!(!o.is_token());
    assert!(!o.tapped);
    assert!(o.is(CardType::Land) && o.is(CardType::Enchantment));
    // A second Copy Land copying the first: a Forest enchantment (copying Copy Land itself
    // would make it a plain enchantment named Copy Land).
    let cl2 = copy_land(&mut t, Some(cl));
    let o = t.obj_now(cl2);
    assert_eq!(o.chars.name, "Forest");
    assert!(o.is(CardType::Land) && o.is(CardType::Enchantment));
    assert!(tap_for_mana(&mut t, P0, cl2, "Add {G}"));
}

#[test]
fn copy_land_copying_a_permanent_with_x_in_its_cost_has_x_zero() {
    cr!("707.2", "202.3e", "107.3g");
    ruling!(
        "Copy Land",
        "In the unusual case where the chosen permanent has {X} in its mana cost, X is 0."
    );
    // Chimeric Mass ({X}), cast with X = 3, is a Forest land while Song of the Dryads
    // ("Enchanted permanent is a colorless Forest land.") enchants it, so Copy Land may copy
    // it. The copy is a Chimeric Mass (the Aura's type change isn't copied) that's also an
    // enchantment, with X = 0: mana value 0, and its "This artifact enters with X charge
    // counters on it" puts none on it.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::C, 3);
    let mass = t.hand(P0, "Chimeric Mass");
    t.cast(P0, mass).x(3).go();
    t.resolve_all();
    let mass = t.named_on_battlefield("Chimeric Mass")[0];
    assert_eq!(t.counters(mass, counters::CHARGE), 3);
    attach_new(&mut t, P0, "Song of the Dryads", mass);
    t.g.recompute();
    assert!(t.obj_now(mass).is(CardType::Land));
    let cl = copy_land(&mut t, Some(mass));
    t.g.recompute();
    let o = t.obj_now(cl);
    assert_eq!(o.chars.name, "Chimeric Mass");
    assert!(o.is(CardType::Artifact) && o.is(CardType::Enchantment));
    assert!(!o.is(CardType::Land));
    assert_eq!(t.g.mana_value_of(t.g.current(cl)), 0);
    assert_eq!(t.counters(cl, counters::CHARGE), 0);
}

#[test]
fn midnight_clocks_twelfth_counter_trigger() {
    cr!("603.2", "122.6");
    ruling!(
        "Midnight Clock",
        "Midnight Clock's last ability triggers after one or more counters are put onto it if it had fewer than twelve counters on it before those counters were put on it and it has twelve or more counters on it after."
    );
    // From ten to thirteen at once: it triggers.
    let mut t = TestGame::new(2);
    let clock = t.battlefield(P0, "Midnight Clock");
    put_counters(&mut t, clock, "hour", 10);
    assert_eq!(t.stack_len(), 0);
    put_counters(&mut t, clock, "hour", 3);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_exile("Midnight Clock"));
    assert_eq!(t.hand_size(P0), 7);
    // Already at twelve: another counter doesn't trigger it.
    let mut t = TestGame::new(2);
    let clock = t.battlefield(P0, "Midnight Clock");
    t.g.objects[clock.0 as usize]
        .counters
        .insert("hour".into(), 12);
    put_counters(&mut t, clock, "hour", 1);
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(clock));
}

#[test]
fn midnight_clock_ticks_in_every_upkeep_once_per_team() {
    cr!("503.1a", "805.4", "805.4d");
    ruling!(
        "Midnight Clock",
        "Midnight Clock's first triggered ability triggers at the beginning of each upkeep, not just your upkeep. In a Two-Headed Giant game, an ability that triggers at the beginning of each upkeep rather than each player's upkeep triggers only once during each team's upkeep."
    );
    let mut t = TestGame::new(2);
    let clock = t.battlefield(P0, "Midnight Clock");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(clock, "hour"), 1);
    // Two-Headed Giant: once in the opposing team's upkeep.
    let mut t = crate::r_p108_common::two_headed_giant();
    let clock = t.battlefield(P0, "Midnight Clock");
    t.advance_to(P2, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(clock, "hour"), 1);
}

#[test]
fn midnight_clock_gone_before_its_last_ability_resolves_isnt_exiled() {
    cr!("608.2h", "400.7");
    ruling!(
        "Midnight Clock",
        "If Midnight Clock leaves the battlefield while its last ability is on the stack, it won't be exiled."
    );
    let mut t = TestGame::new(2);
    let clock = t.battlefield(P0, "Midnight Clock");
    put_counters(&mut t, clock, "hour", 11);
    put_counters(&mut t, clock, "hour", 1);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, clock);
    assert!(t.in_graveyard(P0, "Midnight Clock"));
    t.resolve_all();
    // Shuffled into the library with the graveyard; not exiled.
    assert!(!t.in_exile("Midnight Clock"));
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn strixhaven_stadium_checks_ten_counters_only_as_its_last_ability_resolves() {
    cr!("608.2c");
    ruling!(
        "Strixhaven Stadium",
        "The check for ten or more point counters happens only as the last ability is resolving. Notably, if you add the tenth point counter some other way, no one will lose the game . . . yet."
    );
    let mut t = TestGame::new(2);
    let st = t.battlefield(P0, "Strixhaven Stadium");
    put_counters(&mut t, st, "point", 9);
    assert!(tap_for_mana(&mut t, P0, st, "Add {C}"));
    t.resolve_all();
    assert_eq!(t.counters(st, "point"), 10);
    assert!(!t.has_lost(P1));
}

#[test]
fn strixhaven_stadium_triggers_once_per_creature() {
    cr!("510.2", "603.2c", "603.3b");
    ruling!(
        "Strixhaven Stadium",
        "If multiple creatures deal combat damage to you or an opponent at the same time, the appropriate ability of Strixhaven Stadium will trigger that many times. You can put these abilities on the stack in any order. The last ability put on the stack will be the first to resolve."
    );
    // Two of P0's creatures hit P1: two counters.
    let mut t = TestGame::new(2);
    let st = t.battlefield(P0, "Strixhaven Stadium");
    put_counters(&mut t, st, "point", 5);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.counters(st, "point"), 7);
    // Two of P1's creatures hit P0: two counters removed.
    let c = t.battlefield(P1, "Grizzly Bears");
    let d = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    t.attack(&[(c, Entity::Player(P0)), (d, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(t.counters(st, "point"), 5);
}

#[test]
fn strixhaven_stadium_in_multiplayer_makes_one_player_lose() {
    cr!("510.2", "603.2c", "608.2c");
    ruling!(
        "Strixhaven Stadium",
        "In a multiplayer game, if creatures you control deal combat damage to multiple opponents at the same time, the last ability will trigger multiple times. However, because the counters are removed, it is difficult (but not impossible) to make multiple players lose the game from this ability in the same combat phase."
    );
    let mut t = TestGame::new(3);
    let st = t.battlefield(P0, "Strixhaven Stadium");
    put_counters(&mut t, st, "point", 9);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P2))], &[]);
    t.resolve_all();
    let lost = [P1, P2].iter().filter(|p| t.has_lost(**p)).count();
    assert_eq!(lost, 1);
    assert_eq!(t.counters(st, "point"), 1);
}

#[test]
fn replicating_ring_checks_eight_counters_only_as_its_trigger_resolves() {
    cr!("608.2c");
    ruling!(
        "Replicating Ring",
        "The check of whether Replicating Ring has eight or more night counters on it happens only as part of the resolution of the triggered ability. If Replicating Ring picks up its eighth night counter at another time, you won't immediately create Replicated Rings. You'll have to wait until the next time the triggered ability resolves."
    );
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Replicating Ring");
    put_counters(&mut t, ring, "night", 8);
    t.resolve_all();
    assert!(t.named_on_battlefield("Replicated Ring").is_empty());
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Replicated Ring").len(), 8);
    assert_eq!(t.counters(ring, "night"), 0);
}

#[test]
fn progenitors_icon_without_a_chosen_type_gives_no_flash() {
    cr!("607.2d", "702.8a");
    ruling!(
        "Progenitor's Icon",
        "If Progenitor's Icon is somehow on the battlefield without a chosen type, its last ability won't do anything."
    );
    for chosen in [false, true] {
        let mut t = TestGame::new(2);
        let icon = if chosen {
            crate::r_p125_common::choose_creature_type(&mut t, P0, "Bear");
            let i = t.enter(P0, "Progenitor's Icon");
            t.settle();
            i
        } else {
            t.battlefield(P0, "Progenitor's Icon")
        };
        act(&mut t, P0, icon, "as though it had flash", &[]).unwrap();
        t.resolve_all();
        t.set_step(P0, Step::BeginningOfCombat);
        mana(&mut t, P0, ManaType::G, 1);
        mana(&mut t, P0, ManaType::C, 1);
        let bears = t.hand(P0, "Grizzly Bears");
        assert_eq!(
            can_cast(&mut t, P0, bears, CastMethod::Normal),
            chosen,
            "{chosen}"
        );
    }
}

#[test]
fn chosen_color_artifacts_with_no_chosen_color() {
    cr!("607.2d", "106.5");
    ruling!(
        "Heraldic Banner",
        "If there is somehow no chosen color, Heraldic Banner's static ability has no effect, and its mana ability produces no mana if you activate it. Colorless creatures won't get +1/+0, and you won't add {C}."
    );
    ruling!(
        "Paradise Plume",
        "If there's somehow no color chosen for Paradise Plume, its middle ability can't trigger and its last ability produces no mana. The former doesn't trigger when colorless spells are cast and the latter doesn't produce colorless mana."
    );
    let mut t = TestGame::new(2);
    let banner = t.battlefield(P0, "Heraldic Banner");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let thopter = t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(thopter), (0, 2));
    assert!(!tap_for_mana(&mut t, P0, banner, "Add one mana"));
    assert_eq!(pool_total(&t, P0), 0);
    // Paradise Plume: casting a colorless spell doesn't trigger it; no mana either.
    let mut t = TestGame::new(2);
    let plume = t.battlefield(P0, "Paradise Plume");
    cast_from_hand(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!tap_for_mana(&mut t, P0, plume, "Add one mana"));
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn commander_identity_mana_without_a_commander_or_with_a_colorless_one() {
    cr!("903.4", "903.4f", "106.5");
    ruling!(
        "Arcane Signet",
        "If you don't have a commander, Arcane Signet's ability produces no mana."
    );
    ruling!(
        "Commander's Sphere",
        "If you don't have a commander, Commander's Sphere's first ability produces no mana."
    );
    ruling!(
        "Commander's Sphere",
        "If your commander is a card that has no colors in its color identity, Commander's Sphere's first ability produces no mana. It doesn't produce {C}."
    );
    for name in ["Arcane Signet", "Commander's Sphere"] {
        let mut t = TestGame::new(2);
        let rock = t.battlefield(P0, name);
        assert!(!tap_for_mana(&mut t, P0, rock, "Add one mana"), "{name}");
        assert_eq!(pool_total(&t, P0), 0, "{name}");
    }
    // A Commander game with Kozilek, Butcher of Truth (colorless) as P0's commander.
    let mut t = crate::r_p114_common::commander_game_n(2);
    crate::r_p114_common::commander_on_battlefield(&mut t, P0, "Kozilek, Butcher of Truth");
    let sphere = t.battlefield(P0, "Commander's Sphere");
    assert!(!tap_for_mana(&mut t, P0, sphere, "Add one mana"));
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 0);
}

#[test]
fn bonders_ornament_draws_one_card_per_player() {
    cr!("608.2c");
    ruling!(
        "Bonder's Ornament",
        "If one player controls more than one Bonder’s Ornament, that player draws just one card while resolving its second ability."
    );
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Bonder's Ornament");
    t.battlefield(P0, "Bonder's Ornament");
    t.battlefield(P1, "Bonder's Ornament");
    mana(&mut t, P0, ManaType::C, 4);
    act(&mut t, P0, o, "Each player who controls", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn bounty_boards_each_trigger_for_any_bounty_counter() {
    cr!("603.2", "603.10a");
    ruling!(
        "Bounty Board",
        "Bounty Board’s last ability looks for any bounty counter on any creature, not just one from that Bounty Board. For example, say players A and B each control a Bounty Board and player C controls a creature with a bounty counter on it (no matter how that bounty counter got there). When that creature dies, the ability each Bounty Board will trigger. Each of those abilities will cause players A and B to draw a card and gain 2 life, for a total of two cards and 4 life for each of them. Player C is left only with a creature to mourn and revenge on their mind."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Bounty Board");
    t.battlefield(P1, "Bounty Board");
    let bears = t.battlefield(P2, "Grizzly Bears");
    put_counters(&mut t, bears, "bounty", 1);
    destroy(&mut t, bears);
    t.resolve_all();
    for p in [P0, P1] {
        assert_eq!(t.hand_size(p), 2, "{p:?}");
        assert_eq!(t.life(p), 24, "{p:?}");
    }
    assert_eq!(t.hand_size(P2), 0);
    assert_eq!(t.life(P2), 20);
}

#[test]
fn blitzball_counts_anyones_legendary_creature() {
    cr!("602.1b", "510.2");
    ruling!(
        "Blitzball",
        "Blitzball's last ability doesn't care who controls the legendary creature that dealt combat damage to an opponent this turn."
    );
    // P2's Isamaru hits P1 (P0's opponent) in P2's turn.
    let mut t = TestGame::new(3);
    let ball = t.battlefield(P0, "Blitzball");
    let isamaru = t.battlefield(P2, "Isamaru, Hound of Konda");
    t.advance_to(P2, Step::PrecombatMain);
    assert!(!can_activate_containing(&mut t, P0, ball, "Draw two"));
    t.attack(&[(isamaru, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert!(can_activate_containing(&mut t, P0, ball, "Draw two"));
    act(&mut t, P0, ball, "Draw two", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn orazca_relic_gets_the_blessing_before_the_legend_rule() {
    cr!("702.131b", "704.5j");
    ruling!(
        "Orazca Relic",
        "If your tenth permanent enters the battlefield and then a permanent leaves the battlefield immediately afterwards (most likely due to the “Legend Rule” or due to being a creature with 0 toughness), you get the city's blessing before it leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Orazca Relic");
    t.lands(P0, "Forest", 7);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.g.recompute();
    t.settle();
    assert!(!t.g.player(P0).has_citys_blessing);
    // A second Isamaru: ten permanents, then the legend rule leaves nine.
    t.enter(P0, "Isamaru, Hound of Konda");
    t.settle();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
    assert!(t.g.player(P0).has_citys_blessing);
    assert!(can_activate_containing(&mut t, P0, relic, "gain 3 life"));
}

#[test]
fn coalition_relic_triggers_only_in_the_first_main_phase() {
    cr!("505.1a", "500.8");
    ruling!(
        "Coalition Relic",
        "Only the first main phase each turn is considered a precombat main phase, even if additional main phases or combat phases are created."
    );
    // Relentless Assault adds a combat and a main phase after this (first) main phase.
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Coalition Relic");
    put_counters(&mut t, relic, counters::CHARGE, 2);
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 2);
    cast_from_hand(&mut t, P0, "Relentless Assault", &[]);
    t.resolve_all();
    let m = mark(&t);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let mains = events_since(&t, m)
        .iter()
        .filter(|e| matches!(e, Event::StepBegan { step, .. } if step.is_main()))
        .count();
    assert_eq!(mains, 2, "the additional main phase and the last one");
    assert_eq!(t.counters(relic, counters::CHARGE), 2);
    // The next turn's first main phase: it triggers.
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.counters(relic, counters::CHARGE), 0);
    assert_eq!(pool_total(&t, P0), 2);
}

#[test]
fn ancient_cornucopia_triggers_until_you_gain_life_once() {
    cr!("603.2h", "603.5");
    ruling!(
        "Ancient Cornucopia",
        "Once you choose to gain life using Ancient Cornucopia’s triggered ability, that ability won’t trigger again that turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancient Cornucopia");
    // Declined: it triggers again.
    mana(&mut t, P0, ManaType::R, 1);
    t.answer_yes(P0, false);
    cast_from_hand(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    mana(&mut t, P0, ManaType::R, 1);
    t.answer_yes(P0, true);
    cast_from_hand(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Taken: no more triggers this turn.
    mana(&mut t, P0, ManaType::R, 1);
    cast_from_hand(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn chromatic_lantern_adds_an_ability_and_keeps_the_rest() {
    cr!("613.1f");
    ruling!(
        "Chromatic Lantern",
        "Lands you control won't lose any other abilities they had. They also won't gain or lose any land types."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chromatic Lantern");
    let forest = t.battlefield(P0, "Forest");
    let vault = t.battlefield(P0, "Mutavault");
    t.g.recompute();
    let f = t.obj_now(forest);
    assert_eq!(f.chars.subtypes.len(), 1);
    assert!(f.chars.has_subtype("Forest"));
    assert!(f.chars.abilities.iter().any(|a| a.text.contains("Add {G}")));
    assert!(f
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("any color")));
    let v = t.obj_now(vault);
    assert!(v.chars.subtypes.is_empty());
    assert!(v.chars.abilities.iter().any(|a| a.text.contains("2/2")));
    assert!(tap_for_mana(&mut t, P0, forest, "any color"));
}

#[test]
fn benders_waterskin_untaps_in_other_players_untap_steps_despite_doesnt_untap() {
    cr!("502.3");
    ruling!(
        "Bender's Waterskin",
        "Some effects state that an artifact doesn't untap during its controller's untap step. These effects won't apply and stop Bender's Waterskin from untapping during another player's untap step."
    );
    let mut t = TestGame::new(2);
    let skin = t.battlefield(P0, "Bender's Waterskin");
    attach_new(&mut t, P1, "Encrust", skin);
    t.g.tap(skin);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(skin).tapped, "untapped in P1's untap step");
    t.g.tap(skin);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(skin).tapped, "not in its controller's");
}

fn gain_life(t: &mut TestGame, p: PlayerId, n: i32) {
    use mtg_engine::ability::{Effect, PlayerRef, Value};
    t.g.recompute();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::GainLife {
            who: PlayerRef::You,
            n: Value::Const(n),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}

#[test]
fn phial_of_galadriel_doubles_life_from_setting_the_life_total() {
    cr!("119.5", "614.1a");
    ruling!(
        "Phial of Galadriel",
        "If an effect would set your life total to a specific number that's higher than your current life total, that effect causes you to gain life equal to the difference. If your life total is 5 or less, Phial of Galadriel will then double the amount of life that effect would cause you to gain. For example, if you have 3 life and an effect says that your life total \"becomes 10,\" you would gain 7 life which Phial doubles to 14, so your life total will actually become 17."
    );
    // At 3 life, Blessed Wind ("Target player's life total becomes 20"): 17 doubled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phial of Galadriel");
    t.g.players[0].life = 3;
    mana(&mut t, P0, ManaType::W, 2);
    mana(&mut t, P0, ManaType::C, 7);
    cast_from_hand(&mut t, P0, "Blessed Wind", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 37);
}

#[test]
fn two_phials_of_galadriel_draw_three() {
    cr!("614.5", "616.1", "121.6");
    ruling!(
        "Phial of Galadriel",
        "If you somehow control more than one Phial of Galadriel, each one will effectively add one card. For example, if you control two Phials of Galadriel and would draw a card while you have no cards in hand, the effect of one Phial of Galadriel will replace the event \"draw a card\" with \"draw two cards.\" The effect of the other Phial of Galadriel will replace the drawing of the first of those two cards with \"draw two cards.\" In total, you would draw three cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Phial of Galadriel");
    t.battlefield(P0, "Phial of Galadriel");
    assert_eq!(t.hand_size(P0), 0);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn phial_of_galadriel_ignores_a_teammates_life_gain() {
    cr!("810.9", "810.9a", "614.1a");
    ruling!(
        "Phial of Galadriel",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Phial of Galadriel's second ability to apply, even though it will cause your team's life total to increase."
    );
    let mut t = crate::r_p108_common::two_headed_giant();
    t.battlefield(P0, "Phial of Galadriel");
    crate::r_p096_common::set_team_life(&mut t, P0, 5);
    gain_life(&mut t, P1, 2);
    assert_eq!(t.life(P0), 7);
    crate::r_p096_common::set_team_life(&mut t, P0, 5);
    gain_life(&mut t, P0, 2);
    assert_eq!(t.life(P0), 9);
}

#[test]
fn the_celestus_still_discards_when_the_draw_is_replaced() {
    cr!("121.6", "614.6", "608.2c");
    ruling!(
        "The Celestus",
        "If you choose to draw a card with The Celestus but some effect replaces that card draw with something else, you will still have to discard a card."
    );
    // Day becomes night; P0 dredges Golgari Thug instead of drawing.
    let mut t = TestGame::new(2);
    let c = t.enter(P0, "The Celestus");
    t.settle();
    let thug = t.graveyard(P0, "Golgari Thug");
    t.hand(P0, "Island");
    mana(&mut t, P0, ManaType::C, 3);
    act(&mut t, P0, c, "it becomes night", &[]).unwrap();
    t.answer_yes(P0, true); // draw
    t.answer_yes(P0, true); // dredge instead
    let m = mark(&t);
    let from = t.asked().len();
    t.resolve_all();
    let drew = events_since(&t, m)
        .iter()
        .any(|e| matches!(e, Event::Drew { player, .. } if *player == P0));
    assert!(!drew, "the draw was replaced\n{}", t.dump_log());
    assert_eq!(t.zone(thug), Zone::Hand(P0), "{}", t.dump_log());
    let discarded = events_since(&t, m)
        .iter()
        .filter(|e| matches!(e, Event::Discarded { player, .. } if *player == P0))
        .count();
    assert_eq!(discarded, 1, "{:?}", &t.asked()[from..]);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn dragonstorm_globe_doesnt_see_dragons_entering_with_it() {
    cr!("614.12");
    ruling!(
        "Dragonstorm Globe",
        "Dragons you control that enter at the same time as Dragonstorm Globe won’t enter with an additional +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    let ids = enter_together(&mut t, &[(P0, "Dragonstorm Globe"), (P0, "Shivan Dragon")]);
    let d = ids
        .iter()
        .copied()
        .find(|i| t.obj(*i).chars.name == "Shivan Dragon")
        .unwrap();
    assert_eq!(t.counters(d, counters::PLUS1), 0);
    let later = t.enter(P0, "Shivan Dragon");
    t.settle();
    assert_eq!(t.counters(later, counters::PLUS1), 1);
}

#[test]
fn kellogg_controlled_by_someone_else_steals_nothing() {
    cr!("611.2b");
    ruling!(
        "Kellogg, Dangerous Mind",
        "If you lose control of Kellogg before its last ability resolves, you won't gain control of the target creature at all."
    );
    for lose_it in [false, true] {
        let mut t = TestGame::new(2);
        let kellogg = t.battlefield(P0, "Kellogg, Dangerous Mind");
        for _ in 0..5 {
            crate::r_s02_common::create_token(&mut t, P0, "Treasure");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        act(
            &mut t,
            P0,
            kellogg,
            "Gain control of target creature",
            &[obj(bears)],
        )
        .unwrap();
        assert!(t.named_on_battlefield("Treasure").is_empty());
        if lose_it {
            give_control(&mut t, kellogg, P1);
        }
        t.resolve_all();
        let want = if lose_it { P1 } else { P0 };
        assert_eq!(t.obj_now(bears).controller, want, "{lose_it}");
    }
}

#[test]
fn serum_powder_cards_stay_exiled_face_up() {
    cr!("103.5b", "406.3");
    ruling!(
        "Serum Powder",
        "The cards in your hand are exiled for the rest of the game; they aren't shuffled back into your library if you take another mulligan. Cards exiled are always face up unless the effect that exiled them says they aren't. Be sure to return those cards to your deck for your next game."
    );
    let deck: Vec<std::sync::Arc<mtg_engine::card::CardDef>> = (0..40)
        .map(|_| mtg_engine::card::card("Serum Powder"))
        .collect();
    let fillers: Vec<std::sync::Arc<mtg_engine::card::CardDef>> =
        (0..40).map(|_| filler_card()).collect();
    let config = mtg_engine::game::GameConfig {
        starting_player: Some(P0),
        ..Default::default()
    };
    // Powder (seven cards exiled), mulligan, don't Powder again, keep six.
    let mut t = crate::r_s12_common::pregame(config, vec![deck, fillers]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer_yes(P0, false);
    t.g.start();
    assert_eq!(t.hand_size(P0), 6);
    assert_eq!(t.g.exile.len(), 7);
    assert!(t.g.exile.iter().all(|c| !t.g.obj(*c).face_down));
    assert_eq!(t.library_size(P0), 40 - 7 - 6);
}

/// Whether P0 may look at the card on top of their library now.
fn sees_top(g: &Game) -> bool {
    g.library_top(P0)
        .is_some_and(|c| mtg_engine::zones::can_see_in_library(g, P0, c))
}

#[test]
fn crystal_skull_the_new_top_card_is_hidden_until_casting_ends() {
    cr!("401.5", "601.2b", "601.2i");
    ruling!(
        "Crystal Skull, Isu Spyglass",
        "If the top card of your library changes while you’re casting a spell, playing a land, or activating an ability, you can’t look at the new top card until you finish doing so. This means that if you cast a spell from the top of your library, you can’t look at the next one until you’re done paying for that spell."
    );
    // Astral Cornucopia (an artifact) is cast from the top; while X is chosen, the Forest
    // under it can't be looked at.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crystal Skull, Isu Spyglass");
    let cards = stack_library(&mut t, P0, &["Astral Cornucopia", "Forest"]);
    t.g.recompute();
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    assert!(sees_top(&t.g));
    mana(&mut t, P0, ManaType::C, 3);
    let choosing_x = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseX { .. }),
        sees_top,
    );
    t.cast(P0, cards[0]).x(1).go();
    assert_eq!(choosing_x.lock().unwrap().clone(), vec![false]);
    assert_eq!(t.g.library_top(P0), Some(cards[1]));
    assert!(sees_top(&t.g));
}

#[test]
fn crystal_skull_cant_cast_a_historic_card_face_down() {
    cr!("708.4", "601.3e");
    ruling!(
        "Crystal Skull, Isu Spyglass",
        "If the top card of your library is a historic card with a disguise or morph ability, you can’t cast it face down from the top of your library with the permission granted by Crystal Skull because the resulting spell would not be a historic spell."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crystal Skull, Isu Spyglass");
    let akroma = t.library_top(P0, "Akroma, Angel of Fury");
    mana(&mut t, P0, ManaType::R, 3);
    mana(&mut t, P0, ManaType::C, 5);
    assert!(can_cast(&mut t, P0, akroma, CastMethod::Normal));
    assert!(!can_cast(
        &mut t,
        P0,
        akroma,
        CastMethod::FaceDown(KeywordKind::Morph)
    ));
    // From the hand, it could be cast face down.
    let in_hand = t.hand(P0, "Akroma, Angel of Fury");
    assert!(can_cast(
        &mut t,
        P0,
        in_hand,
        CastMethod::FaceDown(KeywordKind::Morph)
    ));
}

#[test]
fn burdened_aerialist_gaining_flying_after_blocks_stays_blocked() {
    cr!("509.1h", "702.9b");
    ruling!(
        "Burdened Aerialist",
        "If a blocked creature gains flying after blockers are declared, it won't cause that creature to become unblocked, even if the blocking creature doesn't have flying or reach."
    );
    let mut t = TestGame::new(2);
    let aer = t.battlefield(P0, "Burdened Aerialist");
    let treasure = crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(aer, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, aer)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert!(tap_for_mana(&mut t, P0, treasure, "Sacrifice"));
    t.resolve_all();
    assert!(t.obj_now(aer).has_keyword(KeywordKind::Flying));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn rankle_and_torbrans_extra_damage_comes_from_the_original_source() {
    cr!("614.1a");
    ruling!(
        "Rankle and Torbran",
        "The additional 2 damage is dealt by the same source as the original source of damage. The damage isn’t dealt by Rankle and Torbran unless Rankle and Torbran is the original source of damage."
    );
    let mut t = TestGame::new(2);
    let rt = t.battlefield(P0, "Rankle and Torbran");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.attack(&[(rt, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "Rankle and Torbran's own 3 damage");
    t.set_step(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::R, 1);
    let m = mark(&t);
    let bolt = cast_from_hand(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    let dealt: Vec<(ObjectId, u32)> = events_since(&t, m)
        .iter()
        .filter_map(|e| match e {
            Event::Damage { source, amount, .. } => Some((*source, *amount)),
            _ => None,
        })
        .collect();
    assert_eq!(dealt, vec![(bolt, 5)]);
}

#[test]
fn rankle_and_torbran_and_prevention_in_the_order_the_player_chooses() {
    cr!("616.1", "615.1a", "614.1a");
    ruling!(
        "Rankle and Torbran",
        "If another effect modifies how much damage a source would deal, including preventing some of it, the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Rankle and Torbran’s effect no longer applies."
    );
    // Healing Salve prevents the next 3 damage to P1; Lightning Bolt hits P1 for 3.
    for (prevent_first, life) in [(true, 20), (false, 18)] {
        let mut t = TestGame::new(2);
        let rt = t.battlefield(P0, "Rankle and Torbran");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
        t.attack(&[(rt, Entity::Player(P1))], &[]);
        t.resolve_all();
        t.g.players[1].life = 20;
        t.set_step(P0, Step::PostcombatMain);
        mana(&mut t, P1, ManaType::W, 1);
        let salve = t.hand(P1, "Healing Salve");
        t.g.turn.priority = Some(P1);
        t.cast(P1, salve)
            .modes(&[1])
            .target(Entity::Player(P1))
            .go();
        t.resolve_all();
        let log = if prevent_first {
            crate::r_p041_common::apply_first(&mut t, P1, "Healing Salve")
        } else {
            crate::r_p041_common::apply_first_not(&mut t, P1, "Healing Salve")
        };
        mana(&mut t, P0, ManaType::R, 1);
        cast_from_hand(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
        t.resolve_all();
        assert_eq!(
            t.life(P1),
            life,
            "prevent first: {prevent_first}\n{:?}\n{}",
            log.lock().unwrap(),
            t.dump_log()
        );
        assert!(!log.lock().unwrap().is_empty(), "P1 chose the order");
    }
}

const BIRGI_CARD: &str = "Birgi, God of Storytelling // Harnfel, Horn of Bounty";

/// P0's Harnfel, Horn of Bounty (cast as its back face).
fn harnfel(t: &mut TestGame) -> ObjectId {
    mana(t, P0, ManaType::R, 1);
    mana(t, P0, ManaType::C, 4);
    let b = t.hand(P0, BIRGI_CARD);
    let spell = t.cast(P0, b).method(CastMethod::Half(1)).go();
    t.resolve_all();
    t.g.current(spell)
}

#[test]
fn harnfel_exiles_face_up_and_only_those_cards_follow_normal_timing() {
    cr!("406.3", "601.3", "117.1a", "607.2a");
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "The cards exiled by Harnfel’s activated ability are exiled face up."
    );
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "Playing the cards exiled with Harnfel follows the normal rules for playing those cards. You must pay their costs, if any, and you must follow all applicable timing rules. For example, if one of the cards is a creature card, you can cast that card by paying its mana cost only during your main phase while the stack is empty."
    );
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "The card you discard to activate the ability isn’t part of “those cards” and can’t be played, even if it somehow ends up in exile."
    );
    let mut t = TestGame::new(2);
    let h = harnfel(&mut t);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Lightning Bolt"]);
    let discarded = t.hand(P0, "Shock");
    act(&mut t, P0, h, "Exile the top two", &[]).unwrap();
    assert!(t.in_graveyard(P0, "Shock"));
    t.resolve_all();
    let bears = t.g.current(lib[0]);
    let bolt = t.g.current(lib[1]);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!t.obj_now(bears).face_down && !t.obj_now(bolt).face_down);
    // The discarded Shock, moved to exile, can't be cast.
    let shock = t.g.current(discarded);
    t.g.move_object(shock, Zone::Exile, MoveCause::Effect, None);
    let shock = t.g.current(shock);
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(!can_cast(&mut t, P0, shock, CastMethod::Normal));
    // Bears needs its mana and sorcery timing; Bolt can be cast at instant speed.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, bolt, CastMethod::Normal));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.g.players[0].mana_pool = Default::default();
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
}

#[test]
fn eye_of_ojer_taq_offers_only_card_types() {
    cr!("205.2a", "702.167a");
    ruling!(
        "Eye of Ojer Taq // Apex Observatory",
        "Card types that may appear in a normal game of Magic are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Supertypes such as snow and basic may not be chosen. Subtypes such as Bird, Equipment, and Forest also may not be chosen. Land may be chosen, but it's not a good idea."
    );
    // Crafted with two Icehide Golems (snow artifact creature Golems): Artifact and
    // Creature, not Snow or Golem. With two Dryad Arbors (land creature Forest Dryads):
    // Land and Creature, not Forest or Dryad.
    for (material, want) in [
        ("Icehide Golem", ["artifact", "creature"]),
        ("Dryad Arbor", ["creature", "land"]),
    ] {
        let mut t = TestGame::new(2);
        let eye = t.battlefield(P0, "Eye of Ojer Taq // Apex Observatory");
        let a = t.battlefield(P0, material);
        let b = t.battlefield(P0, material);
        mana(&mut t, P0, ManaType::C, 6);
        t.answer_choose(P0, &[obj(a), obj(b)]);
        let from = t.asked().len();
        act(&mut t, P0, eye, "Craft", &[]).unwrap();
        t.resolve_all();
        let offered = crate::r_p125_common::options_asked(&t, P0, from);
        assert_eq!(offered.len(), 1, "{material}\n{}", t.dump_log());
        let mut opts = offered[0].1.clone();
        opts.sort();
        assert_eq!(opts, want.map(String::from).to_vec(), "{material}");
    }
    // Crafted with two Firdoch Cores (kindred artifacts): Kindred and Artifact, not
    // Shapeshifter.
    let mut t = TestGame::new(2);
    let eye = t.battlefield(P0, "Eye of Ojer Taq // Apex Observatory");
    let a = t.battlefield(P0, "Firdoch Core");
    let b = t.battlefield(P0, "Firdoch Core");
    mana(&mut t, P0, ManaType::C, 6);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    let from = t.asked().len();
    act(&mut t, P0, eye, "Craft", &[]).unwrap();
    t.resolve_all();
    let offered = crate::r_p125_common::options_asked(&t, P0, from);
    assert_eq!(offered.len(), 1);
    let mut opts = offered[0].1.clone();
    opts.sort();
    assert_eq!(opts, vec!["artifact".to_string(), "kindred".to_string()]);
}
