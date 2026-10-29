//! Rulings batch P006 — burn and mana: mana that grants haste to what it's spent on
//! (CR 106.6), monocolored hybrid and Phyrexian symbols (CR 107.4e, 107.4f, 202.2d,
//! 202.3), {X}{X} (CR 107.3), "deals combat damage" triggers per damage step (CR 510.2,
//! 510.4), Aura spells with an illegal target (CR 608.3b), convoke (CR 702.51), effects
//! that affect only what's there as they resolve (CR 611.2c), excess damage (CR 120.10,
//! 702.2c), "if it would die this turn, exile it instead" (CR 614.1a), attacking with
//! 0 power (CR 508.1), and spells that can't be countered (CR 101.2, 701.6a).

use crate::r_p006_common::*;
use crate::r_s01_common::{attack_with, supported, tapped_lands};
use crate::r_s06_common::has_kw;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Sacrifices a Generator Servant of P0's for {C}{C}.
fn servant_mana(t: &mut TestGame) {
    let s = t.battlefield(P0, "Generator Servant");
    t.answer_choose(P0, &[obj(s)]);
    assert!(tap_for_mana(t, P0, s, "Add"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

#[test]
fn generator_servant_mana_on_two_creature_spells() {
    cr!("106.6", "601.2h");
    ruling!(
        "Generator Servant",
        "If the mana is spent on two different creature spells, each of those spells (and resulting creatures) will gain haste until end of turn."
    );
    let mut t = TestGame::new(2);
    servant_mana(&mut t);
    t.lands(P0, "Forest", 2);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Grizzly Bears");
    t.cast(P0, a).go();
    t.resolve_all();
    t.cast(P0, b).go();
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert!(has_kw(&t, a, KeywordKind::Haste));
    assert!(has_kw(&t, b, KeywordKind::Haste));
    // A creature cast with other mana doesn't.
    let c = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(!has_kw(&t, c, KeywordKind::Haste));
}

#[test]
fn generator_servant_mana_on_a_kicker_cost() {
    cr!("106.6", "601.2f");
    ruling!(
        "Generator Servant",
        "If a creature spell has generic mana only in its alternative or additional cost, the creature will still gain haste."
    );
    supported("Urborg Skeleton");
    let mut t = TestGame::new(2);
    servant_mana(&mut t);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    let sk = t.hand(P0, "Urborg Skeleton");
    t.cast(P0, sk).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{{C}}{{C}} spent on the kicker");
    assert!(t.on_battlefield(sk));
    assert_eq!(t.counters(sk, counters::PLUS1), 1, "kicked");
    assert!(has_kw(&t, sk, KeywordKind::Haste));
}

#[test]
fn generator_servant_mana_on_a_noncreature_spell() {
    cr!("106.6", "702.122a");
    ruling!(
        "Generator Servant",
        "If the mana is spent on a noncreature spell that becomes a creature later in the turn, that creature won't have haste."
    );
    let mut t = TestGame::new(2);
    servant_mana(&mut t);
    let copter = t.hand(P0, "Smuggler's Copter");
    t.cast(P0, copter).go();
    t.resolve_all();
    let crewer = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(&mut t, P0, copter, &[crewer]));
    t.resolve_all();
    assert!(t.obj_now(copter).is(CardType::Creature));
    assert!(!has_kw(&t, copter, KeywordKind::Haste));
}

#[test]
fn flame_javelin_colors_mana_value_and_payments() {
    cr!("107.4e", "202.2d", "202.3f", "105.2");
    ruling!(
        "Flame Javelin",
        "A card with a monocolored hybrid mana symbol in its mana cost is each of the colors that appears in its mana cost, regardless of what mana was spent to cast it. Thus, Flame Javelin is red even if you spend six green mana to cast it."
    );
    ruling!(
        "Flame Javelin",
        "A card with monocolored hybrid mana symbols in its mana cost has a mana value equal to the highest possible cost it could be cast for. Its mana value never changes. Thus, Flame Javelin has a mana value of 6, even if you spend {R}{R}{R} to cast it."
    );
    ruling!(
        "Flame Javelin",
        "If a cost includes more than one monocolored hybrid mana symbol, you can choose a different way to pay for each symbol. For example, you can pay for Flame Javelin by spending {R}{R}{R}, {2}{R}{R}, {4}{R}, or {6}."
    );
    // Six green: red.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let fj = t.hand(P0, "Flame Javelin");
    let s = t.cast(P0, fj).target(pl(P1)).go();
    assert_eq!(t.obj(s).chars.colors, ColorSet::single(Color::Red));
    assert_eq!(tapped_lands(&t, P0), 6);
    // {R}{R}{R}: mana value 6.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let fj = t.hand(P0, "Flame Javelin");
    let s = t.cast(P0, fj).target(pl(P1)).go();
    assert_eq!(crate::r_s08_common::mana_value(&t, s), 6);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Each way to pay, using all of exactly that many lands.
    for (mountains, wastes) in [(3, 0), (2, 2), (1, 4), (0, 6)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", mountains);
        t.lands(P0, "Wastes", wastes);
        let fj = t.hand(P0, "Flame Javelin");
        let r = t.cast(P0, fj).target(pl(P1)).try_go();
        assert!(r.is_ok(), "{mountains} R + {wastes}: {r:?}");
        assert_eq!(tapped_lands(&t, P0), mountains + wastes);
    }
    // Two Mountains and one Wastes aren't enough.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 1);
    let fj = t.hand(P0, "Flame Javelin");
    assert!(t.cast(P0, fj).target(pl(P1)).try_go().is_err());
}

#[test]
fn lukka_mana_value_is_five_even_paying_life() {
    cr!("107.4f", "202.3f", "702.150a");
    ruling!(
        "Lukka, Bound to Ruin",
        "A hybrid Phyrexian mana symbol contributes 1 toward the mana value of a card, even if life is paid for it. Specifically, Lukka's mana value is always 5."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let lukka = t.hand(P0, "Lukka, Bound to Ruin");
    let s = t.cast(P0, lukka).go();
    assert_eq!(t.life(P0), 18, "life paid for {{R/G/P}}");
    assert_eq!(crate::r_s08_common::mana_value(&t, s), 5);
    t.resolve_all();
    assert!(t.on_battlefield(lukka));
    assert_eq!(crate::r_s08_common::mana_value(&t, lukka), 5);
}

#[test]
fn bonfire_of_the_damned_x_is_paid_twice() {
    cr!("107.3a", "107.3f");
    ruling!(
        "Bonfire of the Damned",
        "A mana cost of {X}{X} means that you pay twice X. If you want X to be 3, you pay {6}{R} to cast Bonfire of the Damned."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let b = t.hand(P0, "Bonfire of the Damned");
    assert!(t.cast(P0, b).target(pl(P1)).x(3).try_go().is_err(), "six lands");
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P1, "Hill Giant");
    t.cast(P0, b).target(pl(P1)).x(3).go();
    assert_eq!(tapped_lands(&t, P0), 7);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(in_graveyard(&t, bears));
}

#[test]
fn five_alarm_fire_once_per_damage_step() {
    cr!("510.2", "510.4", "702.19c", "702.4b");
    ruling!(
        "Five-Alarm Fire",
        "Five-Alarm Fire’s first ability triggers once for each time a creature you control deals combat damage, even if it deals combat damage to more than one creature, planeswalker, or player."
    );
    supported("Five-Alarm Fire");
    // A trampler dealing damage to its blocker and the player: one counter.
    let mut t = TestGame::new(2);
    let fire = t.battlefield(P0, "Five-Alarm Fire");
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(maw, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(bears, maw)]));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "4 trampled over");
    assert_eq!(t.counters(fire, "blaze"), 1);
    // A double striker: two counters.
    let mut t = TestGame::new(2);
    let fire = t.battlefield(P0, "Five-Alarm Fire");
    let ace = t.battlefield(P0, "Fencing Ace");
    attack_with(&mut t, &[(ace, pl(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.counters(fire, "blaze"), 2);
}

#[test]
fn lightning_diadem_with_an_illegal_target_doesnt_enter() {
    cr!("608.3b", "303.4a");
    ruling!(
        "Lightning Diadem",
        "If Lightning Diadem's target is an illegal target when it tries to resolve, it won't resolve and won't enter the battlefield. Its triggered ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 6);
    let d = t.hand(P0, "Lightning Diadem");
    let d = t.cast(P0, d).target(obj(bears)).go();
    crate::r_p046_common::bounce(&mut t, bears);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert!(in_graveyard(&t, d));
    assert_eq!(t.life(P1), 20, "no enters trigger");
}

#[test]
fn joyful_stormsculptor_triggers_on_convoke_without_convoking() {
    cr!("702.51a", "603.2");
    ruling!(
        "Joyful Stormsculptor",
        "Joyful Stormsculptor’s last ability will trigger whether you tapped any creatures to pay for the spell or not, as long as it has convoke."
    );
    let mut t = TestGame::new(2);
    let js = t.battlefield(P0, "Joyful Stormsculptor");
    t.lands(P0, "Mountain", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let stoke = t.hand(P0, "Stoke the Flames");
    let s = t.cast(P0, stoke).target(obj(bears)).go();
    assert!(t.obj(s).stack.as_ref().unwrap().cast.convoked.is_empty());
    assert!(!t.obj_now(js).tapped);
    t.settle();
    assert_eq!(triggers_from(&t, js), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn pump_all_affects_only_creatures_there_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Rally the Peasants",
        "Only creatures you control when Rally the Peasants resolves will be affected. Creatures that enter the battlefield or that you gain control of later in the turn will not."
    );
    ruling!(
        "Gnawing Crescendo",
        "Only creatures you control when Gnawing Crescendo resolves will get +2/+0. However, if a nontoken creature enters the battlefield after Gnawing Crescendo resolves and then dies while under your control later that turn, you'll create a Rat token."
    );
    supported("Gnawing Crescendo");
    // Rally the Peasants.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 3);
    let rally = t.hand(P0, "Rally the Peasants");
    t.cast(P0, rally).go();
    t.resolve_all();
    let later = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let threaten = t.hand(P0, "Threaten");
    t.cast(P0, threaten).target(obj(theirs)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(theirs).controller, P0);
    assert_eq!(t.pt(a), (4, 2));
    assert_eq!(t.pt(later), (2, 2));
    assert_eq!(t.pt(theirs), (3, 3));
    // Gnawing Crescendo.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let gc = t.hand(P0, "Gnawing Crescendo");
    t.cast(P0, gc).go();
    t.resolve_all();
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(a), (4, 2));
    assert_eq!(t.pt(later), (3, 3));
    crate::r_s02_common::destroy(&mut t, later);
    t.resolve_all();
    let rats = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Rat"))
        .count();
    assert_eq!(rats, 1);
}

fn clues(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == P0 && o.chars.has_subtype("Clue"))
        .count()
}

#[test]
fn torch_the_witness_excess_counts_earlier_damage() {
    cr!("120.10", "120.6");
    ruling!(
        "Torch the Witness",
        "A creature has been dealt excess damage if one or more sources dealt more damage to it than the minimum amount of damage required to be lethal damage. In most cases, this means damage greater than its toughness, but consider the damage already dealt to it this turn."
    );
    supported("Torch the Witness");
    for (earlier, clue) in [(0, 0), (2, 1)] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        if earlier > 0 {
            let src = t.battlefield(P0, "Grizzly Bears");
            crate::r_s06_common::damage(&mut t, src, earlier, wurm);
        }
        t.lands(P0, "Mountain", 3);
        let torch = t.hand(P0, "Torch the Witness");
        t.cast(P0, torch).target(obj(wurm)).x(2).go();
        t.resolve_all();
        assert!(in_graveyard(&t, wurm));
        assert_eq!(clues(&t), clue, "with {earlier} earlier damage");
    }
}

#[test]
fn torch_the_witness_with_deathtouch() {
    cr!("120.10", "702.2c");
    ruling!(
        "Torch the Witness",
        "Even 1 damage dealt to a creature from a source with deathtouch is considered lethal damage."
    );
    supported("Judith, Carnage Connoisseur");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Judith, Carnage Connoisseur");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 2);
    let torch = t.hand(P0, "Torch the Witness");
    t.cast(P0, torch).target(obj(wurm)).x(1).go();
    t.resolve_all();
    assert!(in_graveyard(&t, wurm), "2 deathtouch damage");
    assert_eq!(clues(&t), 1, "2 damage is excess over 1");
}

#[test]
fn nahiri_s_warcrafting_excess_counts_earlier_damage() {
    cr!("120.10");
    ruling!(
        "Nahiri's Warcrafting",
        "A creature has been dealt excess damage if Nahiri's Warcrafting deals more damage to it than the minimum amount of damage required to be lethal damage. In most cases, this means damage greater than the creature's toughness, but consider any damage that was already dealt to it that turn."
    );
    supported("Nahiri's Warcrafting");
    for (earlier, looked) in [(0, 1), (2, 3)] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        if earlier > 0 {
            let src = t.battlefield(P0, "Grizzly Bears");
            crate::r_s06_common::damage(&mut t, src, earlier, wurm);
        }
        t.lands(P0, "Mountain", 3);
        let nw = t.hand(P0, "Nahiri's Warcrafting");
        t.cast(P0, nw).target(obj(wurm)).go();
        let from = mark(&t);
        t.resolve_all();
        let c = choices_since(&t, P0, from);
        assert!(!c.is_empty(), "asked to exile one");
        assert_eq!(c[0].0.len(), looked, "with {earlier} earlier damage");
    }
}

#[test]
fn flame_blessed_bolt_exiles_if_it_dies_later_this_turn() {
    cr!("614.1a", "614.6");
    ruling!(
        "Flame-Blessed Bolt",
        "A creature or planeswalker dealt damage this way will be exiled instead of dying if it dies for any reason later in the turn, even if it initially survived the 2 damage."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let fbb = t.hand(P0, "Flame-Blessed Bolt");
    t.cast(P0, fbb).target(obj(wurm)).go();
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    crate::r_s02_common::destroy(&mut t, wurm);
    assert_eq!(t.zone(wurm), Zone::Exile);
}

#[test]
fn zero_power_creatures_can_attack_and_trigger() {
    cr!("508.1a", "302.6");
    ruling!(
        "Raid Bombardment",
        "A creature with 0 power can attack, as long as it doesn't also have defender."
    );
    ruling!(
        "Cavalcade of Calamity",
        "A creature with power 0 or less can attack, as long as no rule or effect prohibits it from doing so."
    );
    // Raid Bombardment: Ornithopter (0/2) attacks.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raid Bombardment");
    let orni = t.battlefield(P0, "Ornithopter");
    attack_with(&mut t, &[(orni, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Cavalcade of Calamity: Ornithopter at -1 power attacks.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cavalcade of Calamity");
    let orni = t.battlefield(P0, "Ornithopter");
    crate::r_p057_common::pump(&mut t, orni, -1, 0);
    assert_eq!(t.pt(orni), (-1, 2));
    attack_with(&mut t, &[(orni, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // A 0/7 with defender can't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raid Bombardment");
    let wall = t.battlefield(P0, "Wall of Junk");
    assert!(!crate::r_s02_common::can_attack(&mut t, wall));
}

#[test]
fn unleash_the_inferno_zero_excess_doesnt_trigger() {
    cr!("120.10", "603.12");
    ruling!(
        "Unleash the Inferno",
        "A permanent cannot be dealt 0 excess damage."
    );
    supported("Unleash the Inferno");
    // 7 damage to a 0/7: no excess, so an artifact with mana value 0 survives.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Junk");
    let orni = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let u = t.hand(P0, "Unleash the Inferno");
    t.cast(P0, u).target(obj(wall)).go();
    t.answer_targets(P0, &[obj(orni)]);
    t.resolve_all();
    assert!(in_graveyard(&t, wall));
    assert!(t.on_battlefield(orni));
    // 7 damage to Grizzly Bears: excess 5.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let orni = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let u = t.hand(P0, "Unleash the Inferno");
    t.cast(P0, u).target(obj(bears)).go();
    t.answer_targets(P0, &[obj(orni)]);
    t.resolve_all();
    assert!(in_graveyard(&t, bears));
    assert!(in_graveyard(&t, orni));
}

#[test]
fn cant_be_countered_spells_still_suffer_the_counterspell_s_other_effects() {
    cr!("101.2", "701.6a", "608.2b");
    ruling!(
        "Chandra, Awakened Inferno",
        "A spell or ability that counters spells can still target Chandra. When that spell or ability resolves, Chandra won't be countered, but any additional effects of that spell or ability will still happen."
    );
    ruling!(
        "Fry",
        "A spell or ability that counters spells can still target Fry. When that spell or ability resolves, Fry won’t be countered, but any additional effects of that spell or ability will still happen."
    );
    supported("Countersquall");
    supported("Undermine");
    // Chandra and Countersquall ("Its controller loses 2 life").
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let chandra = t.hand(P0, "Chandra, Awakened Inferno");
    let chandra = t.cast(P0, chandra).go();
    t.lands(P1, "Island", 1);
    t.lands(P1, "Swamp", 1);
    let cs = t.hand(P1, "Countersquall");
    t.cast(P1, cs).target(obj(chandra)).go();
    t.resolve(); // Countersquall
    assert_eq!(t.zone(chandra), Zone::Stack, "not countered");
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert!(t.on_battlefield(chandra));
    // Fry and Undermine ("Its controller loses 3 life").
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Serra Angel");
    t.lands(P0, "Mountain", 2);
    let fry = t.hand(P0, "Fry");
    let fry = t.cast(P0, fry).target(obj(target)).go();
    t.lands(P1, "Island", 2);
    t.lands(P1, "Swamp", 1);
    let um = t.hand(P1, "Undermine");
    t.cast(P1, um).target(obj(fry)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert!(in_graveyard(&t, target), "Fry resolved");
}
