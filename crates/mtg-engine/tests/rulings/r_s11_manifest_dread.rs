//! Rulings batch S11 — manifest dread (CR 701.62) with Unwanted Remake: "Destroy target
//! creature. Its controller manifests dread." The manifested permanent follows the
//! manifest rules (CR 701.40).

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::run_with;
use crate::r_s11_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Unwanted Remake on P1's Grizzly Bears; P1 manifests dread with `manifested`
/// and `other` on top of their library, choosing `manifested`. Returns the manifested
/// permanent.
fn remake(t: &mut TestGame, manifested: &str, other: &str) -> ObjectId {
    supported("Unwanted Remake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cards = stack_library(t, P1, &[other, manifested]);
    let spell = in_hand_with_mana(t, P0, "Unwanted Remake");
    t.cast(P0, spell).target(bears).go();
    t.answer_choose(P1, &[Entity::Object(cards[1])]);
    t.resolve_all();
    let m = t.g.current(cards[1]);
    assert!(t.obj(m).face_down && t.on_battlefield(m) && t.obj(m).controller == P1);
    assert_eq!(t.zone(t.g.current(cards[0])), mtg_engine::object::Zone::Graveyard(P1));
    m
}

#[test]
fn a_manifested_creature_card_turns_face_up_any_time_its_controller_has_priority() {
    cr!("701.40b", "701.62a", "116.2b");
    ruling!(
        "Unwanted Remake",
        "Any time you have priority, you can turn a manifested permanent you control face up by revealing that it's a creature card (ignoring any copy effects or type-changing effects that might be applying to it) and paying its mana cost. This is a special action. It doesn't use the stack and can't be responded to."
    );
    let mut t = TestGame::new(2);
    let m = remake(&mut t, "Hill Giant", "Lightning Bolt");
    // A type-changing effect makes it a noncreature land.
    run_with(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetTypes {
                types: vec![CardType::Land],
                subtypes: vec![],
            }],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(m)],
    );
    assert!(!t.obj(m).chars.is(CardType::Creature));
    // During P0's turn, with P0's spell on the stack: P1 holds priority.
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    // Not by P0, who doesn't control it.
    t.lands(P0, "Mountain", 4);
    assert!(!can_turn_face_up(&mut t, P0, m));
    t.lands(P1, "Mountain", 3);
    assert!(!can_turn_face_up(&mut t, P1, m));
    t.lands(P1, "Mountain", 1);
    assert!(turn_face_up(&mut t, P1, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Hill Giant");
    // Nothing went on the stack: only the Bolt is there.
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn a_manifested_card_with_disguise_or_morph_turns_up_for_that_cost_too() {
    cr!("701.40c", "701.40d", "702.37e", "702.168d");
    ruling!(
        "Unwanted Remake",
        "If a manifested creature would have disguise or morph if it were face up, you may also turn it face up by paying its disguise or morph cost, as appropriate."
    );
    // Nightdrinker Moroii: {3}{B}, disguise {B}{B}.
    let mut t = TestGame::new(2);
    let m = remake(&mut t, "Nightdrinker Moroii", "Hill Giant");
    t.lands(P1, "Swamp", 2);
    assert!(turn_face_up(&mut t, P1, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Nightdrinker Moroii");
    // Sagu Mauler: {4}{G}{U}, morph {3}{G}{U}: with six lands P1 chooses.
    for (choice, tapped) in [(0, 6), (1, 5)] {
        let mut t = TestGame::new(2);
        let m = remake(&mut t, "Sagu Mauler", "Hill Giant");
        t.lands(P1, "Forest", 1);
        t.lands(P1, "Island", 1);
        t.lands(P1, "Wastes", 4);
        t.answer(P1, DecisionKind::Option, Answer::Index(choice));
        assert!(turn_face_up(&mut t, P1, m));
        assert_eq!(t.obj(m).chars.name.as_str(), "Sagu Mauler");
        assert_eq!(tapped_lands(&t, P1), tapped);
    }
}

#[test]
fn a_face_down_creature_that_lost_its_abilities_cant_use_its_morph_cost() {
    cr!("702.37e", "701.40b", "613.1f");
    ruling!(
        "Unwanted Remake",
        "If a face-down creature loses its abilities, it can't be turned face up with a disguise or morph ability because it will no longer have that ability (or the associated cost) once face up."
    );
    supported("Humility");
    let mut t = TestGame::new(2);
    let m = remake(&mut t, "Sagu Mauler", "Hill Giant");
    // Humility: "All creatures lose all abilities and have base power and toughness 1/1."
    t.battlefield(P0, "Humility");
    t.g.recompute();
    // Five lands would pay the morph cost {3}{G}{U}, but face up it would have no morph.
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Island", 1);
    t.lands(P1, "Wastes", 3);
    assert!(!can_turn_face_up(&mut t, P1, m));
    assert!(!turn_face_up(&mut t, P1, m));
    assert!(t.obj(m).face_down);
    // Its mana cost {4}{G}{U} still can be paid.
    t.lands(P1, "Wastes", 1);
    assert!(turn_face_up(&mut t, P1, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Sagu Mauler");
    assert_eq!(t.pt(m), (1, 1));
}

#[test]
fn a_manifested_creature_card_turns_up_after_losing_its_abilities_unlike_a_disguise() {
    cr!("701.40b", "702.168d", "613.1f");
    ruling!(
        "Unwanted Remake",
        "Unlike a face-down creature that was cast using a disguise or morph ability, a manifested creature may still be turned face up after it loses its abilities if it's a creature card."
    );
    let mut t = TestGame::new(2);
    let m = remake(&mut t, "Hill Giant", "Grizzly Bears");
    // On their turn, P1 casts Nightdrinker Moroii face down with disguise.
    t.advance_to(P1, Step::PrecombatMain);
    t.lands(P1, "Wastes", 3);
    let moroii = t.hand(P1, "Nightdrinker Moroii");
    let spell = t
        .cast(P1, moroii)
        .method(CastMethod::FaceDown(KeywordKind::Disguise))
        .go();
    t.resolve_all();
    let disguised = t.g.current(spell);
    assert!(t.obj(disguised).face_down);
    t.battlefield(P0, "Humility");
    t.g.recompute();
    t.lands(P1, "Swamp", 2);
    t.lands(P1, "Mountain", 4);
    assert!(!can_turn_face_up(&mut t, P1, disguised));
    assert!(can_turn_face_up(&mut t, P1, m));
    assert!(turn_face_up(&mut t, P1, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Hill Giant");
    assert!(t.obj(disguised).face_down);
}
