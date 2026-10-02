//! Rulings batch S33 — life totals: "your starting life total" is the life total you
//! began the game with — 20 in most two-player games, the team's 30 in Two-Headed Giant,
//! 40 in Commander (CR 119.1, 119.1a, 119.1c); halving a negative life total halves 0, so
//! the life total stays the same (CR 107.1b).

use crate::r_s01_common::supported;
use crate::r_s13_common::commander_game;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s33_common::*;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The life-loss events of `p` since index `from` of this turn's events (the amounts).
fn life_losses_since(t: &TestGame, from: usize, p: PlayerId) -> Vec<u32> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::LifeLost { player, amount } if *player == p => Some(*amount),
            _ => None,
        })
        .collect()
}

/// P0 controls Path of Bravery ("As long as your life total is greater than or equal to
/// your starting life total, creatures you control get +1/+1.") and Grizzly Bears (2/2);
/// returns the Bears' power with P0's life total set to `life`.
fn bears_power_with_path(t: &mut TestGame, life: i32) -> i32 {
    supported("Path of Bravery");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Path of Bravery");
    set_life(t, P0, life);
    t.g.dirty = true;
    t.g.recompute();
    t.pt(bears).0
}

#[test]
fn path_of_bravery_your_starting_life_total_is_the_one_you_began_with() {
    cr!("119.1", "119.1a", "119.1c", "810.4");
    ruling!(
        "Path of Bravery",
        "Your starting life total is the life total you began the game with. For most two-player formats, this is 20. For Two-Headed Giant, it’s the life total your team started with, usually 30. In Commander games, your starting life total is 40."
    );
    // A two-player game: 20.
    let mut t = TestGame::new(2);
    assert_eq!(t.life(P0), 20);
    assert_eq!(bears_power_with_path(&mut t, 20), 3);
    let mut t = TestGame::new(2);
    assert_eq!(bears_power_with_path(&mut t, 19), 2);
    // Two-Headed Giant: the team's 30. At 25 life, P0's team is below it.
    let mut t = two_headed_giant();
    assert_eq!(t.life(P0), 30);
    assert_eq!(bears_power_with_path(&mut t, 25), 2);
    let mut t = two_headed_giant();
    assert_eq!(bears_power_with_path(&mut t, 30), 3);
    // Commander: 40.
    let mut t = commander_game();
    assert_eq!(t.life(P0), 40);
    assert_eq!(bears_power_with_path(&mut t, 39), 2);
    let mut t = commander_game();
    assert_eq!(bears_power_with_path(&mut t, 40), 3);
}

#[test]
fn infernal_contract_halving_a_negative_life_total_halves_0() {
    cr!("107.1a", "107.1b");
    ruling!(
        "Infernal Contract",
        "If you attempt to halve a negative life total, you halve 0. This means that the life total stays the same. A life total of -10 would remain -10."
    );
    supported("Infernal Contract");
    // "Draw four cards. You lose half your life, rounded up." P0 controls Platinum Angel
    // ("You can't lose the game and your opponents can't win the game.") at -10 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Platinum Angel");
    set_life(&mut t, P0, -10);
    t.settle();
    assert!(!t.has_lost(P0));
    let hand = t.hand_size(P0);
    let from = t.g.turn_events.len();
    cast_and_resolve(&mut t, P0, "Infernal Contract", &[]);
    assert_eq!(t.hand_size(P0), hand + 4);
    assert_eq!(t.life(P0), -10);
    assert!(life_losses_since(&t, from, P0).is_empty());
    // At 7 life, P0 loses 4 (half, rounded up).
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 7);
    cast_and_resolve(&mut t, P0, "Infernal Contract", &[]);
    assert_eq!(t.life(P0), 3);
}

#[test]
fn ebonblade_reaper_halving_a_negative_life_total_leaves_it_unchanged() {
    cr!("107.1a", "107.1b", "508.1m");
    ruling!(
        "Ebonblade Reaper",
        "If you attempt to halve a negative life total, you halve 0. This means that the life total stays the same. A life total of -10 would remain -10."
    );
    supported("Ebonblade Reaper");
    // "Whenever this creature attacks, you lose half your life, rounded up."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Platinum Angel");
    let reaper = t.battlefield(P0, "Ebonblade Reaper");
    set_life(&mut t, P0, -10);
    t.set_step(P0, Step::BeginningOfCombat);
    let from = t.g.turn_events.len();
    crate::r_s01_common::attack_with(&mut t, &[(reaper, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P0), -10);
    assert!(life_losses_since(&t, from, P0).is_empty());
    assert!(!t.has_lost(P0));
}
