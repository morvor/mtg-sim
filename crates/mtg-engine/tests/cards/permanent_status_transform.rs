//! Transforming named objects (CR 701.27): "transform up to one target Werewolf you
//! control", "transform any number of Human Werewolves you control", "transform all
//! Humans", "then if there are three or more cards exiled with ~, transform it" (patterns
//! in `src/oracle/patterns/face_status_grammar.rs`).

use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn transform_cards_compile() {
    assert_compiles(&[
        "Waxing Moon",
        "Tovolar, Dire Overlord // Tovolar, the Midnight Scourge",
        "Moonmist",
        "Profane Procession // Tomb of the Dusk Rose",
        "Ludevic's Test Subject // Ludevic's Abomination",
    ]);
}

#[test]
fn waxing_moon_transforms_up_to_one_target_werewolf() {
    cr!("701.27a", "712.8");
    let mut t = TestGame::new(2);
    let shepherd = t.battlefield(P0, "Gatstaf Shepherd // Gatstaf Howler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Waxing Moon");
    t.lands(P0, "Forest", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(shepherd)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(shepherd).face, FaceState::Back);
    assert_eq!(t.obj_now(shepherd).chars.name.as_str(), "Gatstaf Howler");
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
}

#[test]
fn waxing_moon_may_be_cast_without_a_target() {
    cr!("115.3", "601.2c");
    ruling!(
        "Waxing Moon",
        "You may cast Waxing Moon without targeting a Werewolf."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Waxing Moon");
    t.lands(P0, "Forest", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
}

#[test]
fn tovolar_transforms_any_number_of_human_werewolves() {
    cr!("701.27a", "603.4");
    ruling!(
        "Tovolar, Dire Overlord // Tovolar, the Midnight Scourge",
        "This ability will allow you to transform Human Werewolf creatures from previous visits to Innistrad as well."
    );
    let mut t = TestGame::new(2);
    // P1's turn, in which a spell is cast: it stays day, and the Shepherds' own "if no
    // spells were cast last turn" abilities don't transform them at P0's upkeep.
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Tovolar, Dire Overlord // Tovolar, the Midnight Scourge");
    let a = t.battlefield(P0, "Gatstaf Shepherd // Gatstaf Howler");
    let b = t.battlefield(P0, "Gatstaf Shepherd // Gatstaf Howler");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(a).face, FaceState::Back);
    assert_eq!(t.obj_now(b).face, FaceState::Front);
}

#[test]
fn seedpod_caretaker_transforms_target_incubator_token() {
    cr!("701.27a", "701.53b");
    let mut t = TestGame::new(2);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    let incubator = mtg_engine::kwa::incubate::incubate(&mut t.g, P0, 2, &mut ctx)[0];
    t.g.flush_events();
    assert!(!t
        .obj_now(incubator)
        .chars
        .card_types
        .contains(mtg_engine::types::CardType::Creature));
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![1]),
    );
    t.answer_targets(P0, &[Entity::Object(incubator)]);
    t.enter(P0, "Seedpod Caretaker");
    t.resolve_all();
    assert_eq!(t.obj_now(incubator).face, FaceState::Back);
    assert_eq!(t.pt(incubator), (2, 2));
}

#[test]
fn moonmist_transforms_humans_and_prevents_combat_damage_by_others() {
    cr!("701.27a", "615.1");
    ruling!(
        "Moonmist",
        "Moonmist causes any double-faced Human to transform, not just Werewolves."
    );
    ruling!(
        "Moonmist",
        "even if that creature wasn’t on the battlefield (or was a Werewolf or a Wolf) when Moonmist resolved."
    );
    let mut t = TestGame::new(2);
    let shepherd = t.battlefield(P0, "Gatstaf Shepherd // Gatstaf Howler");
    let spell = t.hand(P0, "Moonmist");
    t.lands(P0, "Forest", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.obj_now(shepherd).face, FaceState::Back);
    // A creature that wasn't on the battlefield as Moonmist resolved isn't a Werewolf or a
    // Wolf: its combat damage is prevented too.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wolf = t.battlefield(P0, "Timber Wolves");
    let life = t.life(P1);
    t.attack(
        &[
            (shepherd, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (wolf, Entity::Player(P1)),
        ],
        &[],
    );
    // Gatstaf Howler (3/3) and Timber Wolves (1/1) deal theirs.
    assert_eq!(t.life(P1), life - 4);
}

/// Profane Procession with three activations' worth of mana.
fn procession(t: &mut TestGame) -> ObjectId {
    let pp = t.battlefield(P0, "Profane Procession // Tomb of the Dusk Rose");
    t.lands(P0, "Scrubland", 15);
    t.set_step(P0, Step::PrecombatMain);
    pp
}

#[test]
fn profane_procession_transforms_once_three_cards_are_exiled_with_it() {
    cr!("701.27a", "607.2a");
    let mut t = TestGame::new(2);
    let pp = procession(&mut t);
    let foes: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    for (i, f) in foes.iter().enumerate() {
        t.activate(P0, pp, 0, &[Entity::Object(*f)]).unwrap();
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"));
        let back = t.obj_now(pp).face == FaceState::Back;
        assert_eq!(back, i == 2, "after {} activations", i + 1);
    }
    assert_eq!(t.obj_now(pp).chars.name.as_str(), "Tomb of the Dusk Rose");
}

#[test]
fn profane_procession_tokens_dont_count() {
    cr!("111.7", "607.2a");
    ruling!(
        "Profane Procession // Tomb of the Dusk Rose",
        "They won't count towards the number of cards exiled with it."
    );
    let mut t = TestGame::new(2);
    let pp = procession(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let spec = mtg_engine::replacement::TokenCreate {
        chars: t.g.obj(bears).copiable.clone(),
        card: t.g.obj(bears).card.clone(),
        tapped: false,
        attacking: None,
        copy_of: Some(bears),
        copy_exceptions: vec![],
    };
    let token = t.g.create_tokens(P1, spec, 1, None)[0];
    for f in [bears, other, token] {
        t.activate(P0, pp, 0, &[Entity::Object(f)]).unwrap();
        t.resolve_all();
    }
    // Two cards and a token were exiled with it.
    assert_eq!(t.obj_now(pp).face, FaceState::Front);
}

#[test]
fn profane_procession_later_activations_dont_transform_it_back() {
    cr!("701.27f");
    ruling!(
        "Profane Procession // Tomb of the Dusk Rose",
        "Further activations waiting to resolve won't cause Tomb of the Dusk Rose to transform back into Profane Procession"
    );
    let mut t = TestGame::new(2);
    let pp = procession(&mut t);
    let foes: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    t.activate(P0, pp, 0, &[Entity::Object(foes[0])]).unwrap();
    t.resolve_all();
    // Three activations on the stack at once: the second one to resolve exiles the third
    // card and transforms it; the last one sees four cards but doesn't transform it back.
    t.lands(P0, "Scrubland", 5);
    t.activate(P0, pp, 0, &[Entity::Object(foes[1])]).unwrap();
    t.activate(P0, pp, 0, &[Entity::Object(foes[2])]).unwrap();
    let extra = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, pp, 0, &[Entity::Object(extra)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(pp).face, FaceState::Back);
    assert!(!t.on_battlefield(extra));
}

#[test]
fn ludevics_test_subject_transforms_with_five_hatchling_counters() {
    cr!("701.27a", "122.1");
    let mut t = TestGame::new(2);
    let subject = t.battlefield(P0, "Ludevic's Test Subject // Ludevic's Abomination");
    t.g.add_counters(Entity::Object(subject), "hatchling", 3, None);
    t.lands(P0, "Island", 4);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, subject, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(subject, "hatchling"), 4);
    assert_eq!(t.obj_now(subject).face, FaceState::Front);
    t.activate(P0, subject, 0, &[]).unwrap();
    t.resolve_all();
    // All of them were removed, and it transformed.
    assert_eq!(t.obj_now(subject).face, FaceState::Back);
    assert_eq!(t.counters(subject, "hatchling"), 0);
    assert_eq!(t.pt(subject), (13, 13));
}
