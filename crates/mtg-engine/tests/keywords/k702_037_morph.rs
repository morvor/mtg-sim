//! CR 702.37 Morph and megamorph.

use crate::common_k702_011_017::*;
use crate::common_k702_027_037::*;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType, ColorSet};
use mtg_engine::*;

const FACE_DOWN: CastMethod = CastMethod::FaceDown(KeywordKind::Morph);

fn turn_up(id: ObjectId) -> Action {
    Action::Special(SpecialAction::TurnFaceUp { obj: id })
}

/// Whether `p` could turn `id` face up now.
fn can_turn_up(t: &mut TestGame, p: PlayerId, id: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let id = t.g.current(id);
    t.g.legal_actions(p).contains(&turn_up(id))
}

/// Casts `card` face down for {3} and resolves it; returns the face-down permanent.
fn cast_face_down(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    t.cast(p, card).method(FACE_DOWN).go();
    t.resolve();
    let id = t.g.current(card);
    assert!(t.g.obj(id).face_down && t.on_battlefield(id));
    id
}

#[test]
fn morph_casts_the_card_face_down_as_a_nameless_2_2_for_three() {
    cr!("702.37", "702.37a", "702.37c");
    ruling!(
        "Ruthless Ripper",
        "The face-down spell has no mana cost and has a mana value of 0. When you cast a face-down spell, put it on the stack face down so no other player knows what it is, and pay {3}. This is an alternative cost."
    );
    ruling!(
        "Ruthless Ripper",
        "When the spell resolves, it enters the battlefield as a 2/2 creature with no name, mana cost, creature types, or abilities. It's colorless and has a mana value of 0."
    );
    assert_supported("Scornful Egotist");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let egotist = t.hand(P0, "Scornful Egotist");
    assert!(can_cast(&mut t, P0, egotist, FACE_DOWN));
    let spell = t.cast(P0, egotist).method(FACE_DOWN).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    let o = t.obj_now(spell);
    assert!(o.face_down);
    assert_eq!(o.chars.name.as_str(), "");
    assert!(o.chars.is(CardType::Creature));
    assert!(o.chars.subtypes.is_empty());
    assert_eq!(o.chars.mana_value(), 0);
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert_eq!((o.power(), o.toughness()), (2, 2));
    t.resolve();
    // It enters with the characteristics the spell had.
    let o = t.obj_now(egotist);
    assert!(o.face_down && o.zone == Zone::Battlefield);
    assert_eq!(o.chars.name.as_str(), "");
    assert_eq!(t.pt(egotist), (2, 2));
    assert!(o.chars.abilities.is_empty());
}

#[test]
fn a_card_without_morph_cant_be_cast_face_down() {
    cr!("702.37d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!can_cast(&mut t, P0, bears, FACE_DOWN));
    assert!(t.cast(P0, bears).method(FACE_DOWN).try_go().is_err());
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn casting_face_down_uses_the_face_down_spells_characteristics() {
    cr!("702.37c");
    assert_supported("Meddling Mage");
    // Spells named Scornful Egotist can't be cast, but a face-down spell has no name.
    let mut t = TestGame::new(2);
    t.answer(
        P1,
        DecisionKind::Name,
        Answer::Text("Scornful Egotist".into()),
    );
    t.enter(P1, "Meddling Mage");
    t.lands(P0, "Island", 8);
    let egotist = t.hand(P0, "Scornful Egotist");
    assert!(!can_cast(&mut t, P0, egotist, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, egotist, FACE_DOWN));
    cast_face_down(&mut t, P0, egotist);
    // A face-down spell is a creature spell: cast only at sorcery speed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let egotist = t.hand(P0, "Scornful Egotist");
    t.set_step(P0, Step::Upkeep);
    assert!(!can_cast(&mut t, P0, egotist, FACE_DOWN));
}

#[test]
fn a_land_with_morph_can_be_cast_face_down() {
    cr!("702.37c", "702.37e");
    assert_supported("Zoetic Cavern");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let cavern = t.hand(P0, "Zoetic Cavern");
    // A land can't be cast (CR 305.9), but the face-down spell is a creature spell.
    assert!(!can_cast(&mut t, P0, cavern, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, cavern, FACE_DOWN));
    let cavern = cast_face_down(&mut t, P0, cavern);
    assert!(t.obj_now(cavern).is_creature());
    // Turned face up for {2}: a land again.
    assert!(can_turn_up(&mut t, P0, cavern));
    t.g.perform_action(P0, turn_up(cavern)).unwrap();
    assert!(!t.obj_now(cavern).face_down);
    assert!(t.obj_now(cavern).chars.is_land());
    assert!(!t.obj_now(cavern).is_creature());
}

#[test]
fn morph_casts_from_any_zone_the_card_could_be_cast_from() {
    cr!("702.37c");
    let mut t = TestGame::new(2);
    // "You may cast artifact spells and colorless spells from the top of your library":
    // a blue card can't be cast from there, but the face-down spell is colorless.
    t.battlefield(P0, "Mystic Forge");
    t.lands(P0, "Island", 8);
    let egotist = t.library_top(P0, "Scornful Egotist");
    assert!(!can_cast(&mut t, P0, egotist, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, egotist, FACE_DOWN));
    let id = cast_face_down(&mut t, P0, egotist);
    assert_eq!(t.library_size(P0), 30);
    assert!(t.obj_now(id).face_down);
}

#[test]
fn turning_a_morph_face_up_is_a_special_action_for_its_morph_cost() {
    cr!("702.37e");
    ruling!(
        "Ruthless Ripper",
        "Any time you have priority, you may turn the face-down creature face up by revealing what its morph cost is and paying that cost. This is a special action. It doesn't use the stack and can't be responded to. Only a face-down permanent can be turned face up this way; a face-down spell cannot."
    );
    ruling!(
        "Ruthless Ripper",
        "Because the permanent is on the battlefield both before and after it's turned face up, turning a permanent face up doesn't cause any enters-the-battlefield abilities to trigger."
    );
    assert_supported("Soul Warden");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    t.lands(P0, "Island", 4);
    let egotist = t.hand(P0, "Scornful Egotist");
    let spell = t.cast(P0, egotist).method(FACE_DOWN).go();
    // A face-down spell can't be turned face up.
    assert!(!can_turn_up(&mut t, P0, spell));
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    let id = t.g.current(egotist);
    // Any time P0 has priority, even with a spell on the stack.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P1).go();
    assert!(can_turn_up(&mut t, P0, id));
    assert!(!can_turn_up(&mut t, P1, id));
    t.g.perform_action(P0, turn_up(id)).unwrap();
    // It didn't use the stack; the {U} morph cost was paid.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(untapped_lands(&t, P0), 0);
    let o = t.obj_now(id);
    assert!(!o.face_down);
    assert_eq!(o.chars.name.as_str(), "Scornful Egotist");
    assert_eq!(t.pt(id), (1, 1));
    // Soul Warden didn't trigger again: the permanent didn't enter.
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn morph_costs_can_be_non_mana_costs() {
    cr!("702.37e");
    assert_supported("Zombie Cutthroat");
    assert_supported("Ruthless Ripper");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    let cutthroat = t.hand(P0, "Zombie Cutthroat");
    let cutthroat = cast_face_down(&mut t, P0, cutthroat);
    t.g.perform_action(P0, turn_up(cutthroat)).unwrap();
    assert_eq!(t.life(P0), 15);
    assert_eq!(t.obj_now(cutthroat).chars.name.as_str(), "Zombie Cutthroat");
    // "Morph—Reveal a black card in your hand."
    let ripper = t.hand(P0, "Ruthless Ripper");
    let ripper = cast_face_down(&mut t, P0, ripper);
    // No black card in hand: it can't be turned face up.
    assert!(!can_turn_up(&mut t, P0, ripper));
    let black = t.hand(P0, "Walking Corpse");
    t.answer_choose(P0, &[Entity::Object(black)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.perform_action(P0, turn_up(ripper)).unwrap();
    assert!(t.in_hand(P0, "Walking Corpse"));
    // "When ~ is turned face up, target player loses 2 life."
    t.resolve();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_permanent_without_a_morph_cost_face_up_cant_be_turned_face_up_this_way() {
    cr!("702.37e");
    ruling!(
        "Zoetic Cavern",
        "If Blood Moon is on the battlefield and a player controls a face-down Zoetic Cavern, it can't be turned face up since it won't have a morph cost."
    );
    assert_supported("Blood Moon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let cavern = t.hand(P0, "Zoetic Cavern");
    let cavern = cast_face_down(&mut t, P0, cavern);
    t.battlefield(P1, "Blood Moon");
    // Face down it's a 2/2 creature, not a nonbasic land: Blood Moon doesn't affect it.
    assert!(t.obj_now(cavern).is_creature());
    assert!(!can_turn_up(&mut t, P0, cavern));
    assert!(t.g.perform_action(P0, turn_up(cavern)).is_err());
    assert!(t.obj_now(cavern).face_down);
}

#[test]
fn megamorph_puts_a_counter_on_it_as_its_turned_face_up() {
    cr!("702.37b");
    ruling!(
        "Marsh Hulk",
        "Turning a face-down creature with megamorph face up and putting a +1/+1 counter on it is a special action."
    );
    assert_supported("Marsh Hulk");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 10);
    let hulk = t.hand(P0, "Marsh Hulk");
    assert!(can_cast(&mut t, P0, hulk, FACE_DOWN));
    let hulk = cast_face_down(&mut t, P0, hulk);
    assert_eq!(untapped_lands(&t, P0), 7);
    t.g.perform_action(P0, turn_up(hulk)).unwrap();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.counters(hulk, counters::PLUS1), 1);
    assert_eq!(t.pt(hulk), (5, 7));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn x_in_a_morph_cost_is_the_x_of_its_other_abilities() {
    cr!("702.37f");
    ruling!(
        "Warbreak Trumpeter",
        "The X in the ability has the same value as the X paid in the Morph ability."
    );
    assert_supported("Warbreak Trumpeter");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    let trumpeter = t.hand(P0, "Warbreak Trumpeter");
    let trumpeter = cast_face_down(&mut t, P0, trumpeter);
    // X = 2: {2}{2}{R}.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.g.perform_action(P0, turn_up(trumpeter)).unwrap();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(tokens(&t, P0), 2);
}

#[test]
fn effects_can_modify_morph_costs_but_not_the_face_down_casting_cost() {
    cr!("702.37a", "702.37e");
    assert_supported("Exiled Doomsayer");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Exiled Doomsayer");
    t.lands(P0, "Island", 5);
    let egotist = t.hand(P0, "Scornful Egotist");
    // Still {3} to cast face down.
    let egotist = cast_face_down(&mut t, P0, egotist);
    assert_eq!(untapped_lands(&t, P0), 2);
    // Morph {U} costs {2}{U}.
    assert!(!can_turn_up(&mut t, P0, egotist));
    t.lands(P0, "Island", 1);
    assert!(can_turn_up(&mut t, P0, egotist));
    t.g.perform_action(P0, turn_up(egotist)).unwrap();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(!t.obj_now(egotist).face_down);
}
