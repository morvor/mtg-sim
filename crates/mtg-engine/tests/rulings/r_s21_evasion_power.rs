//! Rulings batch S21 — "This creature can't be blocked by creatures with power 2 or less":
//! the restriction is checked as blockers are declared; a blocker whose power drops
//! afterward keeps blocking (CR 506.4a, 509.1b, 509.1h).

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s21_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 attacks with the real card `attacker` and P1's 3/3 Hill Giant blocks it; then the
/// Hill Giant gets -2/-2 (P1 casts Disfigure on it) and combat finishes.
fn shrink_the_blocker(attacker: &str) {
    supported(attacker);
    supported("Disfigure");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, attacker);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    assert!(t.obj_now(a).power() >= 3);
    crate::r_s20_common::to_beginning_of_combat(&mut t, P0);
    to_blockers(&mut t, &[(a, Entity::Player(P1))], &[(giant, a)]);
    assert_eq!(blocks_now(&t), vec![(giant, a)]);
    // P1 shrinks the Hill Giant to 1/1.
    let disfigure = t.hand(P1, "Disfigure");
    t.lands(P1, "Swamp", 1);
    t.cast(P1, disfigure).target(giant).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    // It's still blocking, and the attacker is still blocked.
    assert!(t.g.is_blocking(giant), "{attacker}: the blocker stays in combat");
    assert!(t.g.combat.as_ref().unwrap().is_blocked(a));
    t.advance_to(P0, Step::EndOfCombat);
    // The blocked attacker dealt its damage to the Hill Giant, not to P1.
    assert_eq!(t.life(P1), 20, "{attacker}");
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.g.obj(a).damage, 1, "{attacker}: the 1/1 blocker dealt its damage");
}

#[test]
fn a_blocker_whose_power_drops_below_3_keeps_blocking_garenbrig_paladin() {
    cr!("506.4a", "509.1b", "509.1h");
    ruling!(
        "Garenbrig Paladin",
        "Once a creature with power 3 or greater has blocked this creature, changing the power of the blocking creature won't cause this creature to become unblocked."
    );
    shrink_the_blocker("Garenbrig Paladin");
}

#[test]
fn a_blocker_whose_power_drops_below_3_keeps_blocking_steel_leaf_champion() {
    cr!("506.4a", "509.1b", "509.1h");
    ruling!(
        "Steel Leaf Champion",
        "Once a creature with power 3 or greater has blocked this creature, changing the power of the blocking creature won’t cause this creature to become unblocked."
    );
    shrink_the_blocker("Steel Leaf Champion");
}

#[test]
fn creatures_with_power_2_or_less_cant_be_declared_as_blockers() {
    cr!("509.1b");
    supported("Steel Leaf Champion");
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Steel Leaf Champion");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s20_common::to_beginning_of_combat(&mut t, P0);
    crate::r_s01_common::attack_with(&mut t, &[(champ, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(giant, champ)]));
    assert!(!legal_blocks(&mut t, P1, &[(bears, champ)]));
    assert!(!legal_blocks(&mut t, P1, &[(giant, champ), (bears, champ)]));
}
