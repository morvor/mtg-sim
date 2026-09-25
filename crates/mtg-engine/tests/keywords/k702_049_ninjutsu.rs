//! CR 702.49 Ninjutsu.

use crate::common_k702_011_017::{assert_supported, attack_with};
use crate::common_k702_018_026::declare_blocks;
use mtg_engine::kw::ninjutsu::revealed_cards;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn attack_target(t: &TestGame, id: ObjectId) -> Option<Entity> {
    t.g.combat
        .as_ref()
        .and_then(|c| c.attack_target(t.g.current(id)))
}

/// Attacks with `attacker` (at `target`) with no blocks, stopping in the declare blockers
/// step with P0 holding priority.
fn unblocked_attack(t: &mut TestGame, attacker: ObjectId, target: Entity) {
    attack_with(t, &[(attacker, target)]);
    let dp = match target {
        Entity::Player(p) => p,
        Entity::Object(o) => t.g.obj(o).controller,
    };
    declare_blocks(t, dp, &[]);
}

#[test]
fn ninjutsu_swaps_an_unblocked_attacker_for_the_ninja() {
    cr!("702.49", "702.49a");
    assert_supported("Ninja of the Deep Hours");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    unblocked_attack(&mut t, bears, Entity::Player(P1));
    // {1}{U}, reveal, return the unblocked Bears: the Ninja enters tapped and attacking.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, ninja, 0, &[]).unwrap();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    t.resolve_all();
    let n = t.named_on_battlefield("Ninja of the Deep Hours")[0];
    assert!(t.obj_now(n).tapped);
    assert_eq!(attack_target(&t, n), Some(Entity::Player(P1)));
    // It deals combat damage; its trigger draws a card.
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn ninjutsu_abilities_can_be_made_cheaper() {
    cr!("702.49a");
    assert_supported("Silver-Fur Master");
    let mut t = TestGame::new(2);
    // Silver-Fur Master: "Ninjutsu abilities you activate cost {1} less to activate."
    t.battlefield(P0, "Silver-Fur Master");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    unblocked_attack(&mut t, bears, Entity::Player(P1));
    // Ninjutsu {1}{U} costs {U}.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, ninja, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ninja of the Deep Hours").len(), 1);
}

#[test]
fn only_an_unblocked_attacker_can_be_returned() {
    cr!("702.49a");
    ruling!(
        "Ninja of the Deep Hours",
        "The ninjutsu ability can be activated only after blockers have been declared. Before then, attacking creatures are neither blocked nor unblocked."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    // Declare attackers step: not yet unblocked.
    assert!(t.activate(P0, ninja, 0, &[]).is_err());
    // Blocked: not unblocked either.
    declare_blocks(&mut t, P1, &[(blocker, bears)]);
    assert!(t.activate(P0, ninja, 0, &[]).is_err());
    assert!(t.in_hand(P0, "Ninja of the Deep Hours"));
}

#[test]
fn the_ninja_wasnt_declared_as_an_attacker() {
    cr!("702.49a");
    ruling!(
        "Ninja of the Deep Hours",
        "Although the Ninja is attacking, it was never declared as an attacking creature"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    unblocked_attack(&mut t, bears, Entity::Player(P1));
    t.activate(P0, ninja, 0, &[]).unwrap();
    t.resolve_all();
    let n = t.named_on_battlefield("Ninja of the Deep Hours")[0];
    let info = t.g.combat.as_ref().unwrap().attacker(n).unwrap().clone();
    assert!(!info.declared);
    assert!(!info.blocked);
}

#[test]
fn the_card_stays_revealed_until_the_ability_leaves_the_stack() {
    cr!("702.49b");
    ruling!(
        "Moon-Circuit Hacker",
        "The Ninja card stays revealed and isn't put onto the battlefield until the ability resolves."
    );
    ruling!(
        "Ninja of the Deep Hours",
        "The Ninja isn't put onto the battlefield until the ability resolves. If it leaves your hand before then, it won't enter the battlefield at all."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    unblocked_attack(&mut t, bears, Entity::Player(P1));
    t.activate(P0, ninja, 0, &[]).unwrap();
    assert_eq!(revealed_cards(&t.g), vec![ninja]);
    // The Ninja is discarded in response: the ability does nothing.
    t.g.discard(P0, ninja, None);
    assert!(revealed_cards(&t.g).is_empty());
    t.resolve_all();
    assert!(t.named_on_battlefield("Ninja of the Deep Hours").is_empty());
    assert!(t.in_graveyard(P0, "Ninja of the Deep Hours"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn the_ninja_attacks_what_the_returned_creature_was_attacking() {
    cr!("702.49c");
    ruling!(
        "Ninja of the Deep Hours",
        "The creature put onto the battlefield with ninjutsu enters the battlefield attacking the same player or planeswalker that the returned creature was attacking."
    );
    // Three players: the Bears attack P2.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    unblocked_attack(&mut t, bears, Entity::Player(P2));
    t.activate(P0, ninja, 0, &[]).unwrap();
    t.resolve_all();
    let n = t.named_on_battlefield("Ninja of the Deep Hours")[0];
    assert_eq!(attack_target(&t, n), Some(Entity::Player(P2)));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P2), 18);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn the_ninja_attacks_the_same_planeswalker() {
    cr!("702.49c");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    unblocked_attack(&mut t, bears, Entity::Object(jace));
    t.activate(P0, ninja, 0, &[]).unwrap();
    t.resolve_all();
    let n = t.named_on_battlefield("Ninja of the Deep Hours")[0];
    assert_eq!(attack_target(&t, n), Some(Entity::Object(jace)));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(jace).loyalty(), 1);
}

#[test]
fn ninjutsu_during_the_first_strike_damage_step() {
    cr!("702.49a");
    ruling!(
        "Ninja of the Deep Hours",
        "If a creature in combat has first strike or double strike, you can activate the ninjutsu ability during the first-strike combat damage step. The Ninja will deal combat damage during the regular combat damage step in this case, even if it has first strike."
    );
    let mut t = TestGame::new(2);
    // Kitsune Blademaster: 2/2 first strike.
    let fs = t.battlefield(P0, "Kitsune Blademaster");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    unblocked_attack(&mut t, fs, Entity::Player(P1));
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    t.activate(P0, ninja, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn ninjutsu_after_combat_damage_deals_no_damage() {
    cr!("702.49a");
    ruling!(
        "Ninja of the Deep Hours",
        "The ninjutsu ability can be activated during the declare blockers step, combat damage step, or end of combat step. If you wait until after the declare blockers step, because all combat damage is dealt at once, the Ninja won't normally deal combat damage."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    unblocked_attack(&mut t, bears, Entity::Player(P1));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    // The Bears are still an unblocked attacking creature in the end of combat step.
    let hand = t.hand_size(P0);
    t.activate(P0, ninja, 0, &[]).unwrap();
    t.resolve_all();
    let n = t.named_on_battlefield("Ninja of the Deep Hours")[0];
    assert!(t.g.is_attacking(n));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 18);
    // The Bears went back to hand; no card was drawn.
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn commander_ninjutsu_works_from_the_command_zone() {
    cr!("702.49d");
    ruling!(
        "Yuriko, the Tiger's Shadow",
        "You won’t have to pay the commander tax to activate that ability, and activating that ability won’t increase the commander tax to pay later."
    );
    let mut t = TestGame::new(2);
    let yuriko = t.command(P0, "Yuriko, the Tiger's Shadow");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    unblocked_attack(&mut t, bears, Entity::Player(P1));
    // {U}{B}, return the Bears. (Its first activated ability functions from the hand, the
    // second from the command zone.)
    assert!(t.activate(P0, yuriko, 0, &[]).is_err());
    t.clear_answers();
    t.activate(P0, yuriko, 1, &[]).unwrap();
    t.resolve_all();
    let y = t.named_on_battlefield("Yuriko, the Tiger's Shadow")[0];
    assert_eq!(attack_target(&t, y), Some(Entity::Player(P1)));
    assert!(t.g.player(P0).commander_casts.is_empty());
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn commander_ninjutsu_also_works_from_the_hand() {
    cr!("702.49d");
    let mut t = TestGame::new(2);
    let yuriko = t.hand(P0, "Yuriko, the Tiger's Shadow");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    unblocked_attack(&mut t, bears, Entity::Player(P1));
    t.activate(P0, yuriko, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Yuriko, the Tiger's Shadow").len(), 1);
    assert_eq!(t.zone(bears), Zone::Hand(P0));
}
