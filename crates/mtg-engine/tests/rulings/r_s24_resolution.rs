//! Rulings batch S24 — what "you control" means as a spell or ability resolves: counts
//! and checks made on resolution (CR 608.2h), "creatures you control get ..." locked in
//! on resolution (CR 611.2c), intervening "if" clauses (CR 603.4), and static abilities
//! that apply while you control something (CR 611.3a).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::{activate_containing, has_kw};
use crate::r_s24_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_chosen_creature_type_counts_all_permanents_of_that_type_including_kindred() {
    cr!("608.2h", "308.1");
    ruling!(
        "Distant Melody",
        "Note that this spell counts the number of permanents you control of the chosen type, not just the number of creatures you control. It will also take your kindred permanents of the chosen type into account."
    );
    supported("Distant Melody");
    supported("Boggart Shenanigans");
    // "Choose a creature type. Draw a card for each permanent you control of that type."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Piker");
    // Boggart Shenanigans is a Kindred Enchantment — Goblin.
    t.battlefield(P0, "Boggart Shenanigans");
    // An opponent's Goblin isn't one you control.
    t.battlefield(P1, "Goblin Piker");
    t.lands(P0, "Island", 4);
    let melody = t.hand(P0, "Distant Melody");
    let hand = t.hand_size(P0);
    choose_creature_type(&mut t, P0, "Goblin");
    t.cast(P0, melody).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn a_shrines_count_includes_itself() {
    cr!("608.2h", "505.1");
    ruling!(
        "Sanctum of Stone Fangs",
        "Each Shrine has an ability that counts the number of Shrines you control. These abilities include the Shrine they're printed on."
    );
    supported("Sanctum of Stone Fangs");
    supported("Sanctum of Shattered Heights");
    // "At the beginning of your first main phase, each opponent loses X life and you gain
    // X life, where X is the number of Shrines you control."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanctum of Stone Fangs");
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
    // With a second Shrine, two.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanctum of Stone Fangs");
    t.battlefield(P0, "Sanctum of Shattered Heights");
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn allies_are_counted_as_the_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Ondu Cleric",
        "The ability counts the number of Allies you control as it resolves."
    );
    supported("Ondu Cleric");
    // "Whenever this creature or another Ally you control enters, you may gain life equal
    // to the number of Allies you control."
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Ondu Cleric");
    // Another Ally arrives (without triggering it) before the ability resolves.
    t.battlefield(P0, "Makindi Patrol");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn an_ally_ability_affects_only_allies_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Seascape Aerialist",
        "The ability affects only Allies you control at the time the ability resolves."
    );
    supported("Seascape Aerialist");
    // "Whenever this creature or another Ally you control enters, you may have Ally
    // creatures you control gain flying until end of turn."
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let aerialist = enter(&mut t, P0, "Seascape Aerialist");
    let before = t.battlefield(P0, "Makindi Patrol");
    t.resolve_all();
    assert!(has_kw(&t, aerialist, KeywordKind::Flying));
    assert!(has_kw(&t, before, KeywordKind::Flying));
    // An Ally that comes under P0's control later in the turn doesn't gain flying.
    let later = t.battlefield(P0, "Ondu Cleric");
    t.g.recompute();
    assert!(!has_kw(&t, later, KeywordKind::Flying));
}

/// P0 casts Giant Growth on the Hero `name` ("Whenever you cast a spell that targets this
/// creature, creatures you control get +1/+0 until end of turn."): the creatures P0
/// controls as the trigger resolves get +1/+0, a creature arriving later doesn't.
fn hero_pumps_only_creatures_you_control_as_it_resolves(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, name);
    let hero_pt = t.pt(hero);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Forest");
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(hero).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(hero), (hero_pt.0 + 4, hero_pt.1 + 3));
    let lions = t.battlefield(P0, "Savannah Lions");
    t.g.recompute();
    assert_eq!(t.pt(lions), (2, 1));
}

#[test]
fn a_hero_pumps_only_creatures_you_control_as_the_ability_resolves() {
    cr!("611.2c", "603.2");
    ruling!(
        "Hero of the Pride",
        "This effect affects only creatures you control at the time the ability resolves. Creatures you begin to control later in the turn won’t get +1/+0."
    );
    hero_pumps_only_creatures_you_control_as_it_resolves("Hero of the Pride");
}

#[test]
fn a_hero_pumps_only_creatures_you_control_as_the_ability_resolves_straight() {
    cr!("611.2c", "603.2");
    ruling!(
        "Hero of the Games",
        "This effect affects only creatures you control at the time the ability resolves. Creatures you begin to control later in the turn won't get +1/+0."
    );
    hero_pumps_only_creatures_you_control_as_it_resolves("Hero of the Games");
}

#[test]
fn an_invoker_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Lavafume Invoker",
        "Only creatures you control as the activated ability resolves are affected."
    );
    supported("Lavafume Invoker");
    // "{8}: Creatures you control get +3/+0 until end of turn."
    let mut t = TestGame::new(2);
    let invoker = t.battlefield(P0, "Lavafume Invoker");
    t.lands(P0, "Mountain", 8);
    activate_containing(&mut t, P0, invoker, "+3/+0").expect("activate");
    // A creature arriving while the ability is on the stack is affected.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(invoker), (5, 2));
    assert_eq!(t.pt(bears), (5, 2));
    // One arriving after it resolved isn't.
    let lions = t.battlefield(P0, "Savannah Lions");
    t.g.recompute();
    assert_eq!(t.pt(lions), (2, 1));
}

#[test]
fn the_colors_of_permanents_you_control_are_checked_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Dark Temper",
        "This checks the colors of permanents you control as it resolves."
    );
    supported("Dark Temper");
    // "Dark Temper deals 2 damage to target creature. If you control a black permanent,
    // destroy the creature instead." No black permanent as it's cast, one as it resolves.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 3);
    let temper = t.hand(P0, "Dark Temper");
    t.cast(P0, temper).target(wurm).go();
    t.battlefield(P0, "Vampire Nighthawk");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    // A black permanent as it's cast that's gone as it resolves: only 2 damage.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let nighthawk = t.battlefield(P0, "Vampire Nighthawk");
    t.lands(P0, "Mountain", 3);
    let temper = t.hand(P0, "Dark Temper");
    t.cast(P0, temper).target(wurm).go();
    move_to(&mut t, nighthawk, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj_now(wurm).damage, 2);
}

#[test]
fn a_liege_gives_a_creature_of_both_colors_both_bonuses() {
    cr!("613.4c", "105.2");
    ruling!(
        "Balefire Liege",
        "The abilities are separate and cumulative. If another creature you control is both of the listed colors, it will get a total of +2/+2."
    );
    supported("Balefire Liege");
    // "Other red creatures you control get +1/+1. Other white creatures you control get
    // +1/+1." Boros Reckoner is red and white.
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P0, "Balefire Liege");
    let reckoner = t.battlefield(P0, "Boros Reckoner");
    let lions = t.battlefield(P0, "Savannah Lions");
    t.g.recompute();
    assert_eq!(t.pt(reckoner), (5, 5));
    assert_eq!(t.pt(lions), (3, 2));
    // Not itself (although it's red and white).
    assert_eq!(t.pt(liege), (2, 4));
}

#[test]
fn an_archetype_affects_opponents_creatures_whenever_they_entered() {
    cr!("611.3a", "613.1f");
    ruling!(
        "Archetype of Courage",
        "The Archetype’s second ability applies to each creature controlled by any of your opponents, no matter when it entered the battlefield."
    );
    supported("Archetype of Courage");
    // "Creatures your opponents control lose first strike and can't have or gain first
    // strike."
    let mut t = TestGame::new(2);
    let before = t.battlefield(P1, "White Knight");
    t.battlefield(P0, "Archetype of Courage");
    let after = t.enter(P1, "White Knight");
    t.g.recompute();
    assert!(!has_kw(&t, before, KeywordKind::FirstStrike));
    assert!(!has_kw(&t, after, KeywordKind::FirstStrike));
}

/// With `trial` on the battlefield, P0 casts Cartouche of Solidarity on a creature. If
/// `remove` the Trial is destroyed while its return ability is on the stack.
fn trial_returns_only_if_still_on_the_battlefield(trial: &str) {
    supported(trial);
    supported("Cartouche of Solidarity");
    for remove in [false, true] {
        let mut t = TestGame::new(2);
        let tr = t.battlefield(P0, trial);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Plains");
        let cartouche = t.hand(P0, "Cartouche of Solidarity");
        t.cast(P0, cartouche).target(bears).go();
        t.resolve();
        assert!(t.stack_len() >= 1);
        if remove {
            destroy(&mut t, tr);
        }
        t.resolve_all();
        assert_eq!(t.in_hand(P0, trial), !remove);
        assert_eq!(t.in_graveyard(P0, trial), remove);
    }
}

#[test]
fn a_trial_returns_only_if_its_still_on_the_battlefield_as_the_ability_resolves() {
    cr!("400.7", "603.6a");
    ruling!(
        "Trial of Strength",
        "Each Trial has an ability to return to your hand when a Cartouche enters the battlefield under your control. The Trial is returned to its owner’s hand only if it’s on the battlefield as the ability resolves."
    );
    trial_returns_only_if_still_on_the_battlefield("Trial of Strength");
}

#[test]
fn a_trial_returns_only_if_its_still_on_the_battlefield_as_the_ability_resolves_straight() {
    cr!("400.7", "603.6a");
    ruling!(
        "Trial of Zeal",
        "Each Trial has an ability to return to your hand when a Cartouche enters the battlefield under your control. The Trial is returned to its owner's hand only if it's on the battlefield as the ability resolves."
    );
    trial_returns_only_if_still_on_the_battlefield("Trial of Zeal");
}

#[test]
fn an_end_step_power_check_is_an_intervening_if_clause() {
    cr!("603.4", "514.2");
    ruling!(
        "Drumhunter",
        "The first ability has an “intervening ‘if’ clause.” That means (1) the ability won’t trigger at all unless you control a creature with power 5 or greater as your end step begins, and (2) the ability will do nothing if you don’t control a creature with power 5 or greater by the time it resolves. (It doesn’t have to be the same creature as the one that allowed the ability to trigger.) Power-boosting effects that last “until end of turn” will still be in effect when this kind of ability triggers and resolves. An ability like this will trigger a maximum of once per turn, no matter how many applicable creatures you control."
    );
    supported("Drumhunter");
    // "At the beginning of your end step, if you control a creature with power 5 or
    // greater, you may draw a card."
    // No such creature: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drumhunter");
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A Giant Growth-ed Bears (5/5 until end of turn) alone: the boost is still there as
    // the end step begins and as the ability resolves.
    for wurm in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Drumhunter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        if wurm {
            t.battlefield(P0, "Craw Wurm");
        }
        t.battlefield(P0, "Forest");
        let growth = t.hand(P0, "Giant Growth");
        t.cast(P0, growth).target(bears).go();
        t.resolve_all();
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.pt(bears), (5, 5));
        // With a Craw Wurm too, it still triggers only once.
        assert_eq!(t.stack_len(), 1, "wurm: {wurm}");
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "wurm: {wurm}");
    }
    // The Wurm let it trigger and is gone as it resolves, but a different creature with
    // power 5 or greater is there then: it still draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drumhunter");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, wurm, Zone::Hand(P0));
    t.battlefield(P0, "Colossal Dreadmaw");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // No such creature by the time it resolves: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drumhunter");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, wurm, Zone::Hand(P0));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn a_color_check_on_entering_is_an_intervening_if_clause() {
    cr!("603.4");
    ruling!(
        "Rhox Meditant",
        "The “intervening ‘if’ clause” means that (1) the ability won’t trigger at all unless you control a permanent of the specified color, and (2) the ability will do nothing unless you control a permanent of the specified color at the time it resolves."
    );
    supported("Rhox Meditant");
    // "When this creature enters, if you control a green permanent, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    enter(&mut t, P0, "Rhox Meditant");
    assert_eq!(t.stack_len(), 0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    enter(&mut t, P0, "Rhox Meditant");
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, bears, Zone::Hand(P0));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    t.battlefield(P0, "Grizzly Bears");
    enter(&mut t, P0, "Rhox Meditant");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_color_check_on_entering_is_an_intervening_if_clause_straight() {
    cr!("603.4");
    ruling!(
        "Sedraxis Alchemist",
        "The \"intervening 'if' clause\" means that (1) the ability won't trigger at all unless you control a permanent of the specified color, and (2) the ability will do nothing unless you control a permanent of the specified color at the time it resolves."
    );
    supported("Sedraxis Alchemist");
    // "When this creature enters, if you control a blue permanent, return target nonland
    // permanent to its owner's hand."
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Merfolk of the Pearl Trident");
    enter(&mut t, P0, "Sedraxis Alchemist");
    assert_eq!(t.stack_len(), 0);
    let merfolk = t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    enter(&mut t, P0, "Sedraxis Alchemist");
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, merfolk, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    enter(&mut t, P0, "Sedraxis Alchemist");
    t.resolve_all();
    assert!(t.in_hand(P1, "Craw Wurm"));
}

#[test]
fn a_bonus_works_as_long_as_you_control_a_permanent_of_the_color() {
    cr!("611.3a", "604.2");
    ruling!(
        "Cliffrunner Behemoth",
        "As long as at least one permanent you control is the specified color, the ability will \"work\" and grant this creature the bonus. Otherwise, it won't have the bonus."
    );
    supported("Cliffrunner Behemoth");
    // "This creature has haste as long as you control a red permanent. This creature has
    // lifelink as long as you control a white permanent."
    let mut t = TestGame::new(2);
    let behemoth = t.battlefield(P0, "Cliffrunner Behemoth");
    t.battlefield(P1, "Raging Goblin");
    t.battlefield(P1, "Savannah Lions");
    t.g.recompute();
    assert!(!has_kw(&t, behemoth, KeywordKind::Haste));
    assert!(!has_kw(&t, behemoth, KeywordKind::Lifelink));
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Savannah Lions");
    t.g.recompute();
    assert!(has_kw(&t, behemoth, KeywordKind::Haste));
    assert!(has_kw(&t, behemoth, KeywordKind::Lifelink));
    move_to(&mut t, goblin, Zone::Hand(P0));
    t.g.recompute();
    assert!(!has_kw(&t, behemoth, KeywordKind::Haste));
    assert!(has_kw(&t, behemoth, KeywordKind::Lifelink));
}
