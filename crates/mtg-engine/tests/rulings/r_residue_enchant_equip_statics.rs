//! Rulings residue (batches P207–P213, Auras and Equipment): static abilities of Auras and
//! Equipment and their activated abilities — Blossombind, Overwhelming Splendor, Armament
//! of Nyx, Bound by Moonsilver, Heliod's Punishment, Daybreak Coronet, Dream Leash,
//! Consecrate Land, Wheel of Sun and Moon, Strong Back, Infectious Curse, Detention Vortex,
//! Soul Ransom, Captured by the Consulate, Halvar, Heartseeker, Hammer of Nazahn and more.

use crate::r_p212_common::{grant, move_eq};
use crate::r_residue_enchant_equip_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, destroy, target_candidates};
use crate::r_s06_common::*;
use crate::r_s25_common::cast_new;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Blossombind --------------------------------------------------------------------------

const BLOSSOMBIND: &str = "Blossombind";

#[test]
fn blossombind_wither_and_infect_damage_has_no_effect_but_is_dealt() {
    cr!("702.80a", "702.90c", "702.15b", "120.3d");
    ruling!(
        "Blossombind",
        "Damage from a source with wither or infect has no effect on a creature enchanted with Blossombind. No -1/-1 counters are put on it, and no damage is marked on it. The damage is still dealt for purposes of effects that care about damage, such as lifelink."
    );
    supported(BLOSSOMBIND);
    // Glistener Elf (infect) and Sickle Ripper (wither), each with lifelink.
    for source in ["Glistener Elf", "Sickle Ripper"] {
        supported(source);
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        aura_on(&mut t, P1, BLOSSOMBIND, giant);
        let src = t.battlefield(P1, source);
        grant(&mut t, src, KeywordKind::Lifelink);
        damage(&mut t, src, 2, giant);
        assert_eq!(t.counters(giant, counters::MINUS1), 0, "{source}");
        assert_eq!(t.obj_now(giant).damage, 0, "{source}");
        assert_eq!(t.pt(giant), (3, 3), "{source}");
        assert_eq!(t.life(P1), 22, "{source}");
    }
}

#[test]
fn blossombind_replacement_that_would_give_counters_gives_none() {
    cr!("614.1a", "615.1a", "122.1");
    ruling!(
        "Blossombind",
        "If a replacement effect allows a player to modify or replace an event by untapping or putting counters on a creature enchanted with Blossombind, that player may apply that replacement effect, but they won't get counters. If the original event is entirely replaced, the original event won't happen."
    );
    supported("Vigor");
    // Vigor: "If damage would be dealt to another creature you control, prevent it. Put a
    // +1/+1 counter on that creature for each 1 damage prevented this way."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vigor");
    let giant = t.battlefield(P0, "Hill Giant");
    aura_on(&mut t, P1, BLOSSOMBIND, giant);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 2, giant);
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.counters(giant, counters::PLUS1), 0);
}

#[test]
fn blossombind_costs_that_untap_or_put_counters_cant_be_paid() {
    cr!("118.3", "602.2b", "122.1", "101.2", "608.2c");
    ruling!(
        "Blossombind",
        "If the cost of an ability or an additional cost of a spell requires untapping or putting counters on a creature enchanted with Blossombind, that cost can't be paid."
    );
    supported("Devoted Druid");
    // Devoted Druid: "Put a -1/-1 counter on this creature: Untap this creature."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let druid = t.battlefield(P0, "Devoted Druid");
    t.g.obj_mut(druid).tapped = true;
    assert!(can_activate(&mut t, P0, druid));
    aura_on(&mut t, P1, BLOSSOMBIND, druid);
    assert!(!can_activate(&mut t, P0, druid));
    // Hidden Strings: "You may tap or untap target permanent, then you may tap or untap
    // another target permanent." P0 untaps both: only the other one untaps.
    supported("Hidden Strings");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.g.obj_mut(giant).tapped = true;
    t.g.obj_mut(other).tapped = true;
    aura_on(&mut t, P1, BLOSSOMBIND, giant);
    for _ in 0..2 {
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
    }
    cast_new(&mut t, P0, "Hidden Strings", &[obj(giant), obj(other)]);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert!(!t.obj_now(other).tapped, "{}", t.dump_log());
    // Soldevi Golem: "At the beginning of your upkeep, you may untap target tapped
    // creature an opponent controls. If you do, untap this creature." Aven Mimeomancer:
    // "At the beginning of your upkeep, you may put a feather counter on target creature.
    // If you do, that creature has base power and toughness 3/1 and has flying ...". P0
    // can't choose to untap the enchanted Giant or put a counter on it, so neither "if
    // you do" happens (without Blossombind, both do).
    supported("Soldevi Golem");
    supported("Aven Mimeomancer");
    for bound in [false, true] {
        let mut t = TestGame::new(2);
        let golem = t.battlefield(P0, "Soldevi Golem");
        t.battlefield(P0, "Aven Mimeomancer");
        let giant = t.battlefield(P1, "Hill Giant");
        t.g.obj_mut(golem).tapped = true;
        t.g.obj_mut(giant).tapped = true;
        if bound {
            aura_on(&mut t, P0, BLOSSOMBIND, giant);
        }
        for _ in 0..2 {
            t.answer_yes(P0, true);
            t.answer_targets(P0, &[obj(giant)]);
        }
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        t.resolve_all();
        assert_eq!(t.obj_now(giant).tapped, bound, "{}", t.dump_log());
        assert_eq!(
            t.obj_now(golem).tapped,
            bound,
            "bound {bound}: {}",
            t.dump_log()
        );
        assert_eq!(t.counters(giant, "feather"), if bound { 0 } else { 1 });
        assert_eq!(t.pt(giant), if bound { (3, 3) } else { (3, 1) });
        assert_eq!(has_kw(&t, giant, KeywordKind::Flying), !bound);
    }
}

// --- Overwhelming Splendor ----------------------------------------------------------------

const SPLENDOR: &str = "Overwhelming Splendor";

fn splendor_on(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported(SPLENDOR);
    aura_on(t, P0, SPLENDOR, p)
}

#[test]
fn overwhelming_splendor_leaving_with_a_creature_its_leave_abilities_dont_trigger() {
    cr!("603.10a", "613.1f");
    ruling!(
        "Overwhelming Splendor",
        "If Overwhelming Splendor leaves the battlefield at the same time as a creature enchanted player controls, any \"when [this creature] [dies or leaves the battlefield]\" abilities of that creature won't trigger."
    );
    let mut t = TestGame::new(2);
    let s = splendor_on(&mut t, P1);
    let traveler = t.battlefield(P1, "Doomed Traveler");
    t.g.destroy_all(vec![traveler, s], None, false);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(with_subtype(&t, P1, "Spirit").is_empty());
}

#[test]
fn overwhelming_splendor_creatures_enter_without_their_enters_abilities() {
    cr!("614.12", "603.6a", "613.1f");
    ruling!(
        "Overwhelming Splendor",
        "If a creature enters the battlefield under enchanted player's control, any \"when [this creature] enters the battlefield\" abilities of that creature won't trigger. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities won't be applied."
    );
    let mut t = TestGame::new(2);
    splendor_on(&mut t, P1);
    t.library_top(P1, "Island");
    let hand = t.hand_size(P1);
    t.enter(P1, "Elvish Visionary");
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand);
    let hydra = t.enter(P1, "Kalonian Hydra");
    t.settle();
    assert_eq!(t.counters(hydra, counters::PLUS1), 0);
    let ghoul = t.enter(P1, "Diregraf Ghoul");
    t.settle();
    assert!(!t.obj_now(ghoul).tapped);
    // P0's creatures aren't affected.
    let hydra = t.enter(P0, "Kalonian Hydra");
    t.settle();
    assert_eq!(t.counters(hydra, counters::PLUS1), 4);
}

#[test]
fn overwhelming_splendor_and_riddleform_and_earlier_pt_setting() {
    cr!("613.4b", "613.7", "613.1f", "611.3a");
    ruling!(
        "Overwhelming Splendor",
        "However, if the effect that makes that permanent a creature grants it an ability, it will continue to have that ability. For example, the enchanted player's Riddleform will have flying once it becomes a creature."
    );
    ruling!(
        "Overwhelming Splendor",
        "Overwhelming Splendor overwrites all previous effects that set a creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply to a creature after Overwhelming Splendor becomes attached to its controller will overwrite this effect."
    );
    supported("Riddleform");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    // An earlier base power and toughness of 5/5 is overwritten.
    let bears = t.battlefield(P1, "Grizzly Bears");
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::SetPT(Some(Value::c(5)), Some(Value::c(5)))],
    );
    assert_eq!(t.pt(bears), (5, 5));
    let riddle = t.battlefield(P1, "Riddleform");
    splendor_on(&mut t, P1);
    assert_eq!(t.pt(bears), (1, 1));
    // Riddleform becomes a 3/3 flier: a later P/T-setting effect, and a granted ability.
    t.answer_yes(P1, true);
    t.g.turn.priority = Some(P1);
    cast_new(&mut t, P1, "Divination", &[]);
    t.settle();
    t.resolve_all();
    assert!(t.obj_now(riddle).is(CardType::Creature), "{}", t.dump_log());
    assert_eq!(t.pt(riddle), (3, 3));
    assert!(has_kw(&t, riddle, KeywordKind::Flying));
    // Its own activated ability is gone.
    assert!(!t
        .obj_now(riddle)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("Scry")));
}

#[test]
fn overwhelming_splendor_mana_and_loyalty_abilities() {
    cr!("605.1a", "606.3", "613.1f");
    ruling!(
        "Overwhelming Splendor",
        "While the enchanted player can still activate mana abilities and loyalty abilities, creatures that player controls won't normally have any of those abilities to be activated."
    );
    supported("Liliana of the Veil");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let lili = t.battlefield(P1, "Liliana of the Veil");
    splendor_on(&mut t, P1);
    assert!(t.obj_now(elves).chars.abilities.is_empty());
    assert!(!can_activate(&mut t, P1, elves));
    assert!(can_activate(&mut t, P1, lili));
}

// --- Armament of Nyx, Bound by Moonsilver, Heliod's Punishment, Paralyze -----------------

#[test]
fn armament_of_nyx_checks_continuously_whether_the_creature_is_an_enchantment() {
    cr!("611.3a", "615.1a");
    ruling!(
        "Armament of Nyx",
        "Armament of Nyx continuously checks to see whether the creature it's enchanting is an enchantment or not."
    );
    supported("Armament of Nyx");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let victim = t.battlefield(P1, "Hill Giant");
    aura_on(&mut t, P0, "Armament of Nyx", bears);
    assert!(!has_kw(&t, bears, KeywordKind::DoubleStrike));
    damage(&mut t, bears, 2, victim);
    assert_eq!(t.obj_now(victim).damage, 0);
    // It becomes an enchantment: double strike, and its damage is dealt.
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddTypes(vec![CardType::Enchantment])],
    );
    assert!(has_kw(&t, bears, KeywordKind::DoubleStrike));
    damage(&mut t, bears, 2, victim);
    assert_eq!(t.obj_now(victim).damage, 2);
}

const MOONSILVER: &str = "Bound by Moonsilver";

#[test]
fn bound_by_moonsilver_transforming_abilities_still_do_the_rest() {
    cr!("701.27a", "101.2", "608.2c");
    ruling!(
        "Bound by Moonsilver",
        "Activated and triggered abilities of the enchanted creature that would cause it to transform can still be activated or triggered. If those abilities have any other effects, those effects will happen."
    );
    supported(MOONSILVER);
    supported("Ludevic's Test Subject // Ludevic's Abomination");
    // Ludevic's Test Subject: "{1}{U}: Put a hatchling counter on this creature. Then if
    // there are five or more hatchling counters on it, remove all of them and transform
    // it."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let subject = t.battlefield(P0, "Ludevic's Test Subject // Ludevic's Abomination");
    t.g.add_counters(obj(subject), "hatchling", 4, None);
    aura_on(&mut t, P1, MOONSILVER, subject);
    t.lands(P0, "Island", 2);
    activate_containing(&mut t, P0, subject, "hatchling").expect("activate");
    t.resolve_all();
    assert_eq!(name(&t, subject), "Ludevic's Test Subject");
    // The fifth counter was put on and all of them were removed; it didn't transform.
    assert_eq!(t.counters(subject, "hatchling"), 0, "{}", t.dump_log());
}

#[test]
fn bound_by_moonsilver_only_its_controller_activates_it() {
    cr!("602.2", "303.4e");
    ruling!(
        "Bound by Moonsilver",
        "You control Bound by Moonsilver even while it enchants an opponent’s creature. Only you can activate its last ability."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Island");
    t.battlefield(P1, "Island");
    let aura = aura_on(&mut t, P0, MOONSILVER, bears);
    assert_eq!(t.obj_now(aura).controller, P0);
    assert!(can_activate(&mut t, P0, aura));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P1, aura));
}

const PUNISHMENT: &str = "Heliod's Punishment";

#[test]
fn heliods_punishment_two_of_them_only_the_latest_ability() {
    cr!("613.7", "613.1f", "613.7e");
    ruling!(
        "Heliod's Punishment",
        "If a creature is enchanted by two or more Heliod's Punishments, it has only the activated ability of the one that most recently became attached. Activating that ability removes a task counter from only that one."
    );
    supported(PUNISHMENT);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let first = attach_new(&mut t, P0, PUNISHMENT, bears);
    t.g.add_counters(obj(first), "task", 4, None);
    let second = attach_new(&mut t, P0, PUNISHMENT, bears);
    t.g.add_counters(obj(second), "task", 4, None);
    t.g.recompute();
    let n = t
        .obj_now(bears)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains("task counter"))
        .count();
    assert_eq!(n, 1);
    activate_containing(&mut t, P1, bears, "task counter").expect("activate");
    t.resolve_all();
    assert_eq!(t.counters(second, "task"), 3);
    assert_eq!(t.counters(first, "task"), 4);
}

#[test]
fn heliods_punishment_star_power_is_zero() {
    cr!("604.3", "613.1f", "107.2", "208.2");
    ruling!(
        "Heliod's Punishment",
        "If the enchanted creature has an ability that defines a * in its power or toughness, that * is 0."
    );
    supported("Tarmogoyf");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Island");
    t.graveyard(P0, "Lightning Bolt");
    let goyf = t.battlefield(P1, "Tarmogoyf");
    assert_eq!(t.pt(goyf), (2, 3));
    aura_on(&mut t, P0, PUNISHMENT, goyf);
    assert_eq!(t.pt(goyf), (0, 1));
}

#[test]
fn paralyze_has_the_upkeep_trigger_itself() {
    cr!("603.3a", "613.1f", "113.10");
    ruling!(
        "Paralyze",
        "Paralyze itself has the last triggered ability, rather than granting that ability to the enchanted creature."
    );
    supported("Paralyze");
    // The creature has lost all abilities (for good, not until end of turn): Paralyze's
    // own trigger, controlled by Paralyze's controller, still lets P1 pay {4} to untap it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let paralyze = aura_on(&mut t, P0, "Paralyze", bears);
    t.g.obj_mut(bears).tapped = true;
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![vec![obj(bears)]];
    t.g.exec(
        &mtg_engine::ability::Effect::Modify {
            what: mtg_engine::ability::Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: mtg_engine::ability::Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.lands(P1, "Wastes", 4);
    t.answer_yes(P1, true);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(
        crate::r_s14_common::triggers_from(&t, paralyze),
        1,
        "{}",
        t.dump_log()
    );
    let trigger = *t.g.stack.last().expect("the trigger");
    assert_eq!(t.g.obj(trigger).controller, P0);
    t.resolve_all();
    assert!(t.obj_now(bears).chars.abilities.is_empty());
    assert!(!t.obj_now(bears).tapped, "{}", t.dump_log());
}

// --- Daybreak Coronet, Dream Leash, Consecrate Land, Animate Wall, Goblin Caves ----------

const CORONET: &str = "Daybreak Coronet";

#[test]
fn daybreak_coronet_falls_off_when_the_other_auras_go() {
    cr!("303.4c", "704.5m");
    ruling!(
        "Daybreak Coronet",
        "If all other Auras attached to the enchanted creature stop enchanting it, Daybreak Coronet will be attached to an illegal permanent and will be put into its owner's graveyard."
    );
    supported(CORONET);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = attach_new(&mut t, P0, "Holy Strength", bears);
    let coronet = attach_new(&mut t, P0, CORONET, bears);
    assert_eq!(t.pt(bears), (2 + 1 + 3, 2 + 2 + 3));
    destroy(&mut t, strength);
    assert!(in_zone(&t, coronet, Zone::Graveyard(P0)));
}

#[test]
fn daybreak_coronet_entering_with_another_aura_cant_enchant_an_unenchanted_creature() {
    cr!("303.4f", "303.4g");
    ruling!(
        "Daybreak Coronet",
        "This means that Daybreak Coronet can't be put onto the battlefield enchanting a creature that's not enchanted before the Auras all enter the battlefield."
    );
    supported("Open the Vaults");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = t.graveyard(P0, "Holy Strength");
    let coronet = t.graveyard(P0, CORONET);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Open the Vaults", &[]);
    t.resolve_all();
    assert_eq!(attached_to(&t, strength), Some(obj(bears)));
    assert!(
        in_zone(&t, coronet, Zone::Graveyard(P0)),
        "{}",
        t.dump_log()
    );
}

#[test]
fn dream_leash_tapped_requirement_is_only_for_casting() {
    cr!("303.4f", "601.2c", "707.2");
    ruling!(
        "Dream Leash",
        "If an effect puts Dream Leash onto the battlefield or moves it to another permanent, the permanent it becomes attached to need not be tapped. If you cast Copy Enchantment and copy Dream Leash as it enters, the permanent it becomes attached to need not be tapped."
    );
    ruling!(
        "Dream Leash",
        "The restriction that the permanent must be tapped is checked only if you cast Dream Leash as a spell. Once Dream Leash is on the battlefield, it doesn't matter if the enchanted permanent becomes untapped."
    );
    supported("Dream Leash");
    supported("Copy Enchantment");
    // Cast: only a tapped permanent can be chosen.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let untapped = t.battlefield(P1, "Grizzly Bears");
    let tapped = t.battlefield(P1, "Hill Giant");
    t.g.obj_mut(tapped).tapped = true;
    let leash = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Dream Leash");
    let from = t.asked().len();
    t.cast(P0, leash).target(tapped).go();
    let offered = target_candidates(&t, P0, from);
    assert!(offered[0].contains(&obj(tapped)));
    assert!(!offered[0].contains(&obj(untapped)));
    t.resolve_all();
    assert_eq!(t.obj_now(tapped).controller, P0);
    // It untaps: still P0's.
    crate::r_p209_common::untap(&mut t, tapped);
    assert_eq!(t.obj_now(tapped).controller, P0);
    // Moved by an effect to an untapped permanent.
    let leash = t.g.current(leash);
    assert!(t.g.attach(leash, obj(untapped)));
    t.g.recompute();
    assert_eq!(t.obj_now(untapped).controller, P0);
    // Copy Enchantment copying it, attached to an untapped permanent.
    let other = t.battlefield(P1, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(leash)]);
    t.answer_choose(P0, &[obj(other)]);
    let ce = t.enter(P0, "Copy Enchantment");
    t.settle();
    assert_eq!(name(&t, ce), "Dream Leash");
    assert_eq!(attached_to(&t, ce), Some(obj(other)), "{}", t.dump_log());
    assert_eq!(t.obj_now(other).controller, P0);
}

#[test]
fn consecrate_land_entering_puts_the_other_auras_into_the_graveyard() {
    cr!("303.4c", "704.5m");
    ruling!(
        "Consecrate Land",
        "If Consecrate Land enters attached to a land that's enchanted by other Auras, those Auras are put into their owners' graveyards."
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    let lore = attach_new(&mut t, P1, "Forbidden Lore", land);
    attach_new(&mut t, P0, "Consecrate Land", land);
    t.settle();
    assert!(in_zone(&t, lore, Zone::Graveyard(P1)));
}

#[test]
fn animate_wall_enchants_only_a_wall() {
    cr!("303.4a", "702.5a");
    ruling!("Animate Wall", "Animate Wall can now enchant only a Wall.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P0, "Wall of Stone");
    let aura = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Animate Wall");
    let from = t.asked().len();
    t.cast(P0, aura).target(wall).go();
    let offered = target_candidates(&t, P0, from);
    assert!(offered[0].contains(&obj(wall)));
    assert!(!offered[0].contains(&obj(bears)));
    t.resolve_all();
    assert_eq!(attached_to(&t, aura), Some(obj(wall)));
}

#[test]
fn goblin_caves_works_on_a_snow_covered_mountain() {
    cr!("205.4c", "205.3i");
    ruling!(
        "Goblin Caves",
        "Works if enchanting a Snow-Covered Mountain."
    );
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Snow-Covered Mountain");
    let goblin = t.battlefield(P1, "Goblin Piker");
    aura_on(&mut t, P0, "Goblin Caves", land);
    assert_eq!(t.pt(goblin), (2, 3));
}

#[test]
fn forbidden_lore_ability_is_activated_by_the_lands_controller() {
    cr!("113.10", "602.2");
    ruling!(
        "Forbidden Lore",
        "Forbidden Lore grants an activated ability to the enchanted land. Only that land’s controller can activate the ability."
    );
    supported("Forbidden Lore");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    aura_on(&mut t, P0, "Forbidden Lore", land);
    assert!(can_activate(&mut t, P1, land));
    assert!(!can_activate(&mut t, P0, land));
}

// --- Wheel of Sun and Moon ----------------------------------------------------------------

const WHEEL: &str = "Wheel of Sun and Moon";

#[test]
fn wheel_of_sun_and_moon_applies_from_every_zone() {
    cr!("614.1a", "614.6");
    ruling!(
        "Wheel of Sun and Moon",
        "Wheel of Sun and Moon’s replacement effect will apply to cards that would be put into the enchanted player’s graveyard from any game zone."
    );
    supported(WHEEL);
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    aura_on(&mut t, P0, WHEEL, P1);
    // From the battlefield.
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    // From the stack (a resolving instant).
    t.g.turn.priority = Some(P1);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    // From the hand.
    let card = t.hand(P1, "Island");
    t.g.discard(P1, card, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.graveyard_size(P1), 0, "{}", t.dump_log());
    // The bottom of P1's library (library order is bottom first).
    let bottom: Vec<String> = t.g.players[1]
        .library
        .iter()
        .take(3)
        .map(|o| t.g.obj(*o).chars.name.to_string())
        .collect();
    assert!(bottom.contains(&"Grizzly Bears".to_string()), "{bottom:?}");
    assert!(bottom.contains(&"Lightning Bolt".to_string()));
    assert!(bottom.contains(&"Island".to_string()));
}

#[test]
fn wheel_of_sun_and_moon_simultaneous_cards_are_ordered_by_the_player() {
    cr!("614.1a", "401.4");
    ruling!(
        "Wheel of Sun and Moon",
        "If multiple cards would be put into the enchanted player’s graveyard at the same time (due to Millstone, for example), they are instead all revealed and put on the bottom of the enchanted player’s library at the same time. That player chooses what order to put them in."
    );
    supported("Millstone");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    aura_on(&mut t, P0, WHEEL, P1);
    let mill = t.battlefield(P0, "Millstone");
    t.library_top(P1, "Forest");
    t.library_top(P1, "Island");
    t.lands(P0, "Wastes", 2);
    let lib = t.library_size(P1);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, mill, "mills").expect("activate");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0, "{}", t.dump_log());
    assert_eq!(t.library_size(P1), lib);
    // P1 was asked to order them.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, mtg_engine::decision::Decision::Order { .. })));
    let bottom: Vec<String> = t.g.players[1]
        .library
        .iter()
        .take(2)
        .map(|o| t.g.obj(*o).chars.name.to_string())
        .collect();
    assert!(bottom.contains(&"Forest".to_string()), "{bottom:?}");
    assert!(bottom.contains(&"Island".to_string()));
}

// --- Strong Back, Infectious Curse --------------------------------------------------------

#[test]
fn strong_back_reduces_equip_and_aura_costs_targeting_the_creature() {
    cr!("601.2f", "602.2b", "118.7a");
    ruling!(
        "Strong Back",
        "Strong Back’s abilities reduce only the amount of generic mana in equip abilities that target the enchanted creature and in the total cost of Aura spells that target the enchanted creature. For example, it will reduce the total cost of Animal Friend from {1}{G} to {G}."
    );
    supported("Strong Back");
    supported("Animal Friend");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    aura_on(&mut t, P0, "Strong Back", bears);
    // Animal Friend for {G} on the Bears, not on the Giant.
    t.lands(P0, "Forest", 1);
    let friend = t.hand(P0, "Animal Friend");
    assert!(t.cast(P0, friend).target(other).try_go().is_err());
    t.clear_answers();
    t.cast(P0, friend).target(bears).go();
    t.resolve_all();
    assert_eq!(attached_to(&t, friend), Some(obj(bears)));
    // Bonesplitter's equip {1} costs nothing on the Bears.
    let split = t.battlefield(P0, "Bonesplitter");
    crate::r_p213_common::equip(&mut t, P0, split, bears).expect("equip for free");
    t.resolve_all();
    assert_eq!(attached_to(&t, split), Some(obj(bears)));
}

#[test]
fn infectious_curse_reduces_spells_targeting_the_player_not_abilities() {
    cr!("601.2f", "602.2b");
    ruling!(
        "Accursed Witch // Infectious Curse",
        "Infectious Curse's middle ability affects all spells you cast that target the enchanted player, including Aura spells and spells that have additional targets. It doesn't affect abilities."
    );
    supported("Accursed Witch // Infectious Curse");
    supported("Mind Rot");
    supported("Rod of Ruin");
    let setup = || {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let curse = t.battlefield(P0, "Accursed Witch // Infectious Curse");
        let o = &mut t.g.objects[curse.0 as usize];
        o.face = mtg_engine::object::FaceState::Back;
        o.base = o
            .card
            .clone()
            .expect("card")
            .characteristics(mtg_engine::object::FaceState::Back);
        t.g.obj_mut(curse).attached_to = Some(Entity::Player(P1));
        t.g.recompute();
        assert_eq!(name(&t, curse), "Infectious Curse");
        t
    };
    // Mind Rot ({2}{B}) on P1 for {1}{B}.
    let mut t = setup();
    t.lands(P0, "Swamp", 2);
    let rot = t.hand(P0, "Mind Rot");
    t.cast(P0, rot).target(Entity::Player(P1)).go();
    // An Aura spell (Curse of Shaken Faith, {1}{R}) on P1 for {R}.
    let mut t = setup();
    t.lands(P0, "Mountain", 1);
    let c = t.hand(P0, "Curse of Shaken Faith");
    t.cast(P0, c).target(Entity::Player(P1)).go();
    // Hungry Flames ({2}{R}: 3 damage to target creature and 2 damage to target player or
    // planeswalker) targeting a creature and P1, for {1}{R}.
    let mut t = setup();
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let flames = t.hand(P0, "Hungry Flames");
    t.cast(P0, flames)
        .target(bears)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Rod of Ruin's ability ({3}, {T}) still costs {3}.
    let mut t = setup();
    t.lands(P0, "Wastes", 2);
    let rod = t.battlefield(P0, "Rod of Ruin");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(activate_containing(&mut t, P0, rod, "damage").is_err());
}

// --- Detention Vortex, Soul Ransom, Captured by the Consulate ----------------------------

#[test]
fn detention_vortex_any_opponent_may_activate() {
    cr!("602.2", "806.1");
    ruling!(
        "Detention Vortex",
        "In a multiplayer game, any opponent may activate Detention Vortex's last ability, not just an opponent who controls the permanent that it is enchanting."
    );
    supported("Detention Vortex");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let vortex = aura_on(&mut t, P0, "Detention Vortex", bears);
    for p in [P0, P1, P2] {
        t.lands(p, "Wastes", 3);
    }
    t.set_step(P2, Step::PrecombatMain);
    assert!(can_activate(&mut t, P2, vortex));
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P0, vortex));
}

#[test]
fn soul_ransom_any_opponent_activates_and_you_sacrifice_and_draw() {
    cr!("602.2", "110.2");
    ruling!(
        "Soul Ransom",
        "Any of your opponents can activate the last ability of Soul Ransom by discarding two cards. When that ability resolves, you'll sacrifice Soul Ransom and draw two cards."
    );
    supported("Soul Ransom");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ransom = aura_on(&mut t, P0, "Soul Ransom", bears);
    assert_eq!(t.obj_now(bears).controller, P0);
    // P2 (another opponent) activates it.
    let a = t.hand(P2, "Island");
    let b = t.hand(P2, "Forest");
    let hand0 = t.hand_size(P0);
    t.answer_choose(P2, &[obj(a), obj(b)]);
    t.set_step(P2, Step::PrecombatMain);
    activate_containing(&mut t, P2, ransom, "sacrifices").expect("activate");
    t.resolve_all();
    assert!(in_zone(&t, ransom, Zone::Graveyard(P0)));
    assert_eq!(t.hand_size(P0), hand0 + 2);
    assert_eq!(t.obj_now(bears).controller, P1);
    assert!(t.in_graveyard(P2, "Island") && t.in_graveyard(P2, "Forest"));
}

#[test]
fn captured_by_the_consulate_target_stays_if_it_cant_change() {
    cr!("115.7", "115.7a");
    ruling!(
        "Captured by the Consulate",
        "If the spell can't target the enchanted creature, its target remains unchanged."
    );
    ruling!(
        "Captured by the Consulate",
        "When a player casts an Aura spell, it targets the creature that it will enchant and may cause Captured by the Consulate's last ability to trigger."
    );
    supported("Captured by the Consulate");
    // P1's Mind Rot targets a player: it can't target a creature, so its target stays.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let enchanted = t.battlefield(P1, "Grizzly Bears");
    aura_on(&mut t, P0, "Captured by the Consulate", enchanted);
    t.g.turn.priority = Some(P1);
    let hand = t.hand_size(P0);
    t.hand(P0, "Island");
    t.hand(P0, "Forest");
    cast_new(&mut t, P1, "Mind Rot", &[Entity::Player(P0)]);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "{}", t.dump_log());
    // P1's Aura spell (Holy Strength) on P0's Hill Giant is moved to the enchanted creature.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let enchanted = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    aura_on(&mut t, P0, "Captured by the Consulate", enchanted);
    t.g.turn.priority = Some(P1);
    let hs = cast_new(&mut t, P1, "Holy Strength", &[obj(giant)]);
    t.settle();
    t.resolve_all();
    assert_eq!(
        attached_to(&t, hs),
        Some(obj(enchanted)),
        "{}",
        t.dump_log()
    );
}

// --- Ghastly Haunting, Halvar, Sword of the Meek, Heartseeker, Hammer of Nazahn ----------

#[test]
fn ghastly_haunting_counters_stay_and_dont_affect_the_creature() {
    cr!("712.18", "122.1a");
    ruling!(
        "Soul Seizer // Ghastly Haunting",
        "If Soul Seizer has any +1/+1 counters on it when it transforms, they remain on Ghastly Haunting. They won't affect the power or toughness of the enchanted creature, however."
    );
    supported("Soul Seizer // Ghastly Haunting");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let seizer = t.battlefield(P0, "Soul Seizer // Ghastly Haunting");
    t.g.add_counters(obj(seizer), counters::PLUS1, 2, None);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    t.attack(&[(seizer, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(name(&t, seizer), "Ghastly Haunting", "{}", t.dump_log());
    assert_eq!(attached_to(&t, seizer), Some(obj(bears)));
    assert_eq!(t.counters(seizer, counters::PLUS1), 2);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn halvar_counts_auras_and_equipment_you_dont_control() {
    cr!("303.4", "301.5");
    ruling!(
        "Halvar, God of Battle // Sword of the Realms",
        "A creature you control is enchanted or equipped if there's an Aura or an Equipment attached to it. You don't have to control that Aura or Equipment."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Halvar, God of Battle // Sword of the Realms");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let plain = t.battlefield(P0, "Craw Wurm");
    attach_new(&mut t, P1, "Holy Strength", bears);
    attach_new(&mut t, P1, "Bonesplitter", giant);
    assert!(has_kw(&t, bears, KeywordKind::DoubleStrike));
    assert!(has_kw(&t, giant, KeywordKind::DoubleStrike));
    assert!(!has_kw(&t, plain, KeywordKind::DoubleStrike));
}

#[test]
fn sword_of_the_meek_equips_any_creature() {
    cr!("702.6a", "301.5a");
    ruling!(
        "Sword of the Meek",
        "Any creature can be equipped with Sword of the Meek, not just 1/1 creatures."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let sword = t.battlefield(P0, "Sword of the Meek");
    t.lands(P0, "Wastes", 2);
    crate::r_p213_common::equip(&mut t, P0, sword, wurm).expect("equip");
    t.resolve_all();
    assert_eq!(attached_to(&t, sword), Some(obj(wurm)));
    assert_eq!(t.pt(wurm), (7, 6));
}

#[test]
fn heartseeker_ability_targets_protection_from_artifacts_unless_the_creature_is_one() {
    cr!("702.16b", "113.7");
    ruling!(
        "Heartseeker",
        "Heartseeker grants an ability to the equipped creature. The ability can target a creature with protection from artifacts (unless the equipped creature is an artifact creature)."
    );
    supported("Tel-Jilad Chosen");
    for artifact in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let holder = t.battlefield(
            P0,
            if artifact {
                "Ornithopter"
            } else {
                "Grizzly Bears"
            },
        );
        let chosen = t.battlefield(P1, "Tel-Jilad Chosen");
        attach_new(&mut t, P0, "Heartseeker", holder);
        t.answer_targets(P0, &[obj(chosen)]);
        let from = t.asked().len();
        activate_containing(&mut t, P0, holder, "Destroy target creature").expect("activate");
        let offered = target_candidates(&t, P0, from);
        assert_eq!(
            offered[0].contains(&obj(chosen)),
            !artifact,
            "artifact {artifact}"
        );
        t.resolve_all();
        assert_eq!(t.on_battlefield(chosen), artifact);
    }
}

#[test]
fn hammer_of_nazahn_moving_it_lets_marked_damage_kill() {
    cr!("120.6", "704.5g", "702.12b");
    ruling!(
        "Hammer of Nazahn",
        "nonlethal damage dealt to the equipped creature may become lethal if you attach Hammer of Nazahn to a different creature during that turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let hammer = attach_new(&mut t, P0, "Hammer of Nazahn", bears);
    let src = t.battlefield(P1, "Hill Giant");
    damage(&mut t, src, 3, bears);
    assert!(t.on_battlefield(bears));
    move_eq(&mut t, hammer, giant);
    assert!(!t.on_battlefield(bears));
}

// --- Conjurer's Mantle, Dancer's Chakrams -------------------------------------------------

#[test]
fn conjurers_mantle_shares_a_type_with_one_in_common() {
    cr!("205.3m");
    ruling!(
        "Conjurer's Mantle",
        "A creature \"shares a creature type\" with the equipped creature if they have at least one creature type in common."
    );
    let mut t = TestGame::new(2);
    let soldier = t.battlefield(P0, "Alert Shu Infantry");
    attach_new(&mut t, P0, "Conjurer's Mantle", soldier);
    let lib = stack_library(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Hearthfire Hobgoblin",
            "Island",
            "Forest",
            "Swamp",
            "Plains",
        ],
    );
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[1])]);
    let from = t.asked().len();
    crate::r_s01_common::attack_with(&mut t, &[(soldier, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hearthfire Hobgoblin"), "{}", t.dump_log());
    // The Bears were never offered.
    for (_, d) in &t.asked()[from..] {
        if let mtg_engine::decision::Decision::ChooseEntities { candidates, .. } = d {
            assert!(!candidates.contains(&obj(lib[0])));
        }
    }
}

#[test]
fn dancers_chakrams_granted_ability_works_on_a_noncommander() {
    cr!("903.3", "613.4c");
    ruling!(
        "Dancer's Chakrams",
        "The last ability granted by Dancer's Chakrams to the equipped creature applies whether or not Dancer's Chakrams is attached to a commander."
    );
    let mut t = TestGame::new(2);
    let commander = crate::r_p073_common::bf_commander(&mut t, P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Dancer's Chakrams", bears);
    assert_eq!(t.pt(commander), (5, 5));
    assert!(has_kw(&t, commander, KeywordKind::Lifelink));
}
