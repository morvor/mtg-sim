//! Chaos Moon: "At the beginning of each upkeep, count the number of permanents. If the
//! number is odd, until end of turn, red creatures get +1/+1 and whenever a player taps a
//! Mountain for mana, that player adds an additional {R}. If the number is even, until
//! end of turn, red creatures get -1/-1 and if a player taps a Mountain for mana, that
//! Mountain produces colorless mana instead of any other type."

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort_by_key(|m| format!("{m:?}"));
    v
}

/// Chaos Moon and a red Gray Ogre (P0) and a Mountain (P1), plus `extra` more permanents;
/// on to P1's upkeep, its trigger resolved. Returns (ogre, mountain).
fn upkeep_with(extra: usize) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    for p in [P0, P1] {
        for _ in 0..3 {
            t.library_top(p, "Island");
        }
    }
    t.battlefield(P0, "Chaos Moon");
    let ogre = t.battlefield(P0, "Gray Ogre");
    let mountain = t.battlefield(P1, "Mountain");
    for _ in 0..extra {
        t.battlefield(P1, "Island");
    }
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    (t, ogre, mountain)
}

#[test]
fn an_odd_number_pumps_red_creatures_and_mountains_add_an_extra_red() {
    cr!("608.2h", "611.2a", "605.1b", "106.12a");
    // Chaos Moon, Gray Ogre and the Mountain: 3.
    let (mut t, ogre, mountain) = upkeep_with(0);
    assert_eq!(t.pt(ogre), (3, 3));
    t.activate(P1, mountain, 0, &[]).unwrap();
    assert_eq!(pool(&t, P1), vec![ManaType::R, ManaType::R]);
}

#[test]
fn an_even_number_shrinks_red_creatures_and_mountains_make_colorless() {
    cr!("608.2h", "611.2a", "614.1a");
    // 4 permanents.
    let (mut t, ogre, mountain) = upkeep_with(1);
    assert_eq!(t.pt(ogre), (1, 1));
    t.activate(P1, mountain, 0, &[]).unwrap();
    assert_eq!(pool(&t, P1), vec![ManaType::C]);
}

#[test]
fn the_effects_last_until_end_of_turn() {
    cr!("611.2a", "514.2");
    let (mut t, ogre, _) = upkeep_with(1);
    assert_eq!(t.pt(ogre), (1, 1));
    t.advance_to(P0, Step::Upkeep);
    // P0's upkeep: Chaos Moon triggers again (4 permanents, even): -1/-1 once, not twice.
    t.resolve_all();
    assert_eq!(t.pt(ogre), (1, 1));
}
