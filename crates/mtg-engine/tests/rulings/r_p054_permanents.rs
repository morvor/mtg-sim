//! Rulings batch P054 — pump effects tied to permanents and to how things enter: "as
//! this enters, choose" (CR 614.12, 603.6d), Finale of Devastation's search and bonus,
//! Lion Umbra's "enchant modified creature" (CR 303.4d, 700.9, 704.5m), Lifecraft
//! Awakening on Equipment and Vehicles (CR 301.5c, 704.5n), Invigorate's alternative cost
//! (CR 118.12, 601.2f, 202.3), Garruk's emblem at the end step (CR 603.2, 513.1a) and
//! Aragorn's multicolored triggers (CR 603.3b).

use crate::r_p054_common::*;
use crate::r_s01_common::with_subtype;
use crate::r_s06_common::{attach_new, give_control};
use mtg_engine::ability::{Destination, Effect, Filter, Sel};
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

fn flying(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).has_keyword(KeywordKind::Flying)
}

/// Candidates of the `ChooseEntities` decisions asked of `p` since `from`.
fn choice_candidates(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Tyrannical Pitlord
// ---------------------------------------------------------------------------------------

#[test]
fn tyrannical_pitlord_cant_choose_a_creature_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Tyrannical Pitlord",
        "If Tyrannical Pitlord enters the battlefield at the same time as another creature, that creature can’t be chosen."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let pit = t.graveyard(P0, "Tyrannical Pitlord");
    let giant = t.graveyard(P0, "Hill Giant");
    let mark = t.asked().len();
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::Move {
            what: Sel::All(Filter::Objects(vec![pit, giant])),
            to: Destination::battlefield().under_your_control(),
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    let giant_now = t.g.current(giant);
    assert!(t.on_battlefield(giant_now) && t.on_battlefield(pit));
    for c in choice_candidates(&t, P0, mark) {
        assert!(!c.contains(&o(giant_now)), "the Giant entered with it");
    }
    assert_eq!(t.pt(bear), (5, 5));
    assert!(flying(&t, bear));
    assert_eq!(t.pt(giant_now), (3, 3));
}

#[test]
fn tyrannical_pitlord_chooses_as_it_enters_without_the_stack_and_never_again() {
    cr!("614.12", "603.6d", "400.7");
    ruling!(
        "Tyrannical Pitlord",
        "The ability of Tyrannical Pitlord that allows you to choose a creature isn’t a triggered ability and doesn’t use the stack. Players can’t respond to your choice of which creature it’s affecting."
    );
    ruling!(
        "Tyrannical Pitlord",
        "If the chosen creature leaves the battlefield, you can’t choose a new creature."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[o(bear)]);
    crate::r_s01_common::supported("Tyrannical Pitlord");
    let pit = enter(&mut t, P0, "Tyrannical Pitlord");
    // Already applied, nothing on the stack.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(bear), (5, 5));
    // The chosen creature leaves and comes back: a new object, nothing chosen.
    move_to(&mut t, bear, Zone::Exile);
    let back = t.g.current(bear);
    move_to(&mut t, back, Zone::Battlefield);
    let back = t.g.current(bear);
    assert_eq!(t.pt(back), (2, 2));
    assert_eq!(t.pt(other), (3, 3));
    assert!(!flying(&t, other));
    // The leaves trigger then finds no chosen creature to sacrifice.
    move_to(&mut t, pit, Zone::Exile);
    t.resolve_all();
    assert!(t.on_battlefield(back) && t.on_battlefield(other));
}

// ---------------------------------------------------------------------------------------
// Finale of Devastation
// ---------------------------------------------------------------------------------------

#[test]
fn finale_with_x_ten_pumps_the_found_creature_before_its_enters_triggers() {
    cr!("608.2c", "603.3", "117.3b");
    ruling!(
        "Finale of Devastation",
        "If X is 10 or more, the creature card you just put onto the battlefield will get +X/+X and haste."
    );
    ruling!(
        "Finale of Devastation",
        "No player may take action between the time you reveal which creature card you'll put onto the battlefield and the time it gets +X/+X and haste if X is 10 or more. Any abilities that trigger as it enters the battlefield will be put onto the stack after your creatures get +X/+X and haste."
    );
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let found = t.library_top(P0, "Elvish Visionary");
    t.answer_choose(P0, &[o(found)]);
    cast_how(
        &mut t,
        P0,
        "Finale of Devastation",
        CastMethod::Normal,
        &[],
        Some(10),
        None,
    );
    t.resolve();
    // The Visionary's draw trigger is on the stack, and it's already pumped.
    let v = t.g.current(found);
    assert!(t.on_battlefield(v));
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.pt(v), (11, 11));
    assert!(t.obj_now(v).has_keyword(KeywordKind::Haste));
}

#[test]
fn finale_treats_x_in_a_creature_cards_cost_as_zero() {
    cr!("202.3e", "107.3");
    ruling!(
        "Finale of Devastation",
        "If a creature card in your library or graveyard has {X} in its mana cost, X is considered to be 0."
    );
    let mut t = TestGame::new(2);
    let hydra = t.library_top(P0, "Primordial Hydra");
    let giant = t.library_top(P0, "Hill Giant");
    let mark = t.asked().len();
    t.answer_choose(P0, &[o(hydra)]);
    cast_how(
        &mut t,
        P0,
        "Finale of Devastation",
        CastMethod::Normal,
        &[],
        Some(2),
        None,
    );
    t.resolve_all();
    let cands = choice_candidates(&t, P0, mark);
    assert!(cands.iter().any(|c| c.contains(&o(hydra))));
    assert!(!cands.iter().any(|c| c.contains(&o(giant))));
    // It entered with X = 0 counters, a 0/0, and died.
    assert!(t.in_graveyard(P0, "Primordial Hydra"));
}

// ---------------------------------------------------------------------------------------
// Lion Umbra
// ---------------------------------------------------------------------------------------

#[test]
fn lion_umbra_needs_a_modified_creature_and_keeps_it_modified_while_yours() {
    cr!("702.5a", "303.4c", "700.9", "704.5m");
    ruling!(
        "Lion Umbra",
        "Lion Umbra can't become attached to a creature unless that creature is already modified."
    );
    ruling!(
        "Lion Umbra",
        "If a Lion Umbra you control is attached to a creature you control, that creature will continue to be a \"modified creature\" as long as Lion Umbra stays attached to it, regardless of whether it has any other Auras you control or Equipment attached to it or whether it has counters on it. However, if a Lion Umbra you control is attached to a creature you don't control and that creature has no Auras controlled by that creature's controller attached to it, no Equipment attached to it, and no counters on it, Lion Umbra will be put into its owner's graveyard due to being attached to an illegal object the next time state-based actions are performed."
    );
    // An unmodified creature can't be targeted.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0);
    let umbra = t.hand(P0, "Lion Umbra");
    t.answer_targets(P0, &[o(bear)]);
    let r = t.cast(P0, umbra).try_go();
    assert!(r.is_err() || t.zone(umbra) != Zone::Battlefield && t.stack_len() == 0);
    // A creature with a counter: legal; once attached, the counter can go.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(o(bear), "+1/+1", 1, None);
    cast_at(&mut t, P0, "Lion Umbra", bear);
    t.resolve_all();
    assert_eq!(t.pt(bear), (6, 6));
    t.g.remove_counters(o(bear), "+1/+1", 1);
    t.g.recompute();
    t.settle();
    assert!(!t.named_on_battlefield("Lion Umbra").is_empty());
    assert_eq!(t.pt(bear), (5, 5));
    // The creature changes control: the Umbra is no longer a modification of it.
    give_control(&mut t, bear, P1);
    t.settle();
    assert!(t.in_graveyard(P0, "Lion Umbra"));
}

// ---------------------------------------------------------------------------------------
// Lifecraft Awakening
// ---------------------------------------------------------------------------------------

#[test]
fn lifecraft_awakening_animates_noncreature_artifacts_indefinitely() {
    cr!("611.2c", "704.5f", "122.1a");
    ruling!(
        "Lifecraft Awakening",
        "If the artifact isn’t a creature or Vehicle as Lifecraft Awakening resolves, the artifact becomes a creature indefinitely."
    );
    ruling!(
        "Lifecraft Awakening",
        "You can cast Lifecraft Awakening declaring X as 0. If the artifact isn’t a creature or Vehicle, it’ll become a 0/0 creature and, unless another effect modifies its toughness, be put into your graveyard soon after."
    );
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    cast_how(
        &mut t,
        P0,
        "Lifecraft Awakening",
        CastMethod::Normal,
        &[&[o(ring)]],
        Some(2),
        None,
    );
    t.resolve_all();
    assert!(is_creature(&t, ring) && t.obj_now(ring).chars.has_subtype("Construct"));
    assert_eq!(t.pt(ring), (2, 2));
    t.advance_to(P1, Step::Upkeep);
    assert!(is_creature(&t, ring), "indefinitely");
    // X = 0: a 0/0 that dies.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    cast_how(
        &mut t,
        P0,
        "Lifecraft Awakening",
        CastMethod::Normal,
        &[&[o(ring)]],
        Some(0),
        None,
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sol Ring"));
}

#[test]
fn lifecraft_awakening_on_a_vehicle_and_on_an_equipment() {
    cr!("301.5c", "704.5n", "301.7b", "122.1a");
    ruling!(
        "Lifecraft Awakening",
        "You can put +1/+1 counters on a noncreature Vehicle this way. They’ll be put there, apply if the Vehicle becomes a creature, and remain there when it stops being a creature."
    );
    ruling!(
        "Lifecraft Awakening",
        "If an Equipment becomes a creature, it becomes unattached and it can’t be attached to a creature."
    );
    // Vehicle: counters, still not a creature; crewed it's bigger; afterward the counters stay.
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_how(
        &mut t,
        P0,
        "Lifecraft Awakening",
        CastMethod::Normal,
        &[&[o(copter)]],
        Some(2),
        None,
    );
    t.resolve_all();
    assert!(!is_creature(&t, copter));
    assert_eq!(t.counters(copter, "+1/+1"), 2);
    assert!(crate::r_s04_common::crew(&mut t, P0, copter, &[bear]));
    t.resolve_all();
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(copter), (5, 5));
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_creature(&t, copter));
    assert_eq!(t.counters(copter, "+1/+1"), 2);
    // Equipment: it becomes unattached and can't be attached again.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let sword = attach_new(&mut t, P0, "Bonesplitter", bear);
    cast_how(
        &mut t,
        P0,
        "Lifecraft Awakening",
        CastMethod::Normal,
        &[&[o(sword)]],
        Some(1),
        None,
    );
    t.resolve_all();
    assert!(is_creature(&t, sword));
    assert_eq!(t.obj_now(sword).attached_to, None);
    assert_eq!(t.pt(bear), (2, 2));
    pool(&mut t, P0);
    let _ = t.activate(P0, sword, 0, &[o(other)]);
    t.resolve_all();
    assert_eq!(t.obj_now(sword).attached_to, None);
    assert_eq!(t.pt(other), (3, 3));
}

// ---------------------------------------------------------------------------------------
// Invigorate
// ---------------------------------------------------------------------------------------

fn alternative_method(t: &mut TestGame, card: ObjectId) -> Option<CastMethod> {
    crate::r_s07_common::cast_methods(t, P0, card)
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
}

#[test]
fn invigorates_alternative_cost_and_life_gain_restrictions() {
    cr!("118.3", "119.7", "614.1");
    ruling!(
        "Invigorate",
        "If an effect says that an opponent can't gain life, you can't have that player gain life to pay Invigorate's alternative cost. If an effect instead replaces life gain, such as that of Sulfuric Vortex, you may choose to pay the cost even though the cost's action is replaced with nothing."
    );
    // Normal: the opponent gains 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let inv = t.hand(P0, "Invigorate");
    let m = alternative_method(&mut t, inv).expect("the alternative cost is available");
    t.cast(P0, inv).method(m).target(bear).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
    assert_eq!(t.pt(bear), (6, 6));
    // "Players can't gain life": not available.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Leyline of Punishment");
    let inv = t.hand(P0, "Invigorate");
    assert_eq!(alternative_method(&mut t, inv), None);
    // Sulfuric Vortex replaces the life gain: still available.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Sulfuric Vortex");
    let inv = t.hand(P0, "Invigorate");
    let m = alternative_method(&mut t, inv).expect("available under Sulfuric Vortex");
    t.cast(P0, inv).method(m).target(bear).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.pt(bear), (6, 6));
}

#[test]
fn invigorates_total_cost_adds_increases_and_its_mana_value_is_unchanged() {
    cr!("601.2f", "118.9d", "202.3");
    ruling!(
        "Invigorate",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as the alternative cost of Invigorate), add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let inv = t.hand(P0, "Invigorate");
    let m = alternative_method(&mut t, inv).expect("alternative cost");
    let spell = t.cast(P0, inv).method(m).target(bear).go();
    // Thalia's {1}: the Forest paid it.
    assert!(t.obj_now(forest).tapped);
    assert_eq!(t.g.obj(spell).chars.mana_value(), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
}

// ---------------------------------------------------------------------------------------
// Garruk's emblem, Aragorn
// ---------------------------------------------------------------------------------------

#[test]
fn a_creature_put_onto_the_battlefield_during_the_end_step_misses_that_end_step_trigger() {
    cr!("603.2", "513.1a");
    ruling!(
        "Garruk, Unleashed",
        "If the creature you put onto the battlefield with Garruk's emblem's ability during your end step has an ability that triggers at the beginning of your end step, it won't trigger during that end step."
    );
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let ronin = t.library_top(P0, "Reinforced Ronin");
    let g = t.battlefield(P0, "Garruk, Unleashed");
    t.g.add_counters(o(g), "loyalty", 10, None);
    t.activate(P0, g, 2, &[]).expect("-7");
    t.resolve_all();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(ronin)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let r = t.g.current(ronin);
    assert!(t.on_battlefield(r));
    // Its own "at the beginning of your end step" didn't trigger this end step.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(r));
    // It does at P0's next end step.
    t.advance_to(P0, Step::End);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_hand(P0, "Reinforced Ronin"));
}

#[test]
fn aragorn_multicolored_spell_triggers_each_color_in_the_order_you_choose() {
    cr!("603.3b", "105.2");
    ruling!(
        "Aragorn, the Uniter",
        "If you cast a spell that's multiple colors, more than one of Aragorn, the Uniter's abilities may trigger. In that case, you choose the order in which the abilities are put onto the stack."
    );
    let mut t = TestGame::new(2);
    let ar = t.battlefield(P0, "Aragorn, the Uniter");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let mark = t.asked().len();
    t.answer_targets(P0, &[o(bear)]);
    cast_how(&mut t, P0, "Watchwolf", CastMethod::Normal, &[], None, None);
    t.settle();
    assert_eq!(crate::r_s14_common::triggers_from(&t, ar), 2);
    assert!(t.asked()[mark..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
    t.resolve_all();
    assert_eq!(t.pt(bear), (6, 6));
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 1);
}
