//! Rulings batch P059 — fling effects (sacrificing a creature as a cost, CR 601.2c/h, and
//! using its last known power, CR 608.2h) and flowstone pumps (+N/-N activated abilities
//! whose result is checked by state-based actions, CR 704.5f/g), with Self-Destruct's two
//! targets (CR 608.2b), Bloodfire Infusion's enchant restriction (CR 704.5m), Dromosaur's
//! block triggers (CR 509.3a, 509.3c) and Spatial Contortion's format legality.

use crate::r_p058_common::mana;
use crate::r_p059_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::{can_attack, can_cast, destroy};
use crate::r_s03_common::priority_asked_since;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{activate_containing, attach_new, give_control};
use mtg_engine::card::card;
use mtg_engine::deck::{check_format_legality, DeckProblem};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Fling effects
// ---------------------------------------------------------------------------------------

/// P0 casts Thud sacrificing `sac`, targeting `target`; the spell is left on the stack.
fn thud(t: &mut TestGame, sac: ObjectId, target: Entity) -> ObjectId {
    t.answer_choose(P0, &[o(sac)]);
    cast(t, P0, "Thud", &[target])
}

#[test]
fn thuds_sacrifice_is_paid_before_anyone_can_respond() {
    cr!("601.2h", "601.2i");
    ruling!(
        "Thud",
        "Players can respond only once Thud has been cast and all its costs have been paid. No one can try to destroy the creature you sacrificed to prevent you from casting this spell."
    );
    supported("Thud");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let mark = t.asked().len();
    thud(&mut t, giant, pl(P1));
    assert!(
        priority_asked_since(&t, mark).is_empty(),
        "no priority while casting"
    );
    assert!(t.in_graveyard(P0, "Hill Giant"), "already sacrificed");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn thud_uses_the_sacrificed_creatures_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Thud",
        "The sacrificed creature’s last known existence on the battlefield is checked to determine its power."
    );
    // A Hill Giant (3/3) with Giant Growth (+3/+3) and a +1/+1 counter: 7.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_resolve(&mut t, P0, "Giant Growth", &[o(giant)]);
    put(&mut t, giant, counters::PLUS1, 1);
    assert_eq!(t.pt(giant), (7, 7));
    thud(&mut t, giant, pl(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
}

#[test]
fn thud_needs_exactly_one_creature_to_sacrifice() {
    cr!("601.2h", "118.3");
    ruling!(
        "Thud",
        "You must sacrifice exactly one creature to cast Thud; you can’t cast it without sacrificing a creature, and you can’t sacrifice additional creatures."
    );
    // No creature: it can't be cast.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 1);
    let spell = t.hand(P0, "Thud");
    assert!(!can_cast(&mut t, P0, spell, CastMethod::Normal));
    // Two creatures named: only one is sacrificed.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[o(giant), o(bears)]);
    cast(&mut t, P0, "Thud", &[pl(P1)]);
    let gone = [giant, bears]
        .iter()
        .filter(|c| !t.on_battlefield(**c))
        .count();
    assert_eq!(gone, 1, "exactly one sacrificed");
    t.resolve_all();
    let dealt = 20 - t.life(P1);
    assert!(dealt == 3 || dealt == 2, "{dealt}");
}

#[test]
fn voldaren_thrillseeker_uses_its_last_known_power() {
    cr!("608.2h", "702.165a");
    ruling!(
        "Voldaren Thrillseeker",
        "Use the creature’s power as it last existed on the battlefield to determine how much damage it deals to the target."
    );
    // "Backup 2 ... {1}, Sacrifice this creature: It deals damage equal to its power to
    // any target." Backup on itself: a 3/3; Giant Growth makes it 6/6.
    supported("Voldaren Thrillseeker");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 1);
    let th = t.enter(P0, "Voldaren Thrillseeker");
    t.answer_targets(P0, &[o(th)]);
    t.resolve_all();
    let th = t.g.current(th);
    assert_eq!(t.pt(th), (3, 3), "{}", t.dump_log());
    cast_resolve(&mut t, P0, "Giant Growth", &[o(th)]);
    t.answer_targets(P0, &[pl(P1)]);
    activate_containing(&mut t, P0, th, "Sacrifice").unwrap();
    assert!(t.in_graveyard(P0, "Voldaren Thrillseeker"));
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn self_destruct_with_an_illegal_target() {
    cr!("608.2b", "115.1");
    ruling!(
        "Self-Destruct",
        "If the target creature you control is an illegal target as Self-Destruct tries to resolve, the creature you control won't deal damage. If the creature you control is a legal target but the other target isn't, your creature will still deal damage to itself."
    );
    supported("Self-Destruct");
    // "Target creature you control deals X damage to any other target and X damage to
    // itself, where X is its power." P1 takes control of the Craw Wurm in response: it's
    // no longer a creature P0 controls, so no damage at all.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Self-Destruct", &[o(wurm), o(bears)]);
    give_control(&mut t, wurm, P1);
    t.resolve_all();
    assert!(t.on_battlefield(bears), "no damage to the other target");
    assert_eq!(t.obj_now(wurm).damage, 0, "no damage to itself");
    // The other target is gone: the Wurm (6/4) still deals 6 damage to itself.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Self-Destruct", &[o(wurm), o(bears)]);
    move_to(&mut t, bears, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Craw Wurm"), "{}", t.dump_log());
}

#[test]
fn self_destructs_x_is_its_power_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Self-Destruct",
        "The value of X is calculated only once, as Self-Destruct resolves."
    );
    // Grizzly Bears (2/2) pumped by Giant Growth in response: X is 5 for both.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    cast(&mut t, P0, "Self-Destruct", &[o(bears), o(wall)]);
    cast_resolve(&mut t, P0, "Giant Growth", &[o(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "5 damage to a 5/5");
    assert_eq!(t.obj_now(wall).damage, 5, "5 damage to the 0/8 wall");
}

#[test]
fn bloodfire_infusion_on_a_creature_you_dont_control_is_put_into_the_graveyard() {
    cr!("704.5m", "303.4c");
    ruling!(
        "Bloodfire Infusion",
        "If this card is ever on a creature you don't control, it is put into the graveyard as a State-Based Action."
    );
    supported("Bloodfire Infusion");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Bloodfire Infusion", bears);
    t.settle();
    assert!(
        t.on_battlefield(aura),
        "on a creature its controller controls"
    );
    give_control(&mut t, bears, P1);
    assert!(t.in_graveyard(P0, "Bloodfire Infusion"));
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// Flowstone pumps
// ---------------------------------------------------------------------------------------

/// Activates the ability of `id` containing `needle` and resolves it.
fn pump(t: &mut TestGame, id: ObjectId, needle: &str) {
    activate_containing(t, P0, id, needle).unwrap();
    t.resolve();
}

#[test]
fn flowstone_crusher_dies_once_its_toughness_is_zero() {
    cr!("704.5f", "602.2");
    ruling!(
        "Flowstone Crusher",
        "Flowstone Crusher's ability can be activated multiple times, but Flowstone Crusher is put into the graveyard after an effect resolves that makes its toughness 0."
    );
    supported("Flowstone Crusher");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let crusher = t.battlefield(P0, "Flowstone Crusher");
    for _ in 0..3 {
        pump(&mut t, crusher, "+1/-1");
    }
    assert_eq!(t.pt(crusher), (7, 1));
    pump(&mut t, crusher, "+1/-1");
    assert!(t.in_graveyard(P0, "Flowstone Crusher"));
}

#[test]
fn flowstone_hellion_dies_as_soon_as_its_toughness_is_too_low() {
    cr!("704.5f", "704.5g", "117.5");
    ruling!(
        "Flowstone Hellion",
        "It will be put into its owner's graveyard as soon as one of its abilities resolves that puts its toughness at zero (or less than the current amount of damage on it). This is a state-based action, so there is no way to activate its ability any more to increase its power further before it is put into the graveyard."
    );
    supported("Flowstone Hellion");
    // 3/3 with 1 damage: the second +1/-1 makes it 5/1 with 1 damage.
    let mut t = TestGame::new(2);
    let hellion = t.battlefield(P0, "Flowstone Hellion");
    crate::r_s06_common::damage(&mut t, hellion, 1, hellion);
    pump(&mut t, hellion, "+1/-1");
    assert!(t.on_battlefield(hellion));
    let mark = t.asked().len();
    activate_containing(&mut t, P0, hellion, "+1/-1").unwrap();
    t.g.resolve_top();
    t.settle();
    assert!(t.in_graveyard(P0, "Flowstone Hellion"));
    assert!(
        priority_asked_since(&t, mark).is_empty(),
        "dead before anyone got priority"
    );
    // Toughness 0 without damage: 3 activations.
    let mut t = TestGame::new(2);
    let hellion = t.battlefield(P0, "Flowstone Hellion");
    pump(&mut t, hellion, "+1/-1");
    pump(&mut t, hellion, "+1/-1");
    assert_eq!(t.pt(hellion), (5, 1));
    pump(&mut t, hellion, "+1/-1");
    assert!(t.in_graveyard(P0, "Flowstone Hellion"));
}

#[test]
fn ravenous_bloodseeker_dies_and_its_other_activations_do_nothing() {
    cr!("704.5f", "602.2", "608.2b");
    ruling!(
        "Ravenous Bloodseeker",
        "You can activate Ravenous Bloodseeker's ability any number of times to discard several cards. It will be put into its owner's graveyard once its toughness is 0 or less. Any other instances of the ability still on the stack will resolve with no effect."
    );
    supported("Ravenous Bloodseeker");
    // 1/3: three activations (three discards) on the stack; the second to resolve kills it.
    let mut t = TestGame::new(2);
    let seeker = t.battlefield(P0, "Ravenous Bloodseeker");
    for _ in 0..3 {
        t.hand(P0, "Grizzly Bears");
    }
    for _ in 0..3 {
        activate_containing(&mut t, P0, seeker, "+2/-2").unwrap();
    }
    assert_eq!(t.stack_len(), 3);
    assert_eq!(t.hand_size(P0), 0, "three cards discarded");
    t.resolve();
    assert_eq!(t.pt(seeker), (3, 1));
    t.resolve();
    assert!(t.in_graveyard(P0, "Ravenous Bloodseeker"));
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.stack_len(), 0, "resolved with no effect");
    assert!(t.in_graveyard(P0, "Ravenous Bloodseeker"));
}

#[test]
fn water_servant_can_lower_its_power_below_zero() {
    cr!("107.1b");
    ruling!(
        "Water Servant",
        "You can activate Water Servant's second ability even if doing so would cause Water Servant's power to be less than 0."
    );
    supported("Water Servant");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let servant = t.battlefield(P0, "Water Servant");
    for _ in 0..4 {
        pump(&mut t, servant, "-1/+1");
    }
    assert_eq!(t.pt(servant), (-1, 8));
}

#[test]
fn blood_lusts_toughness_reduction_is_locked_in() {
    cr!("608.2h", "611.2c", "704.5f");
    ruling!(
        "Blood Lust",
        "Has changed slightly in functionality, with the old continuous nature of the toughness reduction gone away; the toughness change remains static. For example, if you have three Swamps and Nightmare on the battlefield and then cast Blood Lust on it, it will become 7/1. If one of your Swamps is subsequently destroyed, your Nightmare will become 6/0 and be put in the graveyard."
    );
    // "If target creature has toughness 5 or greater, it gets +4/-4 until end of turn.
    // Otherwise, it gets +4/-X until end of turn, where X is its toughness minus 1."
    supported("Blood Lust");
    supported("Nightmare");
    let mut t = TestGame::new(2);
    let swamps = t.lands(P0, "Swamp", 3);
    let nightmare = t.battlefield(P0, "Nightmare");
    assert_eq!(t.pt(nightmare), (3, 3));
    cast_resolve(&mut t, P0, "Blood Lust", &[o(nightmare)]);
    assert_eq!(t.pt(nightmare), (7, 1));
    destroy(&mut t, swamps[0]);
    assert!(t.in_graveyard(P0, "Nightmare"), "6/0");
}

#[test]
fn wall_of_wonder_can_attack_only_if_it_isnt_summoning_sick() {
    cr!("302.6", "702.3b");
    ruling!(
        "Wall of Wonder",
        "Paying to make the Wall capable of attacking does not override the normal rule that a creature may not attack unless it began your turn on the battlefield."
    );
    supported("Wall of Wonder");
    for sick in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 4);
        let wall = if sick {
            t.battlefield_sick(P0, "Wall of Wonder")
        } else {
            t.battlefield(P0, "Wall of Wonder")
        };
        pump(&mut t, wall, "+4/-4");
        assert_eq!(t.pt(wall), (5, 1));
        t.set_step(P0, Step::BeginningOfCombat);
        assert_eq!(can_attack(&mut t, wall), !sick, "sick: {sick}");
    }
}

#[test]
fn flowstone_embraces_ability_belongs_to_the_aura() {
    cr!("602.2", "113.6");
    ruling!(
        "Flowstone Embrace",
        "This ability is an ability of the Aura, not an ability granted to the creature. Only the Aura’s controller can activate it."
    );
    // "Enchant creature / {T}: Enchanted creature gets +2/-2 until end of turn." P0's
    // Aura on P1's Hill Giant.
    supported("Flowstone Embrace");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let aura = attach_new(&mut t, P0, "Flowstone Embrace", giant);
    assert!(
        !crate::r_s02_common::can_activate(&mut t, P1, giant),
        "the creature has no such ability"
    );
    assert!(!crate::r_s02_common::can_activate(&mut t, P1, aura));
    assert!(crate::r_s02_common::can_activate(&mut t, P0, aura));
    t.activate(P0, aura, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(aura).tapped, "the Aura taps");
    assert_eq!(t.pt(giant), (5, 1));
}

#[test]
fn dromosaur_gets_the_bonus_each_time_it_blocks_or_becomes_blocked() {
    cr!("509.3a", "509.3c", "506.1");
    ruling!(
        "Dromosaur",
        "If it manages to block or become blocked more than once in a turn, it can get the bonus twice."
    );
    // "Whenever this creature blocks or becomes blocked, it gets +2/-2 until end of turn."
    // Two Glorious Anthems make it 4/5; it's blocked in both combats of P0's turn
    // (Relentless Assault).
    supported("Dromosaur");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    t.battlefield(P0, "Glorious Anthem");
    let dromo = t.battlefield(P0, "Dromosaur");
    let wall = t.battlefield(P1, "Wall of Stone");
    assert_eq!(t.pt(dromo), (4, 5));
    attack_with(&mut t, &[(dromo, pl(P1))]);
    block(&mut t, P1, &[(wall, dromo)]);
    t.resolve_all();
    assert_eq!(t.pt(dromo), (6, 3));
    t.advance_to(P0, Step::PostcombatMain);
    cast_resolve(&mut t, P0, "Relentless Assault", &[]);
    assert!(!t.obj_now(dromo).tapped);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(dromo, pl(P1))]),
    );
    run_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.is_attacking(dromo));
    let wall = t.g.current(wall);
    block(&mut t, P1, &[(wall, dromo)]);
    t.resolve_all();
    assert_eq!(t.pt(dromo), (8, 1), "the bonus twice");
}

#[test]
fn dromosaur_doesnt_retrigger_when_its_blockers_change() {
    cr!("509.1h", "509.3c");
    ruling!(
        "Dromosaur",
        "Switching blockers will not re-trigger this since it only triggers on becoming blocked and switching does not unblock and reblock it."
    );
    // Dromosaur (2/3) is blocked by a Wall of Stone, which then leaves combat; Flash
    // Foliage puts a Saproling onto the battlefield blocking it: still the one bonus.
    supported("Flash Foliage");
    let mut t = TestGame::new(2);
    let dromo = t.battlefield(P1, "Dromosaur");
    let wall = t.battlefield(P0, "Wall of Stone");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(dromo, pl(P0))]);
    block(&mut t, P0, &[(wall, dromo)]);
    t.resolve_all();
    assert_eq!(t.pt(dromo), (4, 1));
    move_to(&mut t, wall, Zone::Hand(P0));
    assert!(t.g.is_attacking(dromo));
    cast(&mut t, P0, "Flash Foliage", &[o(dromo)]);
    t.resolve_all();
    let saproling =
        t.g.permanents()
            .find(|o| o.is_token())
            .map(|o| o.id)
            .expect("a Saproling");
    assert!(t.g.is_blocking(saproling), "{}", t.dump_log());
    assert_eq!(t.pt(dromo), (4, 1), "no second bonus");
    assert!(t.on_battlefield(dromo));
}

#[test]
fn spatial_contortion_cant_be_played_in_pauper() {
    cr!("100.6");
    ruling!(
        "Spatial Contortion",
        "Spatial Contortion is banned in Pauper (not just non-legal), due to a special case on MTGO"
    );
    let deck: Vec<_> = (0..4)
        .map(|_| card("Spatial Contortion"))
        .chain((0..56).map(|_| card("Wastes")))
        .collect();
    let problems = check_format_legality(&deck, &[], "pauper");
    assert!(
        problems.iter().any(|p| matches!(
            p,
            DeckProblem::NotLegalInFormat { name, .. } if name == "Spatial Contortion"
        )),
        "{problems:?}"
    );
    assert!(check_format_legality(&deck, &[], "modern").is_empty());
}
