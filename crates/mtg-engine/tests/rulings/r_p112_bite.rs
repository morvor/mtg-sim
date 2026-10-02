//! Rulings batch P112 — "target creature you control deals damage equal to its power to
//! target creature you don't control" spells and abilities: what happens when one of the
//! two targets is illegal as the spell or ability resolves (CR 608.2b), and the power
//! used for the damage (CR 608.2h).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::give_control;
use crate::r_s07_common::damage_on;
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// How one target of a bite spell is made illegal before it resolves.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Spoil {
    /// Both targets stay legal.
    None,
    /// The creature you control is destroyed.
    MineGone,
    /// The creature you control is still on the battlefield but under the opponent's
    /// control ("target creature you control" is now illegal).
    MineStolen,
    /// The other creature is destroyed.
    TheirsGone,
}

/// P0's Grizzly Bears (2/2) and P1's Colossal Dreadmaw (6/6); P0 casts `spell` targeting
/// them in that order, `spoil` happens in response, then everything resolves.
/// Returns (game, mine, theirs).
fn bite(spell: &str, spoil: Spoil) -> (TestGame, ObjectId, ObjectId) {
    supported(spell);
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, spell, &[mine.into(), theirs.into()]);
    match spoil {
        Spoil::None => {}
        Spoil::MineGone => destroy(&mut t, mine),
        Spoil::MineStolen => give_control(&mut t, mine, P1),
        Spoil::TheirsGone => destroy(&mut t, theirs),
    }
    t.resolve_all();
    (t, mine, theirs)
}

#[test]
fn rabid_bite_style_spells_deal_no_damage_if_either_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Rabid Bite",
        "If either creature is an illegal target as Rabid Bite tries to resolve, the creature you control won't deal damage."
    );
    ruling!(
        "Polliwallop",
        "If either creature is an illegal target as Polliwallop resolves, the creature you control won't deal damage."
    );
    for (spell, mult) in [("Rabid Bite", 1), ("Polliwallop", 2)] {
        // Both legal: the damage is dealt.
        let (t, mine, theirs) = bite(spell, Spoil::None);
        assert_eq!(damage_on(&t, theirs), 2 * mult, "{spell}");
        assert!(t.on_battlefield(mine));
        // The creature you control is gone or no longer yours: no damage.
        for spoil in [Spoil::MineGone, Spoil::MineStolen] {
            let (t, _, theirs) = bite(spell, spoil);
            assert_eq!(damage_on(&t, theirs), 0, "{spell} {spoil:?}");
        }
        // The other creature is gone: your creature deals no damage to anything.
        let (t, mine, _) = bite(spell, Spoil::TheirsGone);
        assert_eq!(damage_on(&t, mine), 0, "{spell}");
        assert_eq!(t.life(P1), 20, "{spell}");
    }
}

#[test]
fn hunters_edge_no_damage_if_either_target_is_illegal() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Hunter's Edge",
        "If either creature is an illegal target as Hunter's Edge tries to resolve, the creature you control won't deal damage."
    );
    // Both legal: the counter makes it a 3/3 first, then it deals 3.
    let (t, mine, theirs) = bite("Hunter's Edge", Spoil::None);
    assert_eq!(t.counters(mine, counters::PLUS1), 1);
    assert_eq!(damage_on(&t, theirs), 3);
    // Your creature is stolen: no counter (it's an illegal target) and no damage.
    let (t, mine, theirs) = bite("Hunter's Edge", Spoil::MineStolen);
    assert_eq!(t.counters(mine, counters::PLUS1), 0);
    assert_eq!(damage_on(&t, theirs), 0);
    // The other creature is gone: the counter is still put on your creature, no damage.
    let (t, mine, _) = bite("Hunter's Edge", Spoil::TheirsGone);
    assert_eq!(t.counters(mine, counters::PLUS1), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn felling_blow_and_knockout_maneuver_counter_only_on_a_legal_target() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Felling Blow",
        "As Felling Blow tries to resolve, if either creature is an illegal target, the creature you control won't deal damage. If the creature you control is a legal target but the other creature isn't, your creature will still get a +1/+1 counter."
    );
    ruling!(
        "Knockout Maneuver",
        "If either creature is an illegal target as Knockout Maneuver resolves, the creature you control won't deal damage. If the creature you control is an illegal target, you won't put a +1/+1 counter on it even if it's still on the battlefield."
    );
    for spell in ["Felling Blow", "Knockout Maneuver"] {
        let (t, mine, theirs) = bite(spell, Spoil::None);
        assert_eq!(t.counters(mine, counters::PLUS1), 1, "{spell}");
        assert_eq!(damage_on(&t, theirs), 3, "{spell}");
        // The other creature is gone: the counter, no damage.
        let (t, mine, _) = bite(spell, Spoil::TheirsGone);
        assert_eq!(t.counters(mine, counters::PLUS1), 1, "{spell}");
        assert_eq!(damage_on(&t, mine), 0, "{spell}");
        // Your creature is on the battlefield but no longer yours: no counter, no damage.
        let (t, mine, theirs) = bite(spell, Spoil::MineStolen);
        assert!(t.on_battlefield(mine));
        assert_eq!(t.counters(mine, counters::PLUS1), 0, "{spell}");
        assert_eq!(damage_on(&t, theirs), 0, "{spell}");
    }
}

#[test]
fn clear_shot_still_pumps_if_only_the_other_target_is_illegal() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Clear Shot",
        "As Clear Shot tries to resolve, if either creature is an illegal target, the creature you control won't deal damage. If the creature you control is a legal target but the other creature isn't, your creature will still get +1/+1."
    );
    let (t, mine, theirs) = bite("Clear Shot", Spoil::None);
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(damage_on(&t, theirs), 3);
    let (t, mine, _) = bite("Clear Shot", Spoil::TheirsGone);
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.life(P1), 20);
    let (t, mine, theirs) = bite("Clear Shot", Spoil::MineStolen);
    assert_eq!(t.pt(mine), (2, 2));
    assert_eq!(damage_on(&t, theirs), 0);
}

#[test]
fn fall_of_the_hammer_does_nothing_if_only_one_target_is_legal() {
    cr!("608.2b");
    ruling!(
        "Fall of the Hammer",
        "As Fall of the Hammer tries to resolve, if only one of the targets is legal, Fall of the Hammer will still resolve but will have no effect"
    );
    let (t, _, theirs) = bite("Fall of the Hammer", Spoil::None);
    assert_eq!(damage_on(&t, theirs), 2);
    for spoil in [Spoil::MineGone, Spoil::MineStolen] {
        let (t, _, theirs) = bite("Fall of the Hammer", spoil);
        assert_eq!(damage_on(&t, theirs), 0, "{spoil:?}");
    }
    let (t, mine, _) = bite("Fall of the Hammer", Spoil::TheirsGone);
    assert_eq!(damage_on(&t, mine), 0);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Fall of the Hammer"));
}

#[test]
fn huatli_minus_three_no_damage_if_either_target_is_illegal() {
    cr!("608.2b", "606.3");
    ruling!(
        "Huatli, Dinosaur Knight",
        "If either creature is an illegal target as Huatli’s second ability resolves, the creature you control won’t deal damage."
    );
    supported("Huatli, Dinosaur Knight");
    for spoil in [Spoil::None, Spoil::MineGone, Spoil::TheirsGone] {
        let mut t = TestGame::new(2);
        let huatli = t.battlefield(P0, "Huatli, Dinosaur Knight");
        let dino = t.battlefield(P0, "Colossal Dreadmaw");
        let theirs = t.battlefield(P1, "Craw Wurm");
        let other = t.battlefield(P1, "Grizzly Bears");
        t.activate(P0, huatli, 1, &[dino.into(), theirs.into()])
            .unwrap();
        match spoil {
            Spoil::MineGone => destroy(&mut t, dino),
            Spoil::TheirsGone => destroy(&mut t, theirs),
            _ => {}
        }
        t.resolve_all();
        match spoil {
            // Craw Wurm (6/4) is dealt 6 and dies.
            Spoil::None => assert!(t.in_graveyard(P1, "Craw Wurm")),
            Spoil::MineGone => assert_eq!(damage_on(&t, theirs), 0),
            _ => {
                assert_eq!(damage_on(&t, dino), 0);
                assert_eq!(damage_on(&t, other), 0);
                assert_eq!(t.life(P1), 20);
            }
        }
    }
}
