//! Rulings batch P115 — phasing (CR 702.26): "for as long as" durations ignore phased-out
//! objects (Divine Smite, March of Swirling Mist, Robe of Stars against Mind Flayer's
//! control effect), and Teferi's Veil's end-of-combat phasing.

use crate::r_p115_common::*;
use crate::r_s06_common::activate_containing;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Mind Flayer has taken P1's Grizzly Bears ("for as long as you control this
/// creature").
fn flayer(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let flayer = t.enter(P0, "Mind Flayer");
    t.resolve_all();
    assert_eq!(ctl(t, bears), P0);
    (flayer, bears)
}

#[test]
fn phasing_out_ends_for_as_long_as_durations() {
    cr!("702.26b", "702.26f", "611.2b");
    ruling!(
        "Divine Smite",
        "Any continuous effects with a “for as long as” duration, such as that of Mind Flayer, ignore phased-out objects. If ignoring those objects causes the effect's conditions to no longer be met, the duration will expire."
    );
    ruling!(
        "March of Swirling Mist",
        "Any continuous effects with a “for as long as” duration, such as that of Mind Flayer, ignore phased-out objects. If ignoring those objects causes the effect's conditions to no longer be met, the duration will expire."
    );
    ruling!(
        "Robe of Stars",
        "Any continuous effects with a “for as long as” duration, such as that of Mind Flayer, ignore phased-out objects. If ignoring those objects causes the effect's conditions to no longer be met, the duration will expire."
    );
    for card in ["Divine Smite", "March of Swirling Mist", "Robe of Stars"] {
        supported(card);
        let mut t = TestGame::new(2);
        let (flayer, bears) = flayer(&mut t);
        match card {
            // "Target creature or planeswalker an opponent controls phases out": P1 casts
            // it on P0's (blue) Mind Flayer.
            "Divine Smite" => {
                cast_at(&mut t, P1, card, &[obj(flayer)]);
            }
            "March of Swirling Mist" => {
                lots(&mut t, P0);
                let c = t.hand(P0, card);
                t.cast(P0, c).x(1).targets(&[obj(flayer)]).go();
            }
            _ => {
                let robe = t.battlefield(P0, card);
                assert!(t.g.attach(robe, obj(flayer)));
                lots(&mut t, P0);
                activate_containing(&mut t, P0, robe, "phases out").expect("Astral Projection");
            }
        }
        t.resolve_all();
        assert!(t.obj_now(flayer).phased_out, "{card}");
        assert_eq!(ctl(&t, bears), P1, "{card}");
        // The effect has ended: it doesn't come back when the Flayer phases in.
        pass_turn(&mut t);
        pass_turn(&mut t);
        assert!(!t.obj_now(flayer).phased_out, "{card}");
        assert_eq!(ctl(&t, bears), P1, "{card}");
    }
}

#[test]
fn teferis_veil_phases_out_after_combat_damage() {
    cr!("511.2", "510.2", "702.26b");
    ruling!(
        "Teferi's Veil",
        "The “at end of combat” ability triggers after combat damage resolves. Creatures dealt lethal damage in combat won’t be saved this way."
    );
    supported("Teferi's Veil");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi's Veil");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let giant = t.battlefield(P1, "Hill Giant");
    t.attack(&[(bears, pl(P1)), (lions, pl(P1))], &[(giant, bears)]);
    assert_eq!(t.g.turn.step, Step::EndOfCombat);
    t.resolve_all();
    // The blocked Bears died in the combat damage step; the Lions dealt their damage and
    // then phased out.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
    assert!(t.obj_now(lions).phased_out);
}
