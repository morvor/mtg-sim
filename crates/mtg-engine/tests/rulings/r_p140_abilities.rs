//! Rulings batch P140 — removing flying, hexproof and indestructible: Radjan Spirit,
//! Gravity Sphere, Grounded, Walking Sponge, Arcane Lighthouse, Detection Tower, Nowhere
//! to Run and Bonds of Mortality.

use crate::r_p140_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn has(t: &mut TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.g.recompute();
    t.obj(t.g.current(id)).has_keyword(k)
}

fn gain(t: &mut TestGame, id: ObjectId, k: KeywordKind) {
    modify(t, id, vec![Modification::AddKeyword(Keyword::new(k))]);
}

/// Whether `p` could target `target` with a fresh Shock now.
fn can_shock(t: &mut TestGame, p: PlayerId, target: ObjectId) -> bool {
    can_target_with(t, p, "Shock", target)
}

fn can_target_with(t: &mut TestGame, p: PlayerId, spell: &str, target: ObjectId) -> bool {
    lots(t, p);
    let card_ = t.hand(p, spell);
    let from = t.asked().len();
    t.answer(p, DecisionKind::Targets, Answer::Default);
    t.g.turn.priority = Some(p);
    let r =
        t.g.cast_spell(p, card_, mtg_engine::object::CastMethod::Normal);
    let offered = target_candidates(t, p, from)
        .concat()
        .contains(&obj(t.g.current(target)));
    if let Ok(s) = r {
        // Take the spell back off the stack.
        move_to(t, s, Zone::Exile);
    }
    t.clear_answers();
    offered
}

#[test]
fn radjan_spirit_on_a_creature_without_flying_does_nothing() {
    cr!("601.2c", "602.2b");
    ruling!(
        "Radjan Spirit",
        "Can be used on a creature without Flying with no effect."
    );
    supported("Radjan Spirit");
    // A creature without flying is a legal target; the ability resolves and does nothing
    // (the flier, which wasn't the target, keeps flying).
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Radjan Spirit");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let drake = t.battlefield(P1, "Wind Drake");
    let from = t.asked().len();
    activate_lots(&mut t, P0, spirit, 0, &[obj(bears)]);
    assert!(target_candidates(&t, P0, from)
        .concat()
        .contains(&obj(bears)));
    let ab = *t.stack.last().expect("the ability is on the stack");
    let targets = &t.obj(ab).stack.as_ref().unwrap().chosen[0].targets;
    assert_eq!(targets.concat(), vec![obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!has(&mut t, bears, KeywordKind::Flying));
    assert!(has(&mut t, drake, KeywordKind::Flying));
    // Control: on the flier, it loses flying.
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Radjan Spirit");
    let drake = t.battlefield(P1, "Wind Drake");
    activate_lots(&mut t, P0, spirit, 0, &[obj(drake)]);
    t.resolve_all();
    assert!(!has(&mut t, drake, KeywordKind::Flying));
}

#[test]
fn gravity_sphere_and_grounded_dont_stop_flying_granted_later() {
    cr!("613.1f", "613.7");
    ruling!(
        "Gravity Sphere",
        "It removes Flying from all creatures on the battlefield when it enters and from each creature as that creature enters. It does not prevent the Flying ability being given to the creature *after* the Sphere is on the battlefield."
    );
    ruling!(
        "Grounded",
        "The creature can later gain flying from another effect. That will override Grounded’s effect."
    );
    supported("Gravity Sphere");
    supported("Grounded");
    // Gravity Sphere: a flier already there, one entering later, then a granted flying.
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    t.battlefield(P0, "Gravity Sphere");
    assert!(!has(&mut t, drake, KeywordKind::Flying));
    let later = t.enter(P1, "Wind Drake");
    t.settle();
    assert!(
        !has(&mut t, later, KeywordKind::Flying),
        "a flier entering later loses it"
    );
    gain(&mut t, drake, KeywordKind::Flying);
    assert!(
        has(&mut t, drake, KeywordKind::Flying),
        "granted afterwards: it flies"
    );
    assert!(!has(&mut t, later, KeywordKind::Flying));
    // Grounded.
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    cast_lots(&mut t, P0, "Grounded", &[obj(drake)]);
    t.resolve_all();
    assert!(!has(&mut t, drake, KeywordKind::Flying));
    gain(&mut t, drake, KeywordKind::Flying);
    assert!(has(&mut t, drake, KeywordKind::Flying));
}

#[test]
fn walking_sponge_removes_just_the_chosen_ability() {
    cr!("608.2d");
    ruling!(
        "Walking Sponge",
        "The target loses just one of the listed abilities."
    );
    supported("Walking Sponge");
    let kinds = [
        KeywordKind::Flying,
        KeywordKind::FirstStrike,
        KeywordKind::Trample,
    ];
    for (i, lost) in kinds.iter().enumerate() {
        let mut t = TestGame::new(2);
        let sponge = t.battlefield(P0, "Walking Sponge");
        let angel = t.battlefield(P1, "Serra Angel");
        gain(&mut t, angel, KeywordKind::FirstStrike);
        gain(&mut t, angel, KeywordKind::Trample);
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
        activate_lots(&mut t, P0, sponge, 0, &[obj(angel)]);
        t.resolve_all();
        for k in kinds {
            assert_eq!(has(&mut t, angel, k), k != *lost, "chose {lost:?}; {k:?}");
        }
        assert!(has(&mut t, angel, KeywordKind::Vigilance));
    }
}

#[test]
fn arcane_lighthouse_stops_later_grants_but_only_for_creatures_there_then() {
    cr!("611.2c", "613.1f");
    ruling!(
        "Arcane Lighthouse",
        "After the second ability resolves, continuous effects generated by the resolution of spells and abilities that would give hexproof or shroud to one of the affected creatures aren't created."
    );
    ruling!(
        "Arcane Lighthouse",
        "Continuous effects generated by static abilities (such as an Aura that grants hexproof to a creature) will not apply during the turn, but will resume applying once the turn ends."
    );
    ruling!(
        "Arcane Lighthouse",
        "The second ability applies only to creatures controlled by your opponents when it resolves. Creatures that enter the battlefield or come under an opponent's control later in the turn won't be affected."
    );
    supported("Arcane Lighthouse");
    let mut t = TestGame::new(2);
    let house = t.battlefield(P0, "Arcane Lighthouse");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let cloaked = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Gladecover Scout");
    cast_lots(&mut t, P1, "Alexi's Cloak", &[obj(cloaked)]);
    t.resolve_all();
    assert!(!can_shock(&mut t, P0, cloaked), "shroud");
    activate_lots(&mut t, P0, house, 1, &[]);
    t.resolve_all();
    assert!(can_shock(&mut t, P0, scout), "printed hexproof is gone");
    assert!(
        can_shock(&mut t, P0, cloaked),
        "the Aura's shroud doesn't apply"
    );
    // A hexproof creature that comes under an opponent's control later is unaffected.
    give_control(&mut t, mine, P1);
    assert!(
        !can_shock(&mut t, P0, mine),
        "P0's former Scout keeps hexproof"
    );
    // A later spell giving hexproof: the +2/+2 applies, the hexproof doesn't.
    cast_lots(&mut t, P1, "Blossoming Defense", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(!has(&mut t, bears, KeywordKind::Hexproof));
    assert!(can_shock(&mut t, P0, bears));
    // A hexproof creature entering later is unaffected.
    let late = t.enter(P1, "Gladecover Scout");
    t.settle();
    assert!(!can_shock(&mut t, P0, late));
    // The next turn, the Aura's shroud applies again.
    t.advance_to(P1, Step::Upkeep);
    assert!(!can_shock(&mut t, P0, cloaked), "shroud is back");
    assert!(!can_shock(&mut t, P0, scout));
}

#[test]
fn detection_tower_covers_creatures_arriving_later_and_hexproof_from_qualities() {
    cr!("702.11b", "702.11d", "702.11e", "611.2c");
    ruling!(
        "Detection Tower",
        "Because Detection Tower’s last ability doesn’t change the characteristics of any permanents, the set of creatures affected by it is constantly updated. Creatures that come under your opponents’ control later in the turn can be targeted as though they didn’t have hexproof."
    );
    ruling!(
        "Detection Tower",
        "Creatures your opponents control with “hexproof from [quality]” abilities can be targeted as though they didn’t have those abilities as well."
    );
    supported("Detection Tower");
    let mut t = TestGame::new(2);
    let tower = t.battlefield(P0, "Detection Tower");
    let knight = t.battlefield(P1, "Knight of Grace");
    let mine = t.battlefield(P0, "Gladecover Scout");
    assert!(
        !can_target_with(&mut t, P0, "Disfigure", knight),
        "hexproof from black"
    );
    activate_lots(&mut t, P0, tower, 1, &[]);
    t.resolve_all();
    assert!(can_target_with(&mut t, P0, "Disfigure", knight));
    let late = t.enter(P1, "Gladecover Scout");
    t.settle();
    assert!(
        can_shock(&mut t, P0, late),
        "a creature arriving later is covered too"
    );
    assert!(
        has(&mut t, late, KeywordKind::Hexproof),
        "it still has hexproof"
    );
    // One that comes under an opponent's control later in the turn is covered too.
    give_control(&mut t, mine, P1);
    assert!(can_shock(&mut t, P0, mine), "now an opponent's: covered");
    assert!(has(&mut t, mine, KeywordKind::Hexproof));
    // Only until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!can_shock(&mut t, P0, late));
}

#[test]
fn nowhere_to_run_leaving_doesnt_trigger_ward_but_makes_hexproof_targets_illegal() {
    cr!("702.21a", "608.2b", "702.11b");
    ruling!(
        "Nowhere to Run",
        "If a spell or ability targeting a creature an opponent controls with a ward ability is on the stack, causing Nowhere to Run to leave the battlefield won't cause that ward ability to trigger."
    );
    ruling!(
        "Nowhere to Run",
        "If a spell or ability you control targets a creature an opponent controls with hexproof and Nowhere to Run leaves the battlefield while that spell or ability is on the stack, that creature becomes an illegal target for that spell or ability. This includes Nowhere to Run's own triggered ability."
    );
    supported("Nowhere to Run");
    // Ward: Shock targets the Honor Guard while Nowhere to Run is out; then it leaves.
    let mut t = TestGame::new(2);
    let ntr = t.battlefield(P0, "Nowhere to Run");
    let guard = t.battlefield(P1, "Tomakul Honor Guard");
    cast_lots(&mut t, P0, "Shock", &[obj(guard)]);
    t.settle();
    assert_eq!(t.stack_len(), 1, "no ward trigger");
    destroy(&mut t, ntr);
    t.settle();
    assert_eq!(t.stack_len(), 1, "still no ward trigger");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Tomakul Honor Guard"), "Shock resolved");
    // Positive control: without Nowhere to Run, ward triggers.
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P1, "Tomakul Honor Guard");
    cast_lots(&mut t, P0, "Shock", &[obj(guard)]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "ward triggers");
    // Hexproof: Nowhere to Run's own trigger targets the Scout, then it leaves.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.answer_targets(P0, &[obj(scout)]);
    let ntr = t.enter(P0, "Nowhere to Run");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, ntr);
    t.resolve_all();
    assert!(t.on_battlefield(scout), "an illegal target: no -3/-3");
    assert_eq!(t.pt(scout), (1, 1));
    // Hexproof: a spell of yours targets the Scout, then Nowhere to Run leaves.
    let mut t = TestGame::new(2);
    let ntr = t.battlefield(P0, "Nowhere to Run");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let shock = cast_lots(&mut t, P0, "Shock", &[obj(scout)]);
    let chosen = &t.obj(shock).stack.as_ref().unwrap().chosen[0].targets;
    assert_eq!(chosen.concat(), vec![obj(scout)], "Shock targets the Scout");
    destroy(&mut t, ntr);
    t.resolve_all();
    assert!(t.on_battlefield(scout), "Shock has an illegal target now");
    assert!(t.in_graveyard(P0, "Shock"));
}

#[test]
fn bonds_of_mortality_later_grants_work_normally() {
    cr!("611.2c", "613.7");
    ruling!(
        "Bonds of Mortality",
        "The last ability causes creatures your opponents control to lose hexproof and indestructible as it resolves. Effects that cause those creatures to gain those abilities after that point will work normally."
    );
    supported("Bonds of Mortality");
    let mut t = TestGame::new(2);
    let bonds = t.battlefield(P0, "Bonds of Mortality");
    let myr = t.battlefield(P1, "Darksteel Myr");
    let other = t.battlefield(P1, "Darksteel Myr");
    let scout = t.battlefield(P1, "Gladecover Scout");
    activate_lots(&mut t, P0, bonds, 0, &[]);
    t.resolve_all();
    assert!(!has(&mut t, myr, KeywordKind::Indestructible));
    assert!(!has(&mut t, scout, KeywordKind::Hexproof));
    gain(&mut t, other, KeywordKind::Indestructible);
    gain(&mut t, scout, KeywordKind::Hexproof);
    assert!(
        has(&mut t, other, KeywordKind::Indestructible),
        "granted later: works"
    );
    assert!(has(&mut t, scout, KeywordKind::Hexproof));
    destroy(&mut t, myr);
    destroy(&mut t, other);
    assert!(!t.on_battlefield(myr), "lost indestructible: destroyed");
    assert!(t.on_battlefield(other), "regained indestructible");
}
