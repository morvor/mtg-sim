//! Rulings batch S33 — paying life (CR 119.4, 119.3): a player can pay life only if their
//! life total is greater than or equal to the amount (so not 2 life at 1 life), even if
//! they can't lose the game; paying life is losing life, so "whenever you lose life"
//! abilities trigger (CR 119.4, 118.3).

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::can_cast;
use crate::r_s04_common::hand_names;
use crate::r_s33_common::*;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether P0, at `life` life and with no mana, could cast Gitaxian Probe ({U/P}: "Look
/// at target player's hand. Draw a card.") by paying 2 life.
fn can_probe_at(t: &mut TestGame, life: i32) -> bool {
    set_life(t, P0, life);
    let probe = t.hand(P0, "Gitaxian Probe");
    can_cast(t, P0, probe, CastMethod::Normal)
}

#[test]
fn gitaxian_probe_at_1_life_you_cant_pay_2_life() {
    cr!("119.4", "107.4f", "601.2h");
    ruling!(
        "Gitaxian Probe",
        "If you're at 1 life or less, you can't pay 2 life."
    );
    supported("Gitaxian Probe");
    for (life, can) in [(20, true), (2, true), (1, false), (0, false), (-3, false)] {
        let mut t = TestGame::new(2);
        assert_eq!(can_probe_at(&mut t, life), can, "at {life} life");
    }
    // At 2 life it can be paid: P0 goes to 0 life.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 2);
    let probe = t.hand(P0, "Gitaxian Probe");
    t.cast(P0, probe).target(Entity::Player(P1)).go();
    assert_eq!(t.life(P0), 0);
}

#[test]
fn herald_of_eternal_dawn_you_cant_pay_more_life_than_you_have() {
    cr!("119.4", "104.3b");
    ruling!(
        "Herald of Eternal Dawn",
        "You can't pay more life than you have, even if you won't lose the game."
    );
    supported("Herald of Eternal Dawn");
    // "You can't lose the game and your opponents can't win the game."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Eternal Dawn");
    assert!(!can_probe_at(&mut t, 1));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Eternal Dawn");
    assert!(!can_probe_at(&mut t, -5));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Eternal Dawn");
    assert!(can_probe_at(&mut t, 2));
}

#[test]
fn angels_grace_you_cant_pay_more_life_than_you_have() {
    cr!("119.4", "104.3b");
    ruling!(
        "Angel's Grace",
        "You can't pay more life than you have, even if you won't lose the game."
    );
    supported("Angel's Grace");
    // "You can't lose the game this turn and your opponents can't win the game this
    // turn." P0 casts it, then goes to 1 life.
    let mut t = TestGame::new(2);
    let grace = t.hand(P0, "Angel's Grace");
    t.lands(P0, "Plains", 1);
    t.cast(P0, grace).go();
    t.resolve_all();
    assert!(!can_probe_at(&mut t, 1));
    set_life(&mut t, P0, -1);
    t.settle();
    assert!(!t.has_lost(P0));
    assert!(!can_probe_at(&mut t, -1));
}

#[test]
fn vilis_paying_life_is_losing_life() {
    cr!("119.4", "119.3", "603.2");
    ruling!("Vilis, Broker of Blood", "A player loses life if they pay life.");
    supported("Vilis, Broker of Blood");
    // "{B}, Pay 2 life: Target creature gets -1/-1 until end of turn. Whenever you lose
    // life, draw that many cards."
    let mut t = TestGame::new(2);
    let vilis = t.battlefield(P0, "Vilis, Broker of Blood");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    let hand = t.hand_size(P0);
    t.activate(P0, vilis, 0, &[Entity::Object(bears)]).unwrap();
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P0), 18);
    assert_eq!(triggers_on_stack(&t, "draw that many cards"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn vengeful_warchief_paying_life_is_losing_life() {
    cr!("119.4", "119.3", "603.2");
    ruling!("Vengeful Warchief", "A player loses life if they pay life.");
    supported("Vengeful Warchief");
    // "Whenever you lose life for the first time each turn, put a +1/+1 counter on this
    // creature." P0 pays 2 life for Gitaxian Probe's {U/P}.
    let mut t = TestGame::new(2);
    let chief = t.battlefield(P0, "Vengeful Warchief");
    let probe = t.hand(P0, "Gitaxian Probe");
    t.cast(P0, probe).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert_eq!(t.counters(chief, "+1/+1"), 1);
    assert!(hand_names(&t, P0).len() == 1);
}

#[test]
fn savage_gorger_an_opponent_paying_life_lost_life() {
    cr!("119.4", "119.3", "603.4");
    ruling!("Savage Gorger", "A player loses life if they pay life.");
    supported("Savage Gorger");
    // "At the beginning of your end step, if an opponent lost life this turn, put a
    // +1/+1 counter on this creature." P1 pays 2 life for Gut Shot ({R/P}: "Gut Shot
    // deals 1 damage to any target.") during P0's turn, targeting P0's Grizzly Bears.
    let mut t = TestGame::new(2);
    let gorger = t.battlefield(P0, "Savage Gorger");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shot = t.hand(P1, "Gut Shot");
    t.g.turn.priority = Some(P1);
    t.cast(P1, shot).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(gorger, "+1/+1"), 1);
}

/// The special action Channel offers `p` now, if any.
fn channel_action(t: &mut TestGame, p: PlayerId) -> Option<mtg_engine::decision::Action> {
    crate::r_s08_common::actions_of(t, p).into_iter().find(|a| {
        matches!(
            a,
            mtg_engine::decision::Action::Special(
                mtg_engine::decision::SpecialAction::Offer { .. }
            )
        )
    })
}

#[test]
fn channel_once_your_life_total_is_0_you_cant_pay_any_more_life() {
    cr!("119.4", "104.3b", "116.2c");
    ruling!(
        "Channel",
        "Once your life total is 0, you can't pay any more life, even if you've somehow not lost the game yet."
    );
    supported("Channel");
    // P0 controls Platinum Angel ("You can't lose the game and your opponents can't win
    // the game.") and, at 2 life, casts Channel ("Until end of turn, any time you could
    // activate a mana ability, you may pay 1 life. If you do, add {C}.").
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Platinum Angel");
    crate::r_s25_common::cast_new(&mut t, P0, "Channel", &[]);
    t.resolve_all();
    set_life(&mut t, P0, 2);
    for life in [1, 0] {
        let pay = channel_action(&mut t, P0).expect("Channel's action");
        t.g.perform_action(P0, pay).unwrap();
        t.settle();
        assert_eq!(t.life(P0), life);
    }
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 2);
    assert!(!t.g.player(P0).has_lost);
    // At 0 life, P0 can't pay 1 life any more: the action isn't offered.
    assert!(channel_action(&mut t, P0).is_none());
    assert_eq!(t.life(P0), 0);
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 2);
    // The two {C} pay for Mind Stone ({2}).
    let stone = t.hand(P0, "Mind Stone");
    t.cast(P0, stone).go();
    t.resolve_all();
    assert!(t.on_battlefield(stone));
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
}
