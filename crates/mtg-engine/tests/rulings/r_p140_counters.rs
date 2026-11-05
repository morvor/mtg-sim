//! Rulings batch P140 — removing counters: Aether Snap, Blitz Leech, Fury Charm, Medicine
//! Runner and Render Inert.

use crate::r_p140_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn total_counters(t: &TestGame, id: ObjectId) -> u32 {
    t.obj(t.g.current(id)).counters.values().sum()
}

#[test]
fn aether_snap_removes_every_kind_of_counter_from_permanents_only() {
    cr!("122.1", "702.62b");
    ruling!(
        "Aether Snap",
        "Aether Snap removes all counters from permanents, no matter what kind they are."
    );
    ruling!(
        "Aether Snap",
        "Counters on objects that aren't permanents, such as time counters on a suspended card or poison counters on a player, are unaffected by Aether Snap."
    );
    supported("Aether Snap");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 2);
    let giant = t.battlefield(P1, "Hill Giant");
    put_counters(&mut t, giant, counters::MINUS1, 1);
    put_counters(&mut t, giant, "flying", 1);
    let stone = t.battlefield(P1, "Millstone");
    put_counters(&mut t, stone, counters::CHARGE, 3);
    let token = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    // A suspended card and a player's poison counters.
    let bolt = t.exile(P0, "Rift Bolt");
    put_counters(&mut t, bolt, counters::TIME, 2);
    crate::r_p142_common::give_player_counters(&mut t, P1, counters::POISON, 3);
    cast_lots(&mut t, P0, "Aether Snap", &[]);
    t.resolve_all();
    for id in [bears, giant, stone] {
        assert_eq!(total_counters(&t, id), 0, "{}", t.obj(t.g.current(id)).chars.name);
    }
    assert_eq!(t.pt(t.g.current(bears)), (2, 2));
    assert_eq!(t.pt(t.g.current(giant)), (3, 3));
    assert!(!t.on_battlefield(token));
    assert!(tokens_of(&t, P1).is_empty(), "tokens are exiled");
    assert_eq!(t.counters(t.g.current(bolt), counters::TIME), 2);
    assert_eq!(t.g.player(P1).counters.get(counters::POISON), Some(&3));
}

#[test]
fn blitz_leech_removes_the_counters_before_state_based_actions() {
    cr!("608.2c", "704.3", "704.5f");
    ruling!(
        "Blitz Leech",
        "Blitz Leech’s ability removes all counters from the target creature before state-based actions are checked to see if that creature should die."
    );
    supported("Blitz Leech");
    // Grizzly Bears with two +1/+1 counters (4/4): -2/-2 and no counters is 0/0.
    // Hill Giant with a -1/-1 counter (2/2): -2/-2 alone would kill it; with the counter
    // removed it's a 1/1 and survives.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 2);
    put_counters(&mut t, bears, "flying", 1);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Blitz Leech");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    put_counters(&mut t, giant, counters::MINUS1, 1);
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Blitz Leech");
    t.resolve_all();
    assert!(t.on_battlefield(giant), "the -1/-1 counter went first");
    assert_eq!(t.pt(giant), (1, 1));
}

#[test]
fn fury_charm_removes_a_lone_time_counter() {
    cr!("700.2a", "702.62a", "702.62b");
    ruling!(
        "Fury Charm",
        "If a permanent or suspended card has only one time counter on it, Fury Charm's last mode removes that one counter."
    );
    supported("Fury Charm");
    // A suspended card: its last time counter is removed, so it's cast.
    let mut t = TestGame::new(2);
    let bolt = t.exile(P0, "Rift Bolt");
    put_counters(&mut t, bolt, counters::TIME, 1);
    lots(&mut t, P0);
    let charm = t.hand(P0, "Fury Charm");
    t.answer_targets(P0, &[obj(bolt)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, charm).modes(&[2]).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "the last counter came off: Rift Bolt was cast");
    // A permanent with one time counter.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Millstone");
    put_counters(&mut t, stone, counters::TIME, 1);
    lots(&mut t, P0);
    let charm = t.hand(P0, "Fury Charm");
    t.answer_targets(P0, &[obj(stone)]);
    t.cast(P0, charm).modes(&[2]).go();
    t.resolve_all();
    assert_eq!(t.counters(stone, counters::TIME), 0);
}

#[test]
fn medicine_runner_any_counter_or_none() {
    cr!("122.1", "115.1");
    ruling!(
        "Medicine Runner",
        "This can remove any kind of counter, not just a -1/-1 counter."
    );
    ruling!(
        "Medicine Runner",
        "You may target a permanent that doesn’t have a counter on it."
    );
    supported("Medicine Runner");
    for kind in [counters::CHARGE, counters::PLUS1, counters::MINUS1] {
        let mut t = TestGame::new(2);
        let target = if kind == counters::CHARGE {
            t.battlefield(P1, "Millstone")
        } else {
            t.battlefield(P1, "Hill Giant")
        };
        put_counters(&mut t, target, kind, 2);
        t.answer_targets(P0, &[obj(target)]);
        t.answer_yes(P0, true);
        t.enter(P0, "Medicine Runner");
        t.resolve_all();
        assert_eq!(t.counters(target, kind), 1, "{kind}");
    }
    // No counters anywhere: the trigger still targets (and does nothing).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    let runner = t.enter(P0, "Medicine Runner");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let offered: Vec<Entity> = target_candidates(&t, P0, from).concat();
    assert!(offered.contains(&obj(bears)) && offered.contains(&obj(runner)));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn render_inert_on_a_permanent_without_counters_still_draws() {
    cr!("115.1", "608.2c");
    ruling!(
        "Render Inert",
        "You can choose a permanent with no counters on it as the target. You’ll still draw a card."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_lots(&mut t, P0, "Render Inert", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(zone_of(&t, bears), Zone::Battlefield);
}
