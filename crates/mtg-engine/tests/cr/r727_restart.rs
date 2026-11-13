//! CR 727: restarting the game (see also r104_restart.rs).

use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

const P4: PlayerId = PlayerId(4);

/// P0 controls Karn Liberated with plenty of loyalty.
fn karn(t: &mut TestGame) -> ObjectId {
    let k = t.battlefield(P0, "Karn Liberated");
    t.g.objects[k.0 as usize]
        .counters
        .insert("loyalty".into(), 30);
    k
}

/// "−14: Restart the game, leaving in exile all non-Aura permanent cards exiled with
/// Karn. Then put those cards onto the battlefield under your control."
fn karn_restart(t: &mut TestGame, k: ObjectId) {
    t.activate(P0, k, 2, &[]).unwrap();
    t.g.resolve_top();
}

/// Leaves `p` owning only `n` cards (the rest of their library ceases to exist).
fn shrink_library(t: &mut TestGame, p: PlayerId, n: usize) {
    let lib = t.g.players[p.idx()].library.clone();
    for id in &lib[n..] {
        t.g.objects[id.0 as usize].zone = Zone::Nowhere;
    }
    t.g.players[p.idx()].library.truncate(n);
}

#[test]
fn a_player_with_fewer_than_seven_cards_loses_the_restarted_game_in_the_first_upkeep() {
    cr!("727.3");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            skip_mulligans: true,
            ..Default::default()
        },
    );
    let k = karn(&mut t);
    shrink_library(&mut t, P1, 5);
    karn_restart(&mut t, k);
    // P1 drew all five of their cards as the new game began...
    assert_eq!(t.g.turn.number, 1);
    assert_eq!(t.hand_size(P1), 5);
    assert!(!t.has_lost(P1));
    // ...and loses when state-based actions are checked in the first turn's upkeep.
    let ok = t.g.run_until(100, |g| {
        g.turn.step == Step::Upkeep && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    assert_eq!(t.g.turn.number, 1);
    t.settle();
    assert!(t.has_lost(P1));
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
}

#[test]
fn all_players_are_involved_regardless_of_range_of_influence() {
    cr!("727.7");
    let mut t = TestGame::with_config(
        5,
        GameConfig {
            range_of_influence: Some(1),
            skip_mulligans: true,
            ..Default::default()
        },
    );
    let k = karn(&mut t);
    // P2 and P3 are outside P0's range of influence (P4 and P1 are next to P0).
    assert!(!t.g.players_in_range(P0).contains(&P2));
    t.g.players[2].life = 7;
    t.g.players[3].life = 3;
    t.battlefield(P2, "Grizzly Bears");
    karn_restart(&mut t, k);
    assert_eq!(t.g.turn.number, 1);
    assert_eq!(t.g.players_in_game(), vec![P0, P1, P2, P3, P4]);
    for p in [P0, P1, P2, P3, P4] {
        assert_eq!(t.life(p), 20);
        assert_eq!(t.hand_size(p), 7);
    }
    assert!(t.g.battlefield.is_empty());
}
