//! Transforming named objects (CR 701.27): "transform up to one target Werewolf you
//! control", "transform any number of Human Werewolves you control", "transform all
//! Humans" (pattern in `src/oracle/patterns/face_status_grammar.rs`).

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
