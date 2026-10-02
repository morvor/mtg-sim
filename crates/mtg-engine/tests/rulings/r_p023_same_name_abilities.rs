//! Rulings batch P023 — permanents that enter as a copy of a creature and gain an ability
//! that refers to creatures "with the same name as this creature" (Evil Twin, Callidus
//! Assassin, Mocking Doppelganger, Pirated Copy): choosing a token copies the original
//! characteristics the effect that created the token gave it, and the permanent doesn't
//! become a token (CR 707.2, 111.3); choosing something that's copying something else
//! copies what it copies (CR 707.3); the granted ability is part of the copy (CR 707.9a)
//! and "the same name" is the copied name (CR 201.2a).

use crate::r_p023_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s18_common::lands_for;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// --- Evil Twin ----------------------------------------------------------------------------

/// P0's Evil Twin activates "{U}{B}, {T}: Destroy target creature with the same name as
/// this creature" targeting `victim`.
fn twin_destroys(t: &mut TestGame, twin: ObjectId, victim: ObjectId) {
    unsick(t, twin);
    lands_for(t, P0, "{U}{B}");
    t.answer_targets(P0, &[obj(victim)]);
    activate_containing(t, P0, twin, "Destroy target creature").expect("activate");
    t.resolve_all();
    assert!(!t.g.is_live(victim));
}

#[test]
fn evil_twin_copying_a_token() {
    cr!("707.2", "111.3", "707.9a", "201.2a");
    ruling!(
        "Evil Twin",
        "If the chosen creature is a token, Evil Twin copies the original characteristics of that token as stated by the effect that created the token. Evil Twin is not a token in this case."
    );
    supported("Evil Twin");
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P1);
    let twin = enter_copying(&mut t, P0, "Evil Twin", wolf);
    assert_wolf(&t, twin, false, true);
    assert!(!t.obj_now(twin).tapped);
    twin_destroys(&mut t, twin, wolf);
}

#[test]
fn evil_twin_copying_a_clone() {
    cr!("707.3", "707.9a", "201.2a");
    ruling!(
        "Evil Twin",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Evil Twin), then your Evil Twin enters the battlefield as whatever the chosen creature copied."
    );
    supported("Evil Twin");
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P1);
    let twin = enter_copying(&mut t, P0, "Evil Twin", clone);
    assert_angel(&t, twin, true);
    assert!(!t.obj_now(twin).is_token());
    // The Clone is named Serra Angel too.
    twin_destroys(&mut t, twin, clone);
}

// --- Callidus Assassin --------------------------------------------------------------------

/// P0's Callidus Assassin enters tapped as a copy of `what`; its granted enters trigger
/// destroys `victim`.
fn assassin_copies(t: &mut TestGame, what: ObjectId, victim: ObjectId) -> ObjectId {
    supported("Callidus Assassin");
    t.answer_targets(P0, &[obj(victim)]);
    let a = enter_copying(t, P0, "Callidus Assassin", what);
    t.resolve_all();
    assert!(t.obj_now(a).tapped);
    assert!(!t.obj_now(a).is_token());
    assert!(!t.g.is_live(victim));
    a
}

#[test]
fn callidus_assassin_copying_a_token() {
    cr!("707.2", "111.3", "707.9a", "201.2a");
    ruling!(
        "Callidus Assassin",
        "If the chosen creature is a token, Callidus Assassin copies the original characteristics of that token as stated by the effect that created the token. Callidus Assassin is not a token in this case."
    );
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P1);
    let a = assassin_copies(&mut t, wolf, wolf);
    assert!(is_wolf(&t, a));
    assert_eq!(t.pt(a), (2, 2));
}

#[test]
fn callidus_assassin_copying_a_clone() {
    cr!("707.3", "707.9a", "201.2a");
    ruling!(
        "Callidus Assassin",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Callidus Assassin), then your Callidus Assassin enters the battlefield as whatever the chosen creature copied."
    );
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P1);
    let a = assassin_copies(&mut t, clone, clone);
    assert_angel(&t, a, true);
}

// --- Mocking Doppelganger -----------------------------------------------------------------

#[test]
fn mocking_doppelganger_copying_a_token() {
    cr!("707.2", "111.3", "707.9a", "701.15b");
    ruling!(
        "Mocking Doppelganger",
        "If the chosen creature is a token, Mocking Doppelganger copies the original characteristics of that token, except for the added ability. Mocking Doppelganger doesn't become a token in this case."
    );
    supported("Mocking Doppelganger");
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P1);
    let d = enter_copying(&mut t, P0, "Mocking Doppelganger", wolf);
    assert_wolf(&t, d, false, true);
    // The Wolf token (named Wolf, like the Doppelganger) is goaded: it must attack.
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[], &[]);
    assert_eq!(t.life(P0), 20 - 7);
}

#[test]
fn mocking_doppelganger_copying_a_clone() {
    cr!("707.3", "707.9a", "701.15b");
    ruling!(
        "Mocking Doppelganger",
        "If the chosen creature is copying something else, Mocking Doppelganger will use the copiable values of the chosen creature. In most cases, it will be a copy of whatever the chosen creature is copying."
    );
    supported("Mocking Doppelganger");
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P1);
    unsick(&mut t, clone);
    let d = enter_copying(&mut t, P0, "Mocking Doppelganger", clone);
    assert_angel(&t, d, true);
    assert!(!t.obj_now(d).is_token());
    // P1's Serra Angel and its Clone (both named Serra Angel) are goaded.
    let angel = t.named_on_battlefield("Serra Angel");
    assert_eq!(angel.len(), 3);
    for a in angel {
        unsick(&mut t, a);
    }
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[], &[]);
    assert_eq!(t.life(P0), 20 - 8);
}

// --- Pirated Copy -------------------------------------------------------------------------

/// P0's Pirated Copy (a copy of something) deals combat damage to P1 and P0 draws a card.
fn pirated_copy_connects(t: &mut TestGame, pc: ObjectId, damage: i32) {
    assert!(has_subtype(t, pc, "Pirate"));
    assert!(!t.obj_now(pc).is_token());
    unsick(t, pc);
    t.library_top(P0, "Island");
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(pc, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - damage);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn pirated_copy_copying_a_token() {
    cr!("707.2", "111.3", "707.9a", "707.9b");
    ruling!(
        "Pirated Copy",
        "If the chosen creature is a token, Pirated Copy copies the original characteristics of that token as stated by the effect that put the token onto the battlefield. Pirated Copy is not a token, even when copying one."
    );
    supported("Pirated Copy");
    let mut t = TestGame::new(2);
    let wolf = dressed_wolf(&mut t, P1);
    let pc = enter_copying(&mut t, P0, "Pirated Copy", wolf);
    assert!(is_wolf(&t, pc));
    assert_eq!(t.pt(pc), (2, 2));
    pirated_copy_connects(&mut t, pc, 2);
}

#[test]
fn pirated_copy_copying_a_clone() {
    cr!("707.3", "707.9a", "707.9b");
    ruling!(
        "Pirated Copy",
        "If the chosen creature is copying something else, then Pirated Copy enters the battlefield as whatever the chosen creature is copying. It will also be a Pirate and have the granted ability."
    );
    supported("Pirated Copy");
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P1);
    let pc = enter_copying(&mut t, P0, "Pirated Copy", clone);
    assert_angel(&t, pc, true);
    assert!(t.obj_now(pc).has_keyword(KeywordKind::Vigilance));
    pirated_copy_connects(&mut t, pc, 4);
}

#[test]
fn pirated_copys_ability_triggers_only_for_creatures_with_its_name() {
    cr!("201.2a", "707.9a");
    supported("Pirated Copy");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Runeclaw Bear");
    let pc = enter_copying(&mut t, P0, "Pirated Copy", bears);
    assert_eq!(t.obj_now(pc).chars.name, "Grizzly Bears");
    unsick(&mut t, bears);
    unsick(&mut t, other);
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    let hand = t.hand_size(P0);
    // Pirated Copy stays home: the other Grizzly Bears triggers it, Runeclaw Bear doesn't.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(bears, Entity::Player(P1)), (other, Entity::Player(P1))],
        &[],
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), hand + 1);
}
