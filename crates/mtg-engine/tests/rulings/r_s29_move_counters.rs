//! Rulings batch S29 — moving counters (CR 122.5): a moved counter is removed from one
//! object and put on another, so abilities that care about counters being removed or put
//! on apply; nothing moves if either object can't take part (an illegal target, CR 608.2b).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s04_common::{add_mana, next_upkeep};
use crate::r_s06_common::activate_containing;
use crate::r_s29_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const GROWTH: &str = "growth";

/// P0 activates Simic Fluxmage's "{1}{U}, {T}: Move a +1/+1 counter from this creature
/// onto target creature." targeting `target`.
fn fluxmage_moves(t: &mut TestGame, fluxmage: ObjectId, target: ObjectId) {
    add_mana(t, P0, ManaType::U, 1);
    add_mana(t, P0, ManaType::C, 1);
    t.answer_targets(P0, &[Entity::Object(target)]);
    activate_containing(t, P0, fluxmage, "Move").expect("Simic Fluxmage's ability");
}

/// P0 activates Daghatar the Adamant's "{1}{B/G}{B/G}: Move a +1/+1 counter from target
/// creature onto a second target creature."
fn daghatar_moves(t: &mut TestGame, daghatar: ObjectId, from: ObjectId, to: ObjectId) {
    add_mana(t, P0, ManaType::B, 2);
    add_mana(t, P0, ManaType::C, 1);
    t.answer_targets(P0, &[Entity::Object(from)]);
    t.answer_targets(P0, &[Entity::Object(to)]);
    activate_containing(t, P0, daghatar, "Move").expect("Daghatar's ability");
}

#[test]
fn a_moved_counter_is_put_on_the_second_creature() {
    cr!("122.5", "122.6");
    ruling!(
        "Simic Fluxmage",
        "To move a counter from one creature to another, the counter is removed from the first creature and placed on the second. Any abilities that care about a counter being placed on the second creature will apply."
    );
    supported("Simic Fluxmage");
    supported("Daghatar the Adamant");
    supported("Simic Ascendancy");
    supported("Hardened Scales");
    // Simic Ascendancy: "Whenever one or more +1/+1 counters are put on a creature you
    // control, put that many growth counters on this enchantment."
    let mut t = TestGame::new(2);
    let ascendancy = t.battlefield(P0, "Simic Ascendancy");
    let fluxmage = t.battlefield(P0, "Simic Fluxmage");
    put_counters(&mut t, fluxmage, counters::PLUS1, 1);
    t.resolve_all();
    let growth = t.counters(ascendancy, GROWTH);
    let bears = t.battlefield(P0, "Grizzly Bears");
    fluxmage_moves(&mut t, fluxmage, bears);
    t.resolve_all();
    assert_eq!(t.counters(fluxmage, counters::PLUS1), 0);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(ascendancy, GROWTH), growth + 1);
    // Daghatar, with Hardened Scales ("If one or more +1/+1 counters would be put on a
    // creature you control, that many plus one ... instead."): the moved counter becomes
    // two.
    let daghatar = t.battlefield(P0, "Daghatar the Adamant");
    put_counters(&mut t, daghatar, counters::PLUS1, 4);
    t.resolve_all();
    t.battlefield(P0, "Hardened Scales");
    daghatar_moves(&mut t, daghatar, daghatar, bears);
    t.resolve_all();
    assert_eq!(t.counters(daghatar, counters::PLUS1), 3);
    assert_eq!(t.counters(bears, counters::PLUS1), 1 + 2);
}

#[test]
fn daghatars_targets_are_two_different_creatures_either_may_be_daghatar() {
    cr!("115.3", "122.5");
    ruling!(
        "Daghatar the Adamant",
        "The two targets of the last ability must be different creatures. Either one may be Daghatar the Adamant."
    );
    let mut t = TestGame::new(2);
    let daghatar = t.battlefield(P0, "Daghatar the Adamant");
    put_counters(&mut t, daghatar, counters::PLUS1, 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    daghatar_moves(&mut t, daghatar, bears, daghatar);
    // The second target's candidates don't include the first target.
    let cands: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(cands.len(), 2);
    assert!(cands[0].contains(&Entity::Object(bears)));
    assert!(!cands[1].contains(&Entity::Object(bears)));
    assert!(cands[1].contains(&Entity::Object(daghatar)));
    // P1's Bears has no counter: nothing moves.
    t.resolve_all();
    assert_eq!(t.counters(daghatar, counters::PLUS1), 4);
    // From Daghatar onto the Bears.
    daghatar_moves(&mut t, daghatar, daghatar, bears);
    t.resolve_all();
    assert_eq!(t.counters(daghatar, counters::PLUS1), 3);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn a_counter_moves_only_if_both_targets_are_still_legal() {
    cr!("122.5", "608.2b");
    ruling!(
        "Daghatar the Adamant",
        "The +1/+1 counter is moved only if both targets are still legal as the ability resolves."
    );
    ruling!(
        "Leech Bonder",
        "If either one of the target creatures becomes an illegal target (because it left the battlefield or for any other reason), the counter doesn’t move. If both targets become illegal, the ability doesn’t resolve."
    );
    supported("Leech Bonder");
    let mut t = TestGame::new(2);
    let daghatar = t.battlefield(P0, "Daghatar the Adamant");
    put_counters(&mut t, daghatar, counters::PLUS1, 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // The receiving creature leaves: Daghatar keeps its counter.
    daghatar_moves(&mut t, daghatar, daghatar, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(daghatar, counters::PLUS1), 4);
    // The creature the counter would come from leaves: nothing is put on the other.
    let giant = t.battlefield(P1, "Hill Giant");
    let donor = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, donor, counters::PLUS1, 1);
    daghatar_moves(&mut t, daghatar, donor, giant);
    destroy(&mut t, donor);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 0);
}

#[test]
fn leech_bonder_moves_any_kind_of_counter_between_any_two_creatures() {
    cr!("122.5", "602.2b");
    ruling!(
        "Leech Bonder",
        "Leech Bonder’s activated ability can move any kind of counter, not just a -1/-1 counter. It can target any two creatures, whether they have counters on them or not."
    );
    // Leech Bonder: "{U}, {Q}: Move a counter from target creature onto a second target
    // creature." A Bears with a flying counter and a +1/+1 counter; the player chooses
    // the flying counter.
    let mut t = TestGame::new(2);
    let bonder = t.battlefield(P0, "Leech Bonder");
    t.g.objects[bonder.0 as usize].tapped = true;
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    put_counters(&mut t, bears, "flying", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    add_mana(&mut t, P0, ManaType::U, 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let pick_flying = |_g: &mtg_engine::game::Game, d: &Decision| match d {
        Decision::ChooseOption { options, .. } => {
            options.iter().position(|o| o == "flying").map(Answer::Index)
        }
        _ => None,
    };
    crate::r_s03_common::respond(&mut t, P0, pick_flying);
    activate_containing(&mut t, P0, bonder, "Move").expect("Leech Bonder's ability");
    assert!(!t.obj_now(bonder).tapped);
    t.resolve_all();
    assert_eq!(t.counters(bears, "flying"), 0);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(giant, "flying"), 1);
    assert!(t
        .obj_now(giant)
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn scrounging_bandar_moves_counters_that_other_abilities_see_removed_and_put() {
    cr!("122.5", "704.5f");
    ruling!(
        "Scrounging Bandar",
        "To move a counter from one creature to another, the counter is removed from the first creature and put onto the second. Any abilities that care about a counter being removed from or placed on a creature will apply."
    );
    ruling!(
        "Scrounging Bandar",
        "You choose a target creature as Scrounging Bandar's triggered ability is put onto the stack. You choose how many counters to move (if any) as that ability resolves."
    );
    supported("Scrounging Bandar");
    // Scrounging Bandar (0/0, two +1/+1 counters): "At the beginning of your upkeep, you
    // may move any number of +1/+1 counters from this creature onto another target
    // creature." It moves both: the other creature gets them (Simic Ascendancy sees
    // them put on), and the Bandar, left 0/0, dies.
    let mut t = TestGame::new(2);
    let ascendancy = t.battlefield(P0, "Simic Ascendancy");
    let bandar = t.battlefield(P0, "Scrounging Bandar");
    put_counters(&mut t, bandar, counters::PLUS1, 2);
    t.resolve_all();
    let growth = t.counters(ascendancy, GROWTH);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.counters(ascendancy, GROWTH), growth + 2);
    assert!(t.in_graveyard(P0, "Scrounging Bandar"));
    // Moving one of two: the Bandar survives as a 1/1.
    let mut t = TestGame::new(2);
    let bandar = t.battlefield(P0, "Scrounging Bandar");
    put_counters(&mut t, bandar, counters::PLUS1, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    next_upkeep(&mut t, P0);
    let asked = t.asked().len();
    t.resolve_all();
    // The number was asked as the ability resolved, after its target was chosen.
    assert!(t.asked()[..asked]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseNumber { .. })));
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.pt(bandar), (1, 1));
}

#[test]
fn a_fluxmage_ability_with_an_illegal_target_removes_no_counter() {
    cr!("608.2b", "122.5");
    ruling!(
        "Simic Fluxmage",
        "If the creature is an illegal target when Simic Fluxmage’s ability tries to resolve, it won’t resolve and none of its effects will happen. No counters will be removed from Simic Fluxmage."
    );
    let mut t = TestGame::new(2);
    let fluxmage = t.battlefield(P0, "Simic Fluxmage");
    put_counters(&mut t, fluxmage, counters::PLUS1, 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    fluxmage_moves(&mut t, fluxmage, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(fluxmage, counters::PLUS1), 1);
}

#[test]
fn weapon_rack_without_counters_stays_and_its_ability_does_nothing() {
    cr!("122.5", "608.2b");
    ruling!(
        "Weapon Rack",
        "Once Weapon Rack runs out of +1/+1 counters, it remains on the battlefield. You can activate its last ability, but it won't do anything."
    );
    ruling!(
        "Weapon Rack",
        "If Weapon Rack has left the battlefield or has no +1/+1 counters on it by the time its activated ability resolves, you won't put a +1/+1 counter on the target creature."
    );
    supported("Weapon Rack");
    // "{T}: Move a +1/+1 counter from this artifact onto target creature. Activate only
    // as a sorcery." One counter left.
    let mut t = TestGame::new(2);
    let rack = t.battlefield(P0, "Weapon Rack");
    put_counters(&mut t, rack, counters::PLUS1, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, rack, "Move").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(rack, counters::PLUS1), 0);
    // Out of counters: still on the battlefield, and activating it does nothing.
    t.g.objects[rack.0 as usize].tapped = false;
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, rack, "Move").expect("it can still be activated");
    t.resolve_all();
    assert!(t.on_battlefield(rack));
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // It leaves the battlefield with the ability on the stack: nothing is put on.
    let mut t = TestGame::new(2);
    let rack = t.battlefield(P0, "Weapon Rack");
    put_counters(&mut t, rack, counters::PLUS1, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, rack, "Move").unwrap();
    destroy(&mut t, rack);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn the_ozolith_moves_all_its_counters_or_none() {
    cr!("122.5", "122.8", "603.4");
    ruling!(
        "The Ozolith",
        "You can't move only some of the counters from The Ozolith onto the target creature."
    );
    ruling!(
        "The Ozolith",
        "As The Ozolith's last ability resolves, you choose whether to move the counters."
    );
    supported("The Ozolith");
    // "Whenever a creature you control leaves the battlefield, if it had counters on it,
    // put those counters on The Ozolith. At the beginning of combat on your turn, if The
    // Ozolith has counters on it, you may move all counters from The Ozolith onto target
    // creature."
    let mut t = TestGame::new(2);
    let ozolith = t.battlefield(P0, "The Ozolith");
    let donor = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, donor, counters::PLUS1, 2);
    put_counters(&mut t, donor, "flying", 1);
    destroy(&mut t, donor);
    t.resolve_all();
    assert_eq!(t.counters(ozolith, counters::PLUS1), 2);
    assert_eq!(t.counters(ozolith, "flying"), 1);
    // First combat: P0 declines as the ability resolves.
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(ozolith, counters::PLUS1), 2);
    // Next turn's combat: all of them move.
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(ozolith, counters::PLUS1), 0);
    assert_eq!(t.counters(ozolith, "flying"), 0);
    assert_eq!(t.counters(giant, counters::PLUS1), 2);
    assert_eq!(t.counters(giant, "flying"), 1);
}
