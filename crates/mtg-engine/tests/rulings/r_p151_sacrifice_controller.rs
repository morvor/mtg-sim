//! Rulings batch P151 — who performs a delayed "sacrifice" instruction (CR 603.7d–e,
//! 608.2c, 701.21a): "sacrifice ~" / "sacrifice it" is the delayed ability's controller's
//! instruction, which they can't follow for a permanent another player controls by then;
//! "its controller sacrifices it" names the player: whoever controls it as the delayed
//! ability resolves (the Goblin Ski Patrol ruling, `r_p063_flying.rs`).

use crate::r_p151_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates Dragon Whelp's firebreathing four times (the fourth creates the delayed
/// "sacrifice ~ at the beginning of the next end step").
fn whelp_four_times(t: &mut TestGame, whelp: ObjectId) {
    for _ in 0..4 {
        t.activate(P0, whelp, 0, &[]).expect("firebreathing");
        t.resolve_all();
    }
}

#[test]
fn dragon_whelps_delayed_sacrifice_needs_its_controller_to_still_control_it() {
    cr!("701.21a", "603.7e");
    supported("Dragon Whelp");
    for stolen in [false, true] {
        let mut t = TestGame::new(2);
        let whelp = t.battlefield(P0, "Dragon Whelp");
        t.lands(P0, "Mountain", 4);
        whelp_four_times(&mut t, whelp);
        if stolen {
            steal_for_good(&mut t, P1, whelp);
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.on_battlefield(whelp), stolen, "stolen: {stolen}");
        if stolen {
            assert_eq!(t.g.obj(t.g.current(whelp)).controller, P1);
        } else {
            assert!(t.in_graveyard(P0, "Dragon Whelp"));
        }
    }
}

#[test]
fn chimeric_coils_delayed_sacrifice_needs_its_controller_to_still_control_it() {
    cr!("701.21a", "603.7e");
    supported("Chimeric Coils");
    for stolen in [false, true] {
        let mut t = TestGame::new(2);
        let coils = t.battlefield(P0, "Chimeric Coils");
        t.lands(P0, "Wastes", 3);
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        t.activate(P0, coils, 0, &[]).expect("Chimeric Coils");
        t.resolve_all();
        assert_eq!(t.pt(coils), (2, 2));
        if stolen {
            steal_for_good(&mut t, P1, coils);
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.on_battlefield(coils), stolen, "stolen: {stolen}");
        if !stolen {
            assert!(t.in_graveyard(P0, "Chimeric Coils"));
        }
    }
}

#[test]
fn celestial_swords_creature_is_sacrificed_by_whoever_controls_it_then() {
    // "Its controller sacrifices it at the beginning of the next end step": the player is
    // the creature's controller as the delayed ability resolves.
    cr!("701.21a", "603.7c");
    supported("Celestial Sword");
    for stolen in [false, true] {
        let mut t = TestGame::new(2);
        let sword = t.battlefield(P0, "Celestial Sword");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Wastes", 3);
        t.activate(P0, sword, 0, &[obj(bears)]).expect("Celestial Sword");
        t.resolve_all();
        assert_eq!(t.pt(bears), (5, 5));
        if stolen {
            steal_for_good(&mut t, P1, bears);
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "sacrificed (stolen: {stolen})");
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "its owner's graveyard");
    }
}
