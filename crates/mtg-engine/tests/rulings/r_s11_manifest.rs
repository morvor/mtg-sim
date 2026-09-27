//! Rulings batch S11 — manifest (CR 701.40) and face-down permanents (CR 708): Reality
//! Shift, Guardian of the Forgotten, Mastery of the Unseen, Unwanted Remake, and the
//! "Forms" of Fate Reforged (Cloudform: "When this enchantment enters, it becomes an Aura
//! with enchant creature. Manifest the top card of your library and attach this
//! enchantment to it.").

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::run_with;
use crate::r_s05_common::colors;
use crate::r_s11_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::facedown;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 controls Grizzly Bears; P0 casts Reality Shift on it, so P1 manifests the top card
/// of their library (`top`). Returns the manifested permanent.
fn reality_shift(t: &mut TestGame, top: &str) -> ObjectId {
    supported("Reality Shift");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.library_top(P1, top);
    let shift = in_hand_with_mana(t, P0, "Reality Shift");
    t.cast(P0, shift).target(bears).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    let m = t.g.current(card);
    assert!(t.obj(m).face_down && t.on_battlefield(m) && t.obj(m).controller == P1);
    m
}

/// P0 casts Cloudform with `top` (if any) on top of their library (otherwise with an empty
/// library). Returns Cloudform and the manifested permanent.
fn cloudform(t: &mut TestGame, top: Option<&str>) -> (ObjectId, Option<ObjectId>) {
    supported("Cloudform");
    let card = match top {
        Some(name) => Some(t.library_top(P0, name)),
        None => {
            empty_library(t, P0);
            None
        }
    };
    let form = in_hand_with_mana(t, P0, "Cloudform");
    t.cast(P0, form).go();
    t.resolve_all();
    let form = t.g.current(form);
    (form, card.map(|c| t.g.current(c)))
}

/// P0's Guardian of the Forgotten ("Whenever a modified creature you control dies,
/// manifest the top card of your library.") manifests `top`: a Grizzly Bears with a +1/+1
/// counter dies. Returns the manifested permanent.
fn guardian_manifests(t: &mut TestGame, top: &str) -> ObjectId {
    supported("Guardian of the Forgotten");
    if t.named_on_battlefield("Guardian of the Forgotten").is_empty() {
        t.battlefield(P0, "Guardian of the Forgotten");
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 1, None);
    let card = t.library_top(P0, top);
    destroy(t, bears);
    t.resolve_all();
    let m = t.g.current(card);
    assert!(t.obj(m).face_down && t.on_battlefield(m));
    m
}

fn transform(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    run_with(
        t,
        p,
        Effect::Transform {
            what: Sel::Target(0),
        },
        &[Entity::Object(id)],
    );
}

fn turn_face_down(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    run_with(
        t,
        p,
        Effect::TurnFaceDown {
            what: Sel::Target(0),
        },
        &[Entity::Object(id)],
    );
}

/// An effect (not the special action) tries to turn `id` face up.
fn effect_turns_face_up(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    run_with(
        t,
        p,
        Effect::TurnFaceUp {
            what: Sel::Target(0),
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn a_manifested_double_faced_card_cant_transform_and_turns_up_front_face_up() {
    cr!("701.40a", "701.40b", "712.15", "712.15a");
    ruling!(
        "Reality Shift",
        "If a double-faced card is manifested, it will be put onto the battlefield face down. While face down, it can't transform. If the front face of the card is a creature card, you can turn it face up by paying its mana cost. If you do, its front face will be up."
    );
    supported("Gatstaf Shepherd // Gatstaf Howler");
    let mut t = TestGame::new(2);
    let m = reality_shift(&mut t, "Gatstaf Shepherd // Gatstaf Howler");
    assert!(is_plain_face_down_2_2(&t, m));
    // An effect trying to transform it does nothing.
    transform(&mut t, P0, m);
    assert!(is_plain_face_down_2_2(&t, m));
    assert_eq!(t.obj(m).face, FaceState::Front);
    // Gatstaf Shepherd's mana cost is {1}{G}.
    t.lands(P1, "Forest", 1);
    assert!(!can_turn_face_up(&mut t, P1, m));
    t.lands(P1, "Wastes", 1);
    assert!(turn_face_up(&mut t, P1, m));
    let o = t.obj(m);
    assert!(!o.face_down);
    assert_eq!(o.face, FaceState::Front);
    assert_eq!(o.chars.name.as_str(), "Gatstaf Shepherd");
    assert_eq!(t.pt(m), (2, 2));
}

#[test]
fn a_manifested_double_faced_card_on_the_battlefield_cant_be_turned_face_down_again() {
    cr!("712.15a", "712.16", "708.2b");
    ruling!(
        "Cloudform",
        "Some previous Magic sets feature double-faced cards, which have a Magic card face on each side rather than a Magic card face on one side and a Magic card back on the other. If a double-faced card is manifested, it will be put onto the battlefield face down. While face down, it can't transform. If the front face of the card is a creature card, you can turn it face up by paying its mana cost. If you do, its front face will be up. Although a double-faced card can enter the battlefield face down, one already on the battlefield can't be turned face down."
    );
    let mut t = TestGame::new(2);
    let (form, m) = cloudform(&mut t, Some("Gatstaf Shepherd // Gatstaf Howler"));
    let m = m.unwrap();
    assert!(t.obj(m).face_down);
    assert_eq!(t.obj(form).attached_to, Some(Entity::Object(m)));
    transform(&mut t, P0, m);
    assert!(t.obj(m).face_down && t.obj(m).face == FaceState::Front);
    t.lands(P0, "Forest", 2);
    assert!(turn_face_up(&mut t, P0, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Gatstaf Shepherd");
    // Face up now, it can't be turned face down again.
    turn_face_down(&mut t, P0, m);
    assert!(!t.obj(m).face_down);
    assert_eq!(t.obj(m).chars.name.as_str(), "Gatstaf Shepherd");
    // A single-faced card can.
    let bears = t.battlefield(P0, "Grizzly Bears");
    turn_face_down(&mut t, P0, bears);
    assert!(t.obj(bears).face_down);
}

/// Checks that turning the manifested instant `m` (controlled by `p`, who also controls
/// Mastery of the Unseen) face up only reveals it.
fn instant_stays_face_down(t: &mut TestGame, p: PlayerId, m: ObjectId) {
    // Mastery of the Unseen: "Whenever a permanent you control is turned face up, you
    // gain 1 life for each creature you control."
    t.battlefield(p, "Mastery of the Unseen");
    let life = t.life(p);
    t.lands(p, "Mountain", 1);
    // Not a creature card: the special action isn't available.
    assert!(!can_turn_face_up(t, p, m));
    effect_turns_face_up(t, p, m);
    t.resolve_all();
    assert!(t.obj(m).face_down && t.on_battlefield(m));
    assert!(t.obj(m).chars.name.is_empty());
    assert_eq!(t.pt(m), (2, 2));
    assert_eq!(revealed_this_turn(t), vec![m]);
    assert!(!turned_face_up_this_turn(t));
    assert_eq!(t.life(p), life);
}

#[test]
fn a_manifested_instant_that_would_turn_face_up_is_revealed_and_stays_face_down() {
    cr!("701.40g", "708.7");
    ruling!(
        "Unwanted Remake",
        "If something tries to turn a face-down instant or sorcery card on the battlefield face up, reveal that card to show all players it's an instant or sorcery card. The permanent remains on the battlefield face down. Abilities that trigger when a permanent turns face up won't trigger, because even though you revealed the card, it never turned face up."
    );
    supported("Unwanted Remake");
    // Unwanted Remake: "Destroy target creature. Its controller manifests dread." P1
    // manifests Lightning Bolt.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cards = stack_library(&mut t, P1, &["Lightning Bolt", "Hill Giant"]);
    let remake = in_hand_with_mana(&mut t, P0, "Unwanted Remake");
    t.cast(P0, remake).target(bears).go();
    t.answer_choose(P1, &[Entity::Object(cards[0])]);
    t.resolve_all();
    let m = t.g.current(cards[0]);
    assert!(t.obj(m).face_down && t.obj(m).controller == P1);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    instant_stays_face_down(&mut t, P1, m);
}

#[test]
fn a_form_manifesting_an_instant_that_would_turn_face_up_stays_face_down() {
    cr!("701.40g", "303.4");
    ruling!(
        "Cloudform",
        "There are no cards in this set that would turn a face-down instant or sorcery card on the battlefield face up, but some older cards can try to do this. If something tries to turn a face-down instant or sorcery card on the battlefield face up, reveal that card to show all players it's an instant or sorcery card. The permanent remains on the battlefield face down."
    );
    let mut t = TestGame::new(2);
    let (form, m) = cloudform(&mut t, Some("Lightning Bolt"));
    let m = m.unwrap();
    instant_stays_face_down(&mut t, P0, m);
    // Still enchanted: a 2/2 with flying and hexproof.
    assert_eq!(t.obj(form).attached_to, Some(Entity::Object(m)));
    assert!(t.obj(m).chars.has_keyword(KeywordKind::Flying));
    assert!(t.obj(m).chars.has_keyword(KeywordKind::Hexproof));
}

#[test]
fn face_down_creatures_have_no_name_so_they_share_it_with_nothing() {
    cr!("201.2a", "708.2a", "701.40a");
    ruling!(
        "Reality Shift",
        "Because face-down creatures don't have names, they can't have the same name as any other creature, even another face-down creature."
    );
    supported("Maelstrom Pulse");
    // Two manifested Grizzly Bears cards and two face-up Grizzly Bears. Maelstrom Pulse:
    // "Destroy target nonland permanent and all other permanents with the same name as
    // that permanent."
    let mut t = TestGame::new(2);
    let a = reality_shift(&mut t, "Grizzly Bears");
    let b = manifest_card(&mut t, P1, "Grizzly Bears");
    let up1 = t.battlefield(P1, "Grizzly Bears");
    let up2 = t.battlefield(P0, "Grizzly Bears");
    let pulse = in_hand_with_mana(&mut t, P0, "Maelstrom Pulse");
    t.cast(P0, pulse).target(a).go();
    t.resolve_all();
    assert!(!t.g.is_live(a));
    assert!(t.on_battlefield(b) && t.on_battlefield(up1) && t.on_battlefield(up2));
    // A face-up Bears shares its name with the other face-up one only.
    let pulse = in_hand_with_mana(&mut t, P0, "Maelstrom Pulse");
    t.cast(P0, pulse).target(up1).go();
    t.resolve_all();
    assert!(!t.g.is_live(up1) && !t.g.is_live(up2));
    assert!(t.on_battlefield(b));
}

#[test]
fn face_down_creatures_share_neither_a_name_nor_a_creature_type() {
    cr!("201.2a", "708.2a", "205.3m");
    ruling!(
        "Cloudform",
        "Because face-down creatures don't have a name, they can't have the same name as any other creature or share any creature types with any other creature, even another face-down creature."
    );
    supported("Coat of Arms");
    // Coat of Arms: "Each creature gets +1/+1 for each other creature on the battlefield
    // that shares at least one creature type with it."
    let mut t = TestGame::new(2);
    let (_, m) = cloudform(&mut t, Some("Grizzly Bears"));
    let m = m.unwrap();
    let other = manifest_card(&mut t, P0, "Grizzly Bears");
    let up1 = t.battlefield(P0, "Grizzly Bears");
    let up2 = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Coat of Arms");
    t.g.recompute();
    assert_eq!(t.pt(m), (2, 2));
    assert_eq!(t.pt(other), (2, 2));
    assert_eq!(t.pt(up1), (3, 3));
    assert_eq!(t.pt(up2), (3, 3));
    // Nor a name: Maelstrom Pulse on one face-down creature destroys only it.
    let pulse = in_hand_with_mana(&mut t, P0, "Maelstrom Pulse");
    t.cast(P0, pulse).target(other).go();
    t.resolve_all();
    assert!(!t.g.is_live(other));
    assert!(t.on_battlefield(m) && t.on_battlefield(up1) && t.on_battlefield(up2));
}

#[test]
fn you_can_look_at_face_down_permanents_you_control_but_not_others() {
    cr!("708.5");
    ruling!(
        "Reality Shift",
        "At any time, you can look at a face-down permanent you control. You can't look at face-down permanents you don't control unless an effect allows you to or instructs you to."
    );
    ruling!(
        "Cloudform",
        "At any time, you can look at a face-down permanent you control. You can't look at face-down permanents you don't control unless an effect instructs you to do so."
    );
    let mut t = TestGame::new(2);
    let theirs = reality_shift(&mut t, "Hill Giant");
    assert!(facedown::can_look_at(&t.g, P1, theirs));
    assert!(!facedown::can_look_at(&t.g, P0, theirs));
    let (_, mine) = cloudform(&mut t, Some("Grizzly Bears"));
    let mine = mine.unwrap();
    assert!(facedown::can_look_at(&t.g, P0, mine));
    assert!(!facedown::can_look_at(&t.g, P1, mine));
    // Once it's turned face up, everyone can see it.
    t.lands(P0, "Forest", 2);
    assert!(turn_face_up(&mut t, P0, mine));
    assert!(facedown::can_look_at(&t.g, P1, mine));
}

#[test]
fn a_face_down_creature_returned_after_leaving_comes_back_face_up() {
    cr!("400.7", "400.4a", "701.40a");
    ruling!(
        "Cloudform",
        "If an effect tries to return a face-down creature to the battlefield after it leaves (such as Aminatou's second ability or Adarkar Valkyrie's delayed triggered ability), that effect returns the card face up. If it tries to put an instant or sorcery card onto the battlefield this way, that card remains in its current zone instead."
    );
    supported("Cloudshift");
    // Cloudshift: "Exile target creature you control, then return that card to the
    // battlefield under your control."
    let mut t = TestGame::new(2);
    let (form, m) = cloudform(&mut t, Some("Hill Giant"));
    let m = m.unwrap();
    let shift = in_hand_with_mana(&mut t, P0, "Cloudshift");
    t.cast(P0, shift).target(m).go();
    t.resolve_all();
    let giant = t.g.current(m);
    assert!(t.on_battlefield(giant));
    assert!(!t.obj(giant).face_down);
    assert_eq!(t.obj(giant).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(giant), (3, 3));
    // The Form, no longer attached to anything, was put into the graveyard.
    assert!(t.in_graveyard(P0, "Cloudform"));
    assert!(!t.on_battlefield(form));
    // An instant card stays in exile.
    let mut t = TestGame::new(2);
    let (_, m) = cloudform(&mut t, Some("Lightning Bolt"));
    let m = m.unwrap();
    let shift = in_hand_with_mana(&mut t, P0, "Cloudshift");
    t.cast(P0, shift).target(m).go();
    t.resolve_all();
    let bolt = t.g.current(m);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(t.in_exile("Lightning Bolt"));
}

#[test]
fn a_manifested_creature_card_turns_face_up_ignoring_type_changing_effects() {
    cr!("701.40b", "116.2b", "708.12");
    ruling!(
        "Cloudform",
        "Any time you have priority, you may turn a manifested creature face up by revealing that it's a creature card (ignoring any type-changing effects that might be applying to it) and paying its mana cost. This is a special action. It doesn't use the stack and can't be responded to."
    );
    let mut t = TestGame::new(2);
    let (_, m) = cloudform(&mut t, Some("Hill Giant"));
    let m = m.unwrap();
    // An effect makes it a noncreature artifact: it's still a creature card.
    run_with(
        &mut t,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetTypes {
                types: vec![CardType::Artifact],
                subtypes: vec![],
            }],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(m)],
    );
    assert!(!t.obj(m).chars.is(CardType::Creature));
    // With an opponent's spell on the stack.
    let bolt = in_hand_with_mana(&mut t, P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.lands(P0, "Mountain", 4);
    assert!(can_turn_face_up(&mut t, P0, m));
    assert!(turn_face_up(&mut t, P0, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Hill Giant");
    // It didn't use the stack.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // A face-down noncreature card made a creature some other way can't be.
    let (_, bolt_m) = cloudform(&mut t, Some("Lightning Bolt"));
    assert!(!can_turn_face_up(&mut t, P0, bolt_m.unwrap()));
}

#[test]
fn a_copy_of_a_face_down_creature_is_a_face_up_2_2_with_no_abilities() {
    cr!("708.2", "707.2", "707.3");
    ruling!(
        "Cloudform",
        "The face-down characteristics of a permanent are copiable values. If another object becomes a copy of a face-down creature or if a token is created that's a copy of a face-down creature, that new object is a 2/2 colorless face-up creature with no abilities."
    );
    supported("Clone");
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    let (_, m) = cloudform(&mut t, Some("Hill Giant"));
    let m = m.unwrap();
    // Clone copies it.
    let clone = in_hand_with_mana(&mut t, P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(m)]);
    t.resolve_all();
    let clone = t.g.current(clone);
    // A token copy of it (Cackling Counterpart: "Create a token that's a copy of target
    // creature you control.").
    let counterpart = in_hand_with_mana(&mut t, P0, "Cackling Counterpart");
    t.cast(P0, counterpart).target(m).go();
    t.resolve_all();
    let token = tokens(&t, P0)[0];
    for copy in [clone, token] {
        let o = t.obj(copy);
        assert!(t.on_battlefield(copy));
        assert!(!o.face_down);
        assert!(o.chars.name.is_empty());
        assert_eq!(t.pt(copy), (2, 2));
        assert_eq!(colors(&t, copy), ColorSet::NONE);
        assert!(o.chars.abilities.is_empty());
        assert!(o.chars.subtypes.is_empty());
        // Not a manifested permanent: it can't be turned face up.
        t.lands(P0, "Mountain", 4);
        assert!(!can_turn_face_up(&mut t, P0, copy));
    }
}

#[test]
fn a_form_with_nothing_to_manifest_goes_to_the_graveyard() {
    cr!("704.5m", "303.4d", "701.40a");
    ruling!(
        "Cloudform",
        "If you have no cards in your library as the ability resolves, the \"Form\" will be put into its owner's graveyard as a state-based action."
    );
    let mut t = TestGame::new(2);
    let (form, m) = cloudform(&mut t, None);
    assert!(m.is_none());
    assert!(!t.on_battlefield(form));
    assert!(t.g.turn_events.iter().any(|e| matches!(
        e,
        Event::ZoneChange {
            from: Zone::Battlefield,
            to: Zone::Graveyard(_),
            cause: MoveCause::StateBased,
            ..
        }
    )));
    assert!(t.in_graveyard(P0, "Cloudform"));
    assert!(t.g.permanents().all(|o| !o.face_down));
    // With a card to manifest, it's an Aura attached to the manifested creature.
    let mut t = TestGame::new(2);
    let (form, m) = cloudform(&mut t, Some("Grizzly Bears"));
    let o = t.obj(form);
    assert!(o.chars.has_subtype("Aura") && o.chars.is(CardType::Enchantment));
    assert_eq!(o.attached_to, m.map(Entity::Object));
}

#[test]
fn a_form_keeps_enchanting_the_creature_once_its_turned_face_up() {
    cr!("708.8", "701.40b");
    ruling!(
        "Cloudform",
        "If the enchanted creature is turned face up, the \"Form\" will continue to enchant it."
    );
    let mut t = TestGame::new(2);
    let (form, m) = cloudform(&mut t, Some("Hill Giant"));
    let m = m.unwrap();
    t.lands(P0, "Mountain", 4);
    assert!(turn_face_up(&mut t, P0, m));
    t.settle();
    assert!(t.on_battlefield(form));
    assert_eq!(t.obj(form).attached_to, Some(Entity::Object(m)));
    // Cloudform: "Enchanted creature has flying and hexproof."
    assert_eq!(t.obj(m).chars.name.as_str(), "Hill Giant");
    assert!(t.obj(m).chars.has_keyword(KeywordKind::Flying));
    assert!(t.obj(m).chars.has_keyword(KeywordKind::Hexproof));
}

#[test]
fn a_form_still_manifests_if_it_left_the_battlefield_before_its_ability_resolved() {
    cr!("603.6a", "701.40a", "608.2h");
    ruling!(
        "Cloudform",
        "You'll still manifest the top card of your library even if the \"Form\" isn't on the battlefield as its enters-the-battlefield ability resolves."
    );
    let mut t = TestGame::new(2);
    let card = t.library_top(P0, "Hill Giant");
    let form = in_hand_with_mana(&mut t, P0, "Cloudform");
    t.cast(P0, form).go();
    // The Form resolves; its enters trigger goes on the stack.
    t.resolve();
    t.settle();
    let form = t.g.current(form);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, form);
    t.resolve_all();
    let m = t.g.current(card);
    assert!(t.on_battlefield(m) && t.obj(m).face_down);
    assert!(t.in_graveyard(P0, "Cloudform"));
    assert!(t.obj(m).chars.abilities.is_empty());
}

#[test]
fn a_face_down_permanent_is_revealed_as_it_leaves_and_when_its_owner_leaves_the_game() {
    cr!("708.9", "800.4a");
    ruling!(
        "Guardian of the Forgotten",
        "If a face-down permanent leaves the battlefield, you must reveal it. You must also reveal all face-down spells and permanents you control if you leave the game or the game end."
    );
    // Leaving the battlefield.
    let mut t = TestGame::new(3);
    let m = guardian_manifests(&mut t, "Hill Giant");
    assert!(revealed_this_turn(&t).is_empty());
    destroy(&mut t, m);
    assert_eq!(revealed_this_turn(&t), vec![m]);
    // Its controller (and owner) leaving the game.
    let m2 = guardian_manifests(&mut t, "Hill Giant");
    t.g.player_loses(P0);
    t.settle();
    assert!(revealed_this_turn(&t).contains(&m2));
    // The game ending.
    let mut t = TestGame::new(2);
    let m = guardian_manifests(&mut t, "Hill Giant");
    t.g.player_loses(P1);
    t.g.flush_events();
    assert!(t.g.result.is_some());
    assert_eq!(revealed_this_turn(&t), vec![m]);
}

#[test]
fn a_manifested_card_is_a_nameless_colorless_2_2_other_effects_can_still_change() {
    cr!("701.40a", "708.2a", "613.1");
    ruling!(
        "Guardian of the Forgotten",
        "To manifest a card, put it onto the battlefield face down. It becomes a 2/2 face-down creature card with no name, mana cost, or creature types. It's colorless and has a mana value of 0. Other effects that apply to the permanent can still grant it any characteristics it doesn't have or change the characteristics it does have."
    );
    supported("Cerulean Wisps");
    let mut t = TestGame::new(2);
    let m = guardian_manifests(&mut t, "Hill Giant");
    assert!(is_plain_face_down_2_2(&t, m));
    assert!(t.obj(m).is_card());
    // Cerulean Wisps ("Target creature becomes blue until end of turn.") and Giant Growth.
    let wisps = in_hand_with_mana(&mut t, P0, "Cerulean Wisps");
    t.cast(P0, wisps).target(m).go();
    t.resolve_all();
    let gg = in_hand_with_mana(&mut t, P0, "Giant Growth");
    t.cast(P0, gg).target(m).go();
    t.resolve_all();
    assert_eq!(colors(&t, m), ColorSet::single(Color::Blue));
    assert_eq!(t.pt(m), (5, 5));
    assert!(t.obj(m).face_down && t.obj(m).chars.name.is_empty());
}

#[test]
fn a_manifested_card_with_morph_or_disguise_turns_up_for_that_cost_too() {
    cr!("701.40c", "701.40d", "702.37e", "702.168d");
    ruling!(
        "Guardian of the Forgotten",
        "If a manifested creature would have morph or disguise if it were face up, you may also turn it face up by paying its morph cost or disguise cost."
    );
    // Sagu Mauler: {4}{G}{U}, morph {3}{G}{U}; five lands pay only the morph cost.
    let mut t = TestGame::new(2);
    let m = guardian_manifests(&mut t, "Sagu Mauler");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    assert!(turn_face_up(&mut t, P0, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Sagu Mauler");
    // Nightdrinker Moroii: {3}{B}, disguise {B}{B}; two Swamps pay only the disguise cost.
    let mut t = TestGame::new(2);
    let m = guardian_manifests(&mut t, "Nightdrinker Moroii");
    t.lands(P0, "Swamp", 1);
    assert!(!can_turn_face_up(&mut t, P0, m));
    t.lands(P0, "Swamp", 1);
    assert!(turn_face_up(&mut t, P0, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Nightdrinker Moroii");
}

#[test]
fn a_manifested_creature_can_be_turned_up_after_losing_its_abilities_unlike_a_morph() {
    cr!("701.40b", "702.37e", "613.1f");
    ruling!(
        "Guardian of the Forgotten",
        "Unlike a face-down creature that was cast using a morph or disguise ability, a manifested creature may still be turned face up after it loses its abilities if it's a creature card."
    );
    supported("Humility");
    let mut t = TestGame::new(2);
    let m = guardian_manifests(&mut t, "Hill Giant");
    // Sagu Mauler cast face down with morph.
    t.lands(P0, "Wastes", 3);
    let mauler = t.hand(P0, "Sagu Mauler");
    let spell = t
        .cast(P0, mauler)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let morph = t.g.current(spell);
    assert!(t.obj(morph).face_down);
    // Humility: "All creatures lose all abilities and have base power and toughness 1/1."
    t.battlefield(P1, "Humility");
    t.g.recompute();
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 4);
    // The morph can't be turned face up: it has no morph cost face up.
    assert!(!can_turn_face_up(&mut t, P0, morph));
    // The manifested Hill Giant can, for its mana cost.
    assert!(can_turn_face_up(&mut t, P0, m));
    assert!(turn_face_up(&mut t, P0, m));
    assert_eq!(t.obj(m).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(m), (1, 1));
}

#[test]
fn target_and_all_other_objects_with_the_same_name_as_that_object() {
    cr!("201.2", "201.2a", "608.2c");
    supported("Bile Blight");
    supported("Echoing Truth");
    supported("Sever the Bloodline");
    // Bile Blight: "Target creature and all other creatures with the same name as that
    // creature get -3/-3 until end of turn." The target gets it once.
    let mut t = TestGame::new(2);
    let wurms: Vec<ObjectId> = (0..2)
        .map(|_| t.battlefield(P1, "Colossal Dreadmaw"))
        .collect();
    let mine = t.battlefield(P0, "Colossal Dreadmaw");
    let other = t.battlefield(P1, "Hill Giant");
    let blight = in_hand_with_mana(&mut t, P0, "Bile Blight");
    t.cast(P0, blight).target(wurms[0]).go();
    t.resolve_all();
    for w in [wurms[0], wurms[1], mine] {
        assert_eq!(t.pt(w), (3, 3));
    }
    assert_eq!(t.pt(other), (3, 3));
    assert_eq!(t.obj(other).chars.power, Some(3));
    // Echoing Truth: "Return target nonland permanent and all other permanents with the
    // same name as that permanent to their owners' hands." Tokens with that name too.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let token = crate::r_s02_common::create_token(&mut t, P1, "Treasure");
    let token2 = crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    let truth = in_hand_with_mana(&mut t, P0, "Echoing Truth");
    t.cast(P0, truth).target(a).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears") && t.in_hand(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(token) && t.on_battlefield(token2));
    let truth = in_hand_with_mana(&mut t, P0, "Echoing Truth");
    t.cast(P0, truth).target(token).go();
    t.resolve_all();
    assert!(!t.on_battlefield(token) && !t.on_battlefield(token2));
    // Sever the Bloodline: "Exile target creature and all other creatures with the same
    // name as that creature."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P1, "Hill Giant");
    let sever = in_hand_with_mana(&mut t, P0, "Sever the Bloodline");
    t.cast(P0, sever).target(b).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(a)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(b)), Zone::Exile);
    assert!(t.on_battlefield(c));
}
