//! Objects that have or gain the abilities of other objects (CR 113.10, 613.1f): which
//! abilities (activated, triggered, keywords that are activated abilities), when they're
//! determined (continuously for a static ability, CR 613.8 dependencies; once for a
//! resolving ability, CR 608.2h), what the gained abilities refer to (CR 201.5b) and their
//! identity (CR 602.5c, 607.5).

use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The activated abilities `id` has, by text.
fn activated(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.clone())
        .collect()
}

fn triggered(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Triggered(_)))
        .map(|a| a.text.clone())
        .collect()
}

/// The index among `id`'s activated abilities of the one with this text.
fn index_of(t: &mut TestGame, id: ObjectId, text: &str) -> usize {
    activated(t, id)
        .iter()
        .position(|a| a.contains(text))
        .unwrap_or_else(|| panic!("no activated ability {text:?}"))
}

#[test]
fn a_static_ability_gives_the_activated_abilities_the_objects_have_now() {
    cr!("113.10", "613.1f", "201.5b");
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Necrotic Ooze");
    assert!(activated(&mut t, ooze).is_empty());
    // A creature card in any graveyard: its activated ability, naming the card itself,
    // refers to Necrotic Ooze on Necrotic Ooze.
    let troll = t.graveyard(P1, "Cudgel Troll");
    assert_eq!(activated(&mut t, ooze).len(), 1);
    t.lands(P0, "Forest", 1);
    let i = index_of(&mut t, ooze, "Regenerate");
    t.activate(P0, ooze, i, &[]).unwrap();
    t.resolve_all();
    t.g.destroy(ooze, None);
    t.settle();
    assert!(t.on_battlefield(ooze), "the regeneration shield is Necrotic Ooze's");
    assert!(t.obj_now(ooze).tapped);
    // Once the card leaves the graveyard, the ability is gone.
    t.g.move_object(troll, Zone::Exile, MoveCause::Effect, None);
    assert!(activated(&mut t, ooze).is_empty());
}

#[test]
fn only_activated_abilities_are_gained_unless_triggered_ones_are_named() {
    cr!("113.10", "613.1f", "702.29a");
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Necrotic Ooze");
    // Flying and vigilance (static keywords), a triggered ability, and cycling — a keyword
    // that is an activated ability.
    t.graveyard(P0, "Serra Angel");
    t.graveyard(P0, "Krosan Tusker");
    t.g.recompute();
    let c = &t.g.obj(ooze).chars;
    assert!(!c.has_keyword(KeywordKind::Flying));
    assert!(!c.has_keyword(KeywordKind::Vigilance));
    assert!(c.has_keyword(KeywordKind::Cycling), "cycling is an activated ability");
    assert!(triggered(&mut t, ooze).is_empty());
}

#[test]
fn activated_and_triggered_abilities_of_the_exiled_card() {
    cr!("113.10", "613.1f", "201.5b", "603.4");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Howling Mine");
    let idris = t.hand(P0, "Idris, Soul of the TARDIS");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    t.cast(P0, idris).go();
    t.resolve(); // Idris
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.resolve_all(); // its imprint trigger exiles Howling Mine
    let idris = t.g.current(idris);
    assert!(t.in_exile("Howling Mine"));
    // Idris has Howling Mine's triggered ability ("if this artifact is untapped" is about
    // Idris) and gets +X/+X for its mana value (2): a 3/3 becomes 5/5.
    let mine_trigger = |t: &mut TestGame| {
        triggered(t, idris)
            .iter()
            .filter(|a| a.contains("draws an additional card"))
            .count()
    };
    assert_eq!(mine_trigger(&mut t), 1);
    assert_eq!(t.pt(idris), (5, 5));
    t.advance_to(P1, mtg_engine::turn::Step::Draw);
    t.resolve_all();
    // P1 drew their card for the turn and the additional card from Idris's ability.
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn a_resolving_ability_gives_the_abilities_the_object_has_as_it_resolves() {
    cr!("608.2h", "611.2c", "613.1f");
    let mut t = TestGame::new(2);
    let qe = t.battlefield(P0, "Quicksilver Elemental");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.lands(P0, "Island", 1);
    let i = index_of(&mut t, qe, "gains all activated abilities");
    t.activate(P0, qe, i, &[Entity::Object(sorcerer)]).unwrap();
    t.resolve_all();
    assert_eq!(activated(&mut t, qe).len(), 2);
    // The creature it copied leaves the battlefield: Quicksilver Elemental keeps the
    // ability until end of turn.
    t.g.move_object(sorcerer, Zone::Exile, MoveCause::Effect, None);
    assert_eq!(activated(&mut t, qe).len(), 2);
    let ping = index_of(&mut t, qe, "deals 1 damage");
    t.activate(P0, qe, ping, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // The effect ends with the turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(activated(&mut t, qe).len(), 1);
}

#[test]
fn each_gained_copy_of_a_once_each_turn_ability_is_a_separate_ability() {
    cr!("602.5c", "113.2c");
    let mut t = TestGame::new(2);
    let qe = t.battlefield(P0, "Quicksilver Elemental");
    let rootwalla = t.battlefield(P1, "Rootwalla");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Forest", 4);
    for _ in 0..2 {
        let i = index_of(&mut t, qe, "gains all activated abilities");
        t.activate(P0, qe, i, &[Entity::Object(rootwalla)]).unwrap();
        t.resolve_all();
    }
    let texts = activated(&mut t, qe);
    assert_eq!(texts.iter().filter(|a| a.contains("+2/+2")).count(), 2);
    // Each copy can be activated once this turn.
    let base = t.pt(qe);
    for k in 0..2 {
        let idx: Vec<usize> = activated(&mut t, qe)
            .iter()
            .enumerate()
            .filter(|(_, a)| a.contains("+2/+2"))
            .map(|(i, _)| i)
            .collect();
        t.activate(P0, qe, idx[k], &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.pt(qe), (base.0 + 4, base.1 + 4));
    // Neither can be activated again this turn.
    let idx = index_of(&mut t, qe, "+2/+2");
    assert!(t.activate(P0, qe, idx, &[]).is_err());
}

#[test]
fn copying_waits_for_effects_that_give_the_copied_object_abilities() {
    cr!("613.8a", "613.8b", "613.1f", "201.5b");
    let mut t = TestGame::new(2);
    // Experiment Kraj's timestamp is earlier than the Aura's.
    let kraj = t.battlefield(P0, "Experiment Kraj");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bear), "+1/+1", 1, None);
    let study = t.battlefield(P0, "Hermetic Study");
    t.g.attach(study, Entity::Object(bear));
    // Kraj's effect depends on the Aura's: it has the ability the Aura grants.
    let texts = activated(&mut t, kraj);
    assert!(
        texts.iter().any(|a| a.contains("deals 1 damage")),
        "Kraj has {texts:?}"
    );
    // The ability refers to Kraj: Kraj taps and deals the damage.
    let i = index_of(&mut t, kraj, "deals 1 damage");
    t.activate(P0, kraj, i, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.obj_now(kraj).tapped);
    assert!(!t.obj_now(bear).tapped);
}

#[test]
fn abilities_gained_from_different_objects_are_distinct_and_unlinked() {
    cr!("607.5", "607.5a", "113.2c");
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Necrotic Ooze");
    t.graveyard(P0, "Prodigal Sorcerer");
    t.graveyard(P1, "Prodigal Sorcerer");
    t.g.recompute();
    let abilities: Vec<Ability> = t
        .g
        .obj(ooze)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .cloned()
        .collect();
    assert_eq!(abilities.len(), 2, "one from each card");
    assert_ne!(abilities[0].uid, abilities[1].uid);
    // Each is linked to neither the Ooze's printed abilities nor the other's.
    assert_ne!(abilities[0].link, 0);
    assert_ne!(abilities[0].link, abilities[1].link);
    // Recomputing keeps their identities (an activation restriction keeps counting).
    t.g.recompute();
    let again: Vec<u64> = t
        .g
        .obj(ooze)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.uid)
        .collect();
    assert_eq!(again, abilities.iter().map(|a| a.uid).collect::<Vec<_>>());
}
