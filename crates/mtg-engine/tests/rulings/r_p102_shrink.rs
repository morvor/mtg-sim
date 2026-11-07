//! Rulings batch P102 — "creatures your opponents control get -N/-0 until end of turn"
//! (the affected set is locked in as the spell resolves, CR 611.2c), Public Execution,
//! and P/T bonuses that count things (Empyrial Armor, Armored Ascension, Walker of the
//! Wastes, Leyline Invocation, Descendant of Masumaro).

use crate::r_p015_common::gain_control;
use crate::r_p102_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::attach_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts the real card `name` (with plenty of mana) at the given targets and
/// resolves everything.
fn cast_resolve(t: &mut TestGame, name: &str, targets: &[Entity]) {
    supported(name);
    lots(t, P0);
    let c = t.hand(P0, name);
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.cast(P0, c).go();
    t.resolve_all();
}

/// The spells and their -N/-0.
const SHRINKS: [(&str, i32); 5] = [
    ("Bewildering Blizzard", 3),
    ("Blinding Spray", 4),
    ("Eliminate the Impossible", 2),
    ("Hysterical Blindness", 4),
    ("Turn the Tide", 2),
];

#[test]
fn mass_shrinks_affect_only_creatures_opponents_controlled_on_resolution() {
    cr!("611.2c", "613.4c");
    ruling!(
        "Bewildering Blizzard",
        "Bewildering Blizzard affects only creatures your opponents control at the time it resolves."
    );
    ruling!(
        "Blinding Spray",
        "Blinding Spray doesn’t affect creatures that enter the battlefield or come under the control of one of your opponents after it resolves."
    );
    ruling!(
        "Eliminate the Impossible",
        "Eliminate the Impossible affects only creatures your opponents control at the time it resolves."
    );
    ruling!(
        "Hysterical Blindness",
        "Only creatures controlled by your opponent when Hysterical Blindness resolves will get -4/-0."
    );
    ruling!(
        "Turn the Tide",
        "Only creatures your opponents control when Turn the Tide resolves are affected."
    );
    for (name, n) in SHRINKS {
        let mut t = TestGame::new(2);
        let theirs = t.battlefield(P1, "Hill Giant");
        let mine = t.battlefield(P0, "Hill Giant");
        cast_resolve(&mut t, name, &[]);
        assert_eq!(t.pt(theirs), (3 - n, 3), "{name}");
        assert_eq!(t.pt(mine), (3, 3), "{name}");
        // Later: a creature enters under P1's control, and P1 gains control of P0's.
        let later = t.battlefield(P1, "Hill Giant");
        gain_control(&mut t, P1, mine);
        assert_eq!(t.pt(later), (3, 3), "{name}");
        assert_eq!(t.pt(mine), (3, 3), "{name}");
    }
}

#[test]
fn eliminate_the_impossible_unsuspects_only_creatures_controlled_on_resolution() {
    cr!("611.2c", "701.60c");
    ruling!(
        "Eliminate the Impossible",
        "Creatures they begin to control later in the turn won't get -2/-0 and won't stop being suspected."
    );
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Hill Giant");
    for id in [theirs, mine] {
        t.g.objects[id.0 as usize].suspected = true;
    }
    t.g.recompute();
    cast_resolve(&mut t, "Eliminate the Impossible", &[]);
    assert!(!t.obj_now(theirs).suspected);
    assert!(t.obj_now(mine).suspected);
    gain_control(&mut t, P1, mine);
    assert!(t.obj_now(mine).suspected);
    assert_eq!(t.pt(mine), (3, 3));
}

#[test]
fn hysterical_blindness_keeps_applying_after_a_control_change() {
    cr!("611.2c");
    ruling!(
        "Hysterical Blindness",
        "Hysterical Blindness's effect will continue to apply to a creature even if you (or a teammate) gains control of that creature later in the turn."
    );
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, "Hysterical Blindness", &[]);
    gain_control(&mut t, P0, theirs);
    assert_eq!(t.obj_now(theirs).controller, P0);
    assert_eq!(t.pt(theirs), (-1, 3));
}

#[test]
fn shrink_and_draw_spells_can_be_cast_without_opposing_creatures() {
    cr!("601.2", "608.2");
    ruling!(
        "Blinding Spray",
        "You can cast Blinding Spray even if your opponents control no creatures just to draw a card."
    );
    ruling!(
        "Roilmage's Trick",
        "You can cast Roilmage’s Trick even if your opponents control no creatures. You’ll simply draw a card."
    );
    for name in ["Blinding Spray", "Roilmage's Trick"] {
        let mut t = TestGame::new(2);
        let hand = t.hand_size(P0);
        cast_resolve(&mut t, name, &[]);
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    }
}

#[test]
fn public_execution_shrinks_the_rest_even_if_the_target_survives() {
    cr!("608.2b", "611.2c", "701.8b");
    ruling!(
        "Public Execution",
        "If the target creature regenerates or has indestructible, each other creature controlled by that creature's controller will still get -2/-0."
    );
    ruling!(
        "Public Execution",
        "Only creatures controlled by the player when Public Execution resolves will get -2/-0."
    );
    supported("Public Execution");
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, "Public Execution", &[o(myr)]);
    assert!(t.on_battlefield(myr));
    assert_eq!(t.pt(myr), (0, 1), "the target itself isn't shrunk");
    assert_eq!(t.pt(giant), (1, 3));
    let later = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3));
}

#[test]
fn public_execution_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Public Execution",
        "Public Execution targets only the creature it destroys. If that creature is an illegal target when Public Execution tries to resolve, it won't resolve"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    lots(&mut t, P0);
    let pe = t.hand(P0, "Public Execution");
    t.cast(P0, pe).target(o(bears)).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn counted_bonuses_change_as_the_count_changes() {
    cr!("611.3a", "613.4c", "205.3i");
    ruling!(
        "Empyrial Armor",
        "The value of X is recalculated constantly, so this card’s bonus varies as the number of cards in your hand varies."
    );
    ruling!(
        "Armored Ascension",
        "This bonus is not fixed; it changes as the number of Plains you control changes."
    );
    ruling!(
        "Armored Ascension",
        "The ability cares about lands with the land type Plains, not necessarily lands named Plains."
    );
    supported("Empyrial Armor");
    supported("Armored Ascension");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Empyrial Armor", bears);
    assert_eq!(t.pt(bears), (2, 2));
    let c = t.hand(P0, "Shock");
    t.hand(P0, "Shock");
    assert_eq!(t.pt(bears), (4, 4));
    t.g.discard(P0, c, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    // Armored Ascension: Hallowed Fountain is a Plains; Gaea's Cradle isn't.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Armored Ascension", bears);
    assert_eq!(t.pt(bears), (2, 2));
    t.battlefield(P0, "Hallowed Fountain");
    t.battlefield(P0, "Gaea's Cradle");
    assert_eq!(t.pt(bears), (3, 3));
    let plains = t.battlefield(P0, "Plains");
    assert_eq!(t.pt(bears), (4, 4));
    destroy(&mut t, plains);
    assert_eq!(t.pt(bears), (3, 3));
    t.battlefield(P1, "Plains");
    assert_eq!(t.pt(bears), (3, 3), "only Plains you control");
}

#[test]
fn walker_of_the_wastes_bonus_only_on_the_battlefield() {
    cr!("611.3a", "113.6");
    ruling!(
        "Walker of the Wastes",
        "The last ability applies only if Walker of the Wastes is on the battlefield."
    );
    supported("Walker of the Wastes");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    let in_hand = t.hand(P0, "Walker of the Wastes");
    assert_eq!(t.pt(in_hand), (4, 4));
    let gy = t.graveyard(P0, "Walker of the Wastes");
    assert_eq!(t.pt(gy), (4, 4));
    let w = t.battlefield(P0, "Walker of the Wastes");
    assert_eq!(t.pt(w), (6, 6));
}

#[test]
fn leyline_invocation_counts_lands_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Leyline Invocation",
        "The number of lands you control is counted as Leyline Invocation resolves."
    );
    supported("Leyline Invocation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let li = t.hand(P0, "Leyline Invocation");
    t.cast(P0, li).go();
    // Two more lands before it resolves (and the six are tapped: still counted).
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Forest");
    t.resolve_all();
    let fractal = crate::r_s01_common::with_subtype(&t, P0, "Fractal")[0];
    assert_eq!(t.counters(fractal, counters::PLUS1), 8);
    assert_eq!(t.pt(fractal), (8, 8));
}

/// P0's Descendant of Masumaro at P0's upkeep with `mine` cards in P0's hand and
/// `theirs` in P1's, after `setup`; returns it after the trigger resolves.
fn descendant(
    mine: usize,
    theirs: usize,
    setup: impl FnOnce(&mut TestGame, ObjectId),
) -> (TestGame, ObjectId) {
    supported("Descendant of Masumaro");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Descendant of Masumaro");
    setup(&mut t, d);
    for _ in 0..mine {
        t.hand(P0, "Shock");
    }
    for _ in 0..theirs {
        t.hand(P1, "Shock");
    }
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.hand_size(P0), mine);
    t.resolve_all();
    (t, d)
}

#[test]
fn descendant_of_masumaro_counters() {
    cr!("704.3", "704.5q", "122.1", "608.2");
    ruling!(
        "Descendant of Masumaro",
        "All +1/+1 counters on Descendant of Masumaro are removed, including those put on it by other effects."
    );
    ruling!(
        "Descendant of Masumaro",
        "Descendant of Masumaro’s printed power and toughness is 1/1, so removing all the +1/+1 counters usually doesn’t reduce the creature’s toughness to 0."
    );
    ruling!(
        "Descendant of Masumaro",
        "The counters are added and then removed without a pause for priority or state-based actions being checked in between."
    );
    // Two counters from elsewhere; P0 has 1 card, P1 has 3: +1 then -3: none left, 1/1.
    let (t, d) = descendant(1, 3, |t, d| {
        crate::r_s29_common::put_counters(t, d, counters::PLUS1, 2)
    });
    assert_eq!(t.counters(d, counters::PLUS1), 0);
    assert_eq!(t.pt(d), (1, 1));
    assert!(t.on_battlefield(d));
    // A -1/-1 counter and Glorious Anthem: +2 then -2 with no state-based actions in
    // between (which would have annihilated a +1/+1 and the -1/-1 counter).
    let (t, d) = descendant(2, 2, |t, d| {
        t.battlefield(P0, "Glorious Anthem");
        crate::r_s29_common::put_counters(t, d, counters::MINUS1, 1);
    });
    assert_eq!(t.counters(d, counters::PLUS1), 0);
    assert_eq!(t.counters(d, counters::MINUS1), 1);
    assert_eq!(t.pt(d), (1, 1));
}
