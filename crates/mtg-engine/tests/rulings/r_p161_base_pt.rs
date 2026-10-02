//! Rulings batch P161 — "shapechange": effects that set base power and toughness apply in
//! layer 7b (CR 613.4b) in timestamp order (CR 613.7), so they overwrite earlier setting
//! effects and are overwritten by later ones, while modifying effects and counters (7c)
//! apply on top no matter when they began. (Helpers from batch P160.)

use crate::r_p160_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn setting_activated_abilities_overwrite_and_are_overwritten() {
    cr!("613.4b", "613.4c", "613.4d", "613.7b");
    ruling!(
        "Creeperhulk",
        "The activated ability overwrites all previous effects that set the target creature’s base power and/or toughness to specific values. Other effects that set its base power and/or toughness that start to apply after Creeperhulk’s ability resolves will overwrite Creeperhulk’s effect."
    );
    ruling!(
        "Gigantomancer",
        "This ability overwrites all previous effects that set the targeted creature’s power and toughness to specific values. Other effects that set its power or toughness to specific values that start to apply after this ability resolves will overwrite this effect."
    );
    helpers_supported();
    supported("Creeperhulk");
    supported("Gigantomancer");

    let mut t = TestGame::new(2);
    let hulk = t.battlefield(P0, "Creeperhulk");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Forest", 2);
        activate_resolve(t, P0, hulk, 0, &[Entity::Object(bears)]);
    };
    layer7(
        &mut t,
        bears,
        "Relic's Roar",
        false,
        apply,
        (5, 5),
        "Mind Transfer Protocol",
    );

    let mut t = TestGame::new(2);
    let mancer = t.battlefield(P0, "Gigantomancer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Wastes", 1);
        activate_resolve(t, P0, mancer, 0, &[Entity::Object(bears)]);
    };
    layer7(
        &mut t,
        bears,
        "Square Up",
        false,
        apply,
        (7, 7),
        "Relic's Roar",
    );
}

#[test]
fn setting_spells_overwrite_and_keep_modifiers() {
    cr!("613.4b", "613.4c", "613.7b");
    ruling!(
        "Zhalfirin Shapecraft",
        "Zhalfirin Shapecraft overwrites all previous effects that set the creature’s base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after the ability resolves will overwrite this effect."
    );
    ruling!(
        "Water Wings",
        "Water Wings overwrites any previous effects that set the creature's power and/or toughness to specific values. Effects that otherwise modify its power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    ruling!(
        "Quandrix Charm",
        "The effect of Quandrix Charm's last mode will overwrite any previous effects that set the creature's power and toughness to specific values. Effects that otherwise modify the target creature's power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    helpers_supported();
    for name in ["Zhalfirin Shapecraft", "Water Wings", "Quandrix Charm"] {
        supported(name);
    }
    for (name, set) in [("Zhalfirin Shapecraft", (4, 3)), ("Water Wings", (4, 4))] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        layer7(
            &mut t,
            bears,
            "Square Up",
            false,
            targeted_spell(name, bears),
            set,
            "Mind Transfer Protocol",
        );
    }

    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        lands_for_cost(t, P0, "Quandrix Charm");
        let charm = t.hand(P0, "Quandrix Charm");
        t.cast(P0, charm).modes(&[2]).target(bears).go();
        t.resolve_all();
    };
    layer7(
        &mut t,
        bears,
        "Relic's Roar",
        false,
        apply,
        (5, 5),
        "Square Up",
    );
}

#[test]
fn eidolon_of_astral_winds_constellation_sets_4_4() {
    cr!("613.4b", "613.4c", "613.7b");
    ruling!(
        "Eidolon of Astral Winds",
        "The effect of Eidolon of Astral Winds's last ability will overwrite any previous effects that set the creature's power and toughness to specific values. Effects that otherwise modify the target creature's power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    helpers_supported();
    supported("Eidolon of Astral Winds");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.enter(P0, "Eidolon of Astral Winds");
        t.resolve_all();
        assert!(t
            .obj(bears)
            .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    };
    layer7(
        &mut t,
        bears,
        "Relic's Roar",
        false,
        apply,
        (4, 4),
        "Square Up",
    );
}

#[test]
fn equipment_and_auras_setting_base_pt_overwrite_earlier_effects() {
    cr!("613.4b", "613.4c", "613.7a", "702.103b");
    ruling!(
        "Wrecking Ball Arm",
        "Wrecking Ball Arm overwrites all previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply afterward will overwrite this effect."
    );
    ruling!(
        "Trickster's Elk",
        "Trickster's Elk overwrites all previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply afterward will overwrite this effect."
    );
    helpers_supported();
    supported("Wrecking Ball Arm");
    supported("Trickster's Elk");

    // "Equip {7}" onto a nonlegendary creature.
    let mut t = TestGame::new(2);
    let arm = t.battlefield(P0, "Wrecking Ball Arm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Wastes", 7);
        activate_resolve(t, P0, arm, 1, &[Entity::Object(bears)]);
        assert_eq!(t.obj(arm).attached_to, Some(Entity::Object(bears)));
    };
    layer7(
        &mut t,
        bears,
        "Relic's Roar",
        false,
        apply,
        (7, 7),
        "Square Up",
    );

    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let apply = move |t: &mut TestGame| {
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Wastes", 1);
        let elk = t.hand(P0, "Trickster's Elk");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.cast(P0, elk)
            .method(mtg_engine::object::CastMethod::Keyword(
                mtg_engine::keywords::KeywordKind::Bestow,
            ))
            .go();
        t.resolve_all();
    };
    layer7(
        &mut t,
        bears,
        "Relic's Roar",
        false,
        apply,
        (3, 3),
        "Square Up",
    );
}

#[test]
fn the_antiquities_war_overwrites_and_affects_only_artifacts_controlled_as_it_resolves() {
    cr!("613.4b", "613.7b", "611.2c", "714.2b");
    ruling!(
        "The Antiquities War",
        "The final chapter ability of The Antiquities War overwrites an artifact creature’s normal base power and toughness and all previous effects that set an artifact creature’s base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after the ability resolves will overwrite this effect."
    );
    ruling!(
        "The Antiquities War",
        "The final chapter ability of The Antiquities War affects only artifacts you control at the time it resolves. Artifacts you begin to control later in the turn won’t become 5/5 creatures."
    );
    helpers_supported();
    supported("The Antiquities War");
    supported("Ornithopter");
    supported("Sol Ring");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let ring = t.battlefield(P0, "Sol Ring");
    let saga = t.battlefield(P0, "The Antiquities War");
    // An earlier setting effect on the Ornithopter.
    cast_resolve(&mut t, P0, "Relic's Roar", &[Entity::Object(thopter)]);
    assert_eq!(t.pt(thopter), (4, 3));
    let saga = t.g.current(saga);
    t.g.add_counters(Entity::Object(saga), counters::LORE, 3, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.pt(thopter), (5, 5));
    assert!(t.obj(ring).is(CardType::Creature));
    assert_eq!(t.pt(ring), (5, 5));
    // A later artifact isn't affected.
    let later = t.battlefield(P0, "Sol Ring");
    t.g.recompute();
    assert!(!t.obj(later).is(CardType::Creature));
    // A later setting effect overwrites it.
    cast_resolve(&mut t, P0, "Square Up", &[Entity::Object(thopter)]);
    assert_eq!(t.pt(thopter), (4, 4));
    assert_eq!(t.pt(ring), (5, 5));
}

#[test]
fn turtle_duck_sets_only_its_power() {
    cr!("613.4b", "613.7b");
    ruling!(
        "Turtle-Duck",
        "Turtle-Duck's ability overwrites all previous effects that set its power to a specific value. Other effects that set its power to a specific value that start to apply after the ability resolves will overwrite that part of the effect."
    );
    supported("Turtle-Duck");
    supported("Gigantomancer");
    supported("Humble");
    let mut t = TestGame::new(2);
    let duck = t.battlefield(P0, "Turtle-Duck");
    let mancer = t.battlefield(P0, "Gigantomancer");
    t.lands(P0, "Wastes", 1);
    activate_resolve(&mut t, P0, mancer, 0, &[Entity::Object(duck)]);
    assert_eq!(t.pt(duck), (7, 7));
    t.lands(P0, "Wastes", 3);
    activate_resolve(&mut t, P0, duck, 0, &[]);
    // Only the power is overwritten.
    assert_eq!(t.pt(duck), (4, 7));
    cast_resolve(&mut t, P0, "Humble", &[Entity::Object(duck)]);
    assert_eq!(t.pt(duck), (0, 1));
}

#[test]
fn riptide_mangler_overwrites_its_previous_activations() {
    cr!("613.4b", "613.7b", "611.2a");
    ruling!(
        "Riptide Mangler",
        "The activated ability overwrites all previous effects that set Riptide Mangler’s base power to specific a value (such as previous activations of the ability). Other effects that set its base power that start to apply after Riptide Mangler’s ability resolves will overwrite this effect."
    );
    supported("Riptide Mangler");
    supported("Humble");
    let mut t = TestGame::new(2);
    let mangler = t.battlefield(P0, "Riptide Mangler");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    activate_resolve(&mut t, P0, mangler, 0, &[Entity::Object(giant)]);
    assert_eq!(t.pt(mangler), (3, 3));
    // The second activation overwrites the first, even with a smaller power.
    t.lands(P0, "Island", 2);
    activate_resolve(&mut t, P0, mangler, 0, &[Entity::Object(bears)]);
    assert_eq!(t.pt(mangler), (2, 3));
    cast_resolve(&mut t, P0, "Humble", &[Entity::Object(mangler)]);
    assert_eq!(t.pt(mangler), (0, 1));
}

#[test]
fn timber_paladin_sets_base_pt_by_aura_count_with_its_own_timestamp() {
    cr!("613.4b", "613.7a", "613.1f");
    ruling!(
        "Timber Paladin",
        "Timber Paladin's abilities each overwrite any previous effects that set creatures' power and/or toughness to specific numbers. Any power- or toughness-setting effects that start to apply after Timber Paladin enters the battlefield will overwrite these effects, regardless of when Timber Paladin became enchanted by one, two, or three or more Auras."
    );
    supported("Timber Paladin");
    supported("Pacifism");
    use mtg_engine::keywords::KeywordKind;
    let kw = |t: &TestGame, id: ObjectId, k: KeywordKind| t.obj_now(id).has_keyword(k);
    // One, two, three Auras: 3/3, 5/5 with vigilance, 10/10 with vigilance and trample.
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Timber Paladin");
    assert_eq!(t.pt(paladin), (1, 1));
    cast_resolve(&mut t, P0, "Pacifism", &[Entity::Object(paladin)]);
    assert_eq!(t.pt(paladin), (3, 3));
    assert!(!kw(&t, paladin, KeywordKind::Vigilance));
    cast_resolve(&mut t, P0, "Pacifism", &[Entity::Object(paladin)]);
    assert_eq!(t.pt(paladin), (5, 5));
    assert!(kw(&t, paladin, KeywordKind::Vigilance));
    assert!(!kw(&t, paladin, KeywordKind::Trample));
    cast_resolve(&mut t, P0, "Pacifism", &[Entity::Object(paladin)]);
    assert_eq!(t.pt(paladin), (10, 10));
    assert!(kw(&t, paladin, KeywordKind::Vigilance));
    assert!(kw(&t, paladin, KeywordKind::Trample));

    // A setting effect that began after it entered wins, even though the Auras were
    // attached after that effect began.
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Timber Paladin");
    cast_resolve(&mut t, P0, "Square Up", &[Entity::Object(paladin)]);
    assert_eq!(t.pt(paladin), (4, 4));
    cast_resolve(&mut t, P0, "Pacifism", &[Entity::Object(paladin)]);
    cast_resolve(&mut t, P0, "Pacifism", &[Entity::Object(paladin)]);
    assert_eq!(t.pt(paladin), (4, 4));
    assert!(kw(&t, paladin, KeywordKind::Vigilance));
}
