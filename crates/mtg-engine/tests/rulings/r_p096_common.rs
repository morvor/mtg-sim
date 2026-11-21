//! Shared helpers for the tests of rulings batch P096 (`r_p096_*.rs`): life gain and the
//! abilities that care about it (replacement effects that change the amount, "whenever
//! you gain life" triggers, "if you gained N or more life this turn" end step triggers),
//! Lich, Chain Lightning, Equipment that creates a creature to attach to, and the
//! "doesn't untap during its controller's untap step" lockdown effects. (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Orders `items` so that the ones containing `text` are put on the stack last (on top:
/// they resolve first).
pub fn order_top(items: &[String], text: &str) -> Answer {
    let (top, rest): (Vec<usize>, Vec<usize>) =
        (0..items.len()).partition(|&i| items[i].contains(text));
    Answer::Indices(rest.into_iter().chain(top).collect())
}

/// Puts `p`'s triggers whose text contains "tapped and attacking" on top of the stack.
pub fn tokens_first(_g: &Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::Order { items, .. } => Some(order_top(items, "tapped and attacking")),
        _ => None,
    }
}

/// The amounts of the life-gain events of `p` since index `from` of this turn's events.
pub fn gains_since(t: &TestGame, from: usize, p: PlayerId) -> Vec<u32> {
    crate::r_s33_common::life_gains_since(t, from, p)
}

/// The amounts of the life-loss events of `p` since index `from` of this turn's events.
pub fn losses_since(t: &TestGame, from: usize, p: PlayerId) -> Vec<u32> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::LifeLost { player, amount, .. } if *player == p => Some(*amount),
            _ => None,
        })
        .collect()
}

/// Sets the shared life total of `p`'s Two-Headed Giant team (both players' life).
pub fn set_team_life(t: &mut TestGame, p: PlayerId, life: i32) {
    for q in t.g.team_members(p) {
        crate::r_s33_common::set_life(t, q, life);
    }
}

/// From P0's precombat main phase, advances to the beginning of `active`'s end step and
/// puts the triggers on the stack.
pub fn to_end_step(t: &mut TestGame, active: PlayerId) {
    t.advance_to(active, Step::End);
    t.settle();
}

/// Moves to P0's combat and declares `attackers` (attacking P1), putting the attack
/// triggers on the stack.
pub fn declare(t: &mut TestGame, attackers: &[ObjectId]) {
    let a: Vec<(ObjectId, Entity)> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    crate::r_s01_common::attack_with(t, &a);
}

/// The number of triggered abilities on the stack whose source is `source`.
pub fn triggers_from(t: &TestGame, source: ObjectId) -> usize {
    crate::r_s25_common::abilities_from(t, source).len()
}

/// P0 attacks P1 with two Child of Night (2/1 lifelink), unblocked, and resolves the
/// triggers: two life-gaining events.
pub fn two_lifelinkers(t: &mut TestGame) {
    let a = t.battlefield(P0, "Child of Night");
    let b = t.battlefield(P0, "Child of Night");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    t.resolve_all();
}

/// P0 attacks P1 with Grizzly Bears wearing Shadowspear (3/3 trample, lifelink); P1's
/// Raging Goblin blocks it: 1 damage to the Goblin and 2 to P1 at the same time, one
/// life-gaining event of 3.
pub fn one_trampling_lifelinker(t: &mut TestGame) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spear = t.battlefield(P0, "Shadowspear");
    t.g.attach(spear, Entity::Object(bears));
    t.g.recompute();
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(goblin, bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Raging Goblin"));
}

/// P0 controls three Grizzly Bears and casts Congregate ("Target player gains 2 life for
/// each creature on the battlefield.") targeting `who`, and the spell resolves; the
/// triggers are put on the stack but don't resolve.
pub fn congregate(t: &mut TestGame, who: PlayerId) {
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    crate::r_s25_common::cast_new(t, P0, "Congregate", &[Entity::Player(who)]);
    t.resolve();
    t.settle();
}

/// Soul Warden ("Whenever another creature enters, you gain 1 life.") under `p`'s control
/// and a Grizzly Bears entering under `p`'s control: `p` gains 1 life, once the trigger
/// resolves (the stack is resolved).
pub fn gain_one(t: &mut TestGame, p: PlayerId) {
    t.battlefield(p, "Soul Warden");
    crate::r_s05_common::enter(t, p, "Grizzly Bears");
    t.resolve_all();
}
