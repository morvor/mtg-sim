//! Rulings batch P064 — activated abilities that grant flying: "activate only once each
//! turn" across a control change (CR 602.5b), animated lands and summoning sickness
//! (302.6), costs paid by tapping or sacrificing (602.2, 118.3), conditions checked only
//! on activation (602.5b), and choices made on activation or on resolution.

use crate::r_p064_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_activate, can_attack};
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s20_common::{entered_this_turn, tap_for_mana};
use crate::r_s24_common::enter_together;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Once each turn, control changes
// ---------------------------------------------------------------------------------------

#[test]
fn beetleform_mage_activated_once_this_turn_cant_be_activated_by_its_new_controller() {
    cr!("602.5b", "400.7");
    ruling!(
        "Beetleform Mage",
        "If Beetleform Mage's ability is activated by one player, then another player gains control of it during the same turn, the second player can't activate its ability that turn."
    );
    supported("Beetleform Mage");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Beetleform Mage");
    pool(&mut t, P0, ManaType::G, 1);
    pool(&mut t, P0, ManaType::U, 1);
    activate_and_resolve(&mut t, P0, m, "+2/+2");
    assert_eq!(t.pt(m), (4, 4));
    give_control(&mut t, m, P1);
    pool(&mut t, P1, ManaType::G, 1);
    pool(&mut t, P1, ManaType::U, 1);
    assert!(
        !can_activate(&mut t, P1, m),
        "already activated this turn, by another player"
    );
}

// ---------------------------------------------------------------------------------------
// Lands that become creatures
// ---------------------------------------------------------------------------------------

#[test]
fn cave_of_the_frost_dragon_ignores_lands_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Cave of the Frost Dragon",
        "If Cave of the Frost Dragon enters the battlefield at the same time as one or more other lands, it doesn't take those lands into consideration"
    );
    supported("Cave of the Frost Dragon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Plains");
    let ids = enter_together(
        &mut t,
        &[
            (P0, "Cave of the Frost Dragon"),
            (P0, "Plains"),
            (P0, "Plains"),
        ],
    );
    assert!(
        !t.obj_now(ids[0]).tapped,
        "only one other land was controlled as it entered"
    );
    let cave = t.enter(P0, "Cave of the Frost Dragon");
    assert!(t.obj_now(cave).tapped, "now three other lands");
}

#[test]
fn animated_lands_need_continuous_control_to_attack_or_tap_for_mana() {
    cr!("302.6", "305.2", "613.1d");
    ruling!(
        "Cave of the Frost Dragon",
        "If you turn Cave of the Frost Dragon into a creature but haven't controlled it continuously since your most recent turn began, you won't be able to activate its mana ability or attack with it."
    );
    ruling!(
        "Inkmoth Nexus",
        "If Inkmoth Nexus entered this turn and it's not a creature, it can be tapped for mana. If it becomes a creature that turn, it can't attack or be tapped for mana."
    );
    supported("Inkmoth Nexus");
    // Inkmoth Nexus: tapped for mana while it isn't a creature.
    let mut t = TestGame::new(2);
    let n = entered_this_turn(&mut t, P0, "Inkmoth Nexus");
    assert!(tap_for_mana(&mut t, P0, n, "Add {C}"));
    // A second one becomes a creature the turn it entered.
    let n2 = entered_this_turn(&mut t, P0, "Inkmoth Nexus");
    activate_and_resolve(&mut t, P0, n2, "becomes");
    assert!(crate::r_s20_common::creature_now(&mut t, n2));
    assert!(!tap_for_mana(&mut t, P0, n2, "Add {C}"));
    crate::r_s20_common::to_beginning_of_combat(&mut t, P0);
    assert!(!can_attack(&mut t, n2));

    // Cave of the Frost Dragon, likewise.
    let mut t = TestGame::new(2);
    let cave = entered_this_turn(&mut t, P0, "Cave of the Frost Dragon");
    let cur = t.g.current(cave);
    t.g.objects[cur.0 as usize].tapped = false;
    pool(&mut t, P0, ManaType::W, 5);
    activate_and_resolve(&mut t, P0, cave, "becomes");
    assert!(crate::r_s20_common::creature_now(&mut t, cave));
    assert!(!tap_for_mana(&mut t, P0, cave, "Add {W}"));
    crate::r_s20_common::to_beginning_of_combat(&mut t, P0);
    assert!(!can_attack(&mut t, cave));
}

#[test]
fn inkmoth_nexus_controlled_since_the_turn_began_can_attack_and_tap() {
    cr!("302.6", "305.2");
    ruling!(
        "Inkmoth Nexus",
        "If you've controlled Inkmoth Nexus continuously since your most recent turn began and it becomes a creature, it can attack or be tapped for mana."
    );
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Inkmoth Nexus");
    pool(&mut t, P0, ManaType::C, 1);
    activate_and_resolve(&mut t, P0, n, "becomes");
    crate::r_s20_common::to_beginning_of_combat(&mut t, P0);
    assert!(can_attack(&mut t, n));
    assert!(tap_for_mana(&mut t, P0, n, "Add {C}"));
}

#[test]
fn celestial_colonnade_taps_itself_for_mana_to_pay_its_attack_cost() {
    cr!("508.1g", "508.1h", "508.1i", "702.20b");
    ruling!(
        "Celestial Colonnade",
        "If Celestial Colonnade can't attack unless you pay a cost that includes a mana payment, you may attack with it and tap it to pay for that cost."
    );
    supported("Celestial Colonnade");
    supported("Propaganda");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Celestial Colonnade");
    let plains = t.battlefield(P0, "Plains");
    t.battlefield(P1, "Propaganda");
    pool(&mut t, P0, ManaType::W, 1);
    pool(&mut t, P0, ManaType::U, 1);
    pool(&mut t, P0, ManaType::C, 3);
    activate_and_resolve(&mut t, P0, c, "becomes");
    let attacks = crate::r_s09_common::declare(&mut t, P0, &[(c, Entity::Player(P1))]);
    assert_eq!(attacks, vec![(c, Entity::Player(P1))], "it attacks");
    assert!(t.obj_now(c).tapped, "tapped for mana to pay Propaganda's {{2}}");
    assert!(t.obj_now(plains).tapped);
}

#[test]
fn celestial_colonnade_tapped_for_mana_while_attacking_stays_in_combat() {
    cr!("506.4", "506.4a", "510.1");
    ruling!(
        "Celestial Colonnade",
        "Once Celestial Colonnade has attacked, tapping it for mana won't remove it from combat."
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Celestial Colonnade");
    pool(&mut t, P0, ManaType::W, 1);
    pool(&mut t, P0, ManaType::U, 1);
    pool(&mut t, P0, ManaType::C, 3);
    activate_and_resolve(&mut t, P0, c, "becomes");
    crate::r_s09_common::declare(&mut t, P0, &[(c, Entity::Player(P1))]);
    assert!(tap_for_mana(&mut t, P0, c, "Add {W}"));
    assert!(crate::r_s10_common::attacking(&t, c));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16, "it still dealt its combat damage");
}

#[test]
fn warden_of_the_wall_taps_for_mana_on_other_turns() {
    cr!("605.1a", "302.6");
    ruling!(
        "Warden of the Wall",
        "Warden of the Wall still has the ability to tap for mana when it’s not your turn."
    );
    supported("Warden of the Wall");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Warden of the Wall");
    t.set_step(P1, Step::PrecombatMain);
    assert!(crate::r_s20_common::creature_now(&mut t, w));
    assert!(tap_for_mana(&mut t, P0, w, "Add {C}"));
}

// ---------------------------------------------------------------------------------------
// Costs: tapping and sacrificing
// ---------------------------------------------------------------------------------------

#[test]
fn warden_of_the_inner_sky_taps_itself_and_summoning_sick_creatures() {
    cr!("302.6", "602.2b");
    ruling!(
        "Warden of the Inner Sky",
        "You can tap any three untapped artifacts and/or creatures you control, including Warden of the Inner Sky itself and other creatures you haven't controlled continuously"
    );
    supported("Warden of the Inner Sky");
    let mut t = TestGame::new(2);
    let w = t.battlefield_sick(P0, "Warden of the Inner Sky");
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let b = t.battlefield_sick(P0, "Savannah Lions");
    activate_and_resolve(&mut t, P0, w, "Tap three");
    assert_eq!(t.counters(w, "+1/+1"), 1);
    for id in [w, a, b] {
        assert!(t.obj_now(id).tapped);
    }
}

#[test]
fn shao_jun_taps_summoning_sick_artifact_creatures() {
    cr!("302.6", "602.2b");
    ruling!(
        "Shao Jun",
        "You can tap any two untapped artifacts you control to pay the cost of Shao Jun’s last ability, including artifact creatures you haven’t controlled continuously"
    );
    supported("Shao Jun");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shao Jun");
    t.battlefield_sick(P0, "Ornithopter");
    t.battlefield_sick(P0, "Ornithopter");
    activate_and_resolve(&mut t, P0, s, "Tap two");
    assert_eq!(t.life(P1), 19);
}

#[test]
fn sacrificing_the_source_itself_to_its_own_ability() {
    cr!("602.2b", "608.2b");
    ruling!(
        "Falkenrath Torturer",
        "You can sacrifice Falkenrath Torturer itself to activate its ability."
    );
    ruling!(
        "Dross Hopper",
        "You may sacrifice Dross Hopper to pay for its own ability. If you do, however, it won't gain flying"
    );
    for name in ["Falkenrath Torturer", "Dross Hopper"] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        t.answer_choose(P0, &[obj(c)]);
        activate_containing(&mut t, P0, c, "Sacrifice a creature").unwrap();
        assert!(t.in_graveyard(P0, name), "{name} was sacrificed");
        assert_eq!(t.stack_len(), 1);
        t.resolve_all();
        assert!(t.in_graveyard(P0, name));
    }
}

#[test]
fn falkenrath_torturer_checks_the_sacrificed_creature_as_it_last_existed() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Falkenrath Torturer",
        "The ability checks whether the sacrificed creature as it last existed on the battlefield was a Human."
    );
    supported("Maskwood Nexus");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Falkenrath Torturer");
    t.battlefield(P0, "Maskwood Nexus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    activate_and_resolve(&mut t, P0, f, "Sacrifice a creature");
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(
        t.counters(f, "+1/+1"),
        1,
        "the Bears were every creature type (a Human) on the battlefield"
    );
    assert!(flies(&mut t, f));
}

// ---------------------------------------------------------------------------------------
// Repeated activations
// ---------------------------------------------------------------------------------------

#[test]
fn flying_granting_abilities_can_be_activated_repeatedly() {
    cr!("602.1", "702.9c");
    ruling!(
        "Locthwain Gargoyle",
        "You can activate Locthwain Gargoyle's ability multiple times during one turn, even if it already has flying."
    );
    ruling!(
        "Olivia's Dragoon",
        "You can activate the ability of Olivia’s Dragoon more than once in a turn, even if it already has flying."
    );
    supported("Locthwain Gargoyle");
    supported("Olivia's Dragoon");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Locthwain Gargoyle");
    for _ in 0..2 {
        pool(&mut t, P0, ManaType::C, 4);
        activate_and_resolve(&mut t, P0, g, "+2/+0");
    }
    assert_eq!(t.pt(g), (4, 3));
    assert!(flies(&mut t, g));

    let d = t.battlefield(P0, "Olivia's Dragoon");
    t.hand(P0, "Plains");
    t.hand(P0, "Plains");
    activate_and_resolve(&mut t, P0, d, "Discard");
    activate_and_resolve(&mut t, P0, d, "Discard");
    assert_eq!(t.hand_size(P0), 0, "activated twice");
    assert!(flies(&mut t, d));
}

#[test]
fn goblin_balloon_brigade_regains_flying_after_losing_it() {
    cr!("613.7", "611.2");
    ruling!(
        "Goblin Balloon Brigade",
        "You can activate the ability repeatedly during a turn. This generally has no additional effect, unless some other effect"
    );
    supported("Goblin Balloon Brigade");
    supported("Radjan Spirit");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goblin Balloon Brigade");
    let w = t.battlefield(P1, "Radjan Spirit");
    pool(&mut t, P0, ManaType::R, 1);
    activate_and_resolve(&mut t, P0, g, "gains flying");
    assert!(flies(&mut t, g));
    t.answer_targets(P1, &[obj(g)]);
    activate_and_resolve(&mut t, P1, w, "loses flying");
    assert!(!flies(&mut t, g));
    pool(&mut t, P0, ManaType::R, 1);
    activate_and_resolve(&mut t, P0, g, "gains flying");
    assert!(flies(&mut t, g), "the newer grant wins");
}

// ---------------------------------------------------------------------------------------
// Conditions checked as the ability is activated
// ---------------------------------------------------------------------------------------

#[test]
fn activation_conditions_are_checked_only_on_activation() {
    cr!("602.5b", "608.2b");
    ruling!(
        "Goblin Bird-Grabber",
        "The only time that Goblin Bird-Grabber's ability checks whether you control a creature with flying is the moment you activate that ability."
    );
    ruling!(
        "Groundling Pouncer",
        "It matters that an opponent controls a creature with flying only as you activate this ability."
    );
    supported("Goblin Bird-Grabber");
    supported("Groundling Pouncer");
    // Goblin Bird-Grabber: your flyer leaves in response.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goblin Bird-Grabber");
    assert!(!can_activate(&mut t, P0, g), "no flyer yet");
    let bird = t.battlefield(P0, "Birds of Paradise");
    pool(&mut t, P0, ManaType::R, 1);
    activate_containing(&mut t, P0, g, "gains flying").unwrap();
    crate::r_s02_common::destroy(&mut t, bird);
    t.resolve_all();
    assert!(flies(&mut t, g));

    // Groundling Pouncer: the opponent's flyer leaves in response.
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Groundling Pouncer");
    let bird = t.battlefield(P1, "Birds of Paradise");
    pool(&mut t, P0, ManaType::G, 1);
    activate_containing(&mut t, P0, p, "+1/+3").unwrap();
    crate::r_s02_common::destroy(&mut t, bird);
    t.resolve_all();
    assert_eq!(t.pt(p), (3, 4));
    assert!(flies(&mut t, p));
}

// ---------------------------------------------------------------------------------------
// When choices are made
// ---------------------------------------------------------------------------------------

#[test]
fn skinshifter_mode_is_chosen_on_activation() {
    cr!("700.2", "602.2b");
    ruling!(
        "Skinshifter",
        "You choose the ability's mode when you activate it."
    );
    supported("Skinshifter");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skinshifter");
    pool(&mut t, P0, ManaType::G, 1);
    crate::r_s29_common::choose_modes(&mut t, P0, &[1]);
    let ab = activate_containing(&mut t, P0, s, "Choose one")
        .unwrap()
        .unwrap();
    assert_eq!(crate::r_s26_common::modes_on_stack(&t, ab), vec![1]);
    t.resolve_all();
    assert_eq!(t.pt(s), (2, 2));
    assert!(flies(&mut t, s));
}

/// Activates the ability of `source` containing `needle`, then checks that no option was
/// offered before the ability resolved, and one ("your choice of") as it resolved.
fn choice_on_resolution(t: &mut TestGame, source: ObjectId, needle: &str, pick: &str) {
    let is_choice = |d: &Decision| matches!(d, Decision::ChooseOption { options, .. } if options.iter().any(|o| o.contains("flying")));
    let from = t.asked().len();
    activate_containing(t, P0, source, needle).unwrap();
    assert_eq!(
        t.asked()[from..].iter().filter(|(_, d)| is_choice(d)).count(),
        0,
        "the keyword was chosen on activation"
    );
    crate::r_s35_common::pick_options(t, P0, &[pick]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(
        t.asked()[from..].iter().filter(|(_, d)| is_choice(d)).count(),
        1,
        "no choice as it resolved"
    );
}

#[test]
fn keyword_choices_are_made_as_the_ability_resolves() {
    cr!("608.2d");
    ruling!(
        "Lunar Avenger",
        "You choose whether Lunar Avenger gains flying, first strike, or haste when the activated ability resolves."
    );
    ruling!(
        "Golem Artisan",
        "You don't choose whether the target artifact creature gains flying, trample, or haste until Golem Artisan's second ability resolves."
    );
    supported("Lunar Avenger");
    supported("Golem Artisan");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Lunar Avenger");
    counters(&mut t, a, "+1/+1", 1);
    choice_on_resolution(&mut t, a, "Remove", "flying");
    assert!(flies(&mut t, a));

    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Golem Artisan");
    pool(&mut t, P0, ManaType::C, 2);
    t.answer_targets(P0, &[obj(g)]);
    choice_on_resolution(&mut t, g, "your choice", "flying");
    assert!(flies(&mut t, g));
    let _ = Answer::Default;
}
