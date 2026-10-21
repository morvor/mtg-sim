//! The worked example of CR 802 (the attack multiple players option), played out as a
//! test.

use super::r600_examples_common::supported;
use super::r800_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Rob (P0) attacks Alex (P1) with Runeclaw Bear and Carissa (P2) with Mountain Goat
/// (mountainwalk). Returns the game at the declare blockers step, the Goat, and which
/// attackers Carissa's creature was offered to block.
fn goat_attack(alex_mountain: bool, carissa_mountain: bool) -> (TestGame, ObjectId, Vec<ObjectId>) {
    let (rob, alex, carissa) = (P0, P1, P2);
    let mut t = TestGame::with_config(3, GameConfig::free_for_all());
    let bear = t.battlefield(rob, "Runeclaw Bear");
    let goat = t.battlefield(rob, "Mountain Goat");
    let blocker = t.battlefield(carissa, "Grizzly Bears");
    if alex_mountain {
        t.battlefield(alex, "Mountain");
    }
    if carissa_mountain {
        t.battlefield(carissa, "Mountain");
    }
    t.set_step(rob, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(bear, Entity::Player(alex)), (goat, Entity::Player(carissa))],
    );
    block(&mut t, carissa, &[(blocker, goat)]);
    go_to(&mut t, Step::DeclareBlockers);
    let offered = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::DeclareBlockers { options } if p == carissa => Some(options),
            _ => None,
        })
        .flatten()
        .find(|(b, _)| *b == blocker)
        .map(|(_, can)| can)
        .unwrap_or_default();
    (t, goat, offered)
}

#[test]
fn mountainwalk_looks_only_at_the_player_the_creature_attacks() {
    cr!("802.2a", "702.14c");
    cr_example!("802.2a", "mountainwalk");
    supported("Mountain Goat");
    supported("Runeclaw Bear");
    // Alex, whom the Goat isn't attacking, controls a Mountain: Carissa can block it.
    let (t, goat, offered) = goat_attack(true, false);
    assert!(offered.contains(&goat), "{offered:?}");
    assert!(is_blocked(&t, goat));
    // Carissa controls a Mountain: she can't, whatever Alex controls.
    let (t, goat, offered) = goat_attack(false, true);
    assert!(!offered.contains(&goat), "{offered:?}");
    assert!(!is_blocked(&t, goat));
}
