//! CR 702.168 Disguise.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::decision::{Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, ColorSet};
use mtg_engine::*;

const DISGUISE: CastMethod = CastMethod::FaceDown(KeywordKind::Disguise);
const MOROII: &str = "Nightdrinker Moroii";

/// Casts `card` face down with disguise, paying {3} with colorless mana.
fn cast_disguised(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    add_mana(t, p, ManaType::C, 3);
    t.cast(p, card).method(DISGUISE).go()
}

/// `name` from P0's hand, cast face down and resolved: the face-down permanent.
fn disguised(t: &mut TestGame, name: &str) -> ObjectId {
    let card = t.hand(P0, name);
    let spell = cast_disguised(t, P0, card);
    t.resolve_all();
    let id = t.g.current(spell);
    assert!(t.obj(id).face_down && t.on_battlefield(id));
    id
}

/// Whether the special action to turn `obj` face up is available to `p`.
fn can_turn_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    special_actions(t, p).contains(&SpecialAction::TurnFaceUp { obj })
}

#[test]
fn a_card_with_disguise_is_cast_face_down_as_a_2_2_with_ward_2_for_3() {
    cr!("702.168", "702.168a", "702.168b", "702.168c", "702.168f");
    assert_supported(MOROII);
    ruling!(
        "Nightdrinker Moroii",
        "The creature spell is a 2/2 creature spell with ward {2} that has no name, mana cost, or creature types. The resulting creature is a 2/2 creature with ward {2} that has no name, mana cost, or creature types. Both the spell and the resulting creature are colorless and have a mana value of 0."
    );
    // Nightdrinker Moroii: {3}{B} 4/2 flying, "When this creature enters, you lose 3
    // life.", disguise {B}{B}.
    let mut t = TestGame::new(2);
    let moroii = t.hand(P0, MOROII);
    // Cards can't normally be cast face down: only the one with disguise can.
    let bears = t.hand(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::C, 3);
    assert!(cast_methods_now(&mut t, P0, moroii).contains(&DISGUISE));
    assert!(!cast_methods_now(&mut t, P0, bears)
        .iter()
        .any(|m| matches!(m, CastMethod::FaceDown(_))));
    // {3} rather than its mana cost.
    let spell = t.cast(P0, moroii).method(DISGUISE).go();
    assert_eq!(pool(&t, P0), 0);
    let o = t.obj(spell);
    assert!(o.face_down);
    assert!(o.chars.name.is_empty());
    assert!(o.chars.subtypes.is_empty());
    assert!(o.chars.mana_cost.is_none());
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert!(o.chars.is(CardType::Creature));
    assert_eq!((o.chars.power, o.chars.toughness), (Some(2), Some(2)));
    assert!(o.chars.has_keyword(KeywordKind::Ward));
    assert!(!o.chars.has_keyword(KeywordKind::Flying));
    assert_eq!(t.g.mana_value_of(spell), 0);
    t.resolve_all();
    // It enters with the same characteristics; its enters ability doesn't exist face down.
    let id = t.g.current(spell);
    assert!(t.obj(id).face_down);
    assert_eq!(t.pt(id), (2, 2));
    assert!(t.obj(id).chars.has_keyword(KeywordKind::Ward));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn effects_apply_to_casting_it_as_the_face_down_spell() {
    cr!("702.168b");
    assert_supported("Concealed Weapon");
    // Concealed Weapon is an Equipment with disguise {2}{R}; Thalia's "Noncreature spells
    // cost {1} more to cast" doesn't apply to it cast face down (a creature spell).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let weapon = t.hand(P0, "Concealed Weapon");
    let spell = cast_disguised(&mut t, P0, weapon);
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    let id = t.g.current(spell);
    assert!(t.obj(id).is_creature());
    // Cast face up, it's a noncreature spell: {1}{R} + {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let weapon = t.hand(P0, "Concealed Weapon");
    add_mana(&mut t, P0, ManaType::R, 2);
    assert!(t.cast(P0, weapon).try_go().is_err());
}

#[test]
fn a_disguised_permanent_has_ward_2() {
    cr!("702.168a", "702.21a");
    let mut t = TestGame::new(2);
    let fd = disguised(&mut t, MOROII);
    // An opponent's spell targeting it is countered unless they pay {2}.
    let bolt = t.hand(P1, "Lightning Bolt");
    add_mana(&mut t, P1, ManaType::R, 1);
    t.cast(P1, bolt).target(fd).go();
    t.resolve_all();
    assert!(t.on_battlefield(fd));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn turning_it_face_up_is_a_special_action_for_its_disguise_cost() {
    cr!("702.168d");
    ruling!(
        "Nightdrinker Moroii",
        "Any time you have priority, you may turn the face-down creature face up by revealing what its disguise cost is and paying that cost. This is a special action. It doesn't use the stack and can't be responded to. Only a face-down permanent can be turned face up this way; a face-down spell cannot."
    );
    ruling!(
        "Nightdrinker Moroii",
        "Because the permanent is on the battlefield both before and after it's turned face up, turning a permanent face up doesn't cause any enters-the-battlefield abilities to trigger."
    );
    let mut t = TestGame::new(2);
    // A face-down spell can't be turned face up.
    let card = t.hand(P0, MOROII);
    let spell = cast_disguised(&mut t, P0, card);
    add_mana(&mut t, P0, ManaType::B, 2);
    assert!(!can_turn_up(&mut t, P0, spell));
    t.resolve_all();
    let fd = t.g.current(spell);
    // During the opponent's turn, with their spell on the stack.
    t.set_step(P1, Step::Upkeep);
    let bolt = t.hand(P1, "Lightning Bolt");
    add_mana(&mut t, P1, ManaType::R, 1);
    add_mana(&mut t, P1, ManaType::C, 2);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    // Only its controller can.
    assert!(!can_turn_up(&mut t, P1, fd));
    assert!(can_turn_up(&mut t, P0, fd));
    let stack = t.stack_len();
    take_special(&mut t, P0, SpecialAction::TurnFaceUp { obj: fd }).unwrap();
    assert_eq!(t.stack_len(), stack);
    assert_eq!(pool(&t, P0), 0);
    assert!(!t.obj(fd).face_down);
    assert_eq!(t.obj(fd).chars.name, MOROII);
    assert!(t.obj(fd).chars.has_keyword(KeywordKind::Flying));
    t.resolve_all();
    // Its enters ability didn't trigger: only the Bolt's 3 damage.
    assert_eq!(t.life(P0), 17);
}

#[test]
fn it_cant_be_turned_face_up_if_it_wouldnt_have_disguise_face_up() {
    cr!("702.168d");
    assert_supported("Humility");
    ruling!(
        "Nightdrinker Moroii",
        "If a face-down creature loses its abilities, it can't be turned face up with a disguise ability because it will no longer have a disguise ability (or a disguise cost) once face up."
    );
    let mut t = TestGame::new(2);
    let fd = disguised(&mut t, MOROII);
    add_mana(&mut t, P0, ManaType::B, 2);
    assert!(can_turn_up(&mut t, P0, fd));
    // Humility: "All creatures lose all abilities ..."
    t.battlefield(P1, "Humility");
    assert!(!can_turn_up(&mut t, P0, fd));
    assert!(take_special(&mut t, P0, SpecialAction::TurnFaceUp { obj: fd }).is_err());
    assert!(t.obj(fd).face_down);
}

#[test]
fn x_in_a_disguise_cost_is_the_x_other_abilities_refer_to() {
    cr!("702.168e");
    assert_supported("Aurelia's Vindicator");
    // Aurelia's Vindicator: disguise {X}{3}{W}, "When this creature is turned face up,
    // exile up to X other target creatures from the battlefield and/or creature cards from
    // graveyards." "When this creature leaves the battlefield, return the exiled cards to
    // their owners' hands."
    let mut t = TestGame::new(2);
    let fd = disguised(&mut t, "Aurelia's Vindicator");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.graveyard(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Llanowar Elves");
    // X = 2: {2}{3}{W}.
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 5);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[Entity::Object(giant), Entity::Object(bears)]);
    take_special(&mut t, P0, SpecialAction::TurnFaceUp { obj: fd }).unwrap();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.on_battlefield(other));
    // It leaves: the exiled cards go to their owners' hands.
    crate::common_k702_052_066::destroy(&mut t, fd);
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}
