//! Rulings batch P030 — a token that's a copy of a creature enters as that creature: its
//! "enters" triggered abilities trigger and its "enters with" and "as [this] enters"
//! abilities apply (CR 707.2, 111.4, 614.1c, 603.6a); tokens created tapped and attacking
//! were never declared as attackers, so "whenever [a creature] attacks" abilities and costs
//! to attack don't apply to them (CR 508.3a, 508.4, 506.3).

use crate::r_p030_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Asserts the token copy of Elvish Visionary ("When this creature enters, draw a card.")
/// drew a card, and the token copy of Chronozoa ("Vanishing 3") entered with three time
/// counters.
fn assert_entered_as_copies(t: &TestGame, hand_before: usize, hand_spent: usize) {
    assert_eq!(t.hand_size(P0), hand_before - hand_spent + 1);
    let zoa = tokens_named(t, P0, "Chronozoa");
    assert_eq!(zoa.len(), 1);
    assert_eq!(t.counters(zoa[0], counters::TIME), 3);
    assert_eq!(tokens_named(t, P0, "Elvish Visionary").len(), 1);
}

#[test]
fn fated_infatuation_s_token_enters_as_the_copied_creature() {
    cr!("707.2", "603.6a", "614.1c");
    ruling!(
        "Fated Infatuation",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the copied creature will also work."
    );
    supported("Fated Infatuation");
    // "Create a token that's a copy of target creature you control."
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Elvish Visionary");
    let zoa = t.battlefield(P0, "Chronozoa");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Fated Infatuation", &[obj(elf)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Fated Infatuation", &[obj(zoa)]);
    t.resolve_all();
    assert_entered_as_copies(&t, hand, 0);
}

#[test]
fn clone_legion_s_tokens_enter_as_the_copied_creatures() {
    cr!("707.2", "603.6a", "614.1c");
    ruling!(
        "Clone Legion",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this permanent] enters the battlefield\" or \"[this permanent] enters the battlefield with\" abilities of the copied creature will also work."
    );
    supported("Clone Legion");
    // "For each creature target player controls, create a token that's a copy of that
    // creature."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Elvish Visionary");
    t.battlefield(P1, "Chronozoa");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Clone Legion", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_entered_as_copies(&t, hand, 0);
}

// --- Tokens that enter attacking ---------------------------------------------------------------

/// A three-player game; P0 controls Druids' Repository ("Whenever a creature you control
/// attacks, put a charge counter on this enchantment.") and P2 controls Propaganda
/// ("Creatures can't attack you unless their controller pays {2} for each creature they
/// control that's attacking you.").
fn three_player_game() -> (TestGame, ObjectId) {
    supported("Druids' Repository");
    supported("Propaganda");
    let mut t = TestGame::new(3);
    let repo = t.battlefield(P0, "Druids' Repository");
    t.battlefield(P2, "Propaganda");
    (t, repo)
}

/// The creatures attacking P2.
fn attacking_p2(t: &TestGame) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| t.g.combat.as_ref().is_some_and(|c| c.attack_target(o.id) == Some(Entity::Player(P2))))
        .map(|o| o.id)
        .collect()
}

/// `creature` (with myriad) attacks P1; P0 creates the myriad token attacking P2. Checks
/// that only the declared attacker triggered "whenever a creature you control attacks",
/// that the token's own myriad didn't trigger, and that P0 paid nothing for Propaganda.
fn myriad_attack(t: &mut TestGame, repo: ObjectId, creature: ObjectId, name: &str) {
    let untapped_lands = t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    attack_with(t, &[(creature, Entity::Player(P1))]);
    t.resolve_all();
    let toks = tokens_named(t, P0, name);
    assert_eq!(toks.len(), 1, "one myriad token (the token's myriad didn't trigger)");
    assert_eq!(attacking_p2(t), toks);
    assert_eq!(t.counters(repo, "charge"), 1);
    assert!(untapped_lands.iter().all(|l| !t.obj_now(*l).tapped));
}

#[test]
fn goldlust_triad_s_myriad_tokens_werent_declared_as_attackers() {
    cr!("702.116a", "508.4", "508.3a", "508.1h");
    ruling!(
        "Goldlust Triad",
        "Although the tokens enter attacking, they were never declared as attackers. Abilities that trigger whenever a creature attacks won’t trigger, including the myriad ability of the tokens. If there are any costs to have a creature attack, those costs won’t apply to the tokens."
    );
    supported("Goldlust Triad");
    let (mut t, repo) = three_player_game();
    let triad = t.battlefield(P0, "Goldlust Triad");
    myriad_attack(&mut t, repo, triad, "Goldlust Triad");
}

#[test]
fn dalek_squadron_s_myriad_tokens_werent_declared_as_attackers() {
    cr!("702.116a", "508.4", "508.3a", "508.1h");
    ruling!(
        "Dalek Squadron",
        "Although the tokens enter the battlefield attacking, they were never declared as attackers. Abilities that trigger whenever a creature attacks won't trigger, including the myriad ability of the tokens. If there any costs to have a creature attack, those costs won't apply to the tokens."
    );
    supported("Dalek Squadron");
    let (mut t, repo) = three_player_game();
    let dalek = t.battlefield(P0, "Dalek Squadron");
    myriad_attack(&mut t, repo, dalek, "Dalek Squadron");
}

#[test]
fn warren_warleader_s_rabbit_wasnt_declared_as_an_attacker() {
    cr!("508.4", "508.3a");
    ruling!(
        "Warren Warleader",
        "Although the token enters attacking, it was never declared as an attacking creature. Abilities that trigger whenever a creature attacks won’t trigger when that creature enters attacking."
    );
    supported("Warren Warleader");
    // "Whenever you attack, choose one — • Create a 1/1 white Rabbit creature token that's
    // tapped and attacking. • ..."
    let mut t = TestGame::new(2);
    let repo = t.battlefield(P0, "Druids' Repository");
    let ww = t.battlefield(P0, "Warren Warleader");
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![0]),
    );
    attack_with(&mut t, &[(ww, Entity::Player(P1))]);
    t.resolve_all();
    let rabbits = crate::r_s01_common::with_subtype(&t, P0, "Rabbit");
    let rabbits: Vec<ObjectId> = rabbits.into_iter().filter(|r| *r != ww).collect();
    assert_eq!(rabbits.len(), 1);
    assert!(t.g.combat.as_ref().unwrap().attack_target(rabbits[0]).is_some());
    assert_eq!(t.counters(repo, "charge"), 1);
}

#[test]
fn a_clone_of_a_creature_with_vanishing_enters_with_time_counters() {
    cr!("614.12", "707.9", "702.63a");
    // Regression: the "enters with" ability a copied keyword stands for (vanishing) applies
    // to a permanent entering as a copy.
    let mut t = TestGame::new(2);
    let zoa = t.battlefield(P1, "Chronozoa");
    t.answer_choose(P0, &[obj(zoa)]);
    let c = t.enter(P0, "Clone");
    t.settle();
    let c = t.g.current(c);
    assert_eq!(t.obj(c).chars.name, "Chronozoa");
    assert_eq!(t.counters(c, counters::TIME), 3);
}
