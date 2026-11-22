//! Rulings batch S01 — afterlife (CR 702.135): "When this permanent is put into a
//! graveyard from the battlefield, create N 1/1 white and black Spirit creature tokens with
//! flying."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn afterlife_spirits_arrive_too_late_to_block() {
    cr!("702.135a", "509.1a");
    ruling!(
        "Seraph of the Scales",
        "Because blockers are chosen all at once, you can't block with a creature with afterlife, wait for it to die, then block with the resulting Spirit tokens."
    );
    supported("Seraph of the Scales");
    let mut t = TestGame::new(2);
    // Two attackers; Seraph of the Scales (4/3, afterlife 2) blocks the Craw Wurm and dies.
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let seraph = t.battlefield(P1, "Seraph of the Scales");
    attack_with(
        &mut t,
        &[(wurm, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(seraph, wurm)]);
    assert!(!t.on_battlefield(seraph));
    let spirits = with_subtype(&t, P1, "Spirit");
    assert_eq!(spirits.len(), 2);
    for s in spirits {
        assert!(!t.g.is_blocking(s));
    }
    // The Bears weren't blocked by the Spirits: 2 damage to P1.
    assert_eq!(t.life(P1), 18);
}
