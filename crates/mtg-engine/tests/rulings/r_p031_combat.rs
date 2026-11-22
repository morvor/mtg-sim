//! Rulings batch P031 — combat interactions of self-copying cards: myriad (CR 702.116),
//! tokens put onto the battlefield attacking (CR 508.4), blocking requirements (CR 509.1c),
//! "defending player" (CR 506.2), double strike (CR 702.4), lifelink (CR 702.15), and
//! delayed combat damage triggers.

use crate::r_p031_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s12_common::attack_target;
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::{cast_new, targets_of};
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn three_players() -> TestGame {
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::BeginningOfCombat);
    t
}

fn pump(t: &mut TestGame, id: ObjectId, p: i32, x: i32) {
    modify_until_eot(t, id, vec![Modification::ModifyPT(Value::c(p), Value::c(x))]);
}

// --- Myriad ------------------------------------------------------------------------------

#[test]
fn myriad_tokens_doubled_each_choose_what_they_attack() {
    cr!("702.116a", "508.4");
    ruling!(
        "Goldlust Triad",
        "If an instance of myriad creates more than one token for any given player (due to an effect such as the one Doubling Season creates), you may choose separately for each token whether it’s attacking the player or a planeswalker they control."
    );
    supported("Parallel Lives");
    let mut t = three_players();
    t.battlefield(P0, "Parallel Lives");
    let jace = t.battlefield(P2, "Jace Beleren");
    let triad = t.battlefield(P0, "Goldlust Triad");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_choose(P0, &[obj(jace)]);
    let before = t.g.battlefield.clone();
    attack_with(&mut t, &[(triad, Entity::Player(P1))]);
    t.resolve_all();
    let tok = tokens_since(&t, P0, &before);
    assert_eq!(tok.len(), 2);
    let mut targets: Vec<Entity> = tok.iter().filter_map(|id| attack_target(&t, *id)).collect();
    targets.sort();
    let mut want = vec![Entity::Player(P2), obj(jace)];
    want.sort();
    assert_eq!(targets, want);
}

#[test]
fn myriad_s_defending_player_is_determined_on_resolution_or_last_known() {
    cr!("702.116a", "506.2");
    ruling!(
        "Scion of Calamity",
        "The term \"defending player\" in the myriad rules (or any other ability of an attacking creature) refers to the player the creature with myriad was attacking, the controller of the planeswalker it was attacking, or the protector of the battle it was attacking at the time the ability resolves. If that creature is no longer attacking, it refers to the appropriate player based on who or what the creature was last attacking."
    );
    ruling!(
        "Goldlust Triad",
        "The term “defending player” in the myriad rules (or any other ability of an attacking creature) refers to the player the creature with myriad was attacking, the controller of the planeswalker it was attacking, or the protector of the battle it was attacking at the time the ability resolves. If that creature is no longer attacking, it refers to the appropriate player based on who or what the creature was last attacking."
    );
    for name in ["Scion of Calamity", "Goldlust Triad"] {
        supported(name);
        // It attacks P1's planeswalker, then leaves the battlefield before myriad resolves:
        // the defending player is still P1, so the only token attacks P2.
        let mut t = three_players();
        let jace = t.battlefield(P1, "Jace Beleren");
        let c = t.battlefield(P0, name);
        let before = t.g.battlefield.clone();
        attack_with(&mut t, &[(c, obj(jace))]);
        assert_eq!(t.stack_len(), 1);
        kill(&mut t, c);
        t.resolve_all();
        let tok = tokens_since(&t, P0, &before);
        assert_eq!(tok.len(), 1, "{name}");
        assert_eq!(attack_target(&t, tok[0]), Some(Entity::Player(P2)), "{name}");
    }
}

// --- Nacatl War-Pride ---------------------------------------------------------------------

#[test]
fn nacatl_war_pride_that_can_t_be_blocked_by_exactly_one() {
    cr!("509.1c");
    ruling!(
        "Nacatl War-Pride",
        "If it's impossible for Nacatl War-Pride to be blocked by exactly one creature, then the defending player may block it with multiple creatures or may leave it unblocked."
    );
    supported("Nacatl War-Pride");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Nacatl War-Pride");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(w, Entity::Player(P1))]);
    // P1 Stifles the token trigger.
    let trig = *t.g.stack.last().unwrap();
    cast_new(&mut t, P1, "Stifle", &[obj(trig)]);
    t.resolve_all();
    assert!(tokens_since(&t, P0, &[w, a, b]).is_empty());
    // Normally exactly one must block it.
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(a, w)]));
    assert!(!legal_blocks(&mut t, P1, &[(a, w), (b, w)]));
    // With menace it can't be blocked by exactly one: two blockers or none.
    modify_until_eot(
        &mut t,
        w,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Menace))],
    );
    assert!(!legal_blocks(&mut t, P1, &[(a, w)]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(a, w), (b, w)]));
}

#[test]
fn nacatl_war_pride_s_tokens_in_multiplayer() {
    cr!("508.4", "802.2", "802.3");
    ruling!(
        "Nacatl War-Pride",
        "In a multiplayer game in which you're allowed to attack only one player each combat, the new creatures will be attacking the same player as Nacatl War-Pride. In a multiplayer game in which you may attack multiple players, you choose the player each new creature is attacking. It must be a player you're allowed to attack."
    );
    for multiple in [false, true] {
        let mut t = TestGame::with_config(
            3,
            GameConfig {
                attack_multiple_players: multiple,
                ..Default::default()
            },
        );
        t.set_step(P0, Step::BeginningOfCombat);
        let w = t.battlefield(P0, "Nacatl War-Pride");
        t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, "Grizzly Bears");
        // When it may, each token attacks P2.
        t.answer_choose(P0, &[Entity::Player(P2)]);
        t.answer_choose(P0, &[Entity::Player(P2)]);
        let before = t.g.battlefield.clone();
        attack_with(&mut t, &[(w, Entity::Player(P1))]);
        t.resolve_all();
        let tok = tokens_since(&t, P0, &before);
        assert_eq!(tok.len(), 2, "X is the creatures P1 controls");
        let want = if multiple { P2 } else { P1 };
        for id in tok {
            assert_eq!(attack_target(&t, id), Some(Entity::Player(want)), "multiple: {multiple}");
        }
    }
}

// --- Coastline Marauders ------------------------------------------------------------------

#[test]
fn coastline_marauders_counts_the_lands_of_the_player_it_attacks() {
    cr!("506.2", "802.2a");
    ruling!(
        "Coastline Marauders",
        "The bonus Coastline Marauders receives is determined by the player it's attacking. Lands controlled by other defending players won't be counted."
    );
    supported("Coastline Marauders");
    let mut t = three_players();
    t.lands(P1, "Forest", 2);
    t.lands(P2, "Forest", 5);
    let m = t.battlefield(P0, "Coastline Marauders");
    attack_with(&mut t, &[(m, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(m), (2, 3));
}

// --- Rust-Shield Rampager -----------------------------------------------------------------

#[test]
fn rust_shield_rampager_stays_blocked_when_the_blocker_shrinks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Rust-Shield Rampager",
        "Once Rust-Shield Rampager has been blocked, reducing the power of a blocking creature to 2 or less won’t remove that creature from combat or cause Rust-Shield Rampager to become unblocked."
    );
    supported("Rust-Shield Rampager");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rust-Shield Rampager");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(r, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(giant, r)]));
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(crate::r_s21_common::blocks_now(&t).contains(&(giant, r)));
    pump(&mut t, giant, -2, 0);
    assert_eq!(t.pt(giant), (1, 3));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "still blocked");
    assert!(!t.on_battlefield(giant), "4 damage to the Giant");
}

// --- Brass Knuckles -----------------------------------------------------------------------

#[test]
fn two_instances_of_double_strike_still_strike_twice() {
    cr!("702.4e", "702.4b");
    ruling!(
        "Brass Knuckles",
        "Multiple instances of double strike do not increase the number of strikes. That creature still only gets to deal combat damage twice."
    );
    supported("Brass Knuckles");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Brass Knuckles", bears);
    attach_new(&mut t, P0, "Fireshrieker", bears);
    let n = t
        .obj_now(bears)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::DoubleStrike)
        .count();
    assert_eq!(n, 2);
    hit(&mut t, bears);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

// --- Starscape Cleric ---------------------------------------------------------------------

#[test]
fn starscape_cleric_triggers_for_each_lifelink_creature() {
    cr!("702.15e", "119.9");
    ruling!(
        "Starscape Cleric",
        "If multiple creatures with lifelink you control deal combat damage at the same time, the damage dealt by each of those creatures is a separate life-gaining event and Starscape Cleric’s ability will trigger that many times."
    );
    supported("Starscape Cleric");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starscape Cleric");
    let a = t.battlefield(P0, "Child of Night");
    let b = t.battlefield(P0, "Child of Night");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.life(P1), 14);
}

// --- Jace, Cunning Castaway ---------------------------------------------------------------

#[test]
fn jace_s_delayed_trigger_can_trigger_more_than_once() {
    cr!("603.2", "510.4");
    ruling!(
        "Jace, Cunning Castaway",
        "The delayed triggered ability created by Jace's first ability can trigger more than once in a turn if creatures you control deal combat damage at different times in a turn (most likely because one or more has first strike) or if creatures you control deal combat damage to more than one player at once."
    );
    supported("Jace, Cunning Castaway");
    // First strike and regular damage: two loots.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Cunning Castaway");
    activate_containing(&mut t, P0, jace, "combat damage").unwrap();
    t.resolve_all();
    let knight = t.battlefield(P0, "White Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gy = t.graveyard_size(P0);
    attack_with(&mut t, &[(knight, Entity::Player(P1)), (bears, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.graveyard_size(P0), gy + 2, "two loots");
    // Two players dealt damage at once: two loots.
    let mut t = TestGame::new(3);
    let jace = t.battlefield(P0, "Jace, Cunning Castaway");
    activate_containing(&mut t, P0, jace, "combat damage").unwrap();
    t.resolve_all();
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let gy = t.graveyard_size(P0);
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P2))]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!((t.life(P1), t.life(P2)), (18, 18));
    assert_eq!(t.graveyard_size(P0), gy + 2, "two loots");
}

#[test]
fn jace_s_illusion_is_sacrificed_before_the_spell_resolves() {
    cr!("608.2b", "603.3");
    ruling!(
        "Jace, Cunning Castaway",
        "If a token created by Jace's second ability becomes the target of a spell, its ability triggers and goes on the stack on top of that spell. The ability will resolve first (causing the token to be sacrificed). Unless the spell has another target, it will then be countered when it tries to resolve for having no legal targets."
    );
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Cunning Castaway");
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, jace, "Illusion").unwrap();
    t.resolve_all();
    let illusion = tokens_since(&t, P0, &before)[0];
    let shock = cast_new(&mut t, P1, "Shock", &[obj(illusion)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), shock, "the trigger is on top");
    t.resolve();
    assert!(!t.on_battlefield(illusion));
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.life(P0), 20);
}

// --- Earthshaker Khenra -------------------------------------------------------------------

/// The trigger of the Earthshaker Khenra that just entered under P0's control; queues
/// `want` as its target.
fn khenra_enters(t: &mut TestGame, want: ObjectId) -> (ObjectId, ObjectId) {
    t.answer_targets(P0, &[obj(want)]);
    let k = t.enter(P0, "Earthshaker Khenra");
    t.g.flush_events();
    t.settle();
    (k, *t.g.stack.last().unwrap())
}

#[test]
fn earthshaker_khenra_s_power_counts_static_abilities_when_targeting() {
    cr!("115.1", "603.3d", "613.4c");
    ruling!(
        "Earthshaker Khenra",
        "Spells, activated abilities, and triggered abilities can't raise Earthshaker Khenra's power before you have to choose a target for its ability. Static abilities, such as that of Lord of the Accursed, will raise its power in time to let you target a larger creature."
    );
    supported("Earthshaker Khenra");
    for anthem in [false, true] {
        let mut t = TestGame::new(2);
        if anthem {
            t.battlefield(P0, "Glorious Anthem");
        }
        let giant = t.battlefield(P1, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let (_, trig) = khenra_enters(&mut t, giant);
        // A 3/2 with the Anthem can target the Hill Giant; a 2/1 can't (the Bears are the
        // only legal target).
        let want = if anthem { giant } else { bears };
        assert_eq!(targets_of(&t, trig), vec![obj(want)], "anthem: {anthem}");
    }
}

#[test]
fn earthshaker_khenra_checks_power_on_targeting_and_resolution_only() {
    cr!("608.2b", "611.2c");
    ruling!(
        "Earthshaker Khenra",
        "The target creature's power is checked when you target it with Earthshaker Khenra's ability and when that ability resolves. Once the ability resolves, if the creature's power increases or Earthshaker Khenra's power decreases, the target creature will still be unable to block."
    );
    for before_resolution in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let (k, _) = khenra_enters(&mut t, bears);
        if before_resolution {
            pump(&mut t, bears, 1, 0);
            t.resolve_all();
        } else {
            t.resolve_all();
            pump(&mut t, bears, 5, 0);
            pump(&mut t, k, -1, 0);
        }
        attack_with(&mut t, &[(k, Entity::Player(P1))]);
        // The ability did nothing if the Bears were too big as it resolved.
        assert_eq!(
            legal_blocks(&mut t, P1, &[(bears, k)]),
            before_resolution,
            "pumped before resolution: {before_resolution}"
        );
    }
}

// --- Sorcerer's Broom ---------------------------------------------------------------------

#[test]
fn sorcerer_s_broom_sacrificed_with_another_permanent_triggers() {
    cr!("603.10a", "702.86a");
    ruling!(
        "Sorcerer's Broom",
        "If you sacrifice Sorcerer's Broom and another permanent at the same time, Sorcerer's Broom's ability triggers for that permanent."
    );
    ruling!(
        "Sorcerer's Broom",
        "The ability of Sorcerer's Broom is a triggered ability, not an activated ability. It doesn't allow you to sacrifice a permanent whenever you want"
    );
    supported("Sorcerer's Broom");
    supported("Ulamog's Crusher");
    let mut t = TestGame::new(2);
    let broom = t.battlefield(P0, "Sorcerer's Broom");
    assert!(t
        .obj_now(broom)
        .chars
        .abilities
        .iter()
        .all(|a| !matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_))));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    let crusher = t.battlefield(P1, "Ulamog's Crusher");
    // Annihilator 2: P0 sacrifices the Broom and the Bears at once.
    t.answer_choose(P0, &[obj(broom), obj(bears)]);
    attack_with(&mut t, &[(crusher, Entity::Player(P0))]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Sorcerer's Broom") && t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 1, "the Broom's trigger, for the Bears");
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(crate::r_p032_common::tokens_named(&t, P0, "Sorcerer's Broom").len(), 1);
}
