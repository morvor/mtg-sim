//! Rulings batch P052 — The Moment and Oubliette: "phases out until [this] leaves the
//! battlefield" (CR 702.26, 610.4), phasing in tapped (CR 702.26c), and a
//! permanent destroyed by its own ability before it's sacrificed (CR 701.8a, 701.21a).

use crate::r_p046_common::*;
use crate::r_p052_common::*;
use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).phased_out
}

#[test]
fn the_moments_creature_phases_in_only_when_the_moment_leaves() {
    cr!("610.4", "610.4a", "702.26c");
    ruling!(
        "The Moment",
        "A creature phased out by The Moment doesn't phase in during its controller's untap step as normal. Rather, it phases in immediately after The Moment leaves the battlefield."
    );
    supported("The Moment");
    let mut t = TestGame::new(2);
    let moment = t.battlefield(P0, "The Moment");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    activate_resolve(&mut t, P0, moment, 0, &[obj(bears)]);
    assert!(phased_out(&t, bears));
    assert!(!t.obj_now(bears).tapped);
    // Two of P0's untap steps pass: still phased out.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(phased_out(&t, bears));
    t.g.destroy(moment, None);
    t.settle();
    assert!(!phased_out(&t, bears));
    assert!(t.on_battlefield(bears));
}

#[test]
fn the_moment_leaving_first_untaps_without_phasing_out() {
    cr!("610.4b");
    ruling!(
        "The Moment",
        "If The Moment leaves the battlefield before its first activated ability resolves, the target creature will be untapped, but won't phase out."
    );
    let mut t = TestGame::new(2);
    let moment = t.battlefield(P0, "The Moment");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    fill_pool(&mut t, P0, 2);
    t.activate(P0, moment, 0, &[obj(bears)]).expect("activate");
    t.g.destroy(moment, None);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert!(!phased_out(&t, bears));
}

#[test]
fn the_moment_destroyed_by_its_own_ability_isnt_sacrificed() {
    cr!("701.8a", "701.21a", "608.2c");
    ruling!(
        "The Moment",
        "The Moment's last ability will destroy itself before you can sacrifice it if there are two or more time counters on it as its last ability resolves. Effects that trigger when you sacrifice a permanent will not trigger when it is destroyed this way."
    );
    supported("Esoteric Duplicator");
    for (time, sacrificed) in [(2u32, false), (1, true)] {
        let mut t = TestGame::new(2);
        let moment = t.battlefield(P0, "The Moment");
        t.g.objects[moment.0 as usize]
            .counters
            .insert("time".into(), time);
        // Esoteric Duplicator: "Whenever you sacrifice this artifact or another artifact,
        // you may pay {2}. ..." (mana value 3: it survives).
        t.battlefield(P0, "Esoteric Duplicator");
        let elves = t.battlefield(P1, "Llanowar Elves");
        fill_pool(&mut t, P0, 3);
        t.activate(P0, moment, 1, &[]).expect("activate");
        t.g.settle();
        t.g.resolve_top();
        t.settle();
        assert!(!t.on_battlefield(elves), "{time}");
        assert!(t.in_graveyard(P0, "The Moment"), "{time}");
        assert_eq!(
            triggers_on_stack(&t, "you may pay {2}"),
            sacrificed as usize,
            "{time} time counters"
        );
    }
}

#[test]
fn phasing_in_tapped_doesnt_trigger_becomes_tapped_abilities() {
    cr!("702.26c", "603.2");
    ruling!(
        "Oubliette",
        "Abilities that trigger when a creature becomes tapped won't trigger when the creature phases in tapped."
    );
    supported("Oubliette");
    supported("Chrome Companion");
    let mut t = TestGame::new(2);
    // Chrome Companion: "Whenever this creature becomes tapped, you gain 1 life."
    let companion = t.battlefield(P1, "Chrome Companion");
    t.answer_targets(P0, &[obj(companion)]);
    let oubliette = t.enter(P0, "Oubliette");
    t.resolve_all();
    assert!(phased_out(&t, companion));
    t.g.destroy(oubliette, None);
    t.settle();
    assert!(!phased_out(&t, companion));
    assert!(t.obj_now(companion).tapped);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
    // Tapping it does trigger.
    let mut t = TestGame::new(2);
    let companion = t.battlefield(P1, "Chrome Companion");
    t.g.tap(companion);
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
}
