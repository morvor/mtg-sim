//! Rulings batch S03 — convert (CR 701.28), More Than Meets the Eye (CR 702.162) and
//! living metal (CR 702.161), the mechanics of the Transformers cards, tested with Cyclonus
//! ("More Than Meets the Eye {5}{U}{B}", 2/5 flying; "Whenever Cyclonus deals combat damage
//! to a player, it connives. Then if Cyclonus's power is 5 or greater, convert it." // a
//! 5/5 flying Vehicle with living metal).

use crate::r_s01_common::*;
use crate::r_s03_common::run_effect;
use mtg_engine::ability::{Effect, Filter, KeywordAction, Sel, Value};
use mtg_engine::casting::CastOption;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

const CYCLONUS: &str = "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter";
const FIGHTER: &str = "Cyclonus, Cybertronian Fighter";
const MTMTE: CastMethod = CastMethod::Keyword(KeywordKind::MoreThanMeetsTheEye);

/// The ways `p` could cast `card` converted now.
fn converted_options(t: &TestGame, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
    t.g.cast_options(p, card)
        .into_iter()
        .filter(|o| o.method == MTMTE)
        .collect()
}

/// Lands for Cyclonus's More Than Meets the Eye cost ({5}{U}{B}) plus `extra` generic.
fn mana_for_converted(t: &mut TestGame, p: PlayerId, extra: usize) {
    t.lands(p, "Island", 1);
    t.lands(p, "Swamp", 1);
    t.lands(p, "Wastes", 5 + extra);
}

/// Converts `id` (an effect with no source).
fn convert(t: &mut TestGame, id: ObjectId) {
    run_effect(
        t,
        None,
        P0,
        Effect::KeywordAction {
            action: KeywordAction::Convert,
            who: mtg_engine::ability::PlayerRef::You,
            what: Sel::All(Filter::Objects(vec![id])),
            n: Value::c(1),
        },
        &[],
    );
}

fn is_creature(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).is(CardType::Creature)
}

#[test]
fn converting_turns_the_permanent_over_to_its_other_face() {
    cr!("701.28a", "712.18");
    ruling!(
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
        "The convert keyword action functions the same way as the transform keyword action found on some other cards; to convert a permanent on the battlefield, turn it over so that its other face is up."
    );
    supported(CYCLONUS);
    // Cyclonus has two +1/+1 counters (a 4/7). It deals combat damage to P1 and connives,
    // discarding a nonland card: a third counter makes it 5/8, so it converts.
    let mut t = TestGame::new(2);
    let cy = t.battlefield(P0, CYCLONUS);
    t.g.add_counters(
        Entity::Object(cy),
        mtg_engine::types::counters::PLUS1,
        2,
        None,
    );
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    t.hand(P0, "Lightning Bolt");
    t.answer_choose(P0, &[]);
    t.attack(&[(cy, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P0, "Lightning Bolt") || t.in_graveyard(P0, "Grizzly Bears"));
    // The same permanent, with its other face up, its counters still on it.
    assert!(t.g.is_live(cy));
    assert_eq!(t.obj_now(cy).face, FaceState::Back);
    assert_eq!(t.obj_now(cy).chars.name, FIGHTER);
    assert_eq!(t.counters(cy, mtg_engine::types::counters::PLUS1), 3);
    assert!(t
        .g
        .turn_events
        .iter()
        .chain(t.g.events.iter())
        .any(|e| matches!(e, Event::Transformed { .. })));
    // Converted again, it's back on its front face.
    convert(&mut t, cy);
    assert_eq!(t.obj_now(cy).face, FaceState::Front);
    assert_eq!(t.obj_now(cy).chars.name, "Cyclonus, the Saboteur");
}

#[test]
fn more_than_meets_the_eye_casts_the_card_converted_from_any_zone_it_can_be_cast_from() {
    cr!("702.162a", "712.11a", "601.3");
    ruling!(
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
        "\"More Than Meets the Eye [cost]\" means \"You may cast this card converted by paying [cost] rather than its mana cost.\" It functions in any zone from which the spell can be cast."
    );
    supported(CYCLONUS);
    // From P0's hand: its back face, for {5}{U}{B}.
    let mut t = TestGame::new(2);
    let cy = t.hand(P0, CYCLONUS);
    let opts = converted_options(&t, P0, cy);
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0].face, FaceState::Back);
    assert_eq!(
        opts[0].alt_cost.as_ref().and_then(|c| c.mana.clone()),
        mtg_engine::mana::ManaCost::parse("{5}{U}{B}")
    );
    mana_for_converted(&mut t, P0, 0);
    let spell = t.cast(P0, cy).method(MTMTE).go();
    assert_eq!(tapped_lands(&t, P0), 7);
    t.resolve_all();
    let cy = t.g.current(spell);
    assert_eq!(t.obj_now(cy).chars.name, FIGHTER);
    assert_eq!(t.obj_now(cy).face, FaceState::Back);

    // From the top of P0's library with Future Sight ("You may play lands and cast spells
    // from the top of your library").
    supported("Future Sight");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Future Sight");
    let cy = t.library_top(P0, CYCLONUS);
    assert_eq!(converted_options(&t, P0, cy).len(), 1);
    mana_for_converted(&mut t, P0, 0);
    t.cast(P0, cy).method(MTMTE).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield(FIGHTER).len(), 1);

    // Not from a zone it can't be cast from.
    let mut t = TestGame::new(2);
    let cy = t.graveyard(P0, CYCLONUS);
    mana_for_converted(&mut t, P0, 0);
    assert!(converted_options(&t, P0, cy).is_empty());
    assert!(t.cast(P0, cy).method(MTMTE).try_go().is_err());
}

#[test]
fn a_spell_cast_converted_has_only_the_characteristics_of_its_back_face() {
    cr!("712.8c", "712.11a", "202.3b", "115.1");
    ruling!(
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
        "When you cast a spell using its More Than Meets the Eye ability, the card is put onto the stack with its back face up. The resulting spell has all characteristics of that face."
    );
    supported(CYCLONUS);
    supported("Essence Scatter");
    supported("Negate");
    let mut t = TestGame::new(2);
    let cy = t.hand(P0, CYCLONUS);
    mana_for_converted(&mut t, P0, 0);
    let spell = t.cast(P0, cy).method(MTMTE).go();
    let o = t.obj_now(spell);
    assert_eq!(o.zone, Zone::Stack);
    assert_eq!(o.face, FaceState::Back);
    assert_eq!(o.chars.name, FIGHTER);
    assert!(o.chars.has_subtype("Vehicle"));
    assert!(o.is(CardType::Artifact) && !o.is(CardType::Creature));
    assert_eq!((o.chars.power, o.chars.toughness), (Some(5), Some(5)));
    // Its mana value is its front face's ({2}{U}{B}).
    assert_eq!(t.g.mana_value_of(spell), 4);
    // It isn't a creature spell: Essence Scatter can't target it, Negate can.
    t.lands(P1, "Island", 4);
    let scatter = t.hand(P1, "Essence Scatter");
    assert!(t.cast(P1, scatter).target(spell).try_go().is_err());
    let negate = t.hand(P1, "Negate");
    t.cast(P1, negate).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Cyclonus, the Saboteur"));
}

#[test]
fn the_total_cost_starts_from_the_more_than_meets_the_eye_cost() {
    cr!("601.2f", "118.9", "202.3b", "712.8c");
    ruling!(
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
        "To determine the total cost of a spell, start with the mana cost or alternative cost (such as a More Than Meets the Eye cost) you're paying, add any cost increases, then apply any cost reductions. The mana value of a spell cast using More Than Meets the Eye is determined by the mana cost on the front face of the card, no matter what the total cost to cast the spell was."
    );
    supported(CYCLONUS);
    supported("Thalia, Guardian of Thraben");
    supported("Etherium Sculptor");
    // Thalia: "Noncreature spells cost {1} more to cast." The converted spell is a
    // noncreature (Vehicle) spell: {5}{U}{B} + {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let cy = t.hand(P0, CYCLONUS);
    mana_for_converted(&mut t, P0, 3);
    let spell = t.cast(P0, cy).method(MTMTE).go();
    assert_eq!(tapped_lands(&t, P0), 8);
    assert_eq!(t.g.mana_value_of(spell), 4);
    // Cast normally it's a creature spell: {2}{U}{B}, no increase.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let cy = t.hand(P0, CYCLONUS);
    mana_for_converted(&mut t, P0, 3);
    let spell = t.cast(P0, cy).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(t.g.mana_value_of(spell), 4);
    // Etherium Sculptor: "Artifact spells you cast cost {1} less to cast." With both:
    // {5}{U}{B} + {1} - {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.battlefield(P0, "Etherium Sculptor");
    let cy = t.hand(P0, CYCLONUS);
    mana_for_converted(&mut t, P0, 3);
    let spell = t.cast(P0, cy).method(MTMTE).go();
    assert_eq!(tapped_lands(&t, P0), 7);
    assert_eq!(t.g.mana_value_of(spell), 4);
    // Without enough mana for the increased cost, it can't be cast converted.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let cy = t.hand(P0, CYCLONUS);
    mana_for_converted(&mut t, P0, 0);
    assert!(t.cast(P0, cy).method(MTMTE).try_go().is_err());
}

#[test]
fn more_than_meets_the_eye_is_an_alternative_cost_and_additional_costs_still_apply() {
    cr!("118.9a", "601.2b", "601.2f", "702.78a");
    ruling!(
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
        "The cost is an alternative cost, so it can't be combined with any other alternative costs. It can be combined with any applicable additional costs."
    );
    supported(CYCLONUS);
    supported("Raiding Schemes");
    // An effect lets P0 cast Cyclonus from exile without paying its mana cost: that's
    // casting it normally (front face up) for free, never converted — that would be
    // another alternative cost, for free or for {5}{U}{B} (the permission is to cast it
    // without paying its mana cost, an alternative cost itself, CR 118.9a).
    let mut t = TestGame::new(2);
    let cy = t.exile(P0, CYCLONUS);
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![cy],
        mtg_engine::ability::Duration::EndOfTurn,
        true,
        None,
    );
    let opts = t.g.cast_options(P0, cy);
    assert!(opts
        .iter()
        .any(|o| o.face == FaceState::Front && o.method == CastMethod::Free));
    assert!(converted_options(&t, P0, cy).is_empty());
    assert!(opts.iter().all(|o| o.face != FaceState::Back));
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 5);
    // Not even with mana for the converted spell's cost.
    assert!(t.cast(P0, cy).method(MTMTE).try_go().is_err());
    assert_eq!(tapped_lands(&t, P0), 0);

    // Raiding Schemes: "Each noncreature spell you cast has conspire." The converted
    // spell is a blue and black noncreature spell: P0 pays its conspire cost (an
    // additional cost) by tapping two blue creatures.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raiding Schemes");
    let a = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let b = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let cy = t.hand(P0, CYCLONUS);
    mana_for_converted(&mut t, P0, 0);
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Bool(true),
    );
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, cy).method(MTMTE).go();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert_eq!(tapped_lands(&t, P0), 7);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "onspire"), 1);
}

#[test]
fn a_copy_of_a_spell_cast_converted_has_the_back_face_characteristics() {
    cr!("707.10", "707.2", "702.78a", "111.13");
    ruling!(
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
        "If you copy a permanent spell cast this way, the copy has the characteristics of the card's back face, even though it isn't itself a double-faced card."
    );
    supported(CYCLONUS);
    supported("Raiding Schemes");
    // P0 casts Cyclonus converted and copies it with conspire (granted by Raiding
    // Schemes).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raiding Schemes");
    let a = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let b = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let cy = t.hand(P0, CYCLONUS);
    mana_for_converted(&mut t, P0, 0);
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Bool(true),
    );
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    let spell = t.cast(P0, cy).method(MTMTE).go();
    // The conspire trigger resolves: the copy is on the stack above the spell.
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    let o = t.obj_now(copy);
    assert_eq!(o.kind, mtg_engine::object::ObjKind::SpellCopy);
    assert_eq!(o.chars.name, FIGHTER);
    assert!(o.chars.has_subtype("Vehicle") && !o.is(CardType::Creature));
    assert_eq!((o.chars.power, o.chars.toughness), (Some(5), Some(5)));
    // It resolves into a token with those characteristics (a 5/5 flying Vehicle with
    // living metal: a creature during P0's turn).
    t.resolve();
    let token = tokens(&t, P0);
    assert_eq!(token.len(), 1);
    let o = t.obj_now(token[0]);
    assert_eq!(o.chars.name, FIGHTER);
    assert!(o.chars.has_subtype("Vehicle"));
    assert!(o.chars.has_keyword(KeywordKind::LivingMetal));
    assert!(o.chars.has_keyword(KeywordKind::Flying));
}

#[test]
fn living_metal_makes_the_vehicle_an_artifact_creature_during_your_turn() {
    cr!("702.161a", "613.1d", "301.7a", "301.7b");
    ruling!(
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
        "\"Living metal\" means \"As long as it's your turn, this permanent is an artifact creature in addition to its other types.\""
    );
    ruling!(
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
        "While it's a creature, the Vehicle has its printed power and toughness."
    );
    supported(CYCLONUS);
    let mut t = TestGame::new(2);
    let cy = t.battlefield(P0, CYCLONUS);
    convert(&mut t, cy);
    assert_eq!(t.obj_now(cy).chars.name, FIGHTER);
    // P0's turn: a 5/5 artifact creature Vehicle.
    t.set_step(P0, Step::PrecombatMain);
    assert!(is_creature(&mut t, cy));
    assert!(t.obj_now(cy).is(CardType::Artifact));
    assert!(t.obj_now(cy).chars.has_subtype("Vehicle"));
    assert_eq!(t.pt(cy), (5, 5));
    // It can attack (it has been under P0's control since the turn began).
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, cy));
    // P1's turn: only an artifact Vehicle, which can't block.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!is_creature(&mut t, cy));
    assert!(t.obj_now(cy).is(CardType::Artifact));
    assert!(t.obj_now(cy).chars.has_subtype("Vehicle"));
    // P0's turn again.
    t.set_step(P0, Step::PrecombatMain);
    assert!(is_creature(&mut t, cy));
}

#[test]
fn effects_on_noncreature_permanents_apply_to_a_living_metal_vehicle_only_on_opponents_turns() {
    cr!("702.161a", "613.8a", "613.8b", "613.4b", "613.6");
    ruling!(
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
        "If a static ability of another permanent applies only to noncreature permanents, that ability applies to a Vehicle with living metal only during your opponents' turns."
    );
    supported(CYCLONUS);
    supported("March of the Machines");
    // March of the Machines: "Each noncreature artifact is an artifact creature with power
    // and toughness each equal to its mana value." It was on the battlefield first.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "March of the Machines");
    let ring = t.battlefield(P0, "Sol Ring");
    let cy = t.battlefield(P0, CYCLONUS);
    convert(&mut t, cy);
    // P0's turn: living metal makes it a creature, so March doesn't apply: 5/5.
    t.set_step(P0, Step::PrecombatMain);
    assert!(is_creature(&mut t, cy));
    assert_eq!(t.pt(cy), (5, 5));
    // P1's turn: a noncreature artifact, which March makes a 4/4 artifact creature (the
    // mana value of its front face, {2}{U}{B}).
    t.set_step(P1, Step::PrecombatMain);
    assert!(is_creature(&mut t, cy));
    assert_eq!(t.pt(cy), (4, 4));
    // Any other noncreature artifact is affected all the time: Sol Ring is a 1/1.
    assert!(is_creature(&mut t, ring));
    assert_eq!(t.pt(ring), (1, 1));
    t.set_step(P0, Step::PrecombatMain);
    assert_eq!(t.pt(ring), (1, 1));
}
