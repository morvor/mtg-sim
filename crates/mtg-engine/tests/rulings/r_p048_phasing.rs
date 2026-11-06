//! Rulings batch P048 — phasing (CR 702.26): Blink Dog's Teleport and The Phasing of
//! Zhalfir. Attachments phase out and in with the permanent (702.26g), "for as long as"
//! durations end (702.26f), a permanent phases in during the untap step of the player it
//! phased out under (702.26a), or later if that player left the game (702.26n), and "can't
//! phase in for as long as you control this Saga".

use crate::r_p048_common::*;
use crate::r_s01_common::custom_card;
use crate::r_s06_common::attached_to;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to `p`'s next upkeep (after `p`'s untap step).
fn next_upkeep(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active == p {
        let other = t.g.next_player(p);
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

/// `p` activates Blink Dog's Teleport ({3}{W}).
fn teleport(t: &mut TestGame, p: PlayerId, dog: ObjectId) {
    t.lands(p, "Plains", 4);
    activate_with(t, p, dog, "phases out", &[]);
    t.resolve_all();
    assert!(phased_out(t, dog));
}

/// P0's The Phasing of Zhalfir enters with read ahead at chapter I, its ability targeting
/// `target`, and resolves.
fn zhalfir_on(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("The Phasing of Zhalfir");
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    t.answer_targets(P0, &[obj(target)]);
    let saga = t.enter(P0, "The Phasing of Zhalfir");
    t.resolve_all();
    assert_eq!(t.counters(saga, "lore"), 1);
    assert!(phased_out(t, target));
    saga
}

#[test]
fn auras_and_equipment_phase_out_and_in_with_the_permanent() {
    cr!("702.26g", "702.26a");
    ruling!(
        "Blink Dog",
        "As Blink Dog phases out, Auras and Equipment attached to it also phase out at the same time."
    );
    ruling!(
        "The Phasing of Zhalfir",
        "As a permanent is phased out, Auras and Equipment attached to it also phase out at the same time."
    );
    // Blink Dog teleports itself with Holy Strength and Bonesplitter attached.
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Blink Dog");
    let aura = attach_new(&mut t, P0, "Holy Strength", obj(dog));
    let eq = attach_new(&mut t, P0, "Bonesplitter", obj(dog));
    teleport(&mut t, P0, dog);
    assert!(phased_out(&t, aura) && phased_out(&t, eq));
    next_upkeep(&mut t, P0);
    for id in [dog, aura, eq] {
        assert!(!phased_out(&t, id));
    }
    assert_eq!(attached_to(&t, aura), Some(obj(dog)));
    assert_eq!(attached_to(&t, eq), Some(obj(dog)));
    // The Phasing of Zhalfir on P1's Bears with P1's Holy Strength and Bonesplitter; they
    // phase in together, still attached, once the Saga is gone.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = attach_new(&mut t, P1, "Holy Strength", obj(bears));
    let eq = attach_new(&mut t, P1, "Bonesplitter", obj(bears));
    let saga = zhalfir_on(&mut t, bears);
    assert!(phased_out(&t, aura) && phased_out(&t, eq));
    bounce(&mut t, saga);
    next_upkeep(&mut t, P1);
    for id in [bears, aura, eq] {
        assert!(!phased_out(&t, id));
    }
    assert_eq!(attached_to(&t, aura), Some(obj(bears)));
    assert_eq!(attached_to(&t, eq), Some(obj(bears)));
}

#[test]
fn zhalfirs_phased_out_permanents_stay_out_while_the_saga_is_controlled() {
    cr!("702.26a", "611.2b");
    ruling!(
        "The Phasing of Zhalfir",
        "Permanents that phase out as The Phasing of Zhalfir's first and second chapter abilities resolve don't phase in during their controllers' untap steps while the Saga is on the battlefield. After it leaves the battlefield, they will phase in during their controller's next untap step as normal."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let saga = zhalfir_on(&mut t, bears);
    next_upkeep(&mut t, P1);
    assert!(phased_out(&t, bears));
    // The Saga leaves during P1's turn: not until P1's next untap step.
    bounce(&mut t, saga);
    assert!(phased_out(&t, bears));
    next_upkeep(&mut t, P0);
    assert!(phased_out(&t, bears));
    next_upkeep(&mut t, P1);
    assert!(!phased_out(&t, bears));
}

#[test]
fn a_permanent_phasing_in_keeps_its_counters_and_can_attack() {
    cr!("702.26a", "702.26d", "302.6");
    ruling!(
        "The Phasing of Zhalfir",
        "Normally, permanents phase back in during their controller's untap step, immediately before that player untaps their permanents. Creatures that phase in this way are able to attack and pay a cost of {T} during that turn. If a permanent had counters on it when it phased out, it will have those counters when it phases back in."
    );
    // P1's Llanowar Elves (tapped, with a +1/+1 counter) phases out to Zhalfir; the Saga
    // leaves; it phases in during P1's untap step and untaps, with its counter, and can
    // attack and tap for mana that turn.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    put_counters(&mut t, elves, "+1/+1", 1);
    tap(&mut t, elves);
    let saga = zhalfir_on(&mut t, elves);
    bounce(&mut t, saga);
    next_upkeep(&mut t, P1);
    assert!(!phased_out(&t, elves));
    assert!(!t.obj_now(elves).tapped);
    assert_eq!(t.counters(elves, "+1/+1"), 1);
    // In P1's main phase it can tap for mana; in combat, it can attack.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(activate_containing(&mut t, P1, elves, "Add {G}").is_ok());
    assert_eq!(t.g.player(P1).mana_pool.total(), 1);
    t.g.objects[elves.0 as usize].tapped = false;
    to_combat(&mut t, P1);
    assert!(crate::r_s09_common::legal_attack(
        &mut t,
        &[(elves, Entity::Player(P0))]
    ));
}

#[test]
fn for_as_long_as_durations_ignore_phased_out_objects() {
    cr!("702.26f", "611.2b");
    ruling!(
        "The Phasing of Zhalfir",
        "Any continuous effects with a \"for as long as\" duration, such as that of Extraction Specialist, ignore phased-out objects."
    );
    ruling!(
        "Vronos, Masked Inquisitor",
        "Any continuous effects with a \"for as long as\" duration, such as that of Extraction Specialist, ignore phased-out objects."
    );
    // P0's Mind Flayer ("gain control of target creature for as long as you control Mind
    // Flayer") took P1's Bears; Zhalfir phases the Mind Flayer out: the Bears go back to
    // P1 for good.
    supported("Mind Flayer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let flayer = t.enter(P0, "Mind Flayer");
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    let saga = zhalfir_on(&mut t, flayer);
    assert_eq!(controller(&mut t, bears), P1);
    bounce(&mut t, saga);
    next_upkeep(&mut t, P0);
    assert!(!phased_out(&t, flayer));
    assert_eq!(controller(&mut t, bears), P1);
    // Vronos's +1 phases out P0's other planeswalker, whose ability took P1's Bears for
    // as long as P0 controls that planeswalker: the Bears go back to P1.
    supported("Vronos, Masked Inquisitor");
    let thief = custom_card(
        "Test Thief",
        "Legendary Planeswalker — Test",
        "{3}",
        None,
        "+1: Gain control of target creature for as long as you control this planeswalker.",
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pw = t.custom(P0, thief, Zone::Battlefield);
    put_counters(&mut t, pw, "loyalty", 3);
    activate_with(&mut t, P0, pw, "Gain control", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(controller(&mut t, bears), P0);
    let vronos = t.battlefield(P0, "Vronos, Masked Inquisitor");
    put_counters(&mut t, vronos, "loyalty", 4);
    t.answer_targets(P0, &[obj(pw)]);
    activate_with(&mut t, P0, vronos, "phase out", &[]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(phased_out(&t, pw));
    assert_eq!(controller(&mut t, bears), P1);
    next_upkeep(&mut t, P0);
    assert!(!phased_out(&t, pw));
    assert_eq!(controller(&mut t, bears), P1);
}

#[test]
fn blink_dog_teleported_by_an_opponent_phases_in_at_that_players_untap_step() {
    cr!("702.26a", "702.26n", "800.4a");
    ruling!(
        "Blink Dog",
        "If an opponent gains control of your Blink Dog, activates its Teleport ability, and the duration of the control-change effect expires before it phases in, Blink Dog phases in under your control as that opponent's next untap step begins."
    );
    ruling!(
        "Blink Dog",
        "If they leave the game before their next untap step, Blink Dog phases in as the next untap step begins after their turn would have begun."
    );
    supported("Act of Treason");
    for leaves in [false, true] {
        // Three players: P1 takes P0's Blink Dog with Act of Treason and teleports it.
        let mut t = TestGame::new(3);
        let dog = t.battlefield(P0, "Blink Dog");
        t.advance_to(P1, Step::PrecombatMain);
        cast_new(&mut t, P1, "Act of Treason", &[obj(dog)]);
        t.resolve_all();
        assert_eq!(controller(&mut t, dog), P1);
        teleport(&mut t, P1, dog);
        // The turn ends: the control change is over; it's still phased out.
        next_upkeep(&mut t, P2);
        assert_eq!(controller(&mut t, dog), P0);
        assert!(phased_out(&t, dog));
        if leaves {
            // P1 leaves during P2's turn; P1's next turn would have begun after P0's, so
            // the Dog phases in during P2's next untap step.
            concede(&mut t, P1);
            assert!(t.g.is_live(dog));
            next_upkeep(&mut t, P0);
            assert!(phased_out(&t, dog));
            next_upkeep(&mut t, P2);
        } else {
            next_upkeep(&mut t, P0);
            assert!(phased_out(&t, dog));
            next_upkeep(&mut t, P1);
        }
        assert!(!phased_out(&t, dog), "leaves: {leaves}");
        assert_eq!(controller(&mut t, dog), P0);
    }
    // P1 gained control of the Dog with no duration and teleported it; P1 leaving the game
    // ends that control change (CR 800.4a): the Dog, P0's again and not exiled, phases in
    // during P2's untap step after P1's turn would have begun (after P0's).
    let mut t = TestGame::new(3);
    let dog = t.battlefield(P0, "Blink Dog");
    t.advance_to(P1, Step::PrecombatMain);
    crate::r_s06_common::give_control(&mut t, dog, P1);
    teleport(&mut t, P1, dog);
    next_upkeep(&mut t, P2);
    assert_eq!(controller(&mut t, dog), P1);
    concede(&mut t, P1);
    assert_eq!(t.zone(dog), Zone::Battlefield);
    assert_eq!(controller(&mut t, dog), P0);
    assert!(phased_out(&t, dog));
    next_upkeep(&mut t, P0);
    assert!(phased_out(&t, dog));
    next_upkeep(&mut t, P2);
    assert!(!phased_out(&t, dog));
    assert_eq!(controller(&mut t, dog), P0);
}
