//! Rulings batch P100 — continuous effects around mana values: effects that set power and
//! toughness in timestamp order within layer 7b (CR 613.4b, 613.7), modifications and
//! counters above them (613.4c), switching last (613.4d), effects with no duration
//! lasting while the object stays (611.2a, 400.7), a "for as long as" duration that ended
//! before its effect began (611.2b), and protection shared by a tie (Favor of the Mighty).

use crate::r_p100_common::*;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Tenth District Hero under P0 with lands and evidence for both abilities.
fn hero(t: &mut TestGame) -> ObjectId {
    let h = t.battlefield(P0, "Tenth District Hero");
    t.lands(P0, "Plains", 5);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    h
}

/// An effect setting the creature's base power and toughness to 1/1 until end of turn
/// (as a resolving spell would, with a timestamp of now; it keeps its abilities).
fn set_1_1(t: &mut TestGame, id: ObjectId) {
    modify(t, id, vec![Modification::SetPT(Some(Value::c(1)), Some(Value::c(1)))]);
}

/// Turn to Frog ("Target creature loses all abilities and becomes a blue Frog with base
/// power and toughness 1/1 until end of turn") on the creature, cast by P1.
fn turn_to_frog(t: &mut TestGame, id: ObjectId) {
    let frog = t.hand(P1, "Turn to Frog");
    mana(t, P1, "1U");
    t.g.turn.priority = Some(P1);
    let id = t.g.current(id);
    t.cast(P1, frog).target(obj(id)).go();
    t.resolve_all();
}

#[test]
fn tenth_district_heros_set_effects_overwrite_earlier_ones_and_yield_to_later_ones() {
    cr!("613.4b", "613.4c", "613.4d", "613.7", "613.7b");
    ruling!(
        "Tenth District Hero",
        "The effects from Tenth District Hero's abilities overwrite other effects that set power and/or toughness if and only if those effects existed before the ability resolved. They will not overwrite effects that modify power or toughness without setting it (whether from a static ability, counters, or a resolved spell or ability), nor will they overwrite effects that set power and toughness which come into existence after they resolve. Effects that switch the creature's power and toughness are always applied after any other power or toughness changing effects, including these two, regardless of the order in which they are created."
    );
    supported("Tenth District Hero");
    supported("Twisted Image");
    // An effect setting it to 1/1, then the first ability: 4/4. A +1/+1 counter on top:
    // 5/5. Then another 1/1 effect: 2/2 (the counter still applies). Then the second
    // ability: 6/6. Then Twisted Image, and +3/+0 after it: the switch applies last, so
    // 6/9.
    let mut t = TestGame::new(2);
    let h = hero(&mut t);
    assert_eq!(t.pt(h), (2, 3));
    set_1_1(&mut t, h);
    assert_eq!(t.pt(h), (1, 1));
    t.activate(P0, h, 0, &[]).expect("first ability");
    t.resolve_all();
    assert_eq!(t.pt(h), (4, 4), "the later set effect wins");
    put(&mut t, h, counters::PLUS1, 1);
    assert_eq!(t.pt(h), (5, 5));
    set_1_1(&mut t, h);
    assert_eq!(t.pt(h), (2, 2), "a set effect created later wins; the counter stays");
    assert!(t.obj_now(h).chars.subtypes.iter().any(|s| s == "Detective"));
    t.activate(P0, h, 1, &[]).expect("second ability");
    t.resolve_all();
    assert_eq!(t.pt(h), (6, 6));
    assert_eq!(t.obj_now(h).chars.name, "Mileva, the Stalwart");
    crate::r_p003_common::switch_pt(&mut t, h);
    assert_eq!(t.pt(h), (6, 6));
    modify(&mut t, h, vec![Modification::ModifyPT(Value::c(3), Value::c(0))]);
    assert_eq!(t.pt(h), (6, 9), "+3/+0 first, then the switch (CR 613.4d)");
}

#[test]
fn tenth_district_heros_effects_have_no_duration() {
    cr!("611.2a", "400.7");
    ruling!(
        "Tenth District Hero",
        "Neither of these abilities have durations. If one of them resolves, it will remain in effect until the game ends, Tenth District Hero leaves the battlefield, or some subsequent effect changes its characteristics, whichever comes first."
    );
    supported("Tenth District Hero");
    let mut t = TestGame::new(2);
    let h = hero(&mut t);
    t.activate(P0, h, 0, &[]).expect("first ability");
    t.resolve_all();
    assert_eq!(t.pt(h), (4, 4));
    // Still a 4/4 Detective with vigilance in the next turn.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.pt(h), (4, 4));
    assert!(t.obj_now(h).chars.subtypes.iter().any(|s| s == "Detective"));
    assert!(has_kw(&t, h, KeywordKind::Vigilance));
    // Bounced and replayed: a new object, 1/1 again.
    bounce(&mut t, h);
    assert_eq!(t.zone(h), Zone::Hand(P0));
    let h = t.g.current(h);
    t.lands(P0, "Plains", 2);
    t.cast(P0, h).go();
    t.resolve_all();
    assert_eq!(t.pt(h), (2, 3));
    assert!(!t.obj_now(h).chars.subtypes.iter().any(|s| s == "Detective"));
}

#[test]
fn tenth_district_heros_second_ability_checks_for_a_detective_however_it_became_one() {
    cr!("608.2c", "613.1d");
    ruling!(
        "Tenth District Hero",
        "Tenth District Hero's second ability checks whether it's a Detective. It doesn't matter how it became a Detective."
    );
    supported("Tenth District Hero");
    // Not a Detective: the second ability does nothing.
    let mut t = TestGame::new(2);
    let h = hero(&mut t);
    t.activate(P0, h, 1, &[]).expect("second ability");
    t.resolve_all();
    assert_eq!(t.pt(h), (2, 3));
    assert_eq!(t.obj_now(h).chars.name, "Tenth District Hero");
    // Made a Detective by another effect: it becomes Mileva, the Stalwart.
    let mut t = TestGame::new(2);
    let h = hero(&mut t);
    modify(&mut t, h, vec![Modification::AddSubtypes(vec!["Detective".into()])]);
    t.activate(P0, h, 1, &[]).expect("second ability");
    t.resolve_all();
    assert_eq!(t.obj_now(h).chars.name, "Mileva, the Stalwart");
    assert_eq!(t.pt(h), (5, 5));
    assert!(t.obj_now(h).chars.is_legendary());
}

#[test]
fn the_capitoline_triads_emblem_overwrites_earlier_set_effects_only() {
    cr!("613.4b", "613.4c", "613.7", "614.1c");
    ruling!(
        "The Capitoline Triad",
        "The ability of the emblem created by The Capitoline Triad overwrites all previous effects that set the power and toughness of creatures you control to specific values. Other effects that set these characteristics to specific values that start to apply after the emblem is created will overwrite this effect."
    );
    supported("The Capitoline Triad");
    supported("Darksteel Forge");
    let mut t = TestGame::new(2);
    let triad = t.battlefield(P0, "The Capitoline Triad");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    // Darksteel Forge (9) three times, Mind Stone (2) and Sol Ring (1): 30.
    let historic: Vec<ObjectId> = ["Darksteel Forge", "Darksteel Forge", "Darksteel Forge", "Mind Stone", "Sol Ring"]
        .iter()
        .map(|c| t.graveyard(P0, c))
        .collect();
    turn_to_frog(&mut t, bears);
    assert_eq!(t.pt(bears), (1, 1));
    put(&mut t, giant, counters::PLUS1, 1);
    let entities: Vec<Entity> = historic.iter().map(|o| obj(*o)).collect();
    t.answer_choose(P0, &entities);
    act(&mut t, P0, triad, "Exile any number of historic cards", &[]).expect("Triad");
    t.resolve_all();
    assert!(historic.iter().all(|c| t.zone(*c) == Zone::Exile));
    assert_eq!(t.pt(bears), (9, 9), "the emblem's later set effect wins over Turn to Frog");
    assert_eq!(t.pt(giant), (10, 10), "the counter applies on top");
    assert_eq!(t.pt(triad), (9, 9));
    // A set effect created after the emblem wins over it.
    turn_to_frog(&mut t, giant);
    assert_eq!(t.pt(giant), (2, 2));
}

#[test]
fn favor_of_the_mighty_protects_every_creature_tied_for_the_greatest_mana_value() {
    cr!("702.16b", "702.16e");
    ruling!(
        "Favor of the Mighty",
        "If there's a tie for highest mana value, all of those creatures have protection from all colors."
    );
    supported("Favor of the Mighty");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Favor of the Mighty");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    assert!(protected_from(&t, a, goblin));
    assert!(protected_from(&t, b, goblin));
    assert!(!protected_from(&t, goblin, a));
    // A Hill Giant (4) breaks the tie: only it has protection.
    let giant = t.battlefield(P1, "Hill Giant");
    t.settle();
    assert!(protected_from(&t, giant, goblin));
    assert!(!protected_from(&t, a, goblin));
    assert!(!protected_from(&t, b, goblin));
}

#[test]
fn hedge_whisperer_leaving_before_resolution_animates_nothing() {
    cr!("611.2b");
    ruling!(
        "Hedge Whisperer",
        "If Hedge Whisperer leaves the battlefield before its activated ability resolves, the target land won't become a creature at all."
    );
    supported("Hedge Whisperer");
    let mut t = TestGame::new(2);
    let whisperer = t.battlefield(P0, "Hedge Whisperer");
    let land = t.battlefield(P0, "Forest");
    let wurm = t.graveyard(P0, "Craw Wurm");
    mana(&mut t, P0, "3G");
    t.answer_choose(P0, &[obj(wurm)]);
    act(&mut t, P0, whisperer, "becomes a 5/5", &[obj(land)]).expect("Hedge Whisperer");
    assert!(t.obj_now(whisperer).tapped);
    destroy(&mut t, whisperer);
    t.resolve_all();
    assert!(!t.obj_now(land).is_creature(), "{}", t.dump_log());
    assert!(t.obj_now(land).chars.is_land());
    // With the Whisperer staying tapped, the land is a 5/5 Plant Boar with haste.
    let mut t = TestGame::new(2);
    let whisperer = t.battlefield(P0, "Hedge Whisperer");
    let land = t.battlefield(P0, "Forest");
    let wurm = t.graveyard(P0, "Craw Wurm");
    mana(&mut t, P0, "3G");
    t.answer_choose(P0, &[obj(wurm)]);
    act(&mut t, P0, whisperer, "becomes a 5/5", &[obj(land)]).expect("Hedge Whisperer");
    t.resolve_all();
    assert!(t.obj_now(land).is_creature());
    assert_eq!(t.pt(land), (5, 5));
}
