//! Rulings batch P162 — state triggers (CR 603.8) and untap-step limits (CR 502.3).

use crate::r_p162_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Cloudshifts `id` (exiles it and returns it at once) with P0's real Cloudshift.
fn cloudshift(t: &mut TestGame, id: ObjectId) {
    crate::r_s01_common::give_mana_for(t, P0, "Cloudshift");
    let c = t.hand(P0, "Cloudshift");
    t.cast(P0, c).target(id).go();
    t.settle();
    t.g.resolve_top();
    t.settle();
}

#[test]
fn a_state_trigger_sees_a_momentary_state_and_doesnt_recheck() {
    cr!("603.8", "603.4");
    ruling!(
        "Emperor Crocodile",
        "Emperor Crocodile’s ability will trigger if you don’t control another creature, even if it’s only for a brief moment during the resolution of another spell or ability (such as that of Brago, King Eternal)."
    );
    ruling!(
        "Emperor Crocodile",
        "The ability only checks if you control no other creatures at the time it triggers. It does not check again on resolution, so gaining control of a creature before then will not save Emperor Crocodile."
    );
    ruling!(
        "Covetous Dragon",
        "The ability will trigger if you don’t control an artifact, even for a brief moment during the resolution of another spell or ability."
    );
    ruling!(
        "Covetous Dragon",
        "The ability only checks if you control no artifacts at the time it triggers. It does not check again on resolution. So gaining control of an artifact before then will not save the Dragon."
    );
    supported("Emperor Crocodile");
    supported("Covetous Dragon");
    supported("Cloudshift");
    for (card, other) in [
        ("Emperor Crocodile", "Grizzly Bears"),
        ("Covetous Dragon", "Ornithopter"),
    ] {
        let mut t = TestGame::new(2);
        let source = t.battlefield(P0, card);
        let other = t.battlefield(P0, other);
        t.settle();
        assert_eq!(t.stack_len(), 0);
        cloudshift(&mut t, other);
        // The other permanent is back, but the ability triggered while it was in exile.
        assert_eq!(t.zone(t.g.current(other)), Zone::Battlefield, "{card}");
        assert_eq!(triggers_on_stack(&t, "sacrifice"), 1, "{card}");
        // Getting another one before it resolves doesn't help either.
        t.battlefield(P0, "Ornithopter");
        t.resolve_all();
        assert_eq!(t.zone(source), Zone::Graveyard(P0), "{card}");
    }
}

#[test]
fn lurebound_scarecrow_without_a_chosen_color_never_triggers() {
    cr!("603.8", "607.2d", "607.5a");
    ruling!(
        "Lurebound Scarecrow",
        "If you control a Lurebound Scarecrow but no color was chosen for it (perhaps because your Cemetery Puca turned into a Lurebound Scarecrow), the trigger condition will have an undefined value in it. It’ll never trigger, so you won’t have to sacrifice the Scarecrow."
    );
    supported("Lurebound Scarecrow");
    // Put onto the battlefield without the "as enters" choice: no color chosen.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Lurebound Scarecrow");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(s));
    // With a chosen color (red) and no red permanents, it triggers.
    let mut t = TestGame::new(2);
    crate::r_s35_common::pick_options(&mut t, P0, &["Red"]);
    let s = crate::r_s05_common::enter(&mut t, P0, "Lurebound Scarecrow");
    assert_eq!(triggers_on_stack(&t, "sacrifice"), 1);
    t.resolve_all();
    assert_eq!(t.zone(s), Zone::Graveyard(P0));
}

#[test]
fn veiled_crocodile_sees_a_momentarily_empty_hand() {
    cr!("603.8");
    ruling!(
        "Veiled Crocodile",
        "It can trigger even if a player's hand is empty momentarily during the middle of the resolution of a spell or ability."
    );
    supported("Veiled Crocodile");
    supported("Wheel of Fortune");
    let mut t = TestGame::new(2);
    let croc = t.battlefield(P0, "Veiled Crocodile");
    for _ in 0..2 {
        t.hand(P0, "Forest");
        t.hand(P1, "Forest");
    }
    cast_resolve(&mut t, P0, "Wheel of Fortune", &[]);
    assert!(t.hand_size(P0) > 0 && t.hand_size(P1) > 0);
    assert!(t.obj_now(croc).is(CardType::Creature));
    assert_eq!(t.pt(croc), (4, 4));
}

#[test]
fn lurking_jackals_becomes_only_a_creature_for_good() {
    cr!("603.8", "613.1d", "611.2a");
    ruling!(
        "Lurking Jackals",
        "Once it becomes a creature, it is no longer an enchantment."
    );
    ruling!(
        "Lurking Jackals",
        "Once it becomes a creature, it will not change back into an enchantment if your opponent's life total goes back above 10."
    );
    supported("Lurking Jackals");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Lurking Jackals");
    t.g.players[1].life = 10;
    t.settle();
    t.resolve_all();
    let o = t.obj_now(j);
    assert!(o.is(CardType::Creature) && !o.is(CardType::Enchantment));
    assert_eq!(t.pt(j), (3, 2));
    t.g.players[1].life = 15;
    t.settle();
    t.g.recompute();
    let o = t.obj_now(j);
    assert!(o.is(CardType::Creature) && !o.is(CardType::Enchantment));
}

/// Passes from P1's end step to P0's upkeep (P0's untap step happens).
fn untap_step(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
}

fn untapped(t: &TestGame, ids: &[ObjectId]) -> usize {
    ids.iter().filter(|i| !t.obj_now(**i).tapped).count()
}

#[test]
fn damping_field_limits_artifact_creatures_only_in_the_untap_step() {
    cr!("502.3");
    ruling!(
        "Damping Field",
        "Artifact creatures are artifacts. They are affected so only one may untap."
    );
    ruling!(
        "Damping Field",
        "Damping Field does not prevent artifacts being untapped by spells or abilities at other times."
    );
    supported("Damping Field");
    supported("Twiddle");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Damping Field");
    let thopters: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    for &a in &thopters {
        t.g.objects[a.0 as usize].tapped = true;
    }
    untap_step(&mut t);
    assert_eq!(untapped(&t, &thopters), 1);
    // In the main phase, Twiddle untaps one.
    t.advance_to(P0, Step::PrecombatMain);
    let tapped = *thopters.iter().find(|i| t.obj_now(**i).tapped).unwrap();
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    cast_resolve(&mut t, P0, "Twiddle", &[Entity::Object(tapped)]);
    assert!(!t.obj_now(tapped).tapped);
    assert_eq!(untapped(&t, &thopters), 2);
}

#[test]
fn stoic_angel_limits_creatures_redundantly_and_not_other_permanents() {
    cr!("502.3");
    ruling!(
        "Stoic Angel",
        "If multiple Stoic Angels are on the battlefield, their effects are redundant. Each player will still be able to untap no more than one creature during their untap step."
    );
    ruling!(
        "Stoic Angel",
        "Noncreature permanents will untap as normal during their controllers’ untap steps."
    );
    supported("Stoic Angel");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Stoic Angel");
    t.battlefield(P1, "Stoic Angel");
    let bears: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let lands = t.lands(P0, "Forest", 3);
    let ring = t.battlefield(P0, "Mind Stone");
    for &a in bears.iter().chain(&lands).chain([&ring]) {
        t.g.objects[a.0 as usize].tapped = true;
    }
    untap_step(&mut t);
    assert_eq!(untapped(&t, &bears), 1);
    assert_eq!(untapped(&t, &lands), 3);
    assert!(!t.obj_now(ring).tapped);
}
