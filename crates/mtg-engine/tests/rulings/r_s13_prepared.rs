//! Rulings batch S13 — prepared (CR 722.3): a permanent with a prepare spell that becomes
//! prepared gets the "prepared" designation, and its controller creates a copy of it in
//! exile with only the prepare spell's characteristics, which they may cast while the
//! permanent stays prepared (CR 722.3a, 722.3c).

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::designations;
use mtg_engine::kwa::evidence_forage::ALT_COST_METHOD;
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Adventurous Eater ({2}{B} 3/2, "This creature enters prepared.") // Have a Bite ({B}
/// sorcery, "Put a +1/+1 counter on target creature. You gain 1 life.").
const EATER: &str = "Adventurous Eater // Have a Bite";
/// Goblin Glasswright ({1}{R} 2/2, "This creature enters prepared.") // Craft with Pride
/// ({R} sorcery, "Create a Treasure token.").
const GLASSWRIGHT: &str = "Goblin Glasswright // Craft with Pride";
/// Encouraging Aviator ({2}{U} 2/3 flier, "Whenever this creature attacks, it becomes
/// prepared.") // Jump ({U} instant, "Target creature gains flying until end of turn.").
const AVIATOR: &str = "Encouraging Aviator // Jump";

/// The exiled prepare-spell copy of a prepared permanent, if it's prepared.
fn prepared_copy(t: &TestGame, perm: ObjectId) -> Option<ObjectId> {
    t.obj_now(perm).prepared
}

/// The prepare-spell copies in exile.
fn exiled_copies(t: &TestGame) -> Vec<ObjectId> {
    t.g.exile
        .iter()
        .copied()
        .filter(|c| t.obj(*c).kind == ObjKind::CardCopy)
        .collect()
}

#[test]
fn a_prepared_creature_that_loses_all_abilities_stays_prepared() {
    cr!("722.3a", "722.3c", "722.2b", "613.1f");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "If a prepared creature loses all abilities, it won't stop being prepared, and nothing will happen to its alternative characteristics or to the copy of its prepare spell in exile."
    );
    supported(EATER);
    supported("Frogify");
    let mut t = TestGame::new(2);
    let eater = t.enter(P0, EATER);
    let copy = prepared_copy(&t, eater).expect("entered prepared");
    // Frogify: "Enchanted creature loses all abilities and is a blue Frog creature with
    // base power and toughness 1/1."
    let frogify = t.hand(P0, "Frogify");
    give_mana_for(&mut t, P0, "Frogify");
    t.cast(P0, frogify).target(eater).go();
    t.resolve_all();
    let frog = t.g.current(eater);
    assert!(t.obj(frog).chars.abilities.is_empty());
    assert_eq!(t.pt(frog), (1, 1));
    // Still prepared, with the same copy in exile, and it still has its prepare spell.
    assert_eq!(prepared_copy(&t, eater), Some(copy));
    assert_eq!(t.obj(copy).zone, Zone::Exile);
    assert_eq!(t.obj(copy).chars.name, "Have a Bite");
    assert!(designations::has_prepare_spell(&t.g, frog));
    // Its controller can still cast the copy.
    t.lands(P0, "Swamp", 1);
    assert!(can_cast(&mut t, P0, copy, CastMethod::Normal));
    t.cast(P0, copy).target(frog).go();
    t.resolve_all();
    assert_eq!(t.counters(frog, counters::PLUS1), 1);
    assert_eq!(t.life(P0), 21);
    assert!(prepared_copy(&t, eater).is_none());
}

#[test]
fn casting_the_prepare_copy_isnt_casting_it_for_an_alternative_cost() {
    cr!("722.3c", "118.9", "118.9a", "601.2b", "601.2f");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "Casting a copy of a prepare spell from exile isn't casting it for an alternative cost. Effects that allow you to cast a spell for an alternative cost or without paying its mana cost may allow you to apply those to a copy of a prepare spell cast from exile."
    );
    supported(EATER);
    supported("Conspiracy Unraveler");
    // Normally, the copy is cast by paying its mana cost, {B}.
    let mut t = TestGame::new(2);
    let eater = t.enter(P0, EATER);
    let copy = prepared_copy(&t, eater).expect("prepared");
    assert!(!can_cast(&mut t, P0, copy, CastMethod::Normal));
    t.lands(P0, "Swamp", 1);
    assert!(can_cast(&mut t, P0, copy, CastMethod::Normal));
    // Conspiracy Unraveler: "You may collect evidence 10 rather than pay the mana cost for
    // spells you cast." That alternative cost can be applied to the copy.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conspiracy Unraveler");
    t.graveyard(P0, "Shivan Dragon");
    t.graveyard(P0, "Shivan Dragon");
    let eater = t.enter(P0, EATER);
    let copy = prepared_copy(&t, eater).expect("prepared");
    let alt = CastMethod::Alternative(ALT_COST_METHOD);
    assert!(can_cast(&mut t, P0, copy, alt.clone()));
    assert!(!can_cast(&mut t, P0, copy, CastMethod::Normal));
    let eater_now = t.g.current(eater);
    t.cast(P0, copy).method(alt).target(eater_now).go();
    // Evidence was collected: both Dragons were exiled; no mana was needed.
    assert_eq!(t.graveyard_size(P0), 0);
    assert!(prepared_copy(&t, eater).is_none());
    t.resolve_all();
    assert_eq!(t.counters(eater, counters::PLUS1), 1);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn the_prepare_spells_name_can_be_chosen_as_a_card_name() {
    cr!("722.5", "201.4a");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "If an effect instructs you to choose a card name, you may choose the alternative prepare spell's name. Consider only the alternative characteristics to determine whether that is an appropriate name to choose."
    );
    supported(EATER);
    supported("Council of the Absolute");
    supported("Meddling Mage");
    // Council of the Absolute: "As this creature enters, choose a noncreature, nonland
    // card name. Your opponents can't cast spells with the chosen name." Have a Bite is a
    // sorcery, so its name is a noncreature name, though Adventurous Eater is a creature
    // card.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Name, Answer::Text("Have a Bite".into()));
    let council = t.enter(P0, "Council of the Absolute");
    assert_eq!(
        t.obj_now(council).choices.card_name.as_deref(),
        Some("Have a Bite")
    );
    t.set_step(P1, Step::PrecombatMain);
    let eater = t.enter(P1, EATER);
    let copy = prepared_copy(&t, eater).expect("prepared");
    t.lands(P1, "Swamp", 1);
    assert!(!can_cast(&mut t, P1, copy, CastMethod::Normal));
    // The card's own (creature) name isn't a noncreature card name.
    let mut t = TestGame::new(2);
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Adventurous Eater".into()),
    );
    let council = t.enter(P0, "Council of the Absolute");
    assert_ne!(
        t.obj_now(council).choices.card_name.as_deref(),
        Some("Adventurous Eater")
    );
    // Meddling Mage ("choose a nonland card name. Spells with the chosen name can't be
    // cast.") naming Have a Bite stops its own controller's copy too.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Name, Answer::Text("Have a Bite".into()));
    t.enter(P0, "Meddling Mage");
    let eater = t.enter(P0, EATER);
    let copy = prepared_copy(&t, eater).expect("prepared");
    t.lands(P0, "Swamp", 1);
    assert!(!can_cast(&mut t, P0, copy, CastMethod::Normal));
    // (Adventurous Eater itself can still be cast.)
    let other = t.hand(P0, EATER);
    t.lands(P0, "Swamp", 3);
    assert!(can_cast(&mut t, P0, other, CastMethod::Normal));
}

#[test]
fn a_prepared_creature_cant_become_prepared_again() {
    cr!("722.3a");
    ruling!(
        "Encouraging Aviator // Jump",
        "A creature with a prepare spell can't become prepared more than once at the same time."
    );
    supported(AVIATOR);
    let mut t = TestGame::new(2);
    let aviator = t.battlefield(P0, AVIATOR);
    // The first attack prepares it.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(aviator, Entity::Player(P1))], &[]);
    let copy = prepared_copy(&t, aviator).expect("prepared");
    assert_eq!(exiled_copies(&t), vec![copy]);
    // Next turn it attacks again while still prepared: its ability resolves, but it
    // doesn't become prepared a second time or create a second copy.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(aviator, Entity::Player(P1))], &[]);
    assert!(t.g.turn_events.iter().any(|e| matches!(
        e,
        mtg_engine::events::Event::AbilityTriggeredOnStack { source, .. } if *source == t.g.current(aviator)
    )));
    assert_eq!(prepared_copy(&t, aviator), Some(copy));
    assert_eq!(exiled_copies(&t), vec![copy]);
}

#[test]
fn becoming_prepared_creates_a_castable_copy_that_lasts_while_prepared() {
    cr!("722.3a", "722.3c", "704.5e", "601.2i");
    ruling!(
        "Goblin Glasswright // Craft with Pride",
        "As an effect causes a creature with a prepare spell to become prepared (including effects that state that a creature \"enters prepared\"), that creature's controller creates a copy of that creature's prepare spell in exile."
    );
    supported(GLASSWRIGHT);
    supported("Unsummon");
    let mut t = TestGame::new(2);
    // It enters prepared: a copy of Craft with Pride, a red sorcery, in exile, controlled
    // by the Glasswright's controller, who may cast it.
    let g = t.enter(P0, GLASSWRIGHT);
    let copy = prepared_copy(&t, g).expect("prepared");
    assert_eq!(exiled_copies(&t), vec![copy]);
    let c = t.obj(copy);
    assert_eq!(c.chars.name, "Craft with Pride");
    assert!(c.chars.is(CardType::Sorcery) && !c.chars.is_creature());
    assert_eq!(c.controller, P0);
    assert!(t.g.permitted_cards(P0).contains(&copy));
    assert!(!t.g.permitted_cards(P1).contains(&copy));
    // It stays through state-based actions while the Glasswright is prepared ...
    t.settle();
    assert_eq!(t.obj(copy).zone, Zone::Exile);
    // ... and ceases to exist when the Glasswright leaves the battlefield.
    let unsummon = t.hand(P1, "Unsummon");
    give_mana_for(&mut t, P1, "Unsummon");
    t.cast(P1, unsummon).target(g).go();
    t.resolve_all();
    assert_eq!(t.zone(g), Zone::Hand(P0));
    assert_eq!(t.obj(copy).zone, Zone::Nowhere);
    assert!(exiled_copies(&t).is_empty());
    // Back on the battlefield, it's a new object that's prepared again, with a new copy.
    let g = t.enter(P0, GLASSWRIGHT);
    let copy = prepared_copy(&t, g).expect("prepared again");
    // As the copy is cast, the Glasswright stops being prepared.
    t.lands(P0, "Mountain", 1);
    let spell = t.cast(P0, copy).go();
    assert_eq!(t.obj(spell).zone, Zone::Stack);
    assert!(prepared_copy(&t, g).is_none());
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    assert!(exiled_copies(&t).is_empty());
}

#[test]
fn a_prepared_creature_that_stops_being_a_creature_or_becomes_a_copy_stays_prepared() {
    cr!("722.3a", "722.3c", "707.2", "305.7");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "If a prepared creature stops being a creature, it will still be prepared, and the copy of its prepare spell will remain in exile. That permanent's controller will still be able to cast it. The same is true if a prepared creature becomes a copy of something else, even if it's then a permanent without a prepare spell."
    );
    supported(EATER);
    supported("Song of the Dryads");
    supported("True Polymorph");
    // Song of the Dryads: "Enchanted permanent is a colorless Forest land."
    let mut t = TestGame::new(2);
    let eater = t.enter(P0, EATER);
    let copy = prepared_copy(&t, eater).expect("prepared");
    let song = t.hand(P0, "Song of the Dryads");
    give_mana_for(&mut t, P0, "Song of the Dryads");
    t.cast(P0, song).target(eater).go();
    t.resolve_all();
    let forest = t.g.current(eater);
    assert!(!t.obj(forest).chars.is_creature());
    assert!(t.obj(forest).chars.is(CardType::Land));
    // It lost its own abilities; it has just the Forest's mana ability (CR 305.7).
    let abilities = &t.obj(forest).chars.abilities;
    assert_eq!(abilities.len(), 1);
    assert!(matches!(&abilities[0].kind, AbilityKind::Activated(a) if a.is_mana_ability));
    assert_eq!(prepared_copy(&t, eater), Some(copy));
    assert_eq!(t.obj(copy).zone, Zone::Exile);
    // (Have a Bite needs a target creature.)
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    assert!(can_cast(&mut t, P0, copy, CastMethod::Normal));
    t.cast(P0, copy).target(bears).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // True Polymorph: "Target artifact or creature becomes a copy of another target
    // artifact or creature." The Eater becomes a Grizzly Bears, which has no prepare
    // spell: it's still prepared, and the copy can still be cast.
    let mut t = TestGame::new(2);
    let eater = t.enter(P0, EATER);
    let copy = prepared_copy(&t, eater).expect("prepared");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tp = t.hand(P0, "True Polymorph");
    give_mana_for(&mut t, P0, "True Polymorph");
    t.cast(P0, tp)
        .targets(&[Entity::Object(eater)])
        .targets(&[Entity::Object(bears)])
        .go();
    t.resolve_all();
    let now = t.g.current(eater);
    assert_eq!(t.obj(now).chars.name, "Grizzly Bears");
    assert!(!designations::has_prepare_spell(&t.g, now));
    assert_eq!(prepared_copy(&t, eater), Some(copy));
    t.lands(P0, "Swamp", 1);
    t.cast(P0, copy).target(now).go();
    t.resolve_all();
    assert_eq!(t.counters(now, counters::PLUS1), 1);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn a_prepare_spell_that_doesnt_resolve_still_unprepared_its_permanent() {
    cr!("722.3c", "608.2b", "601.2i");
    ruling!(
        "Adventurous Eater // Have a Bite",
        "If a prepare spell with one or more targets has no legal targets when it tries to resolve, it won't resolve and none of its effects will happen. Since the spell was cast, the associated permanent will still not be prepared."
    );
    supported(EATER);
    let mut t = TestGame::new(2);
    let eater = t.enter(P0, EATER);
    let copy = prepared_copy(&t, eater).expect("prepared");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.cast(P0, copy).target(bears).go();
    // The target dies in response.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Have a Bite didn't resolve: no life gained ...
    assert_eq!(t.life(P0), 20);
    // ... and the Eater stays unprepared.
    assert!(prepared_copy(&t, eater).is_none());
    assert!(exiled_copies(&t).is_empty());
}
