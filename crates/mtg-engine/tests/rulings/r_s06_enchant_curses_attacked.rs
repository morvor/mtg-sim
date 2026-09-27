//! Rulings batch S06 — the Curses of Commander 2017: "Whenever enchanted player is
//! attacked, [instruction]. Each opponent attacking that player does the same." (Curse of
//! Bounty: "Each opponent attacking that player untaps all nonland permanents they
//! control.") The ability triggers once per combat in which the enchanted player is
//! attacked (CR 508.3b); whether an opponent is attacking that player is checked as it
//! resolves.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A game of `n` players where P0 controls the Curse `name` attached to `enchanted`, and
/// `attacker` is the active player (in their precombat main phase).
fn cursed(n: usize, name: &str, enchanted: PlayerId, attacker: PlayerId) -> (TestGame, ObjectId) {
    supported(name);
    let mut t = TestGame::new(n);
    let curse = attach_new(&mut t, P0, name, Entity::Player(enchanted));
    t.set_step(attacker, Step::PrecombatMain);
    (t, curse)
}

/// Tokens with the subtype `name` ("Gold") that `p` controls.
fn tokens_of_type(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(name))
        .count()
}

/// Zombie tokens `p` controls.
fn zombies(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype("Zombie"))
        .count()
}

// ---------------------------------------------------------------------------------------
// Attacking only a planeswalker of the enchanted player
// ---------------------------------------------------------------------------------------

#[test]
fn curse_of_vitality_doesnt_trigger_if_only_a_planeswalker_is_attacked() {
    cr!("508.3b", "506.2");
    ruling!(
        "Curse of Vitality",
        "The triggered ability of these Curses won’t trigger if only a planeswalker controlled by the enchanted player is attacked."
    );
    // "Whenever enchanted player is attacked, you gain 2 life. Each opponent attacking that
    // player does the same."
    let (mut t, _) = cursed(3, "Curse of Vitality", P1, P2);
    let lili = t.battlefield(P1, "Liliana of the Veil");
    let bears = t.battlefield(P2, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Object(lili))]);
    assert_eq!(on_stack(&t, "gain 2 life"), 0);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P2)), (20, 20));
    // Attacking the player too: it triggers, and the attacking opponent gains life too.
    let (mut t, _) = cursed(3, "Curse of Vitality", P1, P2);
    let lili = t.battlefield(P1, "Liliana of the Veil");
    let bears = t.battlefield(P2, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    attack_with(
        &mut t,
        &[
            (bears, Entity::Object(lili)),
            (giant, Entity::Player(P1)),
        ],
    );
    assert_eq!(on_stack(&t, "gain 2 life"), 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (22, 20, 22));
}

#[test]
fn curse_of_opulence_doesnt_trigger_if_only_a_planeswalker_is_attacked() {
    cr!("508.3b", "506.2");
    ruling!(
        "Curse of Opulence",
        "The triggered ability of these Curses won't trigger if only a planeswalker controlled by the enchanted player is attacked."
    );
    // "Whenever enchanted player is attacked, create a Gold token. Each opponent attacking
    // that player does the same."
    let (mut t, _) = cursed(3, "Curse of Opulence", P1, P2);
    let lili = t.battlefield(P1, "Liliana of the Veil");
    let bears = t.battlefield(P2, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Object(lili))]);
    t.resolve_all();
    assert_eq!(tokens_of_type(&t, P0, "Gold"), 0);
    assert_eq!(tokens_of_type(&t, P2, "Gold"), 0);
    let (mut t, _) = cursed(3, "Curse of Opulence", P1, P2);
    let bears = t.battlefield(P2, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(tokens_of_type(&t, P0, "Gold"), 1);
    assert_eq!(tokens_of_type(&t, P1, "Gold"), 0);
    assert_eq!(tokens_of_type(&t, P2, "Gold"), 1);
}

// ---------------------------------------------------------------------------------------
// Enchanting yourself
// ---------------------------------------------------------------------------------------

#[test]
fn curse_of_disturbance_on_yourself_works_when_another_player_attacks_you() {
    cr!("303.4a", "508.3b");
    ruling!(
        "Curse of Disturbance",
        "If you enchant yourself with one of these Curses, you’ll get its effects whenever another player attacks you."
    );
    // "Whenever enchanted player is attacked, create a 2/2 black Zombie creature token.
    // Each opponent attacking that player does the same."
    let (mut t, _) = cursed(2, "Curse of Disturbance", P0, P1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(zombies(&t, P0), 1);
    assert_eq!(zombies(&t, P1), 1);
}

#[test]
fn curse_of_verbosity_on_yourself_works_when_another_player_attacks_you() {
    cr!("303.4a", "508.3b");
    ruling!(
        "Curse of Verbosity",
        "If you enchant yourself with one of these Curses, you'll get its effects whenever another player attacks you."
    );
    // "Whenever enchanted player is attacked, you draw a card. Each opponent attacking
    // that player does the same."
    let (mut t, _) = cursed(2, "Curse of Verbosity", P0, P1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1 + 1);
}

// ---------------------------------------------------------------------------------------
// Once per combat
// ---------------------------------------------------------------------------------------

#[test]
fn curse_of_bounty_triggers_once_however_many_creatures_attack() {
    cr!("508.3b", "603.2c");
    ruling!(
        "Curse of Bounty",
        "Each Curse’s ability triggers only once, no matter how many creatures are attacking the enchanted player."
    );
    // "Whenever enchanted player is attacked, untap all nonland permanents you control.
    // Each opponent attacking that player untaps all nonland permanents they control."
    let (mut t, _) = cursed(3, "Curse of Bounty", P1, P2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(mine);
    let my_land = t.battlefield(P0, "Forest");
    t.g.tap(my_land);
    let attackers: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P2, "Grizzly Bears")).collect();
    let decl: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    attack_with(&mut t, &decl);
    assert!(attackers.iter().all(|a| t.obj_now(*a).tapped));
    assert_eq!(on_stack(&t, "untap all nonland permanents"), 1);
    t.resolve_all();
    // P0's creature untaps (not its land); the attacking opponent's attackers untap and
    // keep attacking.
    assert!(!t.obj_now(mine).tapped);
    assert!(t.obj_now(my_land).tapped);
    assert!(attackers.iter().all(|a| !t.obj_now(*a).tapped));
    assert!(attackers.iter().all(|a| t.g.is_attacking(*a)));
}

#[test]
fn curse_of_verbosity_triggers_once_however_many_creatures_attack() {
    cr!("508.3b", "603.2c");
    ruling!(
        "Curse of Verbosity",
        "Each Curse's ability triggers only once, no matter how many creatures are attacking the enchanted player."
    );
    let (mut t, _) = cursed(3, "Curse of Verbosity", P1, P2);
    let a = t.battlefield(P2, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    let (h0, h1, h2) = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1);
    assert_eq!(t.hand_size(P2), h2 + 1);
}

// ---------------------------------------------------------------------------------------
// Checked on resolution
// ---------------------------------------------------------------------------------------

#[test]
fn with_no_creature_attacking_on_resolution_only_the_controller_gains_life() {
    cr!("506.2", "608.2h");
    ruling!(
        "Curse of Vitality",
        "A player is attacking another player if they control a creature that’s attacking that player. If no creatures are attacking the enchanted player as a Curse’s ability resolves (most likely because they’ve left the battlefield), only the Curse’s controller performs its actions."
    );
    let (mut t, _) = cursed(3, "Curse of Vitality", P1, P2);
    let bears = t.battlefield(P2, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(on_stack(&t, "gain 2 life"), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P2)), (22, 20));
}

#[test]
fn with_no_creature_attacking_on_resolution_only_the_controller_gets_gold() {
    cr!("506.2", "608.2h");
    ruling!(
        "Curse of Opulence",
        "A player is attacking another player if they control a creature that's attacking that player. If no creatures are attacking the enchanted player as a Curse's ability resolves (most likely because they've left the battlefield), only the Curse's controller performs its actions."
    );
    let (mut t, _) = cursed(3, "Curse of Opulence", P1, P2);
    let bears = t.battlefield(P2, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens_of_type(&t, P0, "Gold"), 1);
    assert_eq!(tokens_of_type(&t, P2, "Gold"), 0);
    // The Gold token works for its controller: "Sacrifice this token: Add one mana of any
    // color."
    let gold = t
        .g
        .permanents()
        .find(|o| o.controller == P0 && o.chars.has_subtype("Gold"))
        .map(|o| o.id)
        .unwrap();
    assert!(activate_containing(&mut t, P0, gold, "Add").is_ok());
    assert!(!t.g.is_live(gold));
}
