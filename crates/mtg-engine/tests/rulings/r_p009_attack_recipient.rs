//! Rulings batch P009 — "deals damage to the player or planeswalker [it's / that creature
//! is] attacking" (CR 506.2, 508.1b): attack triggers (CR 508.3a, 603.2), conditions of the
//! trigger event checked only as it triggers (CR 603.2, 603.4), and choices made while
//! resolving (CR 608.2d).

use crate::r_p009_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn attack_triggers_damage_what_the_creature_attacks() {
    cr!("506.2", "508.1b", "603.2");
    for name in [
        "Raid Bombardment",
        "Cavalcade of Calamity",
        "Hellrider",
        "Mage Slayer",
        "Scorch Spitter",
        "Myr Battlesphere",
    ] {
        supported(name);
    }
    // Raid Bombardment: one Bears attacks the player, one attacks the planeswalker.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raid Bombardment");
    let pw = t.battlefield(P1, "Chandra Nalaar");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, pl(P1)), (b, obj(pw))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.counters(pw, counters::LOYALTY), 5);
    // Hellrider ("it's attacking": the attacking creature) and Fathom Fleet Swordjack
    // ("it deals damage to the player or planeswalker it's attacking equal to ...").
    supported("Fathom Fleet Swordjack");
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P1, "Chandra Nalaar");
    let rider = t.battlefield(P0, "Hellrider");
    let jack = t.battlefield(P0, "Fathom Fleet Swordjack");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    attack_with(&mut t, &[(rider, pl(P1)), (jack, obj(pw))]);
    t.resolve_all();
    // Hellrider: 1 to P1 (for itself), 1 to the planeswalker (for the Swordjack); the
    // Swordjack: 2 (two artifacts) to the planeswalker.
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.counters(pw, counters::LOYALTY), 3);
}

#[test]
fn power_is_checked_only_as_the_ability_triggers() {
    cr!("603.2", "603.10");
    ruling!(
        "Cavalcade of Calamity",
        "The power of the attacking creature is checked only when the ability triggers. Once it triggers, Cavalcade of Calamity will deal 1 damage to the appropriate player or planeswalker even if the creature’s power changes or the creature leaves the battlefield before the ability resolves."
    );
    ruling!(
        "Raid Bombardment",
        "The power of the attacking creature is checked only when the ability triggers. Once it triggers, Raid Bombardment will deal 1 damage to the appropriate player or planeswalker even if the creature's power changes before the ability resolves."
    );
    // Raid Bombardment: the Bears gets +3/+3 in response; still 1 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raid Bombardment");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    crate::r_s25_common::cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve(); // Giant Growth
    assert_eq!(t.pt(bears).0, 5);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Cavalcade of Calamity: the 1/1 attacking a planeswalker leaves the battlefield in
    // response; the planeswalker is still dealt 1 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cavalcade of Calamity");
    let pw = t.battlefield(P1, "Chandra Nalaar");
    let elf = t.battlefield(P0, "Llanowar Elves");
    attack_with(&mut t, &[(elf, obj(pw))]);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, elf);
    t.resolve_all();
    assert_eq!(t.counters(pw, counters::LOYALTY), 5);
    // ... and a power-2 creature doesn't trigger it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cavalcade of Calamity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn mage_slayer_resolves_in_declare_attackers_then_combat_damage_follows() {
    cr!("508.3a", "510.2");
    ruling!(
        "Mage Slayer",
        "This ability triggers and resolves in the declare attackers step. The creature will still deal its combat damage as normal later on during the combat phase."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Mage Slayer", obj(bears));
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn myr_battlesphere_taps_any_untapped_myr_as_it_resolves() {
    cr!("608.2d", "107.1c");
    ruling!(
        "Myr Battlesphere",
        "You can tap any untapped Myr you control as the last ability resolves, not just the Myr tokens you created with the first ability. This includes Myr that haven't been under your control since your most recent turn began."
    );
    ruling!(
        "Myr Battlesphere",
        "You choose the value for X as the last ability resolves. You can't choose a value for X that's greater than the number of untapped Myr you control."
    );
    let mut t = TestGame::new(2);
    let sphere = t.battlefield(P0, "Myr Battlesphere");
    let myr = t.battlefield_sick(P0, "Iron Myr");
    let other = t.battlefield(P0, "Iron Myr");
    t.g.tap(other);
    attack_with(&mut t, &[(sphere, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(myr)]);
    t.resolve_all();
    // Only one untapped Myr: X is at most 1.
    assert!(t.obj_now(myr).tapped, "the summoning-sick Myr was tapped");
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.pt(sphere).0, 5);
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseX { max: 1, .. })));
}

#[test]
fn becomes_blocked_trigger_damages_what_it_attacks_not_the_blocker() {
    cr!("506.2", "509.3c");
    supported("Rakdos Roustabout");
    // "it deals 1 damage to the player or planeswalker it's attacking" as it becomes
    // blocked; if it leaves the battlefield first, what it was attacking as it left.
    for dies in [false, true] {
        let mut t = TestGame::new(2);
        let pw = t.battlefield(P1, "Chandra Nalaar");
        let rouse = t.battlefield(P0, "Rakdos Roustabout");
        let wall = t.battlefield(P1, "Wall of Wood");
        attack_with(&mut t, &[(rouse, obj(pw))]);
        crate::r_p148_common::declare_blocks(&mut t, P1, &[(wall, rouse)]);
        assert_eq!(t.stack_len(), 1);
        if dies {
            kill(&mut t, rouse);
        }
        t.resolve_all();
        assert_eq!(t.counters(pw, counters::LOYALTY), 5, "dies: {dies}");
        assert_eq!(t.obj_now(wall).damage, 0, "the blocker isn't dealt damage");
        assert_eq!(t.life(P1), 20);
    }
}

#[test]
fn removed_from_combat_deals_no_damage_but_leaving_the_battlefield_does() {
    cr!("506.4", "603.10");
    ruling!(
        "Fathom Fleet Swordjack",
        "On the other hand, if it remains on the battlefield but leaves combat before its triggered ability resolves, no player or planeswalker is dealt damage."
    );
    ruling!(
        "Raid Bombardment",
        "If the attacking creature leaves the battlefield before Raid Bombardment's triggered ability resolves, Raid Bombardment deals 1 damage to the player or planeswalker that creature was attacking before it left the battlefield."
    );
    // Fathom Fleet Swordjack with two artifacts: removed from combat -> no damage;
    // destroyed -> damage as normal.
    for leaves_battlefield in [false, true] {
        let mut t = TestGame::new(2);
        let jack = t.battlefield(P0, "Fathom Fleet Swordjack");
        t.battlefield(P0, "Ornithopter");
        t.battlefield(P0, "Ornithopter");
        attack_with(&mut t, &[(jack, pl(P1))]);
        assert_eq!(t.stack_len(), 1);
        if leaves_battlefield {
            kill(&mut t, jack);
        } else {
            mtg_engine::combat::remove_from_combat(&mut t.g, jack);
        }
        t.resolve_all();
        let expected = if leaves_battlefield { 18 } else { 20 };
        assert_eq!(
            t.life(P1),
            expected,
            "left the battlefield: {leaves_battlefield}"
        );
    }
    // Raid Bombardment: the attacking Bears dies in response; still 1 damage to P1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raid Bombardment");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P1))]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn attacking_a_battle_or_a_removed_planeswalker_deals_no_damage() {
    cr!("310.5", "506.4c");
    ruling!(
        "Raid Bombardment",
        "Raid Bombardment's ability won't do anything when a creature you control with power 2 or less attacks a battle."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raid Bombardment");
    let battle = t.battlefield(P0, "Invasion of Azgol");
    t.g.objects[battle.0 as usize].choices.player = Some(P1);
    let defense = t.counters(battle, counters::DEFENSE);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, obj(battle))]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(battle, counters::DEFENSE), defense);
    assert_eq!(t.life(P1), 20);
    // Scorch Spitter attacking a planeswalker that leaves the battlefield before the
    // ability resolves: the creature attacks nothing (CR 506.4c); no player is dealt
    // damage.
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P1, "Chandra Nalaar");
    let spitter = t.battlefield(P0, "Scorch Spitter");
    attack_with(&mut t, &[(spitter, obj(pw))]);
    kill(&mut t, pw);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.g.combat.as_ref().unwrap().attacker(spitter).is_some());
}

#[test]
fn hellrider_damages_each_attacking_creatures_own_recipient() {
    cr!("506.2", "508.1b");
    ruling!(
        "Hellrider",
        "For each attacking creature, Hellrider will deal damage to the corresponding player or planeswalker."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Hellrider");
    let pw = t.battlefield(P2, "Chandra Nalaar");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P2)), (c, obj(pw))]);
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P2), 19);
    assert_eq!(t.counters(pw, counters::LOYALTY), 5);
}

#[test]
fn myr_battlesphere_gone_still_taps_myr_and_deals_x_damage() {
    cr!("608.2h", "107.1c");
    ruling!(
        "Myr Battlesphere",
        "If that has happened, Myr Battlesphere won't be able to get the +X/+0 bonus, but it will still deal X damage to the appropriate player or planeswalker."
    );
    let mut t = TestGame::new(2);
    let sphere = t.battlefield(P0, "Myr Battlesphere");
    let a = t.battlefield(P0, "Iron Myr");
    let b = t.battlefield(P0, "Iron Myr");
    attack_with(&mut t, &[(sphere, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, sphere);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.resolve_all();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert_eq!(t.life(P1), 18);
}
