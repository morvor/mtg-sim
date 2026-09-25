//! Qualified enters and attacks triggers: "enters untapped", "enters from a graveyard"
//! (CR 603.6a: enters-the-battlefield abilities look at the permanent as it entered), and
//! "attacks while saddled" (CR 702.171b); "when you discard ~"; attaching and unattaching
//! (CR 701.3); phasing in (CR 702.26c).

use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_line_supported(name: &str, needle: &str) {
    let c = card(name);
    let bad: Vec<_> = c
        .unsupported_text()
        .into_iter()
        .filter(|u| u.contains(needle))
        .collect();
    assert!(bad.is_empty(), "{name} has unsupported text: {bad:?}");
}

/// Moves a card onto the battlefield through a real zone change.
fn put_onto_battlefield(t: &mut TestGame, p: PlayerId, card: ObjectId, tapped: bool) -> ObjectId {
    t.g.move_object_ev(MoveEv {
        obj: card,
        to: Zone::Battlefield,
        pos: mtg_engine::ability::LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(p),
        etb: EtbInfo {
            tapped,
            controller: Some(p),
            ..Default::default()
        },
        source: None,
    })
    .expect("failed to enter the battlefield")
}

fn count_subtype(t: &TestGame, subtype: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| t.g.obj(**id).chars.has_subtype(subtype))
        .count()
}

#[test]
fn when_this_land_enters_untapped() {
    cr!("603.6a", "603.6d");
    assert_line_supported("Dwarven Mine", "enters untapped");
    for tapped in [false, true] {
        let mut t = TestGame::new(2);
        let mine = t.hand(P0, "Dwarven Mine");
        put_onto_battlefield(&mut t, P0, mine, tapped);
        t.resolve_all();
        let dwarves = if tapped { 0 } else { 1 };
        assert_eq!(count_subtype(&t, "Dwarf"), dwarves, "tapped: {tapped}");
    }
}

#[test]
fn when_this_creature_enters_from_a_graveyard() {
    cr!("603.6a", "400.7");
    assert_line_supported("Triarch Praetorian", "enters from a graveyard");
    // From the graveyard: draw two, lose 2 life.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Triarch Praetorian");
    let hand = t.hand_size(P0);
    put_onto_battlefield(&mut t, P0, card, false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), 18);
    // From the hand: nothing.
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Triarch Praetorian");
    put_onto_battlefield(&mut t, P0, card, false);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn attacks_while_saddled() {
    cr!("702.171b", "508.3a");
    assert_line_supported("Brightfield Mustang", "while saddled");
    for saddled in [false, true] {
        let mut t = TestGame::new(2);
        let horse = t.battlefield(P0, "Brightfield Mustang");
        t.g.objects[horse.0 as usize].saddled = saddled;
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(horse, Entity::Player(P1))], &[]);
        // "Untap it and put a +1/+1 counter on it."
        let expected = if saddled { 1 } else { 0 };
        assert_eq!(t.counters(horse, "+1/+1"), expected, "saddled: {saddled}");
    }
}

#[test]
fn when_you_discard_this_card() {
    cr!("701.9a", "603.2");
    assert_line_supported("Titanbones, Towering Heart", "When you discard");
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Titanbones, Towering Heart");
    t.g.discard(P0, card, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Put into the graveyard some other way: no trigger.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Titanbones, Towering Heart");
    t.g.mill(P0, 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn an_aura_becomes_attached_to_this_creature() {
    cr!("701.3a", "303.4");
    assert_line_supported("Bramble Elemental", "becomes attached");
    let mut t = TestGame::new(2);
    let bramble = t.battlefield(P0, "Bramble Elemental");
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, Entity::Object(bramble));
    t.resolve_all();
    assert_eq!(count_subtype(&t, "Saproling"), 2);
}

#[test]
fn equipment_becomes_unattached_sacrifice_that_permanent() {
    cr!("701.3a", "701.3c");
    assert_line_supported("Grafted Wargear", "becomes unattached");
    let mut t = TestGame::new(2);
    let wargear = t.battlefield(P0, "Grafted Wargear");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.g.attach(wargear, Entity::Object(a));
    t.resolve_all();
    assert!(t.on_battlefield(a));
    // Moving it to another creature unattaches it from the first.
    t.g.attach(wargear, Entity::Object(b));
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(b));
}

#[test]
fn whenever_this_creature_phases_in() {
    cr!("702.26c", "603.2");
    assert_line_supported("Warping Wurm", "phases in");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Warping Wurm");
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![wurm]);
    t.resolve_all();
    assert_eq!(t.counters(wurm, "+1/+1"), 0);
    mtg_engine::keyword_impls::phase_in(&mut t.g, wurm);
    t.resolve_all();
    assert_eq!(t.counters(wurm, "+1/+1"), 1);
}
