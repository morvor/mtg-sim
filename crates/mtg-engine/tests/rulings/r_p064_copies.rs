//! Rulings batch P064 — copies of permanents whose characteristics were chosen as they
//! entered (Corrupted Shapeshifter, Primal Clay, Primal Plasma: CR 707.2, 614.12),
//! animated planeswalkers (Sarkhan, the Dragonspeaker), and Skinshifter's type and P/T
//! changes (613.1d, 613.4b, 707.2).

use crate::r_p017_common::enter_copying;
use crate::r_p064_common::*;
use crate::r_s01_common::supported;
use crate::r_s14_common::triggers_from;
use crate::r_s17_common::become_copy;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::object::Zone;
use mtg_engine::types::*;
use mtg_engine::*;

/// The options of the "choice as it enters" decisions asked since `from`.
fn options_asked(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect()
}

/// `p` puts the shapeshifter `name` onto the battlefield choosing option `pick` (by index,
/// as printed) as it enters.
fn shapeshift(t: &mut TestGame, p: PlayerId, name: &str, pick: usize) -> ObjectId {
    t.answer(p, DecisionKind::Option, Answer::Index(pick));
    let from = t.asked().len();
    let id = t.enter(p, name);
    t.settle();
    assert!(
        !options_asked(t, from).is_empty(),
        "no choice as {name} entered"
    );
    id
}

/// `p`'s Clone enters copying `what`, choosing option `pick` (by index) for the copy's own
/// "as this enters" choice.
fn clone_choosing(t: &mut TestGame, p: PlayerId, what: ObjectId, pick: usize) -> ObjectId {
    t.answer(p, DecisionKind::Option, Answer::Index(pick));
    enter_copying(t, p, "Clone", Some(what))
}

#[test]
fn entering_as_a_copy_of_corrupted_shapeshifter_applies_both_choices() {
    cr!("707.2", "614.12", "613.2a");
    ruling!(
        "Corrupted Shapeshifter",
        "If a permanent enters the battlefield as a copy of Corrupted Shapeshifter, there will be two copy effects to apply to it."
    );
    supported("Corrupted Shapeshifter");
    let mut t = TestGame::new(2);
    let cs = shapeshift(&mut t, P0, "Corrupted Shapeshifter", 2);
    assert_eq!(t.pt(cs), (0, 12));
    assert!(has(&mut t, cs, KeywordKind::Defender));
    let c = clone_choosing(&mut t, P0, cs, 0);
    assert_eq!(t.pt(c), (3, 3), "its own choice sets its P/T last");
    assert!(flies(&mut t, c));
    assert!(has(&mut t, c, KeywordKind::Defender), "and the copied defender");
}

#[test]
fn entering_as_a_copy_of_primal_clay_uses_its_own_pt_choice() {
    cr!("707.2", "614.12");
    ruling!(
        "Primal Clay",
        "If an object enters the battlefield as a copy of Primal Clay, it copies the values determined by its enters-the-battlefield replacement effect, but its power and toughness are determined by the copy's own enters-the-battlefield replacement effect."
    );
    supported("Primal Clay");
    let mut t = TestGame::new(2);
    let pc = shapeshift(&mut t, P0, "Primal Clay", 1);
    assert_eq!(t.pt(pc), (2, 2));
    let c = clone_choosing(&mut t, P0, pc, 2);
    assert_eq!(t.pt(c), (1, 6));
    assert!(flies(&mut t, c));
    assert!(has(&mut t, c, KeywordKind::Defender));
}

#[test]
fn becoming_a_copy_of_a_chosen_shapeshifter_copies_its_choice() {
    cr!("707.2");
    ruling!(
        "Corrupted Shapeshifter",
        "If an object on the battlefield becomes a copy of Corrupted Shapeshifter, it copies the values determined by its enters-the-battlefield replacement effect."
    );
    ruling!(
        "Primal Clay",
        "If an object on the battlefield becomes a copy of Primal Clay, it copies the values determined by its enters-the-battlefield replacement effect."
    );
    ruling!(
        "Primal Plasma",
        "If an object on the battlefield becomes a copy of Primal Plasma, it copies the values determined by its enters-the-battlefield replacement effect."
    );
    supported("Primal Plasma");
    for (name, pick, pt, kw) in [
        (
            "Corrupted Shapeshifter",
            1,
            (2, 5),
            KeywordKind::Vigilance,
        ),
        ("Primal Clay", 1, (2, 2), KeywordKind::Flying),
        ("Primal Plasma", 2, (1, 6), KeywordKind::Defender),
    ] {
        let mut t = TestGame::new(2);
        let s = shapeshift(&mut t, P0, name, pick);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        become_copy(&mut t, bears, s);
        assert!(options_asked(&t, from).is_empty(), "{name}: no new choice");
        assert_eq!(t.pt(bears), pt, "{name}");
        assert!(has(&mut t, bears, kw), "{name}");
    }
}

#[test]
fn chosen_shapeshifters_are_0_0_cards_elsewhere() {
    cr!("614.12", "400.7");
    ruling!(
        "Corrupted Shapeshifter",
        "While not on the battlefield, Corrupted Shapeshifter is a 0/0 creature card. It doesn't have flying, vigilance, or defender."
    );
    ruling!(
        "Primal Clay",
        "While not on the battlefield, Primal Clay is a 0/0 creature card. It doesn't have flying or defender."
    );
    for (name, flying) in [("Corrupted Shapeshifter", 0), ("Primal Clay", 1)] {
        let mut t = TestGame::new(2);
        let h = t.hand(P0, name);
        t.g.recompute();
        assert_eq!(t.pt(h), (0, 0), "{name}");
        assert!(!flies(&mut t, h));
        assert!(!has(&mut t, h, KeywordKind::Defender));
        // One that chose flying, then went to the graveyard.
        let s = shapeshift(&mut t, P0, name, flying);
        assert!(flies(&mut t, s));
        crate::r_s02_common::destroy(&mut t, s);
        assert_eq!(t.zone(s), Zone::Graveyard(P0));
        assert_eq!(t.pt(s), (0, 0), "{name} in the graveyard");
        assert!(!flies(&mut t, s));
    }
}

// ---------------------------------------------------------------------------------------
// Sarkhan, the Dragonspeaker
// ---------------------------------------------------------------------------------------

/// P0's Sarkhan, after his +1 has resolved: a 4/4 Dragon creature.
fn dragon_sarkhan(t: &mut TestGame) -> ObjectId {
    let s = t.battlefield(P0, "Sarkhan, the Dragonspeaker");
    activate_and_resolve(t, P0, s, "+1");
    assert!(crate::r_s20_common::creature_now(t, s));
    s
}

#[test]
fn sarkhan_becoming_a_creature_isnt_a_creature_entering() {
    cr!("603.6a", "613.1d");
    ruling!(
        "Sarkhan, the Dragonspeaker",
        "If Sarkhan, the Dragonspeaker becomes a creature due to his first ability, that doesn't count as having a creature enter the battlefield."
    );
    supported("Sarkhan, the Dragonspeaker");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Soul Warden");
    dragon_sarkhan(&mut t);
    assert_eq!(triggers_from(&t, w), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_copy_of_dragon_sarkhan_is_a_planeswalker_with_four_loyalty() {
    cr!("707.2", "306.5b", "613.1d");
    ruling!(
        "Sarkhan, the Dragonspeaker",
        "If a permanent enters the battlefield as a copy of Sarkhan after his first ability has resolved, the copy will be a Sarkhan planeswalker. It will enter the battlefield with four loyalty counters"
    );
    let mut t = TestGame::new(2);
    let s = dragon_sarkhan(&mut t);
    assert_eq!(t.counters(s, "loyalty"), 5);
    let c = enter_copying(&mut t, P1, "Clone", Some(s));
    assert!(t.obj_now(c).is(CardType::Planeswalker));
    assert!(!t.obj_now(c).is(CardType::Creature));
    assert_eq!(t.counters(c, "loyalty"), 4);
}

// ---------------------------------------------------------------------------------------
// Skinshifter
// ---------------------------------------------------------------------------------------

/// Activates Skinshifter's ability choosing mode `mode` (0: Rhino, 1: Bird, 2: Plant).
fn skinshift(t: &mut TestGame, s: ObjectId, mode: usize) {
    pool(t, P0, ManaType::G, 1);
    crate::r_s29_common::choose_modes(t, P0, &[mode]);
    activate_and_resolve(t, P0, s, "Choose one");
}

#[test]
fn skinshifter_loses_its_creature_types_but_keeps_card_types_and_abilities() {
    cr!("205.1a", "613.1d", "613.4b");
    ruling!(
        "Skinshifter",
        "No matter which mode you choose, Skinshifter will cease to have any creature types it previously had."
    );
    ruling!(
        "Skinshifter",
        "No matter which mode you choose, Skinshifter will retain any other card types it may have."
    );
    ruling!(
        "Skinshifter",
        "Skinshifter's ability doesn't cause it to lose any other abilities it may have."
    );
    supported("Liquimetal Coating");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skinshifter");
    let lc = t.battlefield(P0, "Liquimetal Coating");
    t.answer_targets(P0, &[obj(s)]);
    activate_and_resolve(&mut t, P0, lc, "becomes an artifact");
    crate::r_s29_common::cast_and_resolve(&mut t, P0, "Jump", &[obj(s)]);
    skinshift(&mut t, s, 0);
    assert!(is_a(&mut t, s, "Rhino"));
    assert!(!is_a(&mut t, s, "Human"));
    assert!(!is_a(&mut t, s, "Shaman"));
    assert!(t.obj_now(s).is(CardType::Artifact));
    assert!(t.obj_now(s).is(CardType::Creature));
    assert_eq!(t.pt(s), (4, 4));
    assert!(has(&mut t, s, KeywordKind::Trample));
    assert!(flies(&mut t, s), "it keeps the flying it had");
}

#[test]
fn a_copy_of_a_shifted_skinshifter_is_a_1_1_human_shaman() {
    cr!("707.2", "613.1d");
    ruling!(
        "Skinshifter",
        "Skinshifter's ability doesn't affect its copiable values."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skinshifter");
    skinshift(&mut t, s, 0);
    let c = enter_copying(&mut t, P0, "Clone", Some(s));
    assert_eq!(t.pt(c), (1, 1));
    assert!(is_a(&mut t, c, "Human"));
    assert!(is_a(&mut t, c, "Shaman"));
    assert!(!is_a(&mut t, c, "Rhino"));
}
