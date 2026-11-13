//! A target whose requirement names another target ("target Equipment attached to that
//! creature", "target permanents that player controls", "two target creatures controlled
//! by different players") when that other target has become illegal (CR 608.2b): the
//! requirement is still checked against it, as it is if it's still there (as it last
//! existed if it has left), so the dependent target stays legal while the relationship
//! holds and is affected (register `dependent-target-checked-against-an-illegal-target`,
//! decided by the user Nov 7; work items ruling-conflicts-4, case 1, and
//! user-questions-nov7, Part A). Fiery Annihilation's ruling that its Equipment "won't be
//! exiled" whenever the creature is an illegal target goes the other way: it is set aside
//! as inconsistent (exempt in docs/rulings-exemptions/rulings-P047.tsv, logged in
//! docs/RULING_DISCREPANCIES.md).

use crate::r_p007_common::{pl, player_hexproof};
use crate::r_p047_common::obj;
use crate::r_p113_common::{spoil, Spoil};
use crate::r_p133_common::{fizzles, kill};
use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s06_common::give_control;
use mtg_engine::attraction_cards;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// P1's Grizzly Bears wear a Bonesplitter (colorless). P0 casts Fiery Annihilation at the
/// Bears and the Bonesplitter; in response P1 casts `response` at the Bears (choosing
/// `color`, for Gods Willing); everything resolves. Returns (game, Bears, Bonesplitter).
fn fiery_annihilation_after(
    response: &str,
    color: Option<Color>,
) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sword = t.battlefield(P1, "Bonesplitter");
    t.g.attach(sword, obj(bears));
    let fa = t.hand(P0, "Fiery Annihilation");
    give_mana_for(&mut t, P0, "Fiery Annihilation");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(sword)]);
    t.cast(P0, fa).go();
    let r = t.hand(P1, response);
    give_mana_for(&mut t, P1, response);
    if let Some(c) = color {
        choose_color(&mut t, P1, c);
    }
    t.cast(P1, r).target(bears).go();
    t.resolve_all();
    (t, bears, sword)
}

#[test]
fn fiery_annihilation_exiles_equipment_still_attached_to_an_illegal_creature() {
    cr!("608.2b", "702.11b", "702.16b");
    interp!("dependent-target-checked-against-an-illegal-target");
    supported("Fiery Annihilation");
    supported("Ranger's Guile");
    supported("Gods Willing");
    // "Fiery Annihilation deals 5 damage to target creature. Exile up to one target
    // Equipment attached to that creature. ..." In response the Bears become an illegal
    // target but stay on the battlefield with the Bonesplitter attached, so the Bonesplitter
    // is still "attached to that creature": a legal target, exiled.
    // Ranger's Guile (+1/+1 and hexproof): the Bears aren't dealt damage only because
    // they're an illegal target (5 damage would kill the 3/3, and it would be exiled).
    let (t, bears, sword) = fiery_annihilation_after("Ranger's Guile", None);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.zone(sword), Zone::Exile);
    // Gods Willing, protection from red: the same; the colorless Bonesplitter stays
    // attached, and is exiled. (Protection from red would also prevent the damage, CR
    // 702.16e, so the Bears' survival doesn't by itself show they weren't affected here.)
    let (t, bears, sword) = fiery_annihilation_after("Gods Willing", Some(Color::Red));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(sword), Zone::Exile);
    // Control: protection from blue leaves both targets legal; both are exiled.
    let (t, bears, sword) = fiery_annihilation_after("Gods Willing", Some(Color::Blue));
    assert_eq!(t.zone(sword), Zone::Exile);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn yosei_taps_the_permanents_of_a_player_who_became_an_illegal_target() {
    cr!("608.2b", "702.11c");
    ruling!(
        "Yosei, the Morning Star",
        "If the player becomes an illegal target, the target permanents are tapped, but the player doesn't skip their untap step."
    );
    interp!("dependent-target-checked-against-an-illegal-target");
    supported("Yosei, the Morning Star");
    // "When Yosei dies, target player skips their next untap step. Tap up to five target
    // permanents that player controls." `hexproof`: P1 gains hexproof (Leyline of Sanctity)
    // with the trigger on the stack.
    for hexproof in [true, false] {
        let mut t = TestGame::new(2);
        let yosei = t.battlefield(P0, "Yosei, the Morning Star");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_targets(P0, &[obj(bears), obj(giant)]);
        t.g.destroy(yosei, None);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if hexproof {
            player_hexproof(&mut t, P1);
        }
        t.resolve_all();
        // The permanents "that player controls" are checked against P1, illegal or not, and
        // tapped either way.
        assert!(t.obj_now(bears).tapped, "hexproof {hexproof}");
        assert!(t.obj_now(giant).tapped, "hexproof {hexproof}");
        // P1 skips the untap step only if P1 was a legal target.
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(t.obj_now(bears).tapped, !hexproof, "hexproof {hexproof}");
        assert_eq!(t.obj_now(giant).tapped, !hexproof, "hexproof {hexproof}");
    }
}

/// P1 has a Drop Tower (an Attraction) and a Grizzly Bears card in hand; P0 casts Down for
/// Repairs targeting P1 and (if `tower_too`) the Drop Tower, and `respond` happens before
/// it resolves. Returns (game, Drop Tower, Bears card, whether the spell resolved).
fn down_for_repairs(
    tower_too: bool,
    respond: impl FnOnce(&mut TestGame, ObjectId),
) -> (TestGame, ObjectId, ObjectId, bool) {
    let mut t = TestGame::new(2);
    let tower = t.battlefield(P1, "Drop Tower");
    let bears = t.hand(P1, "Grizzly Bears");
    let card = t.hand(P0, "Down for Repairs");
    give_mana_for(&mut t, P0, "Down for Repairs");
    t.answer_choose(P0, &[obj(bears)]);
    let cast = t.cast(P0, card).target(pl(P1));
    if tower_too {
        cast.target(tower).go();
    } else {
        cast.go();
    }
    respond(&mut t, tower);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Down for Repairs"));
    let resolved = t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, events::Event::SpellResolved { .. }));
    (t, tower, bears, resolved)
}

#[test]
fn down_for_repairs_destroys_the_attraction_of_a_player_who_became_an_illegal_target() {
    cr!("608.2b", "702.11c", "717.6");
    ruling!(
        "Down for Repairs",
        "If only one of those targets is legal as the spell resolves, you’ll still do as much as you can to the remaining legal target. The illegal target will be unaffected."
    );
    interp!("dependent-target-checked-against-an-illegal-target");
    supported("Down for Repairs");
    // "Target opponent reveals their hand. You choose a nonland card from it. That player
    // discards that card. Destroy up to one target Attraction that player controls."
    // P1 gains hexproof (Leyline of Sanctity) in response: the Attraction is still one
    // "that player controls", a legal target, and is destroyed (put into the junkyard);
    // P1, illegal, discards nothing.
    let (t, tower, bears, resolved) = down_for_repairs(true, |t, _| player_hexproof(t, P1));
    assert!(resolved);
    assert!(!t.on_battlefield(tower));
    assert_eq!(attraction_cards::junkyard(&t.g, P1).len(), 1);
    assert_eq!(t.zone(bears), Zone::Hand(P1));
    // The Attraction leaves first: P1, still legal, discards the chosen card.
    let leaves = |t: &mut TestGame, tower: ObjectId| {
        t.g.destroy(tower, None);
        t.settle();
    };
    let (t, _, bears, resolved) = down_for_repairs(true, leaves);
    assert!(resolved);
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    // Both legal: both are affected.
    let (t, tower, bears, _) = down_for_repairs(true, |_, _| {});
    assert!(!t.on_battlefield(tower));
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    // "If both targets become illegal, the spell won’t resolve and none of its effects
    // will happen.": P1 gains hexproof and the Attraction leaves.
    let (t, _, bears, resolved) = down_for_repairs(true, |t, tower| {
        player_hexproof(t, P1);
        leaves(t, tower);
    });
    assert!(!resolved);
    assert_eq!(t.zone(bears), Zone::Hand(P1));
    // "the Attraction, if any": cast with no Attraction targeted, P1 still discards and
    // the Drop Tower stays.
    let (t, tower, bears, resolved) = down_for_repairs(false, |_, _| {});
    assert!(resolved);
    assert!(t.on_battlefield(tower));
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
}

/// P0's Grizzly Bears and P1's Hill Giant, and P0's Run Away Together on the stack
/// targeting both. Returns (game, spell, Bears, Giant).
fn run_away_together() -> (TestGame, ObjectId, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P0, "Run Away Together");
    let card = t.hand(P0, "Run Away Together");
    let s = t.cast(P0, card).target(mine).target(theirs).go();
    (t, s, mine, theirs)
}

#[test]
fn run_away_together_compares_the_other_creature_with_an_illegal_one() {
    cr!("608.2b", "702.11b");
    interp!("dependent-target-checked-against-an-illegal-target");
    ruling!(
        "Run Away Together",
        "If one of the two target creatures becomes an illegal target, Run Away Together can still determine its controller only to check whether the other creature is a legal target. If the illegal target has left the battlefield, use its last known information. If the other creature is still a legal target, it's returned to its owner's hand."
    );
    supported("Run Away Together");
    // "Choose two target creatures controlled by different players. Return those creatures
    // to their owners' hands." The Giant gains hexproof: an illegal target still controlled
    // by P1, so the Bears are still controlled by a different player and are returned.
    let (mut t, _, mine, theirs) = run_away_together();
    spoil(&mut t, theirs, Spoil::Hexproof);
    t.resolve_all();
    assert_eq!(t.zone(mine), Zone::Hand(P0));
    assert!(t.on_battlefield(theirs));
    // The Giant gains hexproof and P1 gains control of the Bears: the illegal Giant's
    // controller is still determined, and it's the Bears' controller now, so the Bears
    // aren't a legal target either and the spell doesn't resolve.
    let (mut t, s, mine, theirs) = run_away_together();
    fizzles(&mut t, s, |t| {
        spoil(t, theirs, Spoil::Hexproof);
        give_control(t, mine, P1);
    });
    assert!(t.on_battlefield(mine) && t.on_battlefield(theirs));
    // The Bears die and P0 gains control of the Giant: the Bears' last known information
    // (controlled by P0) is used, and the Giant is P0's now, so it isn't returned.
    let (mut t, s, mine, theirs) = run_away_together();
    fizzles(&mut t, s, |t| {
        kill(t, mine);
        give_control(t, theirs, P0);
    });
    assert!(t.on_battlefield(theirs));
    assert_eq!(t.obj_now(theirs).controller, P0);
}
