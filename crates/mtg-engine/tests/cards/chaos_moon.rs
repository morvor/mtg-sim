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

#[test]
fn the_odd_turns_pump_and_extra_mana_end_with_that_turn() {
    cr!("611.2a", "514.2");
    // Odd on P1's turn (3 permanents); a fourth permanent makes P0's upkeep even.
    let (mut t, ogre, mountain) = upkeep_with(0);
    assert_eq!(t.pt(ogre), (3, 3));
    t.battlefield(P1, "Island");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // Only the even effects apply: 1/1 (not 2/2), and the Mountain makes only {C} (the
    // odd turn's additional {R} ended with that turn).
    assert_eq!(t.pt(ogre), (1, 1));
    t.activate(P1, mountain, 0, &[]).unwrap();
    assert_eq!(pool(&t, P1), vec![ManaType::C]);
}

#[test]
fn in_two_headed_giant_it_triggers_once_per_upkeep() {
    cr!("805.4");
    ruling!(
        "Chaos Moon",
        "In Two-Headed Giant, triggers only once per upkeep, not once for each player."
    );
    // P0 and P1 against P2 and P3; on P2 and P3's turn, one upkeep: Chaos Moon, Gray Ogre
    // and a Mountain are 3 permanents (odd), so the Ogre gets +1/+1 once (twice would
    // make it 4/4).
    let mut t = TestGame::with_config(
        4,
        mtg_engine::game::GameConfig::two_headed_giant(vec![0, 0, 1, 1]),
    );
    for p in [P0, P1, P2, P3] {
        for _ in 0..3 {
            t.library_top(p, "Island");
        }
    }
    t.battlefield(P0, "Chaos Moon");
    let ogre = t.battlefield(P0, "Gray Ogre");
    t.battlefield(P2, "Mountain");
    t.set_step(P1, Step::End);
    t.advance_to(P2, Step::PrecombatMain);
    assert_eq!(t.pt(ogre), (3, 3));
}
