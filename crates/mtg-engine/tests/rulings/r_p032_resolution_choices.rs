//! Rulings batch P032 — choices made as a spell resolves (CR 608.2): Trait Doctoring's
//! words (CR 612), Stream of Thought's cards, Arcane Heist's cast, Call of the Nightwing's
//! cipher creature; and cards exiled with permission to play them (Rob the Archives,
//! Junk tokens), which follow the normal timing rules and costs (CR 601.3, 305.2).

use crate::r_p032_common::*;
use crate::r_s01_common::{stack_library, supported, watch};
use crate::r_s02_common::{can_cast, can_play_land};
use crate::r_s03_common::{choice_candidates, respond};
use crate::r_s06_common::activate_containing;
use crate::r_s21_common::legal_blocks;
use crate::r_s23_common::choose_named_once;
use crate::r_s25_common::cast_new;
use crate::r_s27_common::can_activate_containing;
use mtg_engine::ability::{Filter, Modification};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// Trait Doctoring offers: white, blue, black, red, green, Plains, Island, Swamp,
// Mountain, Forest; then the other words of the same kind.
const BLACK: usize = 2;
const GREEN: usize = 4;
const SWAMP: usize = 7;
const FOREST: usize = 9;

/// P0 casts Trait Doctoring on `target`, replacing the `from`th word with the `to`th of
/// the remaining words of its kind, and resolves it.
fn doctor(t: &mut TestGame, target: ObjectId, from: usize, to: usize) {
    supported("Trait Doctoring");
    t.answer(P0, DecisionKind::Option, Answer::Index(from));
    t.answer(P0, DecisionKind::Option, Answer::Index(to));
    cast_new(t, P0, "Trait Doctoring", &[obj(target)]);
    t.resolve_all();
}

/// The filters of the object's protection abilities.
fn protections(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Protection)
        .map(|k| format!("{:?}", k.filter))
        .collect()
}

#[test]
fn trait_doctoring_changes_part_of_a_word() {
    cr!("612.2", "612.1");
    ruling!(
        "Trait Doctoring",
        "The type-changing effect can change part of a word such as “nonblack” or “swampwalk” if the part of the word is being used to refer to a color or basic land type."
    );
    // Bog Wraith's swampwalk becomes islandwalk: P0 (with an Island, no Swamp) can't
    // block it.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let wraith = t.battlefield(P1, "Bog Wraith");
    t.set_step(P0, Step::PrecombatMain);
    doctor(&mut t, wraith, SWAMP, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(wraith, Entity::Player(P0))]);
    assert!(!legal_blocks(&mut t, P0, &[(bears, wraith)]));
    // Ascendant Evincar ("Other black creatures get +1/+1. Nonblack creatures get
    // -1/-1."), black → green: green creatures get +1/+1 and nongreen ones -1/-1.
    supported("Ascendant Evincar");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let evincar = t.battlefield(P1, "Ascendant Evincar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let knight = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(bears), (1, 1));
    doctor(&mut t, evincar, BLACK, 3);
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(knight), (2, 2));
}

#[test]
fn trait_doctoring_changes_type_line_and_rules_text_but_not_the_name() {
    cr!("612.2", "612.1", "201.2");
    ruling!(
        "Trait Doctoring",
        "Trait Doctoring changes the color word or basic land type each time it appears in the permanent’s type line and/or rules text. It doesn’t change the name of the card or any instances of the word being used as part of a card’s name."
    );
    // White Knight (protection from black), black → red.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let knight = t.battlefield(P1, "White Knight");
    doctor(&mut t, knight, BLACK, 2);
    assert_eq!(protections(&t, knight), vec!["Some(Color(Red))"]);
    assert_eq!(t.obj_now(knight).chars.name, "White Knight");
    // White Knight, white → red: its name is unchanged.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let knight = t.battlefield(P1, "White Knight");
    doctor(&mut t, knight, 0, 2);
    assert_eq!(t.obj_now(knight).chars.name, "White Knight");
    // A Forest, Forest → Island: its type line says Island; its name is still Forest.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let forest = t.battlefield(P1, "Forest");
    doctor(&mut t, forest, FOREST, 1);
    let o = t.obj_now(forest);
    assert!(o.chars.has_subtype("Island"));
    assert!(!o.chars.has_subtype("Forest"));
    assert_eq!(o.chars.name, "Forest");
}

#[test]
fn trait_doctoring_doesnt_change_granted_abilities() {
    cr!("612.1", "613.1c");
    ruling!(
        "Trait Doctoring",
        "Trait Doctoring’s effect changes only the text printed on the permanent. It can’t change words found in abilities it’s been granted. For example, if you’ve changed “green” to “blue,” and the permanent gains protection from green (as opposed to having protection from green printed on it), that protection ability is unaffected. The permanent will have protection from green."
    );
    // White Knight gains protection from green (cloned from its printed protection
    // ability), then green → blue: the granted ability still says green.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let knight = t.battlefield(P0, "White Knight");
    let mut kw = t
        .obj_now(knight)
        .chars
        .keywords()
        .find(|k| k.kind == KeywordKind::Protection)
        .unwrap()
        .clone();
    kw.filter = Some(Filter::Color(Color::Green));
    kw.text = Some("Protection from green".into());
    crate::r_p021_common::modify(&mut t, knight, vec![Modification::AddKeyword(kw)]);
    doctor(&mut t, knight, GREEN, 1);
    let mut p = protections(&t, knight);
    p.sort();
    assert_eq!(p, vec!["Some(Color(Black))", "Some(Color(Green))"]);
}

#[test]
fn trait_doctoring_words_are_chosen_on_resolution() {
    cr!("608.2", "612.1");
    ruling!(
        "Trait Doctoring",
        "You choose what word you’re changing and what word you’re changing it to as Trait Doctoring resolves."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let knight = t.battlefield(P1, "White Knight");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseOption { .. }),
        |g| g.stack.len(),
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(BLACK));
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    cast_new(&mut t, P0, "Trait Doctoring", &[obj(knight)]);
    assert!(seen.lock().unwrap().is_empty(), "words chosen while casting");
    t.resolve_all();
    // Both words were chosen while Trait Doctoring was on the stack, resolving.
    assert_eq!(*seen.lock().unwrap(), vec![1, 1]);
}

#[test]
fn trait_doctoring_can_target_a_permanent_without_such_words() {
    cr!("115.1", "612.1");
    ruling!(
        "Trait Doctoring",
        "You can target any permanent with Trait Doctoring, including one with no color words or basic land types on it."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let thopter = t.battlefield(P1, "Ornithopter");
    doctor(&mut t, thopter, 0, 0);
    assert!(t.in_graveyard(P0, "Trait Doctoring") || t.in_exile("Trait Doctoring"));
    assert_eq!(t.obj_now(thopter).chars.name, "Ornithopter");
}

#[test]
fn stream_of_thought_chooses_the_cards_on_resolution_including_milled_ones() {
    cr!("608.2", "608.2c");
    ruling!(
        "Stream of Thought",
        "You choose which cards to shuffle into your library while Stream of Thought is resolving. If you target yourself with Stream of Thought, you may choose any of the four cards that you put into your graveyard or any cards that were there before."
    );
    supported("Stream of Thought");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.graveyard(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Hill Giant", "Craw Wurm", "Shock", "Island"]);
    choose_named_once(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    cast_new(&mut t, P0, "Stream of Thought", &[Entity::Player(P0)]);
    t.resolve_all();
    // Both were chosen (choose_named_once picks only offered cards): the one already
    // there and one just milled.
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Hill Giant"));
    assert!(!t.g.find_in_zone(Zone::Library(P0), "Hill Giant").is_empty());
    for n in ["Craw Wurm", "Shock", "Island", "Stream of Thought"] {
        assert!(t.in_graveyard(P0, n), "{n}");
    }
}

#[test]
fn arcane_heist_casts_the_card_during_its_resolution_or_never() {
    cr!("608.2g", "601.2");
    ruling!(
        "Arcane Heist",
        "You choose whether to cast the target instant or sorcery card as Arcane Heist resolves. If you do, you do so as part of the resolution of Arcane Heist. You can’t wait to cast it later in the turn. Timing restrictions based on the card’s types are ignored."
    );
    supported("Arcane Heist");
    // Divination (a sorcery: "Draw two cards.") in P1's graveyard is cast while Arcane
    // Heist is resolving (the stack isn't empty), then exiled instead of going to the
    // graveyard.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let div = t.graveyard(P1, "Divination");
    let hand = t.hand_size(P0);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.stack.len(),
    );
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    cast_new(&mut t, P0, "Arcane Heist", &[obj(div)]);
    t.resolve_all();
    assert_eq!(seen.lock().unwrap()[0], 1, "asked while Arcane Heist resolves");
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.zone(div), Zone::Exile);
    // Declined: it stays in P1's graveyard and can't be cast later.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let div = t.graveyard(P1, "Divination");
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    cast_new(&mut t, P0, "Arcane Heist", &[obj(div)]);
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Graveyard(P1));
    t.lands(P0, "Island", 3);
    for m in [CastMethod::Normal, CastMethod::Free] {
        assert!(!can_cast(&mut t, P0, div, m));
    }
}

/// Encodes a cipher card on the first token offered.
fn cipher_on_a_token(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseEntities {
            prompt, candidates, ..
        } if prompt.contains("cipher") => Some(Answer::Entities(
            candidates
                .iter()
                .copied()
                .filter(|e| e.object().is_some_and(|o| g.obj(o).is_token()))
                .take(1)
                .collect(),
        )),
        _ => None,
    }
}

#[test]
fn call_of_the_nightwing_can_be_encoded_on_the_horror_it_created() {
    cr!("702.99a", "608.2c");
    ruling!(
        "Call of the Nightwing",
        "You can exile Call of the Nightwing encoded on the Horror creature token it just created."
    );
    supported("Call of the Nightwing");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Grizzly Bears");
    respond(&mut t, P0, cipher_on_a_token);
    let card = cast_new(&mut t, P0, "Call of the Nightwing", &[]);
    t.resolve_all();
    let horror = tokens_named(&t, P0, "Horror");
    let horror = if horror.is_empty() {
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Horror"))
            .map(|o| o.id)
            .collect()
    } else {
        horror
    };
    assert_eq!(horror.len(), 1);
    assert_eq!(
        mtg_engine::kw::cipher::encoded_on(&t.g, t.g.current(card)),
        Some(horror[0])
    );
}

/// Rob the Archives exiling Forest and Grizzly Bears from the top of P0's library.
/// Returns the exiled cards.
fn rob(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Rob the Archives");
    stack_library(t, P0, &["Forest", "Grizzly Bears"]);
    cast_new(t, P0, "Rob the Archives", &[]);
    t.resolve_all();
    let forest = t.g.find_in_zone(Zone::Exile, "Forest")[0];
    let bears = t.g.find_in_zone(Zone::Exile, "Grizzly Bears")[0];
    (forest, bears)
}

#[test]
fn rob_the_archives_cards_follow_normal_timing_and_costs() {
    cr!("601.2f", "305.2", "305.3");
    ruling!(
        "Rob the Archives",
        "You must pay all costs and follow all normal timing rules for cards played this way. For example, you may play lands this way only during your main phase while no spells or abilities are on the stack and only if you haven't played a land yet this turn."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let (forest, bears) = rob(&mut t);
    // No mana for the Bears; the land can be played.
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    assert!(can_play_land(&mut t, P0, forest));
    // Not while something is on the stack.
    let shock = t.hand(P0, "Ornithopter");
    t.cast(P0, shock).go();
    assert!(!can_play_land(&mut t, P0, forest));
    t.resolve_all();
    // Not after a land was played this turn.
    let other = t.hand(P0, "Plains");
    t.play_land(P0, other).unwrap();
    assert!(!can_play_land(&mut t, P0, forest));
    // With mana, the Bears can be cast (paying for them).
    t.lands(P0, "Forest", 2);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    // Not in combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
}

/// P0's Thrill-Kill Disciple dies, creating a Junk token. Returns it.
fn junk(t: &mut TestGame) -> ObjectId {
    supported("Thrill-Kill Disciple");
    let d = t.battlefield(P0, "Thrill-Kill Disciple");
    crate::r_s02_common::destroy(t, d);
    t.resolve_all();
    let j: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Junk"))
        .map(|o| o.id)
        .collect();
    assert_eq!(j.len(), 1);
    j[0]
}

#[test]
fn junk_token_cards_follow_normal_timing_and_costs() {
    cr!("601.2f", "305.2", "305.3");
    ruling!(
        "Thrill-Kill Disciple",
        "You pay all costs and follow all normal timing rules for the card played from exile with a Junk token’s ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let j = junk(&mut t);
    stack_library(&mut t, P0, &["Forest"]);
    activate_containing(&mut t, P0, j, "Exile the top card").expect("junk");
    t.resolve_all();
    let forest = t.g.find_in_zone(Zone::Exile, "Forest")[0];
    assert!(can_play_land(&mut t, P0, forest));
    let thopter = t.hand(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    assert!(!can_play_land(&mut t, P0, forest));
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, forest));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, forest));
    // A spell card needs its mana paid.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let j = junk(&mut t);
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    activate_containing(&mut t, P0, j, "Exile the top card").expect("junk");
    t.resolve_all();
    let bears = t.g.find_in_zone(Zone::Exile, "Grizzly Bears")[0];
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
    t.lands(P0, "Forest", 2);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
}

#[test]
fn a_junk_token_cant_pay_two_costs() {
    cr!("602.2b", "701.21a");
    ruling!(
        "Thrill-Kill Disciple",
        "You can’t sacrifice a Junk token to pay multiple costs. For example, you can’t sacrifice a Junk token to activate its own ability and also to activate the second ability of Junk Jet."
    );
    supported("Junk Jet");
    // Junk Jet: "{3}, Sacrifice another artifact: Double equipped creature's power until
    // end of turn." The Junk token is P0's only other artifact.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let j = junk(&mut t);
    let jet = t.battlefield(P0, "Junk Jet");
    t.lands(P0, "Wastes", 3);
    assert!(can_activate_containing(&mut t, P0, jet, "Double"));
    activate_containing(&mut t, P0, j, "Exile the top card").expect("junk");
    assert!(!t.is_live(j));
    assert!(!can_activate_containing(&mut t, P0, jet, "Double"));
}
