//! The worked examples of CR 607 (linked abilities), played out as tests. The CR 607.4
//! example is cited by a test in `r600_linked_abilities.rs`.

use crate::r600_examples_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates the activated ability of `source` whose text contains `text`.
fn activate_text(t: &mut TestGame, p: PlayerId, source: ObjectId, text: &str, targets: &[Entity]) {
    t.g.recompute();
    let i = t
        .obj(source)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains(text))
        .unwrap_or_else(|| panic!("no activated ability {text:?}"));
    t.activate(p, source, i, targets).expect("activation failed");
}

fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn stormscape_battlemages_kicker_triggers_are_each_linked_to_their_kicker() {
    cr!("607.2i", "702.33c");
    cr_example!("607.2i", "Stormscape Battlemage");
    supported("Stormscape Battlemage");
    // Kicked with its {W} kicker only: it gains 3 life, nothing is destroyed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    t.lands(P0, "Plains", 1);
    let mage = t.hand(P0, "Stormscape Battlemage");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, mage).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert!(t.on_battlefield(bears));
    // Kicked with its {2}{B} kicker only: it destroys a nonblack creature, no life.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    t.lands(P0, "Swamp", 3);
    let mage = t.hand(P0, "Stormscape Battlemage");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, mage).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn quicksilver_elementals_sisters_abilities_return_only_what_their_exile_ability_exiled() {
    cr!("607.5", "607.2a");
    cr_example!("607.5", "Quicksilver Elemental");
    for name in [
        "Quicksilver Elemental",
        "Arc-Slogger",
        "Sisters of Stone Death",
    ] {
        supported(name);
    }
    let mut t = TestGame::new(2);
    let qe = t.battlefield(P0, "Quicksilver Elemental");
    let slogger = t.battlefield(P1, "Arc-Slogger");
    let sisters = t.battlefield(P1, "Sisters of Stone Death");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    // Blue mana may be spent as any color for the Elemental's abilities.
    t.lands(P0, "Island", 8);
    // The top ten cards of P0's library: creature cards.
    let library: Vec<ObjectId> = (0..10).map(|_| t.library_top(P0, "Hill Giant")).collect();
    // The Elemental gains Arc-Slogger's ability, and activates it.
    activate_text(&mut t, P0, qe, "gains all activated abilities", &[Entity::Object(slogger)]);
    t.resolve();
    activate_text(&mut t, P0, qe, "Exile the top ten cards", &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(library.iter().all(|c| t.zone(*c) == mtg_engine::object::Zone::Exile));
    // Then it gains Sisters of Stone Death's abilities.
    activate_text(&mut t, P0, qe, "gains all activated abilities", &[Entity::Object(sisters)]);
    t.resolve();
    // It attacks; Grizzly Bears blocks it; the exile ability exiles the blocker.
    crate::r506_common::declare(&mut t, &[(qe, Entity::Player(P1))]);
    crate::r506_common::block(&mut t, P1, &[(blocker, qe)]);
    crate::r506_common::go_to(&mut t, Step::DeclareBlockers);
    activate_text(&mut t, P0, qe, "Exile target creature blocking", &[Entity::Object(blocker)]);
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    // The return ability can return only the creature card exiled by the Elemental's
    // Sisters exile ability, not the cards exiled by Arc-Slogger's ability.
    let bears_card = t.g.current(blocker);
    t.answer_choose(P0, &[Entity::Object(library[0])]);
    activate_text(&mut t, P0, qe, "Put a creature card exiled with", &[]);
    t.resolve();
    assert!(t.zone(library[0]) == mtg_engine::object::Zone::Exile);
    assert!(t.on_battlefield(bears_card));
    assert_eq!(t.obj_now(bears_card).controller, P0);
    assert!(library.iter().all(|c| t.zone(*c) == mtg_engine::object::Zone::Exile));
}

#[test]
fn unstable_shapeshifter_copying_voice_of_all_never_chose_a_color() {
    cr!("607.5a", "607.2d");
    cr_example!("607.5a", "Unstable Shapeshifter copies it");
    supported("Voice of All");
    supported("Unstable Shapeshifter");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Unstable Shapeshifter");
    // Voice of All enters; its controller chooses red.
    choose_color(&mut t, P0, Color::Red);
    let voice = t.enter(P0, "Voice of All");
    t.resolve_all();
    assert_eq!(t.obj_now(shifter).chars.name.as_str(), "Voice of All");
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.g.protected_from(voice, bolt));
    // The Shapeshifter didn't enter as Voice of All: it has no protection.
    assert!(!t.g.protected_from(shifter, bolt));
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(shifter).go();
    t.resolve_all();
    assert!(!t.on_battlefield(shifter));
}
