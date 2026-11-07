//! Rulings batch P038 — creatures exiled for a while: Day of the Dragons, Aligned Hedron
//! Network, Eerie Interlude, Portcullis, and Aerial Assault against Banisher Priest
//! (CR 400.7, 603.6, 610.3, 603.3b).

use crate::r_p038_common::*;
use crate::r_s01_common::{supported, tokens};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn dragons(t: &TestGame, p: PlayerId) -> usize {
    tokens(t, p)
        .into_iter()
        .filter(|o| t.obj_now(*o).chars.has_subtype("Dragon"))
        .count()
}

#[test]
fn day_of_the_dragons_returns_every_exiled_card_even_if_the_dragons_are_gone() {
    cr!("603.6c", "610.3");
    ruling!(
        "Day of the Dragons",
        "All of the nontoken creatures you exiled return to the battlefield when Day of the Dragons leaves the battlefield, even if some or all of the Dragon tokens had already left the battlefield."
    );
    ruling!(
        "Day of the Dragons",
        "Token creatures will count to determine how many Dragon tokens you create, but they won’t be returned to the battlefield when Day of the Dragons leaves the battlefield."
    );
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Raise the Alarm", &[]);
    assert_eq!(creature_count(&t, P0), 3);
    let day = enter(&mut t, P0, "Day of the Dragons");
    t.resolve_all();
    assert_eq!(dragons(&t, P0), 3, "the two Soldier tokens count");
    assert!(t.in_exile("Grizzly Bears"));
    // All the Dragons leave first.
    for d in tokens(&t, P0) {
        kill(&mut t, d);
    }
    assert_eq!(dragons(&t, P0), 0);
    kill(&mut t, day);
    t.resolve_all();
    // The Bears return; the Soldier tokens ceased to exist in exile.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.obj_now(t.named_on_battlefield("Grizzly Bears")[0]).controller, P0);
    assert_ne!(t.named_on_battlefield("Grizzly Bears")[0], bear, "a new object");
    assert_eq!(creature_count(&t, P0), 1);
}

#[test]
fn day_of_the_dragons_leaving_before_its_first_trigger_resolves() {
    cr!("603.6c", "603.3b", "608.2b");
    ruling!(
        "Day of the Dragons",
        "If Day of the Dragons leaves the battlefield before its first ability has resolved, its second ability will trigger and you’ll sacrifice any Dragons you already control."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shivan Dragon");
    t.battlefield(P0, "Grizzly Bears");
    let day = enter(&mut t, P0, "Day of the Dragons");
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, day);
    assert_eq!(t.stack_len(), 2);
    t.resolve(); // leaves-the-battlefield trigger: Shivan Dragon is sacrificed
    assert!(t.in_graveyard(P0, "Shivan Dragon"));
    t.resolve_all(); // enters trigger: the Bears are exiled for good
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(dragons(&t, P0), 1, "that many Dragon tokens: the Bears");
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

/// A creature of `p` with an Aura (Holy Strength), Equipment (Bonesplitter) and a +1/+1
/// counter.
fn decorated(t: &mut TestGame, p: PlayerId, name: &str) -> (ObjectId, ObjectId, ObjectId) {
    let c = t.battlefield(p, name);
    let aura = t.battlefield(p, "Holy Strength");
    assert!(t.g.attach(aura, obj(c)));
    let eq = t.battlefield(p, "Bonesplitter");
    assert!(t.g.attach(eq, obj(c)));
    t.g.add_counters(obj(c), counters::PLUS1, 1, None);
    t.g.recompute();
    (c, aura, eq)
}

fn assert_exiled_cleanly(t: &TestGame, aura: ObjectId, eq: ObjectId) {
    assert!(t.in_graveyard(P0, "Holy Strength"), "the Aura goes to the graveyard");
    assert!(!t.on_battlefield(aura));
    assert!(t.on_battlefield(eq), "the Equipment stays");
    assert_eq!(t.obj_now(eq).attached_to, None);
}

fn assert_returned_without_counters(t: &TestGame, name: &str) {
    let back = t.named_on_battlefield(name);
    assert_eq!(back.len(), 1, "{name} returned");
    assert_eq!(t.counters(back[0], counters::PLUS1), 0);
}

#[test]
fn exiled_creatures_lose_auras_equipment_and_counters() {
    cr!("400.7", "303.4c", "301.5c", "122.2");
    ruling!(
        "Day of the Dragons",
        "Auras attached to the exiled creatures will be put into their owners’ graveyards. Equipment attached to the exiled creatures will become unattached and remain on the battlefield. Any counters on the exiled creatures will cease to exist."
    );
    ruling!(
        "Aligned Hedron Network",
        "Auras attached to the exiled creatures will be put into their owners’ graveyards. Equipment attached to the exiled creatures will become unattached and remain on the battlefield. Any counters on the exiled creatures will cease to exist."
    );
    ruling!(
        "Eerie Interlude",
        "Auras attached to the exiled creatures will be put into their owners’ graveyards. Equipment attached to the exiled creatures will become unattached and remain on the battlefield. Any counters on the exiled creatures will cease to exist."
    );
    // Day of the Dragons: "exile all creatures you control".
    let mut t = TestGame::new(2);
    let (_, aura, eq) = decorated(&mut t, P0, "Grizzly Bears");
    let day = enter(&mut t, P0, "Day of the Dragons");
    t.resolve_all();
    assert_exiled_cleanly(&t, aura, eq);
    kill(&mut t, day);
    t.resolve_all();
    assert_returned_without_counters(&t, "Grizzly Bears");

    // Aligned Hedron Network: "exile all creatures with power 5 or greater until ...".
    let mut t = TestGame::new(2);
    let (_, aura, eq) = decorated(&mut t, P0, "Craw Wurm");
    let net = enter(&mut t, P0, "Aligned Hedron Network");
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm"));
    assert_exiled_cleanly(&t, aura, eq);
    kill(&mut t, net);
    t.resolve_all();
    assert_returned_without_counters(&t, "Craw Wurm");

    // Eerie Interlude: "Exile any number of target creatures you control. Return those
    // cards ... at the beginning of the next end step."
    let mut t = TestGame::new(2);
    let (bear, aura, eq) = decorated(&mut t, P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Eerie Interlude", &[obj(bear)]);
    assert!(t.in_exile("Grizzly Bears"));
    assert_exiled_cleanly(&t, aura, eq);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_returned_without_counters(&t, "Grizzly Bears");
}

/// Portcullis with two other creatures on the battlefield (P1's).
fn portcullis(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Portcullis");
    let port = t.battlefield(p, "Portcullis");
    bears(t, P1, 2);
    port
}

#[test]
fn portcullis_ignores_creatures_phasing_in() {
    cr!("702.26d", "603.6a");
    ruling!(
        "Portcullis",
        "Creatures which are phasing in will not trigger this card's ability."
    );
    let mut t = TestGame::new(2);
    portcullis(&mut t, P0);
    let giant = t.battlefield(P1, "Hill Giant");
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![giant]);
    t.g.recompute();
    assert!(phased_out(&t, giant));
    // P1's untap step phases the Giant in.
    t.advance_to(P1, Step::Upkeep);
    assert!(!phased_out(&t, giant));
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(giant));
}

#[test]
fn portcullis_does_nothing_if_the_creature_already_left() {
    cr!("603.6a", "608.2b");
    ruling!(
        "Portcullis",
        "If the creature that triggered Portcullis is no longer on the battlefield when the ability resolves, then it fails to do anything."
    );
    let mut t = TestGame::new(2);
    let port = portcullis(&mut t, P0);
    let giant = enter(&mut t, P0, "Hill Giant");
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, giant);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(!t.in_exile("Hill Giant"));
    // Nothing was exiled, so nothing returns when Portcullis leaves.
    kill(&mut t, port);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}

#[test]
fn portcullis_trigger_is_ordered_with_other_enters_triggers() {
    cr!("603.3b", "101.4");
    ruling!(
        "Portcullis",
        "The creature does enter, so any other abilities that trigger on it entering trigger."
    );
    // P1 controls Portcullis; P0 (the active player) casts Elvish Visionary. P0's trigger
    // goes on the stack first, so Portcullis's resolves first: the Visionary is exiled
    // and then its own trigger still draws a card.
    let mut t = TestGame::new(2);
    portcullis(&mut t, P1);
    let hand = t.hand_size(P0);
    enter(&mut t, P0, "Elvish Visionary");
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(t.in_exile("Elvish Visionary"));
    assert_eq!(t.hand_size(P0), hand);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);

    // P0 controls both triggers: P0 is asked to order them.
    let mut t = TestGame::new(2);
    portcullis(&mut t, P0);
    let from = t.asked().len();
    enter(&mut t, P0, "Elvish Visionary");
    assert_eq!(t.stack_len(), 2);
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::Order { .. })));
}

#[test]
fn creatures_banisher_priest_exiled_return_before_aerial_assault_gains_life() {
    cr!("610.3c", "608.2c");
    ruling!(
        "Aerial Assault",
        "If any creatures were exiled \"until\" the target creature left the battlefield, those creatures will return before you gain life."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P1, &[obj(angel)]);
    let priest = enter(&mut t, P1, "Banisher Priest");
    t.resolve_all();
    assert!(t.in_exile("Serra Angel"));
    tap(&mut t, priest);
    t.set_step(P0, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Aerial Assault", &[obj(priest)]);
    assert!(t.in_graveyard(P1, "Banisher Priest"));
    assert_eq!(t.named_on_battlefield("Serra Angel").len(), 1);
    assert_eq!(t.life(P0), 21, "the returned Serra Angel counts");
}
