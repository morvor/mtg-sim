//! Rulings batch P057 — "power 4 or greater" in combat: attack and block restrictions are
//! checked only as attackers or blockers are declared (CR 508.1c, 509.1b), so a creature
//! that has attacked or blocked stays in combat when the condition stops holding (CR
//! 506.4); "can't be blocked by more than one creature" with menace; Temur Ascendancy's
//! haste; Warbeast of Gorgoroth's simultaneous deaths; Bitter Work's one card per player;
//! Bonders' Enclave's activation restriction.

use crate::r_p057_common::*;
use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s02_common::{can_attack, destroy};
use crate::r_s03_common::to_blockers;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::{count_asked, is_priority};
use crate::r_s09_common::legal_attack;
use crate::r_s10_common::{attacking, blocking};
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::cast_new;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BIG: &str = "Rumbling Baloth"; // vanilla 4/4
const SMALL: &str = "Grizzly Bears"; // vanilla 2/2

fn at_p1(id: ObjectId) -> (ObjectId, Entity) {
    (id, Entity::Player(P1))
}

#[test]
fn defenders_that_can_attack_with_power_4_count_themselves_and_stay_attacking() {
    cr!("508.1c", "506.4", "702.3b");
    ruling!(
        "Drowsing Tyrannodon",
        "If Drowsing Tyrannodon's power is raised to 4 or greater, it will allow itself to attack."
    );
    ruling!(
        "Drowsing Tyrannodon",
        "Once Drowsing Tyrannodon has attacked, it will remain an attacking creature even if you no longer control a creature with power 4 or greater."
    );
    ruling!(
        "Bristlepack Sentry",
        "Once Bristlepack Sentry has legally attacked, causing its last ability to not apply by removing other creatures from the battlefield or reducing the power of one or more creatures won’t cause Bristlepack Sentry to stop attacking."
    );
    supported("Drowsing Tyrannodon");
    supported("Bristlepack Sentry");
    // Drowsing Tyrannodon (3/3, defender): "As long as you control a creature with power 4
    // or greater, this creature can attack as though it didn't have defender."
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Drowsing Tyrannodon");
    to_beginning_of_combat(&mut t, P0);
    assert!(!can_attack(&mut t, d));
    pump(&mut t, d, 1, 0);
    assert!(can_attack(&mut t, d));
    attack_with(&mut t, &[at_p1(d)]);
    assert!(attacking(&t, d));
    // Its power is lowered again: it remains attacking and deals its damage.
    pump(&mut t, d, -1, 0);
    assert!(attacking(&t, d));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    // Bristlepack Sentry (3/3, defender, the same ability) with a Baloth that then dies.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Bristlepack Sentry");
    let b = t.battlefield(P0, BIG);
    to_beginning_of_combat(&mut t, P0);
    assert!(can_attack(&mut t, s));
    attack_with(&mut t, &[at_p1(s)]);
    destroy(&mut t, b);
    assert!(attacking(&t, s));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn lambholt_pacifist_fulfills_its_own_restriction() {
    cr!("508.1c");
    ruling!(
        "Lambholt Pacifist // Lambholt Butcher",
        "If Lambholt Pacifist's power becomes 4 or greater, it fulfills its own restriction and can attack."
    );
    supported("Lambholt Pacifist // Lambholt Butcher");
    // "This creature can't attack unless you control a creature with power 4 or greater."
    // (3/3)
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lambholt Pacifist // Lambholt Butcher");
    to_beginning_of_combat(&mut t, P0);
    assert!(!can_attack(&mut t, l));
    pump(&mut t, l, 1, 0);
    assert!(can_attack(&mut t, l));
}

#[test]
fn two_wardens_of_the_chained_enable_each_other_and_stay_attacking() {
    cr!("508.1c", "506.4");
    ruling!(
        "Warden of the Chained",
        "If you control two Wardens of the Chained, each will fulfill the other's condition and both will be able to attack."
    );
    ruling!(
        "Warden of the Chained",
        "Once Warden of the Chained is attacking, it will remain attacking even if you no longer control another creature with power 4 or greater."
    );
    supported("Warden of the Chained");
    // "This creature can't attack unless you control another creature with power 4 or
    // greater." (4/4)
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Warden of the Chained");
    to_beginning_of_combat(&mut t, P0);
    assert!(!can_attack(&mut t, a), "itself doesn't count");
    let b = t.battlefield(P0, "Warden of the Chained");
    assert!(legal_attack(&mut t, &[at_p1(a), at_p1(b)]));
    // One attacks; the other is then destroyed: the attacker keeps attacking.
    attack_with(&mut t, &[at_p1(a)]);
    destroy(&mut t, b);
    assert!(attacking(&t, a));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn rhonas_needs_no_companion_in_combat_and_stays_in_combat() {
    cr!("508.1c", "509.1b", "506.4");
    ruling!(
        "Rhonas the Indomitable",
        "You don't have to attack with another creature with power 4 or greater for Rhonas to be able to attack. The same is true of blocking."
    );
    ruling!(
        "Rhonas the Indomitable",
        "Once Rhonas has attacked or blocked, it will remain in combat even if you no longer control another creature with power 4 or greater."
    );
    supported("Rhonas the Indomitable");
    // "Rhonas can't attack or block unless you control another creature with power 4 or
    // greater." (5/5 deathtouch, indestructible)
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rhonas the Indomitable");
    to_beginning_of_combat(&mut t, P0);
    assert!(!can_attack(&mut t, r));
    let b = t.battlefield(P0, BIG);
    // Rhonas attacks alone; the Baloth stays home and is then destroyed.
    attack_with(&mut t, &[at_p1(r)]);
    assert!(attacking(&t, r));
    destroy(&mut t, b);
    assert!(attacking(&t, r));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
    // Blocking: P1's Bears attack P0; Rhonas blocks alone, the Baloth doesn't block.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rhonas the Indomitable");
    let b = t.battlefield(P0, BIG);
    let bears = t.battlefield(P1, SMALL);
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(bears, Entity::Player(P0))], &[(r, bears)]);
    assert!(blocking(&t, r));
    destroy(&mut t, b);
    assert!(blocking(&t, r));
    t.advance_to(P1, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, SMALL));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn challenger_troll_restricts_itself_and_menace_makes_it_unblockable() {
    cr!("509.1b", "702.111b");
    ruling!(
        "Challenger Troll",
        "Challenger Troll’s ability affects itself as long as its power remains 4 or greater."
    );
    ruling!(
        "Challenger Troll",
        "If a creature you control with power 4 or greater has menace, it can’t be blocked by only one creature and it can’t be blocked by more than one creature, so it simply can’t be blocked."
    );
    supported("Challenger Troll");
    supported("Kozilek's Shrieker");
    // "Each creature you control with power 4 or greater can't be blocked by more than one
    // creature." (6/5)
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Challenger Troll");
    // Kozilek's Shrieker (3/2): "{C}: This creature gets +1/+0 and gains menace until end
    // of turn."
    let shrieker = t.battlefield(P0, "Kozilek's Shrieker");
    add_mana(&mut t, P0, ManaType::C, 1);
    activate_containing(&mut t, P0, shrieker, "menace").expect("activated");
    t.resolve_all();
    assert_eq!(t.pt(shrieker), (4, 2));
    let x = t.battlefield(P1, SMALL);
    let y = t.battlefield(P1, SMALL);
    attack_with(&mut t, &[at_p1(troll), at_p1(shrieker)]);
    t.set_step(P0, Step::DeclareBlockers);
    // The Troll: one blocker yes, two no.
    assert!(legal_blocks(&mut t, P1, &[(x, troll)]));
    assert!(!legal_blocks(&mut t, P1, &[(x, troll), (y, troll)]));
    // The 4-power menace Shrieker: neither one nor two.
    assert!(!legal_blocks(&mut t, P1, &[(x, shrieker)]));
    assert!(!legal_blocks(&mut t, P1, &[(x, shrieker), (y, shrieker)]));
    // The Troll's power lowered to 3: two blockers are fine.
    pump(&mut t, troll, -3, 0);
    assert!(legal_blocks(&mut t, P1, &[(x, troll), (y, troll)]));
}

#[test]
fn challenger_troll_doesnt_undo_blocks_when_power_changes() {
    cr!("509.1b", "506.4");
    ruling!(
        "Challenger Troll",
        "Once a creature you control with power 3 or less has become blocked by two or more creatures, changing its power won’t cause either blocking creature to stop blocking it."
    );
    supported("Challenger Troll");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Challenger Troll");
    let bears = t.battlefield(P0, SMALL);
    let x = t.battlefield(P1, SMALL);
    let y = t.battlefield(P1, SMALL);
    to_blockers(&mut t, &[at_p1(bears)], &[(x, bears), (y, bears)]);
    assert!(blocking(&t, x) && blocking(&t, y));
    pump(&mut t, bears, 2, 0);
    assert!(blocking(&t, x) && blocking(&t, y));
}

#[test]
fn temur_ascendancy_leaving_takes_haste_away_but_attackers_keep_attacking() {
    cr!("302.6", "702.10b", "506.4");
    ruling!(
        "Temur Ascendancy",
        "If Temur Ascendancy leaves the battlefield, creatures you control lose haste."
    );
    supported("Temur Ascendancy");
    // "Creatures you control have haste."
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Temur Ascendancy");
    let a = t.battlefield_sick(P0, SMALL);
    let b = t.battlefield_sick(P0, SMALL);
    to_beginning_of_combat(&mut t, P0);
    assert!(can_attack(&mut t, a) && can_attack(&mut t, b));
    attack_with(&mut t, &[at_p1(a)]);
    destroy(&mut t, asc);
    assert!(attacking(&t, a), "already attacking");
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    // The other one can't attack this turn any more (a second combat).
    t.g.combat = None;
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, b));
    assert!(!can_attack(&mut t, a));
}

#[test]
fn warbeast_of_gorgoroth_triggers_for_each_creature_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Warbeast of Gorgoroth",
        "If Warbeast of Gorgoroth and one or more other creatures you control with power 4 or greater die at the same time, its ability will trigger once for each of them."
    );
    supported("Warbeast of Gorgoroth");
    supported("Wrath of God");
    // "Whenever this creature or another creature you control with power 4 or greater dies,
    // amass Orcs 2."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Warbeast of Gorgoroth");
    t.battlefield(P0, BIG);
    t.battlefield(P0, BIG);
    t.battlefield(P0, SMALL);
    t.battlefield(P1, BIG);
    cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "amass Orcs 2"), 3);
    t.resolve_all();
    let army = crate::r_s01_common::with_subtype(&t, P0, "Army");
    assert_eq!(army.len(), 1);
    assert_eq!(t.counters(army[0], "+1/+1"), 6);
}

#[test]
fn bitter_work_draws_one_card_per_player_attacked() {
    cr!("508.3a", "603.2c");
    ruling!(
        "Bitter Work",
        "Bitter Work's first ability has you draw just one card per player you attack with a creature with power 4 or greater"
    );
    supported("Bitter Work");
    // "Whenever you attack a player with one or more creatures with power 4 or greater,
    // draw a card."
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Bitter Work");
    let a = t.battlefield(P0, BIG);
    let b = t.battlefield(P0, BIG);
    let c = t.battlefield(P0, BIG);
    let d = t.battlefield(P0, SMALL);
    attack_with(
        &mut t,
        &[
            at_p1(a),
            at_p1(b),
            (c, Entity::Player(P2)),
            (d, Entity::Player(P2)),
        ],
    );
    assert_eq!(triggers_on_stack(&t, "draw a card"), 2);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Only small creatures attacking: no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bitter Work");
    let d = t.battlefield(P0, SMALL);
    attack_with(&mut t, &[at_p1(d)]);
    assert_eq!(triggers_on_stack(&t, "draw a card"), 0);
}

#[test]
fn bonders_enclave_checks_only_as_it_is_activated() {
    cr!("602.2", "602.5b", "608.2b");
    ruling!(
        "Bonders' Enclave",
        "Once you announce that you're activating the last ability of Bonders' Enclave, no player may take actions until you've finished activating it."
    );
    ruling!(
        "Bonders' Enclave",
        "Once you've activated the last ability of Bonders' Enclave, it doesn't check again at any point whether you control a creature with power 4 or greater."
    );
    supported("Bonders' Enclave");
    // "{3}, {T}: Draw a card. Activate only if you control a creature with power 4 or
    // greater."
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Bonders' Enclave");
    t.lands(P0, "Wastes", 3);
    assert!(activate_containing(&mut t, P0, e, "Draw a card").is_err());
    let b = t.battlefield(P0, BIG);
    let from = t.asked().len();
    activate_containing(&mut t, P0, e, "Draw a card").expect("activated");
    // No player received priority while it was being activated.
    assert_eq!(count_asked(&t, from, is_priority), 0);
    assert_eq!(t.stack_len(), 1);
    // The Baloth is destroyed in response: the ability still draws.
    destroy(&mut t, b);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}
