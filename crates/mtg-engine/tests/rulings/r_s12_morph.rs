//! Rulings batch S12 — morph (CR 702.37, 708): cast face down for {3}, turned face up any
//! time its controller has priority by paying its morph cost (a special action).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::untapped_lands;
use crate::r_s05_common::colors;
use crate::r_s06_common::attach_new;
use crate::r_s11_common::*;
use crate::r_s12_common::*;
use mtg_engine::facedown;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn turning_face_up_or_down_keeps_targeting_spells_and_attachments() {
    cr!("708.8", "702.37e", "708.2a");
    ruling!(
        "Grim Haruspex",
        "A permanent that turns face up or face down changes characteristics but is otherwise the same permanent. Spells and abilities that were targeting that permanent, as well as Auras and Equipment that were attached to the permanent, aren't affected."
    );
    ruling!(
        "Ainok Tracker",
        "A permanent that turns face up or face down changes characteristics but is otherwise the same permanent. Spells and abilities that were targeting that permanent, as well as Auras and Equipment that were attached to the permanent, aren’t affected."
    );
    supported("Grim Haruspex");
    supported("Backslide");
    let mut t = TestGame::new(2);
    let hx = morph(&mut t, P0, "Grim Haruspex");
    // Rancor ("+2/+0 and trample") and Bonesplitter ("+2/+0") on the face-down 2/2.
    let rancor = attach_new(&mut t, P0, "Rancor", hx);
    let splitter = attach_new(&mut t, P0, "Bonesplitter", hx);
    assert_eq!(t.pt(hx), (6, 2));
    // Giant Growth targets it; in response it's turned face up for {B}.
    let gg = in_hand_with_mana(&mut t, P0, "Giant Growth");
    t.cast(P0, gg).target(hx).go();
    t.lands(P0, "Swamp", 1);
    assert!(turn_face_up(&mut t, P0, hx));
    assert!(t.g.is_live(hx));
    assert_eq!(t.obj(hx).chars.name.as_str(), "Grim Haruspex");
    t.resolve_all();
    // Grim Haruspex is 3/2: +2/+0 twice and +3/+3.
    assert_eq!(t.pt(hx), (10, 5));
    assert_eq!(t.obj(rancor).attached_to, Some(Entity::Object(hx)));
    assert_eq!(t.obj(splitter).attached_to, Some(Entity::Object(hx)));
    // Another Giant Growth targets it; in response Backslide ("Turn target creature with
    // a morph ability face down") turns it face down. It's still the same permanent.
    let gg = in_hand_with_mana(&mut t, P0, "Giant Growth");
    t.cast(P0, gg).target(hx).go();
    let slide = in_hand_with_mana(&mut t, P0, "Backslide");
    t.cast(P0, slide).target(hx).go();
    t.resolve();
    assert!(t.g.is_live(hx) && t.obj(hx).face_down);
    assert!(t.obj(hx).chars.name.is_empty());
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Giant Growth") && t.in_graveyard(P0, "Backslide"));
    // A 2/2 with +2/+0 twice and +3/+3 twice.
    assert_eq!(t.pt(hx), (12, 8));
    assert_eq!(t.obj(rancor).attached_to, Some(Entity::Object(hx)));
    assert_eq!(t.obj(splitter).attached_to, Some(Entity::Object(hx)));
}

#[test]
fn a_face_down_morph_is_revealed_when_it_leaves_or_the_game_ends() {
    cr!("708.9");
    ruling!(
        "Grim Haruspex",
        "If a face-down permanent leaves the battlefield, you must reveal it. You must also reveal all face-down spells and permanents you control if you leave the game or if the game ends."
    );
    // Leaving the battlefield.
    let mut t = TestGame::new(3);
    let hx = morph(&mut t, P0, "Grim Haruspex");
    assert!(revealed_this_turn(&t).is_empty());
    destroy(&mut t, hx);
    assert_eq!(revealed_this_turn(&t), vec![hx]);
    assert!(t.in_graveyard(P0, "Grim Haruspex"));
    // Its controller leaving the game: the face-down spell and permanent it controls.
    let hx2 = morph(&mut t, P0, "Grim Haruspex");
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Hooded Hydra");
    let spell = t.cast(P0, card).method(MORPH).go();
    t.g.player_loses(P0);
    t.settle();
    let seen = revealed_this_turn(&t);
    assert!(seen.contains(&hx2) && seen.contains(&spell), "{seen:?}");
    // The game ending.
    let mut t = TestGame::new(2);
    let hx = morph(&mut t, P0, "Grim Haruspex");
    t.g.player_loses(P1);
    t.g.flush_events();
    assert!(t.g.result.is_some());
    assert_eq!(revealed_this_turn(&t), vec![hx]);
}

#[test]
fn morph_casts_for_three_and_turns_up_for_the_morph_cost_whenever_you_have_priority() {
    cr!("702.37a", "702.37c", "702.37e", "116.2b");
    ruling!(
        "Rattleclaw Mystic",
        "Morph lets you cast a card face down by paying {3}, and lets you turn the face-down permanent face up any time you have priority by paying its morph cost."
    );
    supported("Rattleclaw Mystic");
    let mut t = TestGame::new(2);
    // {3}, not its mana cost {1}{G}: three Wastes pay for it.
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Rattleclaw Mystic");
    assert!(can_cast(&mut t, P0, card, MORPH));
    t.cast(P0, card).method(MORPH).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    let rm = t.g.current(card);
    assert!(is_plain_face_down_2_2(&t, rm));
    // During the opponent's upkeep, P0 has priority: it turns face up for morph {2}.
    t.advance_to(P1, Step::Upkeep);
    assert!(!can_turn_face_up(&mut t, P0, rm));
    t.lands(P0, "Wastes", 2);
    assert!(can_turn_face_up(&mut t, P0, rm));
    assert!(turn_face_up(&mut t, P0, rm));
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(!t.obj(rm).face_down);
    assert_eq!(t.obj(rm).chars.name.as_str(), "Rattleclaw Mystic");
    assert_eq!(t.pt(rm), (2, 1));
    // "When this creature is turned face up, add {G}{U}{R}."
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn you_can_look_at_your_face_down_spells_and_permanents_but_not_others() {
    cr!("708.5");
    ruling!(
        "Grim Haruspex",
        "At any time, you can look at a face-down spell or permanent you control. You can't look at face-down spells or permanents you don't control unless an effect instructs you to do so."
    );
    ruling!(
        "Ainok Tracker",
        "At any time, you can look at a face-down spell or permanent you control. You can’t look at face-down spells or permanents you don’t control unless an effect instructs you to do so."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Grim Haruspex");
    let spell = t.cast(P0, card).method(MORPH).go();
    assert!(facedown::can_look_at(&t.g, P0, spell));
    assert!(!facedown::can_look_at(&t.g, P1, spell));
    t.resolve_all();
    let hx = t.g.current(spell);
    assert!(facedown::can_look_at(&t.g, P0, hx));
    assert!(!facedown::can_look_at(&t.g, P1, hx));
    // P1's face-down Ainok Tracker: P1 may look at it; P0 may not.
    t.advance_to(P1, Step::PrecombatMain);
    let tracker = morph(&mut t, P1, "Ainok Tracker");
    assert!(facedown::can_look_at(&t.g, P1, tracker));
    assert!(!facedown::can_look_at(&t.g, P0, tracker));
}

#[test]
fn turned_face_up_triggers_on_the_special_action_or_an_effect_not_on_leaving() {
    cr!("702.37e", "708.8", "603.2");
    ruling!(
        "Echo Tracer",
        "The trigger occurs when you use the Morph ability to turn the card face up, or when an effect turns it face up. It will not trigger on being revealed or on leaving the battlefield."
    );
    supported("Echo Tracer");
    supported("Ugin's Mastery");
    // Echo Tracer: "When this creature is turned face up, return target creature to its
    // owner's hand."
    // The morph special action.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tracer = morph(&mut t, P0, "Echo Tracer");
    t.lands(P0, "Island", 3);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert!(turn_face_up(&mut t, P0, tracer));
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // An effect: Ugin's Mastery ("Whenever you attack with creatures with total power 6 or
    // greater, you may turn a face-down creature you control face up.").
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ugin's Mastery");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tracer = morph(&mut t, P0, "Echo Tracer");
    let wurm = t.battlefield(P0, "Craw Wurm");
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(tracer)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(!t.obj(tracer).face_down);
    assert_eq!(triggered_from(&t, tracer), 1);
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Leaving the battlefield (and being revealed as it does): no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let tracer = morph(&mut t, P0, "Echo Tracer");
    destroy(&mut t, tracer);
    assert_eq!(revealed_this_turn(&t), vec![tracer]);
    t.settle();
    assert_eq!(triggered_from(&t, tracer), 0);
    assert_eq!(t.stack_len(), 0);
    assert!(!turned_face_up_this_turn(&t));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn a_morph_can_be_turned_up_in_response_to_a_split_second_spell() {
    cr!("702.61a", "702.61b", "116.2b");
    ruling!(
        "Voidmage Apprentice",
        "If a spell with split second is on the stack, you can still respond by turning this creature face up and targeting that spell with the trigger. This is because split second only stops players from casting spells or activating abilities, while turning a creature face up is a special action."
    );
    supported("Voidmage Apprentice");
    supported("Sudden Shock");
    let mut t = TestGame::new(2);
    let va = morph(&mut t, P0, "Voidmage Apprentice");
    t.lands(P0, "Island", 6);
    let cs = t.hand(P0, "Counterspell");
    // P1's Sudden Shock (split second) at P0.
    let shock = in_hand_with_mana(&mut t, P1, "Sudden Shock");
    t.g.turn.priority = Some(P1);
    let shock = t.cast(P1, shock).target(P0).go();
    // P0 can't cast Counterspell, but can turn Voidmage Apprentice face up ({2}{U}{U}).
    assert!(!can_cast(&mut t, P0, cs, mtg_engine::object::CastMethod::Normal));
    assert!(can_turn_face_up(&mut t, P0, va));
    t.answer_targets(P0, &[Entity::Object(shock)]);
    assert!(turn_face_up(&mut t, P0, va));
    t.settle();
    // "When this creature is turned face up, counter target spell."
    assert_eq!(triggered_from(&t, va), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sudden Shock"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.zone(cs), Zone::Hand(P0));
}

#[test]
fn turning_ainok_tracker_face_up_is_a_special_action_only_for_the_permanent() {
    cr!("702.37e", "116.2b");
    ruling!(
        "Ainok Tracker",
        "Any time you have priority, you may turn the face-down creature face up by revealing what its morph cost is and paying that cost. This is a special action. It doesn’t use the stack and can’t be responded to. Only a face-down permanent can be turned face up this way; a face-down spell cannot."
    );
    supported("Ainok Tracker");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    t.lands(P0, "Mountain", 5);
    let card = t.hand(P0, "Ainok Tracker");
    let spell = t.cast(P0, card).method(MORPH).go();
    // A face-down spell can't be turned face up.
    assert!(!can_turn_face_up(&mut t, P0, spell));
    t.resolve_all();
    let tracker = t.g.current(spell);
    // With a spell on the stack, P0 turns it face up for {4}{R}: no stack object is added.
    let bolt = in_hand_with_mana(&mut t, P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P1).go();
    assert!(turn_face_up(&mut t, P0, tracker));
    assert_eq!(t.stack_len(), 1);
    assert!(!t.obj(tracker).face_down);
    assert_eq!(t.obj(tracker).chars.name.as_str(), "Ainok Tracker");
    assert_eq!(t.pt(tracker), (3, 3));
    assert!(t.obj(tracker).has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike));
}

#[test]
fn a_face_down_ainok_tracker_is_a_plain_2_2_other_effects_can_still_change() {
    cr!("702.37c", "708.2a", "613.1");
    ruling!(
        "Ainok Tracker",
        "When the spell resolves, it enters the battlefield as a 2/2 creature with no name, mana cost, creature types, or abilities. It’s colorless and has a mana value of 0. Other effects that apply to the creature can still grant it any of these characteristics."
    );
    let mut t = TestGame::new(2);
    let tracker = morph(&mut t, P0, "Ainok Tracker");
    assert!(is_plain_face_down_2_2(&t, tracker));
    // Cerulean Wisps ("Target creature becomes blue until end of turn.") and Giant Growth.
    let wisps = in_hand_with_mana(&mut t, P0, "Cerulean Wisps");
    t.cast(P0, wisps).target(tracker).go();
    t.resolve_all();
    let gg = in_hand_with_mana(&mut t, P0, "Giant Growth");
    t.cast(P0, gg).target(tracker).go();
    t.resolve_all();
    assert_eq!(colors(&t, tracker), ColorSet::single(Color::Blue));
    assert_eq!(t.pt(tracker), (5, 5));
    assert!(t.obj(tracker).face_down && t.obj(tracker).chars.name.is_empty());
    assert!(t.obj(tracker).chars.subtypes.is_empty());
}
