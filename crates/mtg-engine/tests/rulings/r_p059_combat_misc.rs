//! Rulings batch P059 — assorted combat rulings: Renegade Silent's phasing (CR 702.26),
//! Zurgo Helmsmasher's damage history (CR 603.10a), Marisi's and Basandra's casting
//! restrictions during combat (CR 101.2, 506.1), Marisi's goad when its controller leaves
//! the game (CR 800.4m), Alela's trigger per damaged player (CR 510.2, 603.2c),
//! Bloodthirsty Blade's control and activation (CR 301.5d, 602.2), and priority after
//! Howling Golem's trigger (CR 508.2, 509.2).

use crate::r_p058_common::{decorate, mana};
use crate::r_p059_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::{can_activate, can_cast, destroy};
use crate::r_s06_common::{attached_to, damage, give_control};
use crate::r_s09_common::{declare, declare_in_next_combat};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).phased_out
}

// ---------------------------------------------------------------------------------------
// Renegade Silent: "At the beginning of your end step, goad up to one target creature you
// don't control and put a +1/+1 counter on this creature. This creature phases out."
// ---------------------------------------------------------------------------------------

#[test]
fn renegade_silent_phases_out_with_its_auras_equipment_and_counters() {
    cr!("702.26g", "702.26d", "702.26a");
    ruling!(
        "Renegade Silent",
        "As Renegade Silent phases out, Auras and Equipment attached to it also phase out at the same time. Those Auras and Equipment will phase in at the same time Renegade Silent does, and they'll phase in still attached to Renegade Silent. Similarly, it will phase in with any counters it had on it."
    );
    supported("Renegade Silent");
    let mut t = TestGame::new(2);
    let silent = t.battlefield(P0, "Renegade Silent");
    let d = decorate(&mut t, P0, silent);
    t.answer_targets(P0, &[]);
    end_step_of(&mut t, P0);
    assert!(phased_out(&t, silent), "{}", t.dump_log());
    assert!(phased_out(&t, d.aura) && phased_out(&t, d.equipment));
    assert_eq!(t.counters(silent, counters::PLUS1), 2);
    t.advance_to(P1, Step::Upkeep);
    assert!(phased_out(&t, silent), "not on P1's turn");
    t.advance_to(P0, Step::Upkeep);
    assert!(!phased_out(&t, silent));
    assert!(!phased_out(&t, d.aura) && !phased_out(&t, d.equipment));
    assert_eq!(attached_to(&t, d.aura), Some(o(silent)));
    assert_eq!(attached_to(&t, d.equipment), Some(o(silent)));
    assert_eq!(t.counters(silent, counters::PLUS1), 2);
    assert_eq!(t.g.current(silent), silent, "the same object");
}

#[test]
fn renegade_silent_phases_in_before_its_controller_untaps_and_can_attack() {
    cr!("702.26a", "502.1", "502.3", "302.6");
    ruling!(
        "Renegade Silent",
        "Permanents phase back in during their controller's untap step, immediately before that player untaps their permanents. Creatures that phase in this way are able to attack and pay a cost of {T} during that turn."
    );
    let mut t = TestGame::new(2);
    let silent = t.battlefield(P0, "Renegade Silent");
    t.g.tap(silent);
    t.answer_targets(P0, &[]);
    end_step_of(&mut t, P0);
    assert!(phased_out(&t, silent));
    t.advance_to(P0, Step::Upkeep);
    assert!(!phased_out(&t, silent));
    assert!(!t.obj_now(silent).tapped, "phased in, then untapped");
    let attacks = declare(&mut t, P0, &[(silent, pl(P1))]);
    assert_eq!(attacks, vec![(silent, pl(P1))], "not summoning sick");
}

#[test]
fn a_stolen_renegade_silent_phases_in_under_its_owners_control() {
    cr!("702.26a", "702.26e", "611.2a", "702.26n");
    ruling!(
        "Renegade Silent",
        "If an opponent gains control of your Renegade Silent until end of turn, it will phase out while they still have control of it. The control-changing effect ends while it is phased out, and it phases in under your control as that opponent's next untap step begins. If they leave the game before their next untap step, Renegade Silent phases in as the next untap step begins after their turn would have begun."
    );
    // P1 casts Threaten on P0's Renegade Silent on P1's turn; it phases out at P1's end
    // step (its trigger is P1's), stays out through P0's turn, and phases in under P0's
    // control as P1's next untap step begins.
    let mut t = TestGame::new(2);
    let silent = t.battlefield(P0, "Renegade Silent");
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Threaten", &[o(silent)]);
    assert_eq!(t.obj_now(silent).controller, P1);
    t.answer_targets(P1, &[]);
    end_step_of(&mut t, P1);
    assert!(phased_out(&t, silent), "phased out under P1's control");
    t.advance_to(P0, Step::Upkeep);
    assert!(phased_out(&t, silent), "not in P0's untap step");
    t.advance_to(P1, Step::Upkeep);
    assert!(!phased_out(&t, silent), "P1's untap step");
    assert_eq!(t.obj_now(silent).controller, P0);

    // Three players: P1 steals it and phases it out, then leaves the game during P2's
    // turn. P1's next turn would have begun after P0's turn, so it phases in as P2's
    // untap step begins.
    let mut t = TestGame::new(3);
    let silent = t.battlefield(P0, "Renegade Silent");
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Threaten", &[o(silent)]);
    t.answer_targets(P1, &[]);
    end_step_of(&mut t, P1);
    assert!(phased_out(&t, silent));
    t.advance_to(P2, Step::Upkeep);
    crate::r_p079_common::concede(&mut t, P1);
    assert!(phased_out(&t, silent));
    t.advance_to(P0, Step::Upkeep);
    assert!(phased_out(&t, silent), "P1's turn hasn't yet been due");
    t.advance_to(P2, Step::Upkeep);
    assert!(!phased_out(&t, silent), "{}", t.dump_log());
    assert_eq!(t.obj_now(silent).controller, P0);
}

// ---------------------------------------------------------------------------------------
// Zurgo Helmsmasher: "Whenever a creature dealt damage by Zurgo this turn dies, put a
// +1/+1 counter on Zurgo."
// ---------------------------------------------------------------------------------------

#[test]
fn zurgo_checks_its_damage_history_whenever_a_creature_dies() {
    cr!("603.10a", "603.2");
    ruling!(
        "Zurgo Helmsmasher",
        "Each time a creature dies, check whether Zurgo had dealt any damage to it at any time during that turn. If so, Zurgo's ability will trigger. It doesn't matter who controlled the creature or whose graveyard it went to."
    );
    let mut t = TestGame::new(2);
    let zurgo = t.battlefield(P0, "Zurgo Helmsmasher");
    // P1's Wall of Stone survives Zurgo's combat damage, and dies later.
    let wall = t.battlefield(P1, "Wall of Stone");
    attack_with(&mut t, &[(zurgo, pl(P1))]);
    block(&mut t, P1, &[(wall, zurgo)]);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(t.obj_now(wall).damage, 7);
    assert_eq!(t.counters(zurgo, counters::PLUS1), 0);
    destroy(&mut t, wall);
    t.resolve_all();
    assert_eq!(t.counters(zurgo, counters::PLUS1), 1);
    // P0's own Hill Giant (owned by P1) dealt 1 damage by Zurgo, then destroyed.
    let giant = t.battlefield(P1, "Hill Giant");
    give_control(&mut t, giant, P0);
    damage(&mut t, zurgo, 1, giant);
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.counters(zurgo, counters::PLUS1), 2);
    // A creature Zurgo didn't damage: nothing.
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(zurgo, counters::PLUS1), 2);
}

// ---------------------------------------------------------------------------------------
// Marisi, Breaker of the Coil and Basandra, Battle Seraph
// ---------------------------------------------------------------------------------------

#[test]
fn marisi_stops_opponents_spells_during_combat_but_not_abilities() {
    cr!("101.2", "506.1", "602.5");
    ruling!(
        "Marisi, Breaker of the Coil",
        "If an effect allows or instructs an opponent to cast a spell during a combat phase, they can't do so. Marisi's restriction takes precedence over that permission."
    );
    ruling!(
        "Marisi, Breaker of the Coil",
        "Opponents can still activate abilities during combat."
    );
    // "Your opponents can't cast spells during combat."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Marisi, Breaker of the Coil");
    t.battlefield(P1, "Vedalken Orrery");
    let fanatic = t.battlefield(P1, "Mogg Fanatic");
    mana(&mut t, P1, 2);
    mana(&mut t, P0, 2);
    let bears = t.hand(P1, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    let mine = t.hand(P0, "Giant Growth");
    // Precombat main phase: P1 may cast the Bears (Orrery) and the Bolt.
    assert!(can_cast(&mut t, P1, bears, CastMethod::Normal));
    assert!(can_cast(&mut t, P1, bolt, CastMethod::Normal));
    for step in [
        Step::BeginningOfCombat,
        Step::DeclareAttackers,
        Step::EndOfCombat,
    ] {
        t.g.combat = None;
        t.set_step(P0, step);
        assert!(!can_cast(&mut t, P1, bears, CastMethod::Normal), "{step:?}");
        assert!(!can_cast(&mut t, P1, bolt, CastMethod::Normal), "{step:?}");
        assert!(can_activate(&mut t, P1, fanatic), "{step:?}: abilities");
        assert!(
            can_cast(&mut t, P0, mine, CastMethod::Normal),
            "Marisi's controller"
        );
    }
}

#[test]
fn basandra_stops_all_spells_during_combat_but_not_abilities() {
    cr!("101.2", "506.1", "702.8a");
    ruling!(
        "Basandra, Battle Seraph",
        "While Basandra, Battle Seraph is on the battlefield, players can’t cast any spells during the combat phase, including permanent spells with flash."
    );
    ruling!(
        "Basandra, Battle Seraph",
        "Players may still activate abilities during combat."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Basandra, Battle Seraph");
    let fanatic = t.battlefield(P1, "Mogg Fanatic");
    mana(&mut t, P0, 2);
    mana(&mut t, P1, 2);
    let viper = t.hand(P1, "Ambush Viper");
    let growth = t.hand(P0, "Giant Growth");
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(
        can_cast(&mut t, P1, viper, CastMethod::Normal),
        "main phase: flash"
    );
    for step in [
        Step::BeginningOfCombat,
        Step::DeclareBlockers,
        Step::EndOfCombat,
    ] {
        t.g.combat = None;
        t.set_step(P0, step);
        assert!(!can_cast(&mut t, P1, viper, CastMethod::Normal), "{step:?}");
        assert!(!can_cast(&mut t, P1, bolt, CastMethod::Normal), "{step:?}");
        assert!(
            !can_cast(&mut t, P0, growth, CastMethod::Normal),
            "{step:?}"
        );
        assert!(can_activate(&mut t, P1, fanatic), "{step:?}");
    }
}

#[test]
fn marisis_goad_lasts_until_its_controllers_turn_would_have_begun() {
    cr!("701.15a", "800.4m");
    ruling!(
        "Marisi, Breaker of the Coil",
        "In a multiplayer game, if you leave the game after Marisi's second ability resolves but before your next turn begins, its effect lasts until your next turn would have begun. It neither expires immediately nor lasts indefinitely."
    );
    // P0's Marisi deals combat damage to P1: P1's Grizzly Bears is goaded. P0 leaves
    // during P1's turn; the Bears must still attack that turn (P2), but not on P1's next
    // turn (after the point P0's turn would have begun).
    let mut t = TestGame::new(3);
    let marisi = t.battlefield(P0, "Marisi, Breaker of the Coil");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(marisi, pl(P1))]);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.g.goaders(bears), vec![P0]);
    t.advance_to(P1, Step::Upkeep);
    crate::r_p079_common::concede(&mut t, P0);
    assert!(t.has_lost(P0));
    assert!(!t.g.goaders(bears).is_empty(), "it doesn't expire at once");
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(declare(&mut t, P1, &[]), vec![(bears, pl(P2))]);
    t.advance_to(P2, Step::Upkeep);
    assert!(!t.g.goaders(bears).is_empty(), "still during P2's turn");
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.goaders(bears).is_empty(), "expired");
    assert!(declare_in_next_combat(&mut t, P1, &[]).is_empty());
}

// ---------------------------------------------------------------------------------------
// Alela, Cunning Conqueror: "Whenever one or more Faeries you control deal combat damage
// to a player, goad target creature that player controls."
// ---------------------------------------------------------------------------------------

#[test]
fn alela_triggers_once_per_damaged_player_per_damage_step() {
    cr!("510.2", "603.2c", "702.4b");
    ruling!(
        "Alela, Cunning Conqueror",
        "Normally, combat damage is dealt all at the same time. In that case, Alela, Cunning Conqueror's last ability triggers once for each player that Faeries you control dealt combat damage to, regardless of how many creatures were dealing that damage. If any of those Faeries have double strike or first strike, the ability will trigger once for each player dealt damage in each combat damage step."
    );
    supported("Alela, Cunning Conqueror");
    // Three Faerie Miscreants: two attack P1, one attacks P2: two triggers.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Alela, Cunning Conqueror");
    let f: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Faerie Miscreant"))
        .collect();
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    attack_with(&mut t, &[(f[0], pl(P1)), (f[1], pl(P1)), (f[2], pl(P2))]);
    t.answer_targets(P0, &[o(b1)]);
    t.answer_targets(P0, &[o(b2)]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.g.goaders(b1), vec![P0]);
    assert_eq!(t.g.goaders(b2), vec![P0]);
    // A double striker (Fireshrieker) and a plain Faerie attacking P1: a trigger in each
    // combat damage step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alela, Cunning Conqueror");
    let a = t.battlefield(P0, "Faerie Miscreant");
    let b = t.battlefield(P0, "Faerie Miscreant");
    let shrieker = t.battlefield(P0, "Fireshrieker");
    t.g.attach(shrieker, o(a));
    t.g.recompute();
    t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1))]);
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.stack_len(), 1, "first-strike damage step");
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 17);
    assert_eq!(
        t.stack_len(),
        1,
        "regular damage step: one trigger for two Faeries"
    );
}

// ---------------------------------------------------------------------------------------
// Bloodthirsty Blade: "Equipped creature gets +2/+0 and is goaded. {1}: Attach this
// Equipment to target creature an opponent controls. Activate only as a sorcery."
// ---------------------------------------------------------------------------------------

#[test]
fn bloodthirsty_blade_has_no_equip_ability_for_your_creatures() {
    cr!("702.6a", "115.1");
    ruling!(
        "Bloodthirsty Blade",
        "Bloodthirsty Blade has no equip ability. There's normally no way to have it attached to your own creature, although certain combinations of other cards make this possible."
    );
    supported("Bloodthirsty Blade");
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Bloodthirsty Blade");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(
        !t.obj_now(blade)
            .chars
            .abilities
            .iter()
            .any(|a| a.text.to_lowercase().starts_with("equip ")),
        "no equip ability"
    );
    let candidates = crate::r_s04_common::ability_targets(&mut t, blade, 0);
    assert!(candidates.contains(&o(theirs)));
    assert!(!candidates.contains(&o(mine)));
}

#[test]
fn bloodthirsty_blade_stays_yours_and_only_you_can_move_it() {
    cr!("301.5d", "602.2", "602.5d");
    ruling!(
        "Bloodthirsty Blade",
        "You still control Bloodthirsty Blade while it's equipped to an opponent's creature. You can move it any time you could cast a sorcery; the equipped creature's controller can't."
    );
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 1);
    mana(&mut t, P1, 1);
    let blade = t.battlefield(P0, "Bloodthirsty Blade");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.activate(P0, blade, 0, &[o(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, blade), Some(o(bears)));
    assert_eq!(t.obj_now(blade).controller, P0);
    assert_eq!(t.pt(bears), (4, 2));
    // Not at instant speed.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_activate(&mut t, P0, blade));
    t.set_step(P0, Step::PostcombatMain);
    t.activate(P0, blade, 0, &[o(ogre)]).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, blade), Some(o(ogre)));
    // P1 can't, even in P1's own main phase.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P1, blade));
}

// ---------------------------------------------------------------------------------------
// Howling Golem: "Whenever this creature attacks or blocks, each player draws a card."
// ---------------------------------------------------------------------------------------

#[test]
fn players_get_priority_after_howling_golems_trigger() {
    cr!("508.2", "509.2", "117.3b");
    ruling!(
        "Howling Golem",
        "After Howling Golem's triggered ability resolves, players can cast spells and activate abilities before blockers are declared if it's attacking, or before damage is dealt if it's blocking."
    );
    supported("Howling Golem");
    // Attacking: after the draws, P1 Bolts it in the declare attackers step.
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P0, "Howling Golem");
    attack_with(&mut t, &[(golem, pl(P1))]);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1, "drew");
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    cast_resolve(&mut t, P1, "Lightning Bolt", &[o(golem)]);
    assert!(t.in_graveyard(P0, "Howling Golem"), "before blockers");
    // Blocking: after the draws, P0 Bolts it in the declare blockers step.
    let mut t = TestGame::new(2);
    let golem = t.battlefield(P1, "Howling Golem");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(giant, pl(P1))]);
    block(&mut t, P1, &[(golem, giant)]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    cast_resolve(&mut t, P0, "Lightning Bolt", &[o(golem)]);
    assert!(t.in_graveyard(P1, "Howling Golem"), "before damage");
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "the Giant was blocked");
}
