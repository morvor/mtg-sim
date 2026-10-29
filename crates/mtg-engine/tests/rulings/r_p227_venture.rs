//! Rulings batch P227 — venture into the dungeon (CR 701.49): Acererak the Archlich's
//! intervening "if you haven't completed Tomb of Annihilation" (CR 603.4), and Zalto,
//! Fire Giant Duke's "Whenever Zalto is dealt damage".

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use mtg_engine::card::card;
use mtg_engine::dungeons;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const TOMB: &str = "Tomb of Annihilation";

/// The dungeon P0's venture marker is in, and its room.
fn at(t: &TestGame, p: PlayerId) -> Option<(String, usize)> {
    dungeons::marker(&t.g, p).map(|(d, r)| (t.obj_now(d).chars.name.to_string(), r))
}

/// P0 ventures into Tomb of Annihilation until they complete it, resolving only the room
/// abilities (whatever was on the stack before stays there).
fn complete_tomb(t: &mut TestGame) {
    let base = t.stack_len();
    for _ in 0..10 {
        dungeons::venture_into(&mut t.g, P0, None, Some(TOMB));
        t.g.flush_events();
        t.settle();
        while t.stack_len() > base {
            t.resolve();
        }
        if t.g.player(P0).completed_dungeons.iter().any(|n| n == TOMB) {
            break;
        }
    }
    assert_eq!(t.g.player(P0).completed_dungeons, vec![TOMB]);
    assert_eq!(at(t, P0), None);
}

/// Acererak's first ability compiles (its attack trigger is the only unsupported text).
fn acererak_compiles() {
    let c = card("Acererak the Archlich");
    assert!(
        c.unsupported_text()
            .iter()
            .all(|u| u.starts_with("Whenever ~ attacks")),
        "{:?}",
        c.unsupported_text()
    );
}

#[test]
fn acererak_doesnt_trigger_or_does_nothing_once_tomb_is_completed() {
    cr!("603.4", "309.7", "701.49a");
    ruling!(
        "Acererak the Archlich",
        "Acererak the Archlich has an intervening if clause in its first triggered ability."
    );
    acererak_compiles();
    // Not completed: it returns to hand and P0 ventures into the dungeon.
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Acererak the Archlich");
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(t.in_hand(P0, "Acererak the Archlich"));
    assert!(at(&t, P0).is_some());
    // Completed: the ability doesn't trigger at all.
    let mut t = TestGame::new(2);
    complete_tomb(&mut t);
    let a = t.enter(P0, "Acererak the Archlich");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(a));
    // Completing another dungeon doesn't matter.
    let mut t = TestGame::new(2);
    t.g.players[0].dungeons_completed = 1;
    t.g.players[0]
        .completed_dungeons
        .push("Lost Mine of Phandelver".into());
    t.enter(P0, "Acererak the Archlich");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Completing Tomb of Annihilation with the ability on the stack: it does nothing.
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Acererak the Archlich");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    complete_tomb(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert_eq!(at(&t, P0), None);
    assert_eq!(t.g.player(P0).completed_dungeons.len(), 1);
}

#[test]
fn acererak_gone_still_ventures() {
    cr!("603.4", "608.2b", "701.49a");
    ruling!(
        "Acererak the Archlich",
        "If Acererak is no longer on the battlefield as its first ability resolves and you haven't completed Tomb of Annihilation, you will still venture into the dungeon."
    );
    acererak_compiles();
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Acererak the Archlich");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Acererak the Archlich"));
    assert!(!t.in_hand(P0, "Acererak the Archlich"));
    assert!(at(&t, P0).is_some());
}

/// P0's Zalto attacks P1 and is blocked by P1's `blockers` Grizzly Bears.
fn zalto_blocked_by_bears(blockers: usize) -> (TestGame, ObjectId) {
    supported("Zalto, Fire Giant Duke");
    let mut t = TestGame::new(2);
    let zalto = t.battlefield(P0, "Zalto, Fire Giant Duke");
    let bears: Vec<ObjectId> = (0..blockers)
        .map(|_| t.battlefield(P1, "Grizzly Bears"))
        .collect();
    attack_with(&mut t, &[(zalto, Entity::Player(P1))]);
    let blocks: Vec<(ObjectId, ObjectId)> = bears.iter().map(|b| (*b, zalto)).collect();
    block_and_finish(&mut t, P1, &blocks);
    t.resolve_all();
    (t, zalto)
}

#[test]
fn zalto_damaged_by_several_sources_at_once_ventures_once() {
    cr!("603.2c", "510.2", "701.49a");
    ruling!(
        "Zalto, Fire Giant Duke",
        "If Zalto is dealt damage by multiple sources at the same time (usually creatures blocking it), you will only venture into the dungeon once."
    );
    // Blocked by one Grizzly Bears: Zalto survives (2 damage) and ventures once.
    let (t, zalto) = zalto_blocked_by_bears(1);
    assert!(t.on_battlefield(zalto));
    assert_eq!(at(&t, P0).map(|(_, r)| r), Some(0));
    // By two Grizzly Bears at the same time: still just once.
    let (t, _) = zalto_blocked_by_bears(2);
    assert_eq!(at(&t, P0).map(|(_, r)| r), Some(0));
}

#[test]
fn zalto_dealt_lethal_damage_still_ventures() {
    cr!("603.2", "704.5g", "701.49a");
    ruling!(
        "Zalto, Fire Giant Duke",
        "If Zalto is dealt lethal damage, you will still get to venture into the dungeon."
    );
    // Two Grizzly Bears deal it 4 damage in combat.
    let (t, zalto) = zalto_blocked_by_bears(2);
    assert_eq!(t.zone(zalto), Zone::Graveyard(P0));
    assert!(at(&t, P0).is_some());
    // P1's Lightning Bolt deals it 3.
    let mut t = TestGame::new(2);
    let zalto = t.battlefield(P0, "Zalto, Fire Giant Duke");
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Lightning Bolt");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Object(zalto)).go();
    t.resolve_all();
    assert_eq!(t.zone(zalto), Zone::Graveyard(P0));
    assert!(at(&t, P0).is_some());
}
