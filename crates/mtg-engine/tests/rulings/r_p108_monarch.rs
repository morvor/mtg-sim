//! Rulings batch P108 — abilities that care about the monarch (CR 725): "exile target
//! creature an opponent controls until an opponent becomes the monarch" (Palace Jailer,
//! CR 610.3), "if you're the monarch" / "if an opponent is the monarch" upkeep triggers
//! (intervening "if", CR 603.4), and "can't be blocked by creatures the monarch controls".

use crate::r_p108_common::*;
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Answer;
use mtg_engine::designations::become_monarch;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Makes `p` the monarch and processes the event.
fn monarch(t: &mut TestGame, p: PlayerId) {
    become_monarch(&mut t.g, p);
    t.g.flush_events();
    t.settle();
}

/// The text of the ability on top of the stack.
fn top_text(t: &TestGame) -> String {
    let top = *t.g.stack.last().expect("stack");
    match &t.g.obj(top).stack.as_deref().expect("stack info").kind {
        StackKind::Triggered { ability, .. } => ability.text.clone(),
        _ => String::new(),
    }
}

/// `n`-player game where P0's Palace Jailer enters and exiles `victim` (P1's creature,
/// made by `victim`) with its exile trigger resolving first if `exile_first`. Both
/// triggers resolve.
fn jailer(
    n: usize,
    exile_first: bool,
    setup: impl Fn(&mut TestGame),
    victim: impl Fn(&mut TestGame) -> ObjectId,
) -> (TestGame, ObjectId, ObjectId) {
    supported("Palace Jailer");
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(n);
        setup(&mut t);
        let v = victim(&mut t);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.answer_targets(P0, &[obj(v)]);
        let j = t.enter(P0, "Palace Jailer");
        t.settle();
        if top_text(&t).contains("exile") != exile_first {
            continue;
        }
        t.resolve_all();
        return (t, j, v);
    }
    panic!("no trigger order puts the exile trigger where wanted");
}

#[test]
fn palace_jailer_exile_drops_auras_and_counters_and_keeps_equipment() {
    cr!("610.3", "303.4c", "704.5m", "704.5n", "122.2");
    ruling!(
        "Palace Jailer",
        "Auras attached to the exiled creature will be put into their owners' graveyards. Equipment attached to the exiled creature will become unattached and remain on the battlefield. Counters on the exiled creature will cease to exist."
    );
    let (mut t, _, v) = jailer(
        2,
        true,
        |_| {},
        |t| {
            let v = t.battlefield(P1, "Hill Giant");
            put_counters(t, v, "+1/+1", 2);
            attach_new(t, P1, "Bonesplitter", v);
            attach_new(t, P1, "Holy Strength", v);
            v
        },
    );
    assert_eq!(t.zone(v), Zone::Exile);
    assert!(t.in_graveyard(P1, "Holy Strength"));
    let bs = t.named_on_battlefield("Bonesplitter");
    assert_eq!(bs.len(), 1);
    assert!(t.obj_now(bs[0]).attached_to.is_none());
    // It returns as a new object without counters.
    monarch(&mut t, P1);
    assert!(t.on_battlefield(v));
    assert_eq!(t.counters(v, "+1/+1"), 0);
}

#[test]
fn palace_jailer_exiled_token_ceases_to_exist() {
    cr!("111.8", "704.5d");
    ruling!(
        "Palace Jailer",
        "If a creature token is exiled, it ceases to exist. It won't return to the battlefield."
    );
    let (mut t, _, v) = jailer(2, true, |_| {}, |t| create_token(t, P1, "Soldier"));
    assert!(!t.on_battlefield(v));
    monarch(&mut t, P1);
    assert_eq!(tokens(&t, P1).len(), 0);
}

#[test]
fn palace_jailer_waits_for_an_opponent_to_become_the_monarch() {
    cr!("610.3", "725.1");
    ruling!(
        "Palace Jailer",
        "If you're not the monarch as Palace Jailer's second ability resolves, the creature will be exiled until there's a new monarch and that player is one of your opponents. The creature won't immediately return just because an opponent is the monarch."
    );
    // P1 is the monarch as the exile trigger resolves; P0 then becomes the monarch.
    let (mut t, _, v) = jailer(2, true, |t| monarch(t, P1), |t| t.battlefield(P1, "Hill Giant"));
    assert_eq!(t.zone(v), Zone::Exile);
    assert_eq!(t.g.monarch, Some(P0));
    // An opponent becoming the monarch returns it.
    monarch(&mut t, P1);
    assert!(t.on_battlefield(v));
    assert_eq!(t.obj_now(v).controller, P1);
}

#[test]
fn palace_jailer_leaving_doesnt_return_the_creature() {
    cr!("610.3");
    ruling!(
        "Palace Jailer",
        "Palace Jailer leaving the battlefield won't cause the exiled creature to return. The game will continue to watch for the next time an opponent becomes the monarch."
    );
    let (mut t, j, v) = jailer(2, false, |_| {}, |t| t.battlefield(P1, "Hill Giant"));
    destroy(&mut t, j);
    assert_eq!(t.zone(v), Zone::Exile);
    monarch(&mut t, P1);
    assert!(t.on_battlefield(v));
}

#[test]
fn palace_jailer_returns_when_any_opponent_becomes_the_monarch() {
    cr!("610.3", "725.1");
    ruling!(
        "Palace Jailer",
        "The opponent that controlled the exiled card doesn't have to be the same opponent that becomes the monarch in order to cause that card to return to the battlefield. Any opponent becoming the monarch will cause the card to return."
    );
    let (mut t, _, v) = jailer(3, false, |_| {}, |t| t.battlefield(P1, "Hill Giant"));
    assert_eq!(t.zone(v), Zone::Exile);
    monarch(&mut t, P2);
    assert!(t.on_battlefield(v));
    assert_eq!(t.obj_now(v).controller, P1);
}

// --- Upkeep triggers -----------------------------------------------------------------------

/// Advances to P0's next upkeep (from P1's turn) with the triggers put on the stack.
fn to_upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

#[test]
fn queen_marchesa_checks_an_opponent_is_the_monarch() {
    cr!("603.4", "725.1");
    ruling!(
        "Queen Marchesa",
        "checks to see if an opponent is the monarch as your upkeep begins. If no opponent is the monarch, Queen Marchesa's (long may she reign) ability won't trigger at all."
    );
    supported("Queen Marchesa");
    // You're the monarch: no trigger.
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Queen Marchesa");
    monarch(&mut t, P0);
    to_upkeep(&mut t);
    assert!(crate::r_s25_common::abilities_from(&t, q).is_empty());
    // An opponent is: an Assassin.
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Queen Marchesa");
    monarch(&mut t, P1);
    to_upkeep(&mut t);
    assert_eq!(crate::r_s25_common::abilities_from(&t, q).len(), 1);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Assassin"), 1);
    // The opponent stops being the monarch before it resolves: no effect.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Queen Marchesa");
    monarch(&mut t, P1);
    to_upkeep(&mut t);
    monarch(&mut t, P0);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Assassin"), 0);
}

#[test]
fn skyline_despot_checks_you_are_the_monarch() {
    cr!("603.4", "725.1");
    ruling!(
        "Skyline Despot",
        "The last ability of Skyline Despot checks to see if you're the monarch as your upkeep begins. If you're not, the ability won't trigger at all."
    );
    supported("Skyline Despot");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Skyline Despot");
    monarch(&mut t, P1);
    to_upkeep(&mut t);
    assert!(crate::r_s25_common::abilities_from(&t, d).is_empty());
    // Becoming the monarch during the upkeep is too late.
    monarch(&mut t, P0);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Dragon"), 0);
    // Monarch as the upkeep begins, but not as it resolves: no Dragon.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Skyline Despot");
    monarch(&mut t, P0);
    to_upkeep(&mut t);
    assert_eq!(crate::r_s25_common::abilities_from(&t, d).len(), 1);
    monarch(&mut t, P1);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Dragon"), 0);
    // Monarch throughout: a Dragon.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skyline Despot");
    monarch(&mut t, P0);
    to_upkeep(&mut t);
    t.resolve_all();
    assert_eq!(tokens_with(&t, P0, "Dragon"), 1);
}

// --- Azure Fleet Admiral -------------------------------------------------------------------

#[test]
fn azure_fleet_admiral_can_be_blocked_by_the_monarchs_teammate() {
    cr!("509.1b", "725.1", "810.2");
    ruling!(
        "Azure Fleet Admiral",
        "In some variants with shared team turns, such as Two-Headed Giant, creatures controlled by the monarch's teammate can block Azure Fleet Admiral, even if it's attacking the monarch."
    );
    supported("Azure Fleet Admiral");
    let mut t = two_headed_giant();
    let admiral = t.battlefield(P0, "Azure Fleet Admiral");
    let monarchs = t.battlefield(P2, "Grizzly Bears");
    let mates = t.battlefield(P3, "Grizzly Bears");
    monarch(&mut t, P2);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(admiral, Entity::Player(P2))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.g.recompute();
    assert!(!t.g.can_block(monarchs, admiral));
    assert!(t.g.can_block(mates, admiral));
}
