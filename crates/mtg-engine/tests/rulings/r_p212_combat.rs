//! Rulings batch P212 — Equipment (CR 301.5, 702.6) in combat: triggers on blocks and
//! attacks (CR 509.3, 508.3), what the defending player is (CR 506.2), damage
//! replacement and prevention stacking (CR 614, 615, 616.1), continuous effects that
//! change as the equipped creature taps (CR 611.3a), and attack/block requirements
//! (CR 508.1d, 509.1c).

use crate::r_p210_common::deal;
use crate::r_p212_common::*;
use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::*;
use crate::r_s30_common::damage_events;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Mirror Shield: "Whenever a creature with deathtouch blocks or becomes blocked by this
// creature, destroy that creature."
// ---------------------------------------------------------------------------------------

#[test]
fn mirror_shield_destroys_the_deathtouch_blocker_before_damage_and_stays_blocked() {
    cr!("509.3c", "509.1h", "506.4", "702.19e");
    ruling!(
        "Mirror Shield",
        "If a creature with deathtouch blocks the equipped creature and is then destroyed, the equipped creature remains blocked. Unless the equipped creature has trample, it won’t deal combat damage to the player or planeswalker it’s attacking."
    );
    ruling!(
        "Mirror Shield",
        "The ability Mirror Shield grants triggers and resolves before combat damage is dealt. That creature won’t deal combat damage to the equipped creature or be dealt damage by it."
    );
    supported("Mirror Shield");
    for trample in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, "Mirror Shield", bears);
        if trample {
            grant(&mut t, bears, KeywordKind::Trample);
        }
        let rats = t.battlefield(P1, "Typhoid Rats");
        to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(rats, bears)]);
        // The trigger is on the stack in the declare blockers step.
        assert_eq!(t.stack_len(), 1);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Typhoid Rats"));
        t.advance_to(P0, Step::EndOfCombat);
        assert!(t.on_battlefield(bears));
        assert_eq!(t.obj_now(bears).damage, 0);
        // No combat damage between the two creatures.
        assert!(damage_events(&t)
            .iter()
            .all(|e| e.1 != Entity::Object(bears) && e.0 != rats));
        assert_eq!(t.life(P1), if trample { 18 } else { 20 });
    }
}

#[test]
fn mirror_shield_doesnt_trigger_when_the_blocker_gains_deathtouch_later() {
    cr!("509.3c", "603.2");
    ruling!(
        "Mirror Shield",
        "Once a creature without deathtouch has blocked or become blocked by the equipped creature, causing the other creature to gain deathtouch won’t cause the ability granted by Mirror Shield to trigger."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Mirror Shield", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(giant, bears)]);
    assert_eq!(t.stack_len(), 0);
    grant(&mut t, giant, KeywordKind::Deathtouch);
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.on_battlefield(giant));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------------------
// Attack and block triggers
// ---------------------------------------------------------------------------------------

#[test]
fn trepanation_blade_the_planeswalkers_controller_is_the_defending_player() {
    cr!("506.2", "508.5");
    ruling!(
        "Trepanation Blade",
        "If the equipped creature is attacking a planeswalker, the controller of the planeswalker is the defending player."
    );
    supported("Trepanation Blade");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Trepanation Blade", bears);
    let jace = t.battlefield(P2, "Jace, the Mind Sculptor");
    for p in [P0, P1, P2] {
        stack_library(&mut t, p, &["Grizzly Bears", "Grizzly Bears", "Forest"]);
    }
    attack_with(&mut t, &[(bears, Entity::Object(jace))]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P2), 3);
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.pt(bears), (5, 2));
}

#[test]
fn infiltration_lens_triggers_for_each_blocker() {
    cr!("509.3c", "603.2c");
    ruling!(
        "Infiltration Lens",
        "If the equipped creature becomes blocked by multiple creatures, Infiltration Lens’s ability triggers that many times."
    );
    supported("Infiltration Lens");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Infiltration Lens", bears);
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(a, bears), (b, bears)]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn thirsting_axe_sacrifices_unless_it_dealt_combat_damage_to_a_creature() {
    cr!("603.4", "510.2");
    ruling!(
        "Thirsting Axe",
        "If the equipped creature dealt combat damage only to a player or planeswalker, Thirsting Axe’s triggered ability triggers. The same is true if the creature didn’t deal combat damage at all."
    );
    supported("Thirsting Axe");
    // 0: attacks unblocked; 1: doesn't attack; 2: blocked by a creature (control).
    for case in 0..3 {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P0, "Craw Wurm");
        attach_new(&mut t, P0, "Thirsting Axe", wurm);
        let elves = t.battlefield(P1, "Llanowar Elves");
        match case {
            0 => {
                attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
                block_and_finish(&mut t, P1, &[]);
                assert_eq!(t.life(P1), 10);
            }
            1 => {}
            _ => {
                attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
                block_and_finish(&mut t, P1, &[(elves, wurm)]);
            }
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.in_graveyard(P0, "Craw Wurm"), case != 2, "case {case}");
    }
}

#[test]
fn armory_of_iroas_counter_arrives_before_combat_damage() {
    cr!("508.3a", "510.1");
    ruling!(
        "Armory of Iroas",
        "The +1/+1 counter is put on the equipped creature before combat damage is dealt."
    );
    supported("Armory of Iroas");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Armory of Iroas", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    // The Bears dealt 3 damage: the Giant dies.
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn mask_of_riddles_on_an_opponents_creature_still_draws_for_you() {
    cr!("603.3a", "301.5c");
    ruling!(
        "Mask of Riddles",
        "If a Mask of Riddles you control somehow winds up equipping a creature you don't control, Mask of Riddles's triggered ability will still trigger whenever that creature deals combat damage to a player (even if that player is you). You will control that ability, so you may draw a card."
    );
    supported("Mask of Riddles");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Mask of Riddles", bears);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1);
}

// ---------------------------------------------------------------------------------------
// Damage replacement and prevention
// ---------------------------------------------------------------------------------------

#[test]
fn inquisitors_flails_multiply() {
    cr!("614.1a", "616.1");
    ruling!(
        "Inquisitor's Flail",
        "If a creature is equipped with a second Inquisitor’s Flail, combat damage dealt by and dealt to that creature will be multiplied by four. A third Inquisitor’s Flail would multiply the combat damage by eight, and so on."
    );
    supported("Inquisitor's Flail");
    // Dealt by: Grizzly Bears (2) unblocked.
    for (n, life) in [(1, 16), (2, 12), (3, 4)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        for _ in 0..n {
            attach_new(&mut t, P0, "Inquisitor's Flail", bears);
        }
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        block_and_finish(&mut t, P1, &[]);
        assert_eq!(t.life(P1), life, "{n} Flails");
    }
    // Dealt to: Llanowar Elves (1) blocks a Craw Wurm (6/4) with two Flails: 4 damage.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    attach_new(&mut t, P0, "Inquisitor's Flail", wurm);
    attach_new(&mut t, P0, "Inquisitor's Flail", wurm);
    let elves = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(elves, wurm)]);
    assert!(damage_events(&t)
        .iter()
        .any(|e| e.1 == Entity::Object(wurm) && e.2 == 4));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
}

#[test]
fn shields_of_the_realm_add_up() {
    cr!("615.7", "616.1");
    ruling!(
        "Shield of the Realm",
        "If a creature is equipped with two Shields of the Realm, 4 damage will be prevented. Three Shields of the Realm will prevent 6 damage, and so on."
    );
    supported("Shield of the Realm");
    for (n, dealt) in [(1, 3), (2, 5), (3, 7)] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P0, "Craw Wurm");
        for _ in 0..n {
            attach_new(&mut t, P0, "Shield of the Realm", wurm);
        }
        let shocker = t.battlefield(P1, "Prodigal Pyromancer");
        deal(&mut t, shocker, dealt, wurm);
        assert_eq!(t.obj_now(wurm).damage, 1, "{n} Shields");
    }
}

#[test]
fn shield_of_the_avatar_the_damaged_creatures_controller_orders_the_effects() {
    cr!("616.1", "615.7");
    ruling!(
        "Shield of the Avatar",
        "If multiple effects modify how damage is dealt, the player being dealt damage or the controller of the permanent being dealt damage chooses the order to apply the effects."
    );
    supported("Shield of the Avatar");
    supported("Dictate of the Twin Gods");
    // Craw Wurm (6/4) with the Shield, and one other creature: prevent 2. Dictate doubles.
    // 3 damage: prevent first → 2; double first → 4 (lethal).
    for prevent_first in [true, false] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P0, "Craw Wurm");
        attach_new(&mut t, P0, "Shield of the Avatar", wurm);
        t.battlefield(P0, "Llanowar Elves");
        t.battlefield(P1, "Dictate of the Twin Gods");
        let shocker = t.battlefield(P1, "Prodigal Pyromancer");
        respond_order(&mut t, prevent_first);
        deal(&mut t, shocker, 3, wurm);
        let marked = damage_events(&t)
            .iter()
            .filter(|e| e.1 == Entity::Object(wurm))
            .map(|e| e.2)
            .sum::<u32>();
        assert_eq!(marked, if prevent_first { 2 } else { 4 });
        assert_eq!(t.on_battlefield(wurm), prevent_first);
    }
}

/// P0 (the damaged creature's controller) orders the replacement/prevention effects:
/// Shield of the Avatar's first or Dictate of the Twin Gods's first.
fn respond_order(t: &mut TestGame, prevent_first: bool) {
    use mtg_engine::decision::{Answer, Decision};
    fn pick(d: &Decision, prevent: bool) -> Option<Answer> {
        match d {
            Decision::ChooseReplacement { options } => options
                .iter()
                .position(|o| o.contains("prevent") == prevent)
                .map(Answer::Index),
            _ => None,
        }
    }
    let f: fn(&mtg_engine::game::Game, &Decision) -> Option<Answer> = if prevent_first {
        |_, d| pick(d, true)
    } else {
        |_, d| pick(d, false)
    };
    crate::r_s03_common::respond(t, P0, f);
}

// ---------------------------------------------------------------------------------------
// Sword of the Paruns
// ---------------------------------------------------------------------------------------

#[test]
fn sword_of_the_paruns_losing_the_toughness_bonus_can_make_damage_lethal() {
    cr!("611.3a", "704.5g");
    ruling!(
        "Sword of the Paruns",
        "If equipped creature becomes tapped, untapped creatures you control will immediately lose the +0/+2 bonus. This may cause nonlethal damage already dealt to them that turn to become lethal."
    );
    supported("Sword of the Paruns");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = attach_new(&mut t, P0, "Sword of the Paruns", bears);
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert_eq!(t.pt(elves), (1, 3));
    let shocker = t.battlefield(P1, "Prodigal Pyromancer");
    deal(&mut t, shocker, 2, elves);
    assert!(t.on_battlefield(elves));
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    activate_containing(&mut t, P0, sword, "tap or untap").unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}
