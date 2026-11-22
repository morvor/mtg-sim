//! CR 801.11, 801.13: with the limited range of influence option, an effect that makes
//! permanents enter as copies of a chosen creature (CR 614.1d) doesn't see that creature
//! while a player outside its controller's range controls it: the copy instruction can't be
//! carried out, and the permanents enter as themselves. (Mystic Reflection's ruling on a
//! chosen creature that has left the battlefield, whose last known copiable values are
//! used, is about a creature that's gone, not one that's out of range.)

use crate::r614_audit_common::supported;
use crate::r800_common::ranged;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn mystic_reflection_doesnt_copy_its_chosen_creature_once_a_player_out_of_range_controls_it() {
    cr!("801.11", "801.13", "614.1d");
    interp!("copy-effect-doesnt-see-an-original-out-of-range");
    supported("Mystic Reflection");
    supported("Ray of Command");
    for stolen in [false, true] {
        // Five players, each with a range of influence of 1: P0's range is P4, P0 and P1;
        // P2 is out of it.
        let mut t = ranged(5, 1);
        let giant = t.battlefield(P1, "Hill Giant");
        // P0's Mystic Reflection: "Choose target nonlegendary creature. The next time one or
        // more creatures or planeswalkers enter this turn, they enter as copies of the chosen
        // creature."
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 1);
        let reflection = t.hand(P0, "Mystic Reflection");
        t.cast(P0, reflection).target(giant).go();
        t.resolve_all();
        if stolen {
            // P2's Ray of Command: "Untap target creature an opponent controls and gain
            // control of it until end of turn. ..."
            t.lands(P2, "Island", 1);
            t.lands(P2, "Wastes", 3);
            let ray = t.hand(P2, "Ray of Command");
            t.cast(P2, ray).target(giant).go();
            t.resolve_all();
            t.g.recompute();
            assert_eq!(t.obj_now(giant).controller, P2);
        }
        let bears = t.enter(P0, "Grizzly Bears");
        let name = t.obj_now(bears).chars.name.to_string();
        if stolen {
            // (Copying the Giant anyway, as though it had left the battlefield, would make
            // the Bears a Hill Giant.)
            assert_eq!(name, "Grizzly Bears", "the Giant is out of P0's range");
            assert_eq!(t.pt(bears), (2, 2));
        } else {
            assert_eq!(name, "Hill Giant", "the Giant is within P0's range");
            assert_eq!(t.pt(bears), (3, 3));
        }
    }
}
