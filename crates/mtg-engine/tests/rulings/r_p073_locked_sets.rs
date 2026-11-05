//! Rulings batch P073 — "creatures you control gain ..." effects whose affected set is
//! locked in as they resolve (CR 611.2c): Flame-Kin Zealot, Burst of Speed, Goblin
//! Bushwhacker, Deafening Clarion, Vito, Victory's Herald; and statics that aren't locked
//! in (CR 611.3a): Regal Caracal.

use crate::r_p073_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s04_common::crew;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn flame_kin_zealot_affects_creatures_controlled_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Flame-Kin Zealot",
        "All creatures under your control when Flame-Kin Zealot’s triggered ability resolves are affected, including Flame-Kin Zealot itself. Ones that come under your control or become creatures later in the turn are not."
    );
    supported("Flame-Kin Zealot");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let zealot = crate::r_s05_common::enter(&mut t, P0, "Flame-Kin Zealot");
    t.resolve_all();
    assert_eq!(t.pt(zealot), (3, 3));
    assert!(kw(&t, zealot, HASTE));
    assert_eq!(t.pt(bears), (3, 3));
    assert!(kw(&t, bears, HASTE));
    // A creature that comes under P0's control later, and a Vehicle that becomes a
    // creature later, aren't affected.
    let later = t.battlefield_sick(P0, "Hill Giant");
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert_eq!(t.pt(later), (3, 3));
    assert!(!kw(&t, later, HASTE));
    assert_eq!(t.pt(copter), (3, 3), "Copter is a 3/3 with no bonus");
}

#[test]
fn burst_of_speed_and_deafening_clarion_affect_only_creatures_controlled_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Burst of Speed",
        "Burst of Speed affects only creatures you control at the time it resolves. It won’t affect creatures that come under your control later in the turn."
    );
    ruling!(
        "Deafening Clarion",
        "Deafening Clarion’s second mode affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t gain lifelink."
    );
    supported("Burst of Speed");
    supported("Deafening Clarion");
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Burst of Speed", &[]);
    let later = t.battlefield_sick(P0, "Grizzly Bears");
    assert!(kw(&t, bears, HASTE));
    assert!(!kw(&t, theirs, HASTE));
    assert!(!kw(&t, later, HASTE));
    crate::r_s09_common::to_combat(&mut t, P0);
    assert!(crate::r_s09_common::legal_attack(&mut t, &[(bears, pl(P1))]));
    assert!(!crate::r_s09_common::legal_attack(&mut t, &[(later, pl(P1))]));

    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let c = t.hand(P0, "Deafening Clarion");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Deafening Clarion");
    t.cast(P0, c).modes(&[1]).go();
    t.resolve_all();
    let later = t.battlefield(P0, "Grizzly Bears");
    assert!(kw(&t, bears, LIFELINK));
    assert!(!kw(&t, theirs, LIFELINK));
    assert!(!kw(&t, later, LIFELINK));
}

#[test]
fn deafening_clarion_lethally_damaged_creature_still_gains_lifelink() {
    cr!("608.2c", "704.3", "704.5g");
    ruling!(
        "Deafening Clarion",
        "A creature you control that’s dealt lethal damage by Deafening Clarion’s first mode will still be on the battlefield to gain lifelink from the spell’s second mode before that creature dies."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Craw Wurm");
    let c = t.hand(P0, "Deafening Clarion");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Deafening Clarion");
    t.cast(P0, c).modes(&[0, 1]).go();
    t.resolve_all();
    // The Bears died to the damage; its last known information has lifelink: it gained
    // it from the second mode before state-based actions put it into the graveyard.
    assert!(!t.on_battlefield(bears));
    assert!(t.g.obj(bears).has_keyword(LIFELINK));
    assert!(kw(&t, giant, LIFELINK));
}

#[test]
fn goblin_bushwhacker_includes_itself_unless_it_left() {
    cr!("611.2c", "702.33d");
    ruling!(
        "Goblin Bushwhacker",
        "Only creatures you control as Goblin Bushwhacker's ability resolves are affected. This includes Goblin Bushwhacker itself, unless it already left the battlefield by then."
    );
    supported("Goblin Bushwhacker");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Mountain", 2);
        let c = t.hand(P0, "Goblin Bushwhacker");
        t.cast(P0, c).kicked(true).go();
        t.resolve(); // the creature spell; its enters trigger goes on the stack
        assert_eq!(t.stack_len(), 1, "kicked: the trigger is on the stack");
        let gob = t.named_on_battlefield("Goblin Bushwhacker")[0];
        if leaves {
            destroy_now(&mut t, gob);
        }
        t.resolve_all();
        assert_eq!(t.pt(bears), (3, 2), "leaves: {leaves}");
        assert!(kw(&t, bears, HASTE));
        if !leaves {
            assert_eq!(t.pt(gob), (2, 1));
            assert!(kw(&t, gob, HASTE));
        }
        let later = t.battlefield_sick(P0, "Grizzly Bears");
        assert_eq!(t.pt(later), (2, 2));
        assert!(!kw(&t, later, HASTE));
    }
}

#[test]
fn victorys_herald_bonuses_last_for_the_turn() {
    cr!("611.2c", "511.3");
    ruling!(
        "Victory's Herald",
        "Even though the triggered ability affects only attacking creatures, the bonuses will remain for the rest of the turn."
    );
    supported("Victory's Herald");
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Victory's Herald");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(herald, pl(P1)), (bears, pl(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P0), 26, "lifelink from both attackers");
    // Combat is over: the Bears is no longer an attacking creature, and keeps the bonuses.
    assert!(t
        .g
        .combat
        .as_ref()
        .is_none_or(|c| c.attackers.iter().all(|a| a.id != bears)));
    assert!(kw(&t, bears, FLYING) && kw(&t, bears, LIFELINK), "after combat");
    assert!(!kw(&t, home, FLYING) && !kw(&t, home, LIFELINK));
}

#[test]
fn regal_caracal_affects_every_other_cat() {
    cr!("611.3a");
    ruling!(
        "Regal Caracal",
        "All other Cats you control get +1/+1 and have lifelink, not just those created by Regal Caracal's last ability."
    );
    supported("Regal Caracal");
    let mut t = TestGame::new(2);
    let caracal = crate::r_s05_common::enter(&mut t, P0, "Regal Caracal");
    t.resolve_all();
    let lions = t.battlefield(P0, "Savannah Lions");
    let theirs = t.battlefield(P1, "Savannah Lions");
    assert_eq!(t.pt(lions), (3, 2));
    assert!(kw(&t, lions, LIFELINK));
    assert_eq!(t.pt(theirs), (2, 1));
    assert_eq!(t.pt(caracal), (3, 3), "other Cats");
    let cats = crate::r_s05_common::tokens_with_subtype(&t, P0, "Cat");
    assert_eq!(cats.len(), 2);
    assert!(cats.iter().all(|c| t.pt(*c) == (2, 2)));
}
