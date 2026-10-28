//! cards4-referents-statics, family `copies`: static copy effects on another permanent
//! (CR 707.2, 613.2a) and copies of chosen objects.

use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// Casts Metamorphic Alteration on `host`, choosing `chosen`.
fn alter(t: &mut TestGame, host: ObjectId, chosen: ObjectId) -> ObjectId {
    t.lands(P0, "Island", 3);
    let aura = t.hand(P0, "Metamorphic Alteration");
    t.answer_choose(P0, &[Entity::Object(chosen)]);
    t.cast(P0, aura).target(host).go();
    t.resolve_all();
    t.g.current(aura)
}

#[test]
fn metamorphic_alteration_enchanted_creature_is_a_copy_of_the_chosen_creature() {
    cr!("707.2", "613.2a", "303.4");
    supported("Metamorphic Alteration");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    let aura = alter(&mut t, bears, angel);
    assert!(t.on_battlefield(aura), "{}", t.dump_log());
    assert_eq!(t.g.obj(bears).chars.name.as_str(), "Serra Angel");
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.g.obj(bears).has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    // The Aura leaves: the creature is itself again.
    t.lands(P0, "Plains", 2);
    let dis = t.hand(P0, "Disenchant");
    t.cast(P0, dis).target(aura).go();
    t.resolve_all();
    assert!(!t.on_battlefield(aura));
    assert_eq!(t.g.obj(bears).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn metamorphic_alteration_values_are_locked_in_as_the_effect_begins() {
    cr!("707.2", "613.2a");
    ruling!(
        "Metamorphic Alteration",
        "if that creature leaves the battlefield, the enchanted creature's characteristics are unaffected"
    );
    ruling!(
        "Metamorphic Alteration",
        "It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    t.g.add_counters(Entity::Object(angel), "+1/+1", 2, None);
    alter(&mut t, bears, angel);
    // Counters on the chosen creature aren't copied.
    assert_eq!(t.pt(bears), (4, 4));
    // The chosen creature leaves the battlefield: still a copy of it.
    t.lands(P0, "Swamp", 2);
    let murder = t.hand(P0, "Murder");
    t.lands(P0, "Swamp", 1);
    t.cast(P0, murder).target(angel).go();
    t.resolve_all();
    assert!(!t.on_battlefield(angel));
    assert_eq!(t.g.obj(bears).chars.name.as_str(), "Serra Angel");
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn spirit_of_resilience_may_become_a_copy_of_a_card_that_left_the_graveyard() {
    cr!("603.2c", "603.10a", "707.2");
    supported("Spirit of Resilience");
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Spirit of Resilience");
    let angel = t.graveyard(P0, "Serra Angel");
    let island = t.graveyard(P0, "Island");
    let moved = t.g.move_objects(
        [angel, island]
            .iter()
            .map(|o| mtg_engine::replacement::MoveEv {
                obj: *o,
                to: mtg_engine::object::Zone::Exile,
                pos: mtg_engine::ability::LibraryPosition::Top,
                cause: mtg_engine::events::MoveCause::Effect,
                by: Some(P1),
                etb: Default::default(),
                source: None,
            })
            .collect(),
    );
    let angel_now = moved[0].expect("exiled");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(angel_now)]);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(spirit, "+1/+1"), 1, "{}", t.dump_log());
    assert_eq!(t.g.obj(spirit).chars.name.as_str(), "Serra Angel");
    assert_eq!(t.pt(spirit), (5, 5));
}

fn fourteenth_doctor(copy: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let seventh = t.library_top(P0, "The Seventh Doctor");
    let milled = t
        .g
        .move_object(
            seventh,
            mtg_engine::object::Zone::Graveyard(P0),
            mtg_engine::events::MoveCause::Mill,
            Some(P0),
        )
        .expect("milled");
    if copy {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(milled)]);
    } else {
        t.answer_yes(P0, false);
        t.answer_choose(P0, &[]);
    }
    let doc = t.enter(P0, "The Fourteenth Doctor");
    t.settle();
    t.resolve_all();
    (t, doc)
}

#[test]
fn the_fourteenth_doctor_gains_haste_only_if_it_entered_as_a_copy() {
    cr!("614.1c", "707.9");
    supported("The Fourteenth Doctor");
    let haste = mtg_engine::keywords::KeywordKind::Haste;
    let (t, doc) = fourteenth_doctor(true);
    assert_eq!(t.obj_now(doc).chars.name.as_str(), "The Seventh Doctor", "{}", t.dump_log());
    assert!(t.obj_now(doc).has_keyword(haste));
    let (t, doc) = fourteenth_doctor(false);
    assert_eq!(t.obj_now(doc).chars.name.as_str(), "The Fourteenth Doctor");
    assert!(!t.obj_now(doc).has_keyword(haste));
}

#[test]
fn ulalek_copies_all_spells_you_control() {
    cr!("707.10", "603.2");
    supported("Ulalek, Fused Atrocity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ulalek, Fused Atrocity");
    t.lands(P0, "Wastes", 10);
    let dev = t.hand(P0, "Eldrazi Devastator");
    t.answer_yes(P0, true);
    t.cast(P0, dev).go();
    t.resolve_all();
    // The Eldrazi spell and its copy both resolve.
    assert_eq!(t.named_on_battlefield("Eldrazi Devastator").len(), 2, "{}", t.dump_log());
}
