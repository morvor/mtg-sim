//! CR 702.168 Disguise.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::ability::Duration;
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
    ruling!(
        "Arno Dorian",
        "The creature spell is a 2/2 creature spell with ward {2} that has no name, mana cost, or creature types. The resulting creature is a 2/2 creature with ward {2} that has no name, mana cost, or creature types. Both the spell and the resulting creature are colorless and have a mana value of 0."
    );
    ruling!(
        "Alley Assailant",
        "The face-down spell has no mana cost and a mana value of 0. When you cast a face-down spell, put it on the stack face down so no other player knows what it is, and pay {3} to cast it. This is an alternative cost."
    );
    ruling!(
        "Arno Dorian",
        "The face-down spell has no mana cost and a mana value of 0. When you cast a face-down spell, put it on the stack face down so no other player knows what it is, and pay {3} to cast it. This is an alternative cost."
    );
    ruling!(
        "Arno Dorian",
        "At any time, you can look at a face-down spell or permanent you control. You can't look at face-down permanents or spells you don't control unless an effect instructs or allows you to do so."
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
    // Only its controller may look at it.
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, spell));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, spell));
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
fn casting_it_face_down_is_paying_an_alternative_cost() {
    cr!("702.168b");
    assert_supported("Sphere of Resistance");
    // Sphere of Resistance: "Spells cost {1} more to cast." It applies to the {3}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    let moroii = t.hand(P0, MOROII);
    add_mana(&mut t, P0, ManaType::C, 3);
    assert!(t.cast(P0, moroii).method(DISGUISE).try_go().is_err());
    add_mana(&mut t, P0, ManaType::C, 1);
    let spell = t.cast(P0, moroii).method(DISGUISE).go();
    assert_eq!(pool(&t, P0), 0);
    assert!(t.obj(spell).face_down);
    // An effect that lets it be cast without paying its mana cost can't be combined with
    // the disguise cost: cast that way, it's cast face up.
    let mut t = TestGame::new(2);
    let moroii = t.hand(P0, MOROII);
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![moroii],
        Duration::EndOfTurn,
        true,
        None,
    );
    let spell = t.cast(P0, moroii).method(CastMethod::Free).go();
    assert!(!t.obj(spell).face_down);
    assert_eq!(t.obj(spell).chars.name, MOROII);
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
    ruling!(
        "Arno Dorian",
        "Any time you have priority, you may turn the face-down creature face up by revealing what its disguise cost is and paying that cost. This is a special action. It doesn't use the stack and can't be responded to. Only a face-down permanent can be turned face up this way; a face-down spell cannot."
    );
    ruling!(
        "Arno Dorian",
        "Because the permanent is on the battlefield both before and after it's turned face up, turning a permanent face up doesn't cause any enters-the-battlefield abilities to trigger."
    );
    ruling!(
        "Alley Assailant",
        "A disguise ability lets you cast a card face down by paying {3} and announcing that you are using a disguise ability. Any time you have priority, you can turn a face-down permanent with disguise face up by paying its disguise cost."
    );
    ruling!(
        "Arno Dorian",
        "A disguise ability lets you cast a card face down by paying {3} and announcing that you are using a disguise ability. Any time you have priority, you can turn a face-down permanent you control face up by paying its disguise cost."
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
    ruling!(
        "Arno Dorian",
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

#[test]
fn a_disguise_cost_can_be_reduced() {
    cr!("702.168d");
    assert_supported("Fugitive Codebreaker");
    // Fugitive Codebreaker: "Disguise {5}{R}. This cost is reduced by {1} for each instant
    // and sorcery card in your graveyard." "When this creature is turned face up, discard
    // your hand, then draw three cards."
    let mut t = TestGame::new(2);
    let fd = disguised(&mut t, "Fugitive Codebreaker");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Divination");
    t.graveyard(P0, "Grizzly Bears");
    // {5}{R} reduced by {2}: {3}{R}.
    add_mana(&mut t, P0, ManaType::R, 3);
    assert!(!can_turn_up(&mut t, P0, fd));
    add_mana(&mut t, P0, ManaType::R, 1);
    assert!(can_turn_up(&mut t, P0, fd));
    take_special(&mut t, P0, SpecialAction::TurnFaceUp { obj: fd }).unwrap();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.obj(fd).chars.name, "Fugitive Codebreaker");
}

#[test]
fn turning_it_face_up_leaves_it_the_same_permanent() {
    cr!("702.168d", "708.8");
    assert_supported("Bonesplitter");
    ruling!(
        "Alley Assailant",
        "Turning a permanent face up or face down doesn't change whether that permanent is tapped or untapped."
    );
    ruling!(
        "Arno Dorian",
        "Turning a permanent face up or face down doesn't change whether that permanent is tapped or untapped."
    );
    ruling!(
        "Alley Assailant",
        "A permanent that turns face up or face down changes characteristics but is otherwise the same permanent. Spells and abilities that were targeting that permanent and Auras and Equipment that were attached to that permanent aren't affected unless the new characteristics of the object change the legality of those targets or attachments."
    );
    ruling!(
        "Arno Dorian",
        "A permanent that turns face up or face down changes characteristics but is otherwise the same permanent. Spells and abilities that were targeting that permanent and Auras and Equipment that were attached to that permanent aren't affected unless the new characteristics of the object change the legality of those targets or attachments."
    );
    let mut t = TestGame::new(2);
    let fd = disguised(&mut t, MOROII);
    // Bonesplitter ("Equipped creature gets +2/+0.") is attached to it, and it's tapped.
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(splitter, Entity::Object(fd)));
    t.g.tap(fd);
    t.g.recompute();
    assert_eq!(t.pt(fd), (4, 2));
    // Giant Growth targets it; in response, it's turned face up.
    let growth = t.hand(P0, "Giant Growth");
    add_mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, growth).target(fd).go();
    add_mana(&mut t, P0, ManaType::B, 2);
    take_special(&mut t, P0, SpecialAction::TurnFaceUp { obj: fd }).unwrap();
    t.resolve_all();
    // Still the same permanent: tapped, equipped, and the spell's target: 4/2 + 2/0 + 3/3.
    assert!(t.on_battlefield(fd));
    assert_eq!(t.obj(fd).chars.name, MOROII);
    assert!(t.obj(fd).tapped);
    assert_eq!(t.g.attachments_of(Entity::Object(fd)), vec![splitter]);
    assert_eq!(t.pt(fd), (9, 5));
    assert!(t.in_graveyard(P0, "Giant Growth"));
}

#[test]
fn face_down_permanents_have_no_name() {
    cr!("702.168a");
    assert_supported("Meddling Mage");
    ruling!(
        "Alley Assailant",
        "Because face-down creatures don't have a name, they can't have the same name as any other creature, even another face-down creature."
    );
    ruling!(
        "Arno Dorian",
        "Because face-down creatures don't have a name, they can't have the same name as any other creature, even another face-down creature."
    );
    let mut t = TestGame::new(2);
    let a = disguised(&mut t, "Arno Dorian");
    let b = disguised(&mut t, "Arno Dorian");
    let c = disguised(&mut t, MOROII);
    // Neither shares a name with another face-down creature, nor with itself.
    for (x, y) in [(a, b), (a, c), (a, a)] {
        assert!(!t.obj(x).chars.shares_name_with(&t.obj(y).chars));
    }
    // Two face-down legendary cards: the "legend rule" doesn't apply to them.
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    // Meddling Mage naming Nightdrinker Moroii: it can't be cast face up, but a face-down
    // spell has no name.
    t.answer(P1, DecisionKind::Name, Answer::Text(MOROII.into()));
    t.enter(P1, "Meddling Mage");
    t.resolve_all();
    let moroii = t.hand(P0, MOROII);
    add_mana(&mut t, P0, ManaType::B, 4);
    assert!(t.cast(P0, moroii).try_go().is_err());
    add_mana(&mut t, P0, ManaType::C, 3);
    let spell = t.cast(P0, moroii).method(DISGUISE).go();
    assert!(t.obj(spell).face_down);
}
