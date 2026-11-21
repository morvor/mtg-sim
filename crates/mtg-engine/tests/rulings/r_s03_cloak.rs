//! Rulings batch S03 — cloak (CR 701.58): "To cloak a card, put it onto the battlefield
//! face down as a 2/2 creature with ward {2}. Turn it face up any time for its mana cost if
//! it's a creature card."

use crate::r_s01_common::*;
use crate::r_s03_common::*;
use mtg_engine::ability::{Effect, Sel};
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::ColorSet;
use mtg_engine::*;

/// `p` turns the face-down permanent `obj` face up (the special action). Whether it was
/// allowed.
fn turn_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let ok =
        t.g.perform_action(p, Action::Special(SpecialAction::TurnFaceUp { obj }))
            .is_ok();
    t.g.recompute();
    ok
}

/// Cryptic Coat enters under P0's control with `top` on top of P0's library: the card is
/// cloaked (and equipped). Returns the cloaked permanent.
fn cloak_with_coat(t: &mut TestGame, top: &str) -> ObjectId {
    supported("Cryptic Coat");
    let card = t.library_top(P0, top);
    t.enter(P0, "Cryptic Coat");
    t.resolve_all();
    let c = t.g.current(card);
    assert!(t.obj(c).face_down && t.on_battlefield(c));
    c
}

/// P0 activates Ransom Note's "{2}, Sacrifice this artifact: Choose one — • Cloak the top
/// card of your library. ..." with `top` on top of P0's library. Returns the cloaked
/// permanent.
fn cloak_with_ransom_note(t: &mut TestGame, top: &str) -> ObjectId {
    supported("Ransom Note");
    let note = t.battlefield(P0, "Ransom Note");
    let card = t.library_top(P0, top);
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.activate(P0, note, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ransom Note"));
    let c = t.g.current(card);
    assert!(t.obj(c).face_down && t.on_battlefield(c));
    c
}

#[test]
fn a_cloaked_double_faced_card_cant_transform_and_turns_face_up_front_face_up() {
    cr!("701.58a", "701.58b", "712.15", "712.15a", "701.27a");
    ruling!(
        "Cryptic Coat",
        "If a double-faced card is cloaked, it will be put onto the battlefield face down. While face down, it can't transform. If the front face of the card is a creature card, you can turn it face up by paying its mana cost. If you do, its front face will be up."
    );
    ruling!(
        "Ransom Note",
        "If a double-faced card is cloaked, it will be put onto the battlefield face down. While face down, it can’t transform. If the front face of the card is a creature card, you can turn it face up by paying its mana cost. If you do, its front face will be up."
    );
    supported("Kruin Outlaw // Terror of Kruin Pass");
    // Kruin Outlaw ({1}{R}{R}): "At the beginning of each upkeep, if no spells were cast
    // last turn, transform this creature." // Terror of Kruin Pass.
    let mut t = TestGame::new(2);
    let c = cloak_with_coat(&mut t, "Kruin Outlaw // Terror of Kruin Pass");
    // An effect instructing to transform it does nothing.
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Transform {
            what: Sel::Target(0),
        },
        &[Entity::Object(c)],
    );
    assert!(t.obj(c).face_down);
    assert_eq!(t.obj(c).face, FaceState::Front);
    assert_eq!(t.pt(c), (3, 2)); // a 2/2 equipped with the Coat (+1/+0)
    t.lands(P0, "Mountain", 3);
    assert!(turn_up(&mut t, P0, c));
    assert!(!t.obj(c).face_down);
    assert_eq!(t.obj(c).face, FaceState::Front);
    assert_eq!(t.obj(c).chars.name, "Kruin Outlaw");
    assert_eq!(tapped_lands(&t, P0), 3);

    // Cloaked by Ransom Note, it has no werewolf trigger while face down: after a turn
    // with no spells, it doesn't transform in the upkeep.
    let mut t = TestGame::new(2);
    let c = cloak_with_ransom_note(&mut t, "Kruin Outlaw // Terror of Kruin Pass");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.obj(c).face_down);
    assert_eq!(t.obj(c).face, FaceState::Front);
    t.lands(P0, "Mountain", 3);
    assert!(turn_up(&mut t, P0, c));
    assert_eq!(t.obj(c).chars.name, "Kruin Outlaw");
    assert_eq!(t.obj(c).face, FaceState::Front);
    // Face up, it's a werewolf that transforms normally.
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj(c).face, FaceState::Back);
    assert_eq!(t.obj(c).chars.name, "Terror of Kruin Pass");
}

#[test]
fn a_cloaked_creature_card_that_lost_its_abilities_can_still_be_turned_face_up() {
    cr!("701.58b", "116.2b", "613.1f");
    ruling!(
        "Cryptic Coat",
        "Unlike a face-down creature that was cast using a disguise or morph ability, a cloaked creature may still be turned face up after it loses its abilities if it's a creature card."
    );
    ruling!(
        "Ransom Note",
        "Unlike a face-down creature that was cast using a disguise or morph ability, a cloaked creature may still be turned face up after it loses its abilities if it’s a creature card."
    );
    supported("Humility");
    for with_note in [false, true] {
        let mut t = TestGame::new(2);
        let c = if with_note {
            cloak_with_ransom_note(&mut t, "Hill Giant")
        } else {
            cloak_with_coat(&mut t, "Hill Giant")
        };
        assert!(t.obj(c).has_keyword(KeywordKind::Ward));
        // Humility: "All creatures lose all abilities and have base power and toughness
        // 1/1." The cloaked permanent loses ward {2}.
        t.battlefield(P1, "Humility");
        t.g.recompute();
        assert!(!t.obj(c).has_keyword(KeywordKind::Ward));
        assert!(t.obj(c).chars.abilities.is_empty());
        let tapped = tapped_lands(&t, P0);
        t.lands(P0, "Mountain", 4);
        assert!(turn_up(&mut t, P0, c));
        assert!(!t.obj(c).face_down);
        assert_eq!(t.obj(c).chars.name, "Hill Giant");
        assert_eq!(tapped_lands(&t, P0), tapped + 4);
    }
}

#[test]
fn a_cloaked_card_is_a_nameless_colorless_2_2_with_ward_that_effects_can_change() {
    cr!("701.58a", "708.2", "202.3a", "613.1d", "613.4c");
    ruling!(
        "Ransom Note",
        "To cloak a card, put it onto the battlefield face down. It becomes a 2/2 face-down creature card with ward {2} and no name, mana cost, or creature types. It’s colorless and has a mana value of 0. Other effects that apply to the permanent can still grant it any characteristics it doesn’t have or change the characteristics it does have."
    );
    supported("Glorious Anthem");
    supported("Kenrith's Transformation");
    // Hill Giant: a red {3}{R} Giant.
    let mut t = TestGame::new(2);
    let c = cloak_with_ransom_note(&mut t, "Hill Giant");
    let o = t.obj(c);
    assert!(o.is_card() && o.is_creature());
    assert_eq!((o.power(), o.toughness()), (2, 2));
    assert!(o.chars.name.is_empty());
    assert!(o.chars.mana_cost.is_none());
    assert!(o.chars.subtypes.is_empty());
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert_eq!(t.g.mana_value_of(c), 0);
    let ward = o.chars.keyword(KeywordKind::Ward).unwrap();
    assert_eq!(
        format!("{:?}", ward.cost.as_ref().unwrap().mana),
        format!("{:?}", mtg_engine::mana::ManaCost::parse("{2}"))
    );
    // An anthem changes its power and toughness.
    t.battlefield(P0, "Glorious Anthem");
    t.g.recompute();
    assert_eq!(t.pt(c), (3, 3));
    // An Aura makes it a green Elk with base power and toughness 3/3 (and it loses ward).
    let aura = in_hand_with_mana(&mut t, P0, "Kenrith's Transformation");
    t.cast(P0, aura).target(c).go();
    t.resolve_all();
    let o = t.obj(c);
    assert!(o.face_down);
    assert!(o.chars.has_subtype("Elk"));
    assert_eq!(
        o.chars.colors,
        ColorSet::single(mtg_engine::types::Color::Green)
    );
    assert_eq!(t.pt(c), (4, 4));
    assert!(!o.has_keyword(KeywordKind::Ward));
    assert!(o.chars.name.is_empty());
}

#[test]
fn turning_a_cloaked_permanent_face_up_is_a_special_action_that_cant_be_responded_to() {
    cr!("701.58b", "116.2b", "116.1");
    ruling!(
        "Ransom Note",
        "Any time you have priority, you can turn a cloaked permanent you control face-up by revealing that it’s a creature card (ignoring any copy effects or type-changing effects that might be applying to it) and paying its mana cost. This is a special action. It doesn’t use the stack and can’t be responded to."
    );
    // During P1's turn, P1 casts Shock at the cloaked 2/2; in response P0 turns it face
    // up: nothing goes on the stack, and the Hill Giant (3/3) survives the Shock.
    let mut t = TestGame::new(2);
    let c = cloak_with_ransom_note(&mut t, "Hill Giant");
    t.advance_to(P1, Step::PrecombatMain);
    let shock = in_hand_with_mana(&mut t, P1, "Shock");
    t.lands(P1, "Wastes", 2); // ward {2}
    t.cast(P1, shock).target(c).go();
    t.settle();
    t.answer_yes(P1, true); // pay for ward
    t.resolve(); // the ward trigger
    assert_eq!(t.stack_len(), 1);
    t.lands(P0, "Mountain", 4);
    assert!(turn_up(&mut t, P0, c));
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.obj(c).chars.name, "Hill Giant");
    t.resolve_all();
    assert!(t.on_battlefield(c));
    assert_eq!(t.obj(c).damage, 2);

    // Only a creature card can be turned face up this way: an Aura that makes the
    // cloaked Hill Giant a Treefolk doesn't matter, the card's a creature card, while a
    // cloaked noncreature card (a creature while face down) can't be.
    supported("Lignify");
    let mut t = TestGame::new(2);
    let c = cloak_with_ransom_note(&mut t, "Hill Giant");
    let aura = in_hand_with_mana(&mut t, P0, "Lignify");
    t.cast(P0, aura).target(c).go();
    t.resolve_all();
    assert!(t.obj(c).chars.has_subtype("Treefolk"));
    t.lands(P0, "Mountain", 4);
    assert!(turn_up(&mut t, P0, c));
    assert_eq!(t.obj(c).chars.name, "Hill Giant");
    let mut t = TestGame::new(2);
    let c = cloak_with_ransom_note(&mut t, "Lightning Bolt");
    assert!(t.obj(c).is_creature());
    let tapped = tapped_lands(&t, P0);
    t.lands(P0, "Mountain", 4);
    assert!(!turn_up(&mut t, P0, c));
    assert!(t.obj(c).face_down);
    assert_eq!(tapped_lands(&t, P0), tapped);
    assert_eq!(t.zone(c), Zone::Battlefield);
}
