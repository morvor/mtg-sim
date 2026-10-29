//! Rulings batch P039 — "whenever a creature enters" abilities (creaturefall): creatures
//! entering at the same time as the permanent with the ability, who is affected, and
//! "if a creature entered the battlefield under your control this turn".

use crate::r_p039_common::*;
use crate::r_p017_common::enter_copying;
use crate::r_s01_common::{give_mana_for, supported, triggers_on_stack};
use crate::r_s02_common::{can_activate, create_token, destroy, target_candidates};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Rally the Ancestors with X = 6, returning the creature cards `names` from
/// P0's graveyard to the battlefield at the same time; the spell resolves and the
/// triggers are put on the stack (unresolved). Returns the creatures, in order.
fn return_together(t: &mut TestGame, names: &[&str]) -> Vec<ObjectId> {
    supported("Rally the Ancestors");
    let cards: Vec<ObjectId> = names.iter().map(|n| t.graveyard(P0, n)).collect();
    give_mana_for(t, P0, "Rally the Ancestors");
    t.lands(P0, "Wastes", 6);
    let c = t.hand(P0, "Rally the Ancestors");
    t.cast(P0, c).x(6).go();
    t.settle();
    t.g.resolve_top();
    t.g.flush_events();
    t.settle();
    let now: Vec<ObjectId> = cards.iter().map(|c| t.g.current(*c)).collect();
    for c in &now {
        assert!(t.on_battlefield(*c));
    }
    now
}

#[test]
fn beast_kin_ranger_triggers_for_each_creature_entering_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "Beast-Kin Ranger",
        "If Beast-Kin Ranger enters at the same time as one or more other creatures you control, its last ability will trigger for each of those other creatures."
    );
    let mut t = TestGame::new(2);
    let c = return_together(
        &mut t,
        &["Beast-Kin Ranger", "Grizzly Bears", "Grizzly Bears"],
    );
    assert_eq!(triggers_on_stack(&t, "gets +1/+0"), 2);
    t.resolve_all();
    assert_eq!(t.pt(c[0]), (5, 3));
}

#[test]
fn griffin_protector_triggers_for_each_creature_entering_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "Griffin Protector",
        "If Griffin Protector enters under your control at the same time as other creatures, its ability will trigger once for each of those creatures."
    );
    ruling!(
        "Griffin Protector",
        "If more than one creature enters the battlefield at the same time, or if one or more other creatures enter the battlefield at the same time that Griffin Protector enters the battlefield, Griffin Protector's triggered ability triggers for each of them."
    );
    // Entering with it.
    let mut t = TestGame::new(2);
    let c = return_together(
        &mut t,
        &["Griffin Protector", "Grizzly Bears", "Grizzly Bears", "Hill Giant"],
    );
    assert_eq!(triggers_on_stack(&t, "gets +1/+1"), 3);
    t.resolve_all();
    assert_eq!(t.pt(c[0]), (5, 6));
    // Several creatures entering together while it is on the battlefield.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Griffin Protector");
    return_together(&mut t, &["Grizzly Bears", "Grizzly Bears"]);
    t.resolve_all();
    assert_eq!(t.pt(g), (4, 5));
}

#[test]
fn orc_sureshot_triggers_for_a_creature_entering_with_it() {
    cr!("603.6a");
    ruling!(
        "Orc Sureshot",
        "If Orc Sureshot enters the battlefield under your control at the same time as another creature, its ability will trigger for that creature."
    );
    supported("Orc Sureshot");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(a)]);
    t.answer_targets(P0, &[o(b)]);
    return_together(&mut t, &["Orc Sureshot", "Hill Giant", "Llanowar Elves"]);
    assert_eq!(triggers_on_stack(&t, "-1/-1"), 2);
    t.resolve_all();
    assert_eq!((t.pt(a), t.pt(b)), ((1, 1), (1, 1)));
}

#[test]
fn kapsho_kitefins_triggers_for_creatures_entering_with_it() {
    cr!("603.6a");
    ruling!(
        "Kapsho Kitefins",
        "The triggered ability will trigger for any creature entering the battlefield under your control at the same time as Kapsho Kitefins."
    );
    let mut t = TestGame::new(2);
    let foes: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    for f in &foes {
        t.answer_targets(P0, &[o(*f)]);
    }
    return_together(&mut t, &["Kapsho Kitefins", "Grizzly Bears", "Hill Giant"]);
    // Itself and the two others.
    assert_eq!(triggers_on_stack(&t, "tap target creature"), 3);
    t.resolve_all();
    assert!(foes.iter().all(|f| t.obj_now(*f).tapped));
}

#[test]
fn legolas_greenleaf_triggers_for_a_legend_entering_with_it() {
    cr!("603.6a");
    ruling!(
        "Legolas Greenleaf",
        "If Legolas Greenleaf enters the battlefield at the same time as another legendary creature you control, Legolas's Greenleaf's third ability will trigger and put a +1/+1 counter on it."
    );
    supported("Legolas Greenleaf");
    let mut t = TestGame::new(2);
    let c = return_together(
        &mut t,
        &[
            "Legolas Greenleaf",
            "Isamaru, Hound of Konda",
            "Grizzly Bears",
        ],
    );
    t.resolve_all();
    assert_eq!(t.counters(c[0], counters::PLUS1), 1);
}

// ---------------------------------------------------------------------------
// Which creature, which player
// ---------------------------------------------------------------------------

#[test]
fn boltwing_marauder_can_target_the_creature_that_entered() {
    cr!("603.3d", "115.1");
    ruling!(
        "Boltwing Marauder",
        "Any creature on the battlefield can be the target of Boltwing Marauder’s triggered ability, including the one that entered the battlefield and caused the ability to trigger."
    );
    supported("Boltwing Marauder");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Boltwing Marauder");
    let foe = t.battlefield(P1, "Grizzly Bears");
    let from = mark(&t);
    let id = new_card(&mut t, P0, "Hill Giant");
    let giant = enter_quiet(&mut t, P0, id);
    t.answer_targets(P0, &[o(giant)]);
    t.settle();
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1);
    for c in [giant, m, foe] {
        assert!(cands[0].contains(&o(c)));
    }
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 3));
}

#[test]
fn poisonbelly_ogre_makes_the_entering_creatures_controller_lose_life() {
    cr!("603.2");
    ruling!(
        "Poisonbelly Ogre",
        "The controller of the creature that entered loses the life. This may or may not be the same player as Poisonbelly Ogre's controller."
    );
    supported("Poisonbelly Ogre");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Poisonbelly Ogre");
    enter(&mut t, P1, "Grizzly Bears");
    enter(&mut t, P1, "Grizzly Bears");
    enter(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (19, 18, 20));
}

#[test]
fn goldnight_commander_pumps_the_creature_that_entered_too() {
    cr!("611.2c");
    ruling!(
        "Goldnight Commander",
        "The creature that entered will also get +1/+1 until end of turn if it's still on the battlefield when the ability resolves."
    );
    supported("Goldnight Commander");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goldnight Commander");
    let giant = enter(&mut t, P0, "Hill Giant");
    let elf = enter(&mut t, P0, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 5));
    assert_eq!(t.pt(elf), (3, 3));
    assert_eq!(t.pt(g), (4, 4));
}

#[test]
fn aether_flash_damages_creatures_that_cant_be_targeted() {
    cr!("115.10", "702.11b");
    ruling!(
        "Aether Flash",
        "This is not a targeted ability, so it deals damage to creatures that can’t be targeted by spells and abilities."
    );
    supported("Aether Flash");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aether Flash");
    let s = enter(&mut t, P1, "Silhana Ledgewalker");
    t.resolve_all();
    assert!(!t.on_battlefield(s));
    assert!(t.in_graveyard(P1, "Silhana Ledgewalker"));
}

#[test]
fn soul_of_the_harvest_sees_a_nontoken_copy_of_a_token() {
    cr!("111.1", "707.2");
    ruling!(
        "Soul of the Harvest",
        "A creature card that enters as a copy of a token creature is still a nontoken creature. You'll be able to draw a card."
    );
    supported("Soul of the Harvest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul of the Harvest");
    let tok = create_token(&mut t, P0, "Cat");
    t.settle();
    // (The token entering doesn't trigger it: it's a token.)
    assert_eq!(triggers_on_stack(&t, "draw a card"), 0);
    let hand = t.hand_size(P0);
    yes(&mut t, P0, true);
    let c = enter_copying(&mut t, P0, "Clone", Some(tok));
    assert_eq!(t.obj_now(c).chars.name, t.obj_now(tok).chars.name);
    assert!(!t.obj_now(c).is_token());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn in_the_web_of_war_ignores_creatures_you_gain_control_of() {
    cr!("603.6a", "603.2");
    ruling!(
        "In the Web of War",
        "In the Web of War doesn’t affect creatures you take control of which are already on the battlefield."
    );
    supported("In the Web of War");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "In the Web of War");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_named(&mut t, P0, "Act of Treason", &[o(giant)]);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.pt(giant), (3, 3));
    // A creature entering under P0's control does get it.
    let b = enter(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(b), (4, 2));
}

#[test]
fn valor_in_akros_pumps_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Valor in Akros",
        "Valor in Akros's ability affects only creatures you control at the time the ability resolves, including the creature that caused it to trigger. Creatures you begin to control later in the turn won't get +1/+1."
    );
    supported("Valor in Akros");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valor in Akros");
    let a = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let b = enter(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!((t.pt(a), t.pt(b)), ((3, 3), (3, 3)));
    cast_named(&mut t, P0, "Act of Treason", &[o(giant)]);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn blood_seekers_life_loss_is_not_damage_for_bloodthirst() {
    cr!("120.3", "702.54a");
    ruling!(
        "Blood Seeker",
        "Life loss is not the same as damage. Blood Seeker's ability will not cause creatures with bloodthirst to enter with +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Seeker");
    yes(&mut t, P0, true);
    enter(&mut t, P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    let v = cast_named(&mut t, P0, "Bloodrage Vampire", &[]);
    t.resolve_all();
    assert_eq!(t.counters(v, counters::PLUS1), 0);
    // Once P1 is dealt damage, bloodthirst applies.
    let s = cast_named(&mut t, P0, "Shock", &[pl(P1)]);
    let _ = s;
    t.resolve_all();
    let v = cast_named(&mut t, P0, "Bloodrage Vampire", &[]);
    t.resolve_all();
    assert_eq!(t.counters(v, counters::PLUS1), 1);
}

// ---------------------------------------------------------------------------
// "If a creature entered the battlefield under your control this turn"
// ---------------------------------------------------------------------------

#[test]
fn saddled_rimestag_gets_the_bonus_once() {
    cr!("611.3a");
    ruling!(
        "Saddled Rimestag",
        "Saddled Rimestag doesn’t get more than +2/+2 if you had more than one other creature enter the battlefield under your control during a turn."
    );
    supported("Saddled Rimestag");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Saddled Rimestag");
    assert_eq!(t.pt(r), (2, 2));
    enter(&mut t, P0, "Grizzly Bears");
    enter(&mut t, P0, "Grizzly Bears");
    enter(&mut t, P0, "Hill Giant");
    assert_eq!(t.pt(r), (4, 4));
}

#[test]
fn saddled_rimestag_counts_creatures_that_entered_before_it_or_left() {
    cr!("611.3a");
    ruling!(
        "Saddled Rimestag",
        "Saddled Rimestag’s effect applies even if the other creature entered the battlefield under your control before Saddled Rimestag entered the battlefield, and it continues to apply even if that creature leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let b = enter(&mut t, P0, "Grizzly Bears");
    // A creature entering under the opponent's control doesn't count.
    enter(&mut t, P1, "Grizzly Bears");
    let r = enter(&mut t, P0, "Saddled Rimestag");
    assert_eq!(t.pt(r), (4, 4));
    destroy(&mut t, b);
    assert_eq!(t.pt(r), (4, 4));
    // Rimestag entering alone doesn't count for itself.
    let mut t = TestGame::new(2);
    enter(&mut t, P1, "Grizzly Bears");
    let r = enter(&mut t, P0, "Saddled Rimestag");
    assert_eq!(t.pt(r), (2, 2));
}

#[test]
fn saddled_rimestag_checks_each_turn() {
    cr!("611.3a");
    ruling!(
        "Saddled Rimestag",
        "Saddled Rimestag’s effect checks each turn whether it applies during that turn. It doesn’t check only during the turn it enters the battlefield."
    );
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Grizzly Bears");
    let r = enter(&mut t, P0, "Saddled Rimestag");
    assert_eq!(t.pt(r), (4, 4));
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.pt(r), (2, 2));
    // In the opponent's turn, a creature entering under P0's control counts.
    enter(&mut t, P0, "Llanowar Elves");
    assert_eq!(t.pt(r), (4, 4));
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.pt(r), (2, 2));
}

#[test]
fn zhalfirin_decoy_counts_a_creature_that_entered_before_it_and_left() {
    cr!("602.1b");
    ruling!(
        "Zhalfirin Decoy",
        "Zhalfirin Decoy’s ability can be activated even if a creature entered the battlefield under your control only before Zhalfirin Decoy entered the battlefield, and even if that creature has left the battlefield."
    );
    supported("Zhalfirin Decoy");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let d = old(&mut t, P0, "Zhalfirin Decoy");
    assert!(!can_activate(&mut t, P0, d));
    let b = enter(&mut t, P0, "Llanowar Elves");
    destroy(&mut t, b);
    assert!(can_activate(&mut t, P0, d));
    // Entered before the Decoy itself did (the Decoy entering doesn't count: "a
    // creature" other than through its own entry is what's needed here only in that it
    // is a creature entering under your control).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let b = enter(&mut t, P0, "Llanowar Elves");
    destroy(&mut t, b);
    let d = old(&mut t, P0, "Zhalfirin Decoy");
    assert!(can_activate(&mut t, P0, d));
}
