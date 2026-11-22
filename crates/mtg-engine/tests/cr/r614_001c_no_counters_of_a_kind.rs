//! CR 614.1c, 122.6: an "enters with" effect whose number comes to 0 gives the permanent no
//! counters, and the permanent has no counters of that kind: nothing that looks for the
//! kinds of counters on it finds one.
//!
//! - Gorma, the Gullet's "Nontoken creatures you control enter with an additional +1/+1
//!   counter on them for each creature that died under your control this turn." with no
//!   creature dead: Aven Riftwatcher (vanishing 3) has only its time counters, so Nesting
//!   Grounds' "Move a counter" has no kind to ask about (CR 122.5).
//! - A permanent returned "with X additional +1/+1 counters" for X = 0 (Abuelo's
//!   Awakening) as a copy of Blue, Loyal Raptor ("For each kind of counter on Blue, Loyal
//!   Raptor, each other Dinosaur you control enters with a counter of that kind on it.") has
//!   no kind of counter on it, so a Dinosaur entering afterward gets none.
//! - Sphere of the Suns returned that way for X = 0 has only its charge counters: Nesting
//!   Grounds has no kind to ask about.

use crate::r614_audit_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn gorma_with_no_creature_dead_leaves_aven_riftwatcher_only_its_time_counters_to_move() {
    cr!("614.1c", "614.1d", "122.5", "702.63a");
    supported("Gorma, the Gullet");
    supported("Aven Riftwatcher");
    supported("Nesting Grounds");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gorma, the Gullet");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Gorma's effect applies to Aven Riftwatcher as it enters, for 0 counters.
    let rift = cast_resolve(&mut t, P0, "Aven Riftwatcher", &[]);
    let rift = t.g.current(rift);
    assert!(t.on_battlefield(rift));
    assert_eq!(t.counters(rift, "time"), 3);
    assert_eq!(t.counters(rift, "+1/+1"), 0);
    // Nesting Grounds: "{1}, {T}: Move a counter from target permanent you control onto a
    // second target permanent. Activate only as a sorcery." The only kind of counter on the
    // Riftwatcher is time: no kind is asked for, and a time counter moves. (Were the
    // Riftwatcher left with an empty "+1/+1" kind, it would be offered, and the default
    // choice, the first kind, would move nothing.)
    let grounds = t.battlefield(P0, "Nesting Grounds");
    t.lands(P0, "Wastes", 1);
    let from = t.asked().len();
    t.activate(P0, grounds, 1, &[obj(rift), obj(bears)])
        .expect("Nesting Grounds's second ability");
    t.resolve_all();
    let kinds_asked: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt.contains("kind of counter") => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert!(kinds_asked.is_empty(), "{kinds_asked:?}");
    assert_eq!(t.counters(rift, "time"), 2);
    assert_eq!(t.counters(bears, "time"), 1);
}

#[test]
fn a_copy_of_blue_loyal_raptor_returned_with_no_counters_gives_other_dinosaurs_none() {
    cr!("614.1c", "614.12", "122.6", "707.2");
    supported("Abuelo's Awakening");
    supported("Phyrexian Metamorph");
    supported("Blue, Loyal Raptor");
    let mut t = TestGame::new(2);
    // P1's Blue, Loyal Raptor is the creature P0's Phyrexian Metamorph copies ("You may have
    // this creature enter as a copy of any artifact or creature on the battlefield, except
    // it's an artifact in addition to its other types."), returned by P0's Abuelo's Awakening
    // ("Return target artifact or non-Aura enchantment card from your graveyard to the
    // battlefield with X additional +1/+1 counters on it. It's a 1/1 Spirit creature with
    // flying in addition to its other types.") for X = 0.
    let blue = t.battlefield(P1, "Blue, Loyal Raptor");
    let metamorph = t.graveyard(P0, "Phyrexian Metamorph");
    t.lands(P0, "Plains", 4);
    t.answer_choose(P0, &[obj(blue)]);
    let awakening = t.hand(P0, "Abuelo's Awakening");
    t.cast(P0, awakening).x(0).target(metamorph).go();
    t.resolve_all();
    let copy = t.g.current(metamorph);
    assert!(t.on_battlefield(copy), "the Metamorph returns");
    assert_eq!(t.obj_now(copy).chars.name, "Blue, Loyal Raptor");
    assert_eq!(t.counters(copy, "+1/+1"), 0);
    // A Dinosaur entering under P0's control: P0's Blue has no kind of counter on it (an
    // empty "+1/+1" kind would count as one and give the Dreadmaw a +1/+1 counter).
    let wurm = t.enter(P0, "Colossal Dreadmaw");
    assert_eq!(t.counters(wurm, "+1/+1"), 0);
    assert_eq!(t.pt(wurm), (6, 6));
}

/// Regression test: a destination's "with X additional +1/+1 counters" for X = 0 (Abuelo's
/// Awakening) gave the permanent an empty "+1/+1" kind (`Game::perform_move` proposed an
/// event of 0 counters and recorded its kind), which Nesting Grounds' "Move a counter"
/// offered beside its real one; the default, the first kind, moved nothing (CR 122.5: "if
/// the first object doesn't have the appropriate kind of counter on it").
#[test]
fn sphere_of_the_suns_returned_with_no_additional_counters_has_only_charge_counters_to_move() {
    cr!("614.1c", "122.5", "122.6");
    supported("Abuelo's Awakening");
    supported("Sphere of the Suns");
    supported("Nesting Grounds");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Sphere of the Suns: "This artifact enters tapped and with three charge counters on
    // it." Abuelo's Awakening: "Return target artifact or non-Aura enchantment card from
    // your graveyard to the battlefield with X additional +1/+1 counters on it. It's a 1/1
    // Spirit creature with flying in addition to its other types." X = 0.
    let card = t.graveyard(P0, "Sphere of the Suns");
    t.lands(P0, "Plains", 4);
    let awakening = t.hand(P0, "Abuelo's Awakening");
    t.cast(P0, awakening).x(0).target(card).go();
    t.resolve_all();
    let sphere = t.g.current(card);
    assert!(t.on_battlefield(sphere), "the Sphere returns");
    assert_eq!(t.counters(sphere, "charge"), 3);
    assert_eq!(t.counters(sphere, "+1/+1"), 0);
    // Nesting Grounds moves a counter from the Sphere to the Bears: the only kind is charge,
    // so none is asked for, and a charge counter moves.
    let grounds = t.battlefield(P0, "Nesting Grounds");
    t.lands(P0, "Wastes", 1);
    let from = t.asked().len();
    t.activate(P0, grounds, 1, &[obj(sphere), obj(bears)])
        .expect("Nesting Grounds's second ability");
    t.resolve_all();
    let kinds_asked: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt.contains("kind of counter") => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert!(kinds_asked.is_empty(), "{kinds_asked:?}");
    assert_eq!(t.counters(sphere, "charge"), 2);
    assert_eq!(t.counters(bears, "charge"), 1);
}
