//! Vesuvan Shapeshifter (hand-written, `src/cards/vesuvan_shapeshifter.rs`): "As ~ enters
//! or is turned face up, you may choose another creature on the battlefield. If you do,
//! until ~ is turned face down, it becomes a copy of that creature, except it has "At the
//! beginning of your upkeep, you may turn ~ face down."" Morph {1}{U}.

use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn turn_up(id: ObjectId) -> Action {
    Action::Special(SpecialAction::TurnFaceUp { obj: id })
}

/// Whether `p` could turn `id` face up now.
fn can_turn_up(t: &mut TestGame, p: PlayerId, id: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let id = t.g.current(id);
    t.g.legal_actions(p).contains(&turn_up(id))
}

/// Casts Vesuvan Shapeshifter for P0 choosing `copy` (or nothing) as it enters.
fn cast_choosing(t: &mut TestGame, copy: Option<ObjectId>) -> ObjectId {
    t.lands(P0, "Island", 5);
    let v = t.hand(P0, "Vesuvan Shapeshifter");
    let chosen: Vec<Entity> = copy.into_iter().map(Entity::Object).collect();
    t.answer_choose(P0, &chosen);
    t.cast(P0, v).go();
    t.resolve();
    t.g.current(v)
}

/// Passes to P0's next upkeep, answering its "you may turn ~ face down" trigger.
fn next_upkeep(t: &mut TestGame, turn_down: bool) {
    t.answer_yes(P0, turn_down);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

fn has_upkeep_trigger(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| a.text == "At the beginning of your upkeep, you may turn ~ face down.")
}

#[test]
fn it_enters_as_a_copy_until_it_is_turned_face_down() {
    cr!("707.5", "614.1c", "708.2a");
    ruling!(
        "Vesuvan Shapeshifter",
        "Any enters-the-battlefield abilities of the copied creature will trigger if Vesuvan Shapeshifter is entering the battlefield"
    );
    let mut t = TestGame::new(2);
    // "When Elvish Visionary enters, draw a card."
    let visionary = t.battlefield(P1, "Elvish Visionary");
    let hand = t.hand_size(P0);
    let v = cast_choosing(&mut t, Some(visionary));
    t.resolve_all();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Elvish Visionary");
    assert_eq!(t.pt(v), (1, 1));
    assert!(has_upkeep_trigger(&t, v));
    // The copy's enters ability triggered: P0 drew a card.
    assert_eq!(t.hand_size(P0), hand + 1);
    // Declining keeps it a copy.
    next_upkeep(&mut t, false);
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Elvish Visionary");
    // Turned face down: a 2/2 with no name or abilities, and no longer a copy.
    next_upkeep(&mut t, true);
    let o = t.obj_now(v);
    assert!(o.face_down);
    assert_eq!(o.chars.name.as_str(), "");
    assert!(o.chars.abilities.is_empty());
    assert_eq!(t.pt(v), (2, 2));
}

#[test]
fn turned_face_up_it_chooses_again() {
    cr!("614.1e", "707.2", "708.8", "702.37e");
    let mut t = TestGame::new(2);
    let visionary = t.battlefield(P1, "Elvish Visionary");
    let giant = t.battlefield(P1, "Hill Giant");
    let v = cast_choosing(&mut t, Some(visionary));
    t.resolve_all();
    next_upkeep(&mut t, true);
    assert!(t.obj_now(v).face_down);
    // Its own morph turns it face up again ({1}{U}): the copy effect ended for good when
    // it was turned face down.
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 2);
    assert!(can_turn_up(&mut t, P0, v));
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.g.perform_action(P0, turn_up(t.g.current(v))).unwrap();
    t.resolve_all();
    let o = t.obj_now(v);
    assert!(!o.face_down);
    assert_eq!(o.chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(v), (3, 3));
    assert!(has_upkeep_trigger(&t, v));
    // Turning face up isn't entering: no enters ability triggered.
    assert_eq!(t.hand_size(P0), hand);
    // And it can be turned face down again.
    next_upkeep(&mut t, true);
    assert!(t.obj_now(v).face_down);
    assert_eq!(t.pt(v), (2, 2));
}

#[test]
fn cast_face_down_and_turned_face_up_choosing_nothing_it_is_a_0_0() {
    cr!("702.37e", "704.5f");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 5);
    let v = t.hand(P0, "Vesuvan Shapeshifter");
    t.cast(P0, v)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve();
    let v = t.g.current(v);
    assert!(t.obj_now(v).face_down);
    t.answer_choose(P0, &[]);
    t.g.perform_action(P0, turn_up(v)).unwrap();
    t.settle();
    // A 0/0 Vesuvan Shapeshifter: put into the graveyard.
    assert!(!t.on_battlefield(v));
}

#[test]
fn its_copy_turned_face_up_gets_the_copied_creatures_face_up_abilities() {
    cr!("603.2", "708.8");
    ruling!(
        "Vesuvan Shapeshifter",
        "any \"As [this creature] is turned face up\" or \"When [this creature] is turned face up\" abilities will apply"
    );
    let mut t = TestGame::new(2);
    // "When Fathom Seer is turned face up, draw two cards."
    let seer = t.battlefield(P1, "Fathom Seer");
    t.lands(P0, "Island", 5);
    let v = t.hand(P0, "Vesuvan Shapeshifter");
    t.cast(P0, v)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve();
    let v = t.g.current(v);
    t.lands(P0, "Island", 2);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[Entity::Object(seer)]);
    t.g.perform_action(P0, turn_up(v)).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Fathom Seer");
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn as_turned_face_up_abilities_of_the_copied_creature_apply() {
    cr!("614.1e", "613.2a");
    ruling!(
        "Vesuvan Shapeshifter",
        "any \"As [this creature] is turned face up\" or \"When [this creature] is turned face up\" abilities will apply"
    );
    let mut t = TestGame::new(2);
    // "As Aquamorph Entity enters or is turned face up, it becomes your choice of 5/1 or
    // 1/5."
    t.lands(P1, "Island", 4);
    let entity = t.hand(P1, "Aquamorph Entity");
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, entity).go();
    t.resolve();
    let entity = t.g.current(entity);
    assert_eq!(t.pt(entity), (5, 1));
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 5);
    let v = t.hand(P0, "Vesuvan Shapeshifter");
    t.cast(P0, v)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve();
    let v = t.g.current(v);
    t.lands(P0, "Island", 2);
    t.answer_choose(P0, &[Entity::Object(entity)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.g.perform_action(P0, turn_up(v)).unwrap();
    t.settle();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Aquamorph Entity");
    assert_eq!(t.pt(v), (1, 5));
}

#[test]
fn it_copies_only_the_copiable_values() {
    cr!("707.2");
    ruling!(
        "Vesuvan Shapeshifter",
        "whether it has any counters on it"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 2, None);
    t.g.objects[bears.0 as usize].tapped = true;
    let v = cast_choosing(&mut t, Some(bears));
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.pt(v), (2, 2));
    assert!(!t.obj_now(v).tapped);
}

#[test]
fn a_copy_of_it_has_the_trigger_but_no_morph_once_face_down() {
    cr!("707.9b", "702.37e");
    ruling!(
        "Vesuvan Shapeshifter",
        "the new copy will also have the triggered ability to turn it face down"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let v = cast_choosing(&mut t, Some(bears));
    t.resolve_all();
    // Clone copies it: a Grizzly Bears with the upkeep trigger.
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.answer_choose(P0, &[Entity::Object(v)]);
    t.cast(P0, clone).go();
    t.resolve();
    let clone = t.g.current(clone);
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Grizzly Bears");
    assert!(has_upkeep_trigger(&t, clone));
    // Both trigger; both are turned face down.
    t.answer_yes(P0, true);
    next_upkeep(&mut t, true);
    assert!(t.obj_now(clone).face_down);
    assert!(t.obj_now(v).face_down);
    // The Shapeshifter has its morph face down; the Clone has none.
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 4);
    assert!(can_turn_up(&mut t, P0, v));
    assert!(!can_turn_up(&mut t, P0, clone));
}

#[test]
fn the_copy_effect_ends_for_good_when_it_is_turned_face_down() {
    cr!("611.2b", "708.8");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let v = cast_choosing(&mut t, Some(bears));
    t.resolve_all();
    next_upkeep(&mut t, true);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 2);
    // Turned face up choosing nothing, it's a Vesuvan Shapeshifter again (0/0), not
    // Grizzly Bears.
    t.answer_choose(P0, &[]);
    t.g.perform_action(P0, turn_up(t.g.current(v))).unwrap();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Vesuvan Shapeshifter");
    assert_eq!(t.pt(v), (0, 0));
    t.settle();
    assert!(!t.on_battlefield(v));
}

#[test]
fn copying_a_copy_it_copies_what_that_creature_is_copying() {
    cr!("707.3");
    ruling!(
        "Vesuvan Shapeshifter",
        "In most cases, Vesuvan Shapeshifter will copy whatever that creature is copying"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    // P1's Clone enters as a copy of Hill Giant.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 4);
    let clone = t.hand(P1, "Clone");
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.cast(P1, clone).go();
    t.resolve();
    let clone = t.g.current(clone);
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Hill Giant");
    // Entering as a copy of the Clone: a Hill Giant.
    t.set_step(P0, Step::PrecombatMain);
    let v = cast_choosing(&mut t, Some(clone));
    t.resolve_all();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(v), (3, 3));
    // Turned face up as a copy of the Clone: a Hill Giant again.
    next_upkeep(&mut t, true);
    assert!(t.obj_now(v).face_down);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 2);
    t.answer_choose(P0, &[Entity::Object(clone)]);
    t.g.perform_action(P0, turn_up(t.g.current(v))).unwrap();
    t.settle();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(v), (3, 3));
}

#[test]
fn x_in_the_copied_mana_cost_is_0() {
    cr!("707.2", "107.3g");
    ruling!("Vesuvan Shapeshifter", "If a copied creature has {X} in its mana cost, X is 0.");
    let mut t = TestGame::new(2);
    // {X}{U}{U}{U} 3/3: "When Gadwick enters, draw X cards."
    let gadwick = t.battlefield(P1, "Gadwick, the Wizened");
    let hand = t.hand_size(P0);
    let v = cast_choosing(&mut t, Some(gadwick));
    t.resolve_all();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Gadwick, the Wizened");
    assert_eq!(t.g.mana_value_of(v), 3);
    // Its enters ability triggered with X = 0: no cards drawn.
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn copying_a_token_it_copies_the_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Vesuvan Shapeshifter",
        "Vesuvan Shapeshifter copies the original characteristics of that token"
    );
    let mut t = TestGame::new(2);
    // "Create two 1/1 white Soldier creature tokens."
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 3);
    let raise = t.hand(P1, "Raise the Alarm");
    t.cast(P1, raise).go();
    t.resolve();
    let soldier = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|&id| t.g.obj(id).is_token())
        .expect("a Soldier token");
    // A +1/+1 counter isn't copied.
    t.g.add_counters(Entity::Object(soldier), counters::PLUS1, 1, None);
    t.set_step(P0, Step::PrecombatMain);
    let v = cast_choosing(&mut t, Some(soldier));
    t.resolve_all();
    let name = t.obj_now(soldier).chars.name.clone();
    let o = t.obj_now(v);
    assert_eq!(o.chars.name, name);
    assert!(o.chars.subtypes.iter().any(|s| s.as_str() == "Soldier"));
    assert!(o.chars.colors.contains(Color::White));
    assert!(!o.is_token());
    assert_eq!(t.pt(v), (1, 1));
}
