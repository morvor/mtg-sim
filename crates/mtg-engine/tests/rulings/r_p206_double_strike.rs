//! Rulings batch P206 — double strike (CR 702.4) and first strike (CR 702.7): a creature
//! that loses double strike (or first strike) after the first combat damage step deals no
//! regular combat damage; triggers on each combat damage step; trample after the blockers
//! die; effects that affect "creatures you control" lock in the set as they resolve
//! (CR 611.2c), with domain counted then.

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::destroy;
use crate::r_s06_common::{attach_new, has_kw};
use crate::r_s25_common::cast_new;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `attacker` attacks P1 unblocked; the game stops in the first-strike combat damage
/// step after that damage was dealt.
fn to_first_strike_damage(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.settle();
}

#[test]
fn warren_instigator_triggers_in_each_combat_damage_step() {
    cr!("702.4b", "510.4", "603.2");
    ruling!(
        "Warren Instigator",
        "If Warren Instigator attacks an opponent and isn’t blocked, its double strike ability will cause it to deal combat damage to that opponent twice, once during each combat damage step. Its triggered ability will thus trigger twice: Warren Instigator deals first-strike combat damage, its ability triggers and resolves, it deals regular combat damage, and its ability triggers and resolves again."
    );
    supported("Warren Instigator");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Warren Instigator");
    t.hand(P0, "Goblin Piker");
    t.hand(P0, "Goblin Piker");
    to_first_strike_damage(&mut t, w);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.named_on_battlefield("Goblin Piker").len(), 1);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.named_on_battlefield("Goblin Piker").len(), 2);
}

#[test]
fn losing_double_or_first_strike_after_first_strike_damage_means_no_regular_damage() {
    cr!("702.4c", "702.7c", "510.4");
    ruling!(
        "Boldwyr Aggressor",
        "If a creature loses double strike after assigning damage in the first strike combat damage step (due to Boldwyr Aggressor leaving the battlefield, for example), that creature won't assign damage in the normal combat damage step."
    );
    ruling!(
        "Kor Blademaster",
        "If a creature loses double strike after assigning damage in the first strike combat damage step (due to Kor Blademaster leaving the battlefield, for example), that creature won’t assign damage in the normal combat damage step."
    );
    ruling!(
        "Aragorn, Hornburg Hero",
        "If Aragorn, Hornburg Hero leaves the battlefield after first strike damage is dealt but before regular combat damage, creatures you control that dealt first strike damage but no longer have first strike won't also deal regular combat damage (unless they have double strike for some reason)."
    );
    // Boldwyr Aggressor: "Other Giants you control have double strike." Hill Giant (3/3).
    supported("Boldwyr Aggressor");
    let mut t = TestGame::new(2);
    let boldwyr = t.battlefield(P0, "Boldwyr Aggressor");
    let giant = t.battlefield(P0, "Hill Giant");
    to_first_strike_damage(&mut t, giant);
    assert_eq!(t.life(P1), 17);
    destroy(&mut t, boldwyr);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    // Kor Blademaster: "Equipped Warriors you control have double strike." Goblin Piker
    // (a 2/1 Warrior) with Leonin Scimitar (+1/+1).
    supported("Kor Blademaster");
    let mut t = TestGame::new(2);
    let kor = t.battlefield(P0, "Kor Blademaster");
    let piker = t.battlefield(P0, "Goblin Piker");
    attach_new(&mut t, P0, "Leonin Scimitar", piker);
    to_first_strike_damage(&mut t, piker);
    assert_eq!(t.life(P1), 17);
    destroy(&mut t, kor);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
    // Aragorn: attacking creatures you control have first strike.
    supported("Aragorn, Hornburg Hero");
    let mut t = TestGame::new(2);
    let aragorn = t.battlefield(P0, "Aragorn, Hornburg Hero");
    let giant = t.battlefield(P0, "Hill Giant");
    to_first_strike_damage(&mut t, giant);
    assert_eq!(t.life(P1), 17);
    destroy(&mut t, aragorn);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn kwende_gives_double_strike_to_creatures_that_gain_first_strike_later() {
    cr!("702.4a", "613.1f");
    ruling!(
        "Kwende, Pride of Femeref",
        "If a creature you control gains first strike after Kwende has entered the battlefield, that creature also gains double strike."
    );
    supported("Kwende, Pride of Femeref");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kwende, Pride of Femeref");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!has_kw(&t, bears, KeywordKind::DoubleStrike));
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddKeyword(Keyword::new(
            KeywordKind::FirstStrike,
        ))],
    );
    assert!(has_kw(&t, bears, KeywordKind::DoubleStrike));
}

#[test]
fn double_strike_trample_after_the_blocker_died_assigns_all_to_the_player() {
    cr!("702.4b", "702.19d");
    ruling!(
        "Swiftblade Vindicator",
        "If an attacking creature with double strike and trample destroys all of its blocking creatures with first-strike combat damage, all of its normal combat damage is assigned to the player or planeswalker that creature's attacking."
    );
    supported("Swiftblade Vindicator");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Swiftblade Vindicator");
    let elf = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(v, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(elf, v)]);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn terror_of_mount_velus_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Terror of Mount Velus",
        "The triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't gain double strike."
    );
    supported("Terror of Mount Velus");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Terror of Mount Velus");
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::DoubleStrike));
    let later = t.battlefield(P0, "Hill Giant");
    assert!(!has_kw(&t, later, KeywordKind::DoubleStrike));
}

#[test]
fn tromp_the_domains_locks_in_its_bonus_and_its_creatures() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Tromp the Domains",
        "The size of the bonus is determined only as Tromp the Domains resolves. Once that happens, the bonus won't change later in the turn even if the number of basic land types among lands you control changes."
    );
    ruling!(
        "Tromp the Domains",
        "Tromp the Domains affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't gain trample or get a power and toughness boost."
    );
    supported("Tromp the Domains");
    let mut t = TestGame::new(2);
    // Forest (and the Forests and Wastes paying for it): one basic land type... plus an
    // Island: two.
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Tromp the Domains", &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    // More basic land types later: the bonus doesn't change.
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    assert_eq!(t.pt(bears), (4, 4));
    // A creature P0 begins to control later gets nothing.
    let later = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(later), (3, 3));
    assert!(!has_kw(&t, later, KeywordKind::Trample));
}
