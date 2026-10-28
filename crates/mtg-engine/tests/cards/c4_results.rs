//! cards4-referents-statics, family `results`: referents to what earlier steps did.

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

#[test]
fn ambitious_augmenter_moves_its_last_known_counters_to_the_fractal() {
    cr!("608.2h", "122.1");
    supported("Ambitious Augmenter");
    let mut t = TestGame::new(2);
    let aug = t.battlefield(P0, "Ambitious Augmenter");
    t.g.add_counters(Entity::Object(aug), "+1/+1", 2, None);
    t.g.add_counters(Entity::Object(aug), "flying", 1, None);
    t.g.destroy(aug, None);
    t.settle();
    t.resolve_all();
    let fractal: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Fractal"))
        .collect();
    assert_eq!(fractal.len(), 1, "{}", t.dump_log());
    assert_eq!(t.counters(fractal[0], "+1/+1"), 2);
    assert_eq!(t.counters(fractal[0], "flying"), 1);
    assert_eq!(t.pt(fractal[0]), (2, 2));
}

#[test]
fn taigam_exiles_the_second_spell_with_time_counters_and_suspend() {
    cr!("702.62a", "707.10");
    supported("Taigam, Master Opportunist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Taigam, Master Opportunist");
    t.lands(P0, "Mountain", 2);
    let a = t.hand(P0, "Shock");
    let b = t.hand(P0, "Shock");
    t.cast(P0, a).target(Entity::Player(P1)).go();
    t.resolve_all();
    t.cast(P0, b).target(Entity::Player(P1)).go();
    t.resolve_all();
    // The first Shock and the copy of the second dealt damage; the second is exiled.
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    let exiled = t.g.current(b);
    assert_eq!(t.zone(exiled), mtg_engine::object::Zone::Exile);
    assert_eq!(t.counters(exiled, "time"), 4);
    assert!(t
        .g
        .obj(exiled)
        .has_keyword(mtg_engine::keywords::KeywordKind::Suspend));
}

#[test]
fn zimone_turns_a_face_down_permanent_face_up_the_second_time() {
    cr!("701.62a", "708.8");
    supported("Zimone, Mystery Unraveler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zimone, Mystery Unraveler");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Grizzly Bears");
    // First land: manifest dread.
    t.enter(P0, "Forest");
    t.resolve_all();
    let down: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).face_down)
        .collect();
    assert_eq!(down.len(), 1, "{}", t.dump_log());
    // Second land: you may turn it face up.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(down[0])]);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert!(!t.obj_now(down[0]).face_down, "{}", t.dump_log());
    assert_eq!(t.obj_now(down[0]).chars.name.as_str(), "Grizzly Bears");
}

#[test]
fn paranormal_analyst_returns_the_card_manifest_dread_put_into_the_graveyard() {
    cr!("701.62a", "701.62b", "603.2");
    supported("Paranormal Analyst");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Paranormal Analyst");
    t.battlefield(P0, "Zimone, Mystery Unraveler");
    t.library_top(P0, "Island");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Mountain");
    // Zimone's landfall: manifest dread (the Bears manifested, the Mountain put into the
    // graveyard); Paranormal Analyst then returns the Mountain to your hand.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert!(t.in_hand(P0, "Mountain"), "{}", t.dump_log());
    assert_eq!(t.graveyard_size(P0), 0);
}
