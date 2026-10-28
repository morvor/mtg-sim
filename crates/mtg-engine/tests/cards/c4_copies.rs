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
