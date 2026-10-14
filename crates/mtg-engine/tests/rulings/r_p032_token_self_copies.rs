//! Rulings batch P032 — permanents that create token copies of themselves: the token
//! copies only the copiable values (CR 707.2) — so it has the ability that made it and can
//! make copies of itself — but not counters, damage, Auras, Equipment, monstrosity, or
//! non-copy effects; copy effects that already apply to the original are copied (CR
//! 707.3). "You may pay {N}. If you do" can be paid once per resolution (CR 603.5).

use crate::r_p032_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, attach_new, damage};
use crate::r_s17_common::become_copy;
use crate::r_s26_common::new_tokens;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn battlefield_now(t: &TestGame) -> Vec<ObjectId> {
    t.g.battlefield.clone()
}

/// Splitting Slime monstrosity: P0 has mana for it and activates it on `slime`.
fn monstrosity(t: &mut TestGame, slime: ObjectId) {
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 4);
    activate_containing(t, P0, slime, "Monstrosity").expect("monstrosity");
    t.resolve_all();
}

#[test]
fn splitting_slime_token_isnt_monstrous_and_can_become_monstrous_itself() {
    cr!("701.37a", "701.37b", "707.2");
    ruling!(
        "Splitting Slime",
        "The token that’s put onto the battlefield isn’t monstrous at first. If you activate the token’s ability, it will get counters, it will become monstrous, and a new token will be put onto the battlefield."
    );
    supported("Splitting Slime");
    let mut t = TestGame::new(2);
    let slime = t.battlefield(P0, "Splitting Slime");
    let before = battlefield_now(&t);
    monstrosity(&mut t, slime);
    assert!(t.obj_now(slime).monstrous);
    assert_eq!(t.pt(slime), (6, 6));
    let tok = new_tokens(&t, P0, &before);
    assert_eq!(tok.len(), 1);
    let tok = tok[0];
    assert!(!t.obj_now(tok).monstrous);
    assert_eq!(t.pt(tok), (3, 3));
    // The token's own ability: it becomes monstrous and creates another token.
    let before = battlefield_now(&t);
    monstrosity(&mut t, tok);
    assert!(t.obj_now(tok).monstrous);
    assert_eq!(t.pt(tok), (6, 6));
    let second = new_tokens(&t, P0, &before);
    assert_eq!(second.len(), 1);
    assert!(!t.obj_now(second[0]).monstrous);
    assert_eq!(t.pt(second[0]), (3, 3));
}

/// P0 activates Pack Rat's ability ({2}{B}, discard a card) on `rat`; returns the new
/// token.
fn pack_rat_copy(t: &mut TestGame, rat: ObjectId) -> ObjectId {
    t.lands(P0, "Swamp", 3);
    t.hand(P0, "Island");
    t.answer_choose(P0, &[]);
    let before = battlefield_now(t);
    activate_containing(t, P0, rat, "Create a token").expect("pack rat");
    t.resolve_all();
    let new = new_tokens(t, P0, &before);
    assert_eq!(new.len(), 1);
    new[0]
}

#[test]
fn pack_rat_token_has_both_abilities_and_counts_rats_continuously() {
    cr!("707.2", "604.3", "613.4a");
    ruling!(
        "Pack Rat",
        "The token will copy Pack Rat's two abilities. Its power and toughness will be equal to the number of Rats you control (not the number of Rats you controlled when the token entered the battlefield). It will also be able to create copies of itself."
    );
    supported("Pack Rat");
    let mut t = TestGame::new(2);
    let rat = t.battlefield(P0, "Pack Rat");
    assert_eq!(t.pt(rat), (1, 1));
    let tok = pack_rat_copy(&mut t, rat);
    assert_eq!(t.pt(tok), (2, 2));
    assert_eq!(t.pt(rat), (2, 2));
    // The token makes a copy of itself: now every Rat is a 3/3.
    let third = pack_rat_copy(&mut t, tok);
    assert_eq!(t.obj_now(third).chars.name, "Pack Rat");
    for id in [rat, tok, third] {
        assert_eq!(t.pt(id), (3, 3));
    }
    // Another Rat (not a Pack Rat) counts too.
    t.battlefield(P0, "Typhoid Rats");
    assert_eq!(t.pt(tok), (4, 4));
}

#[test]
fn pack_rat_token_doesnt_copy_counters_or_non_copy_effects() {
    cr!("707.2", "707.3");
    ruling!(
        "Pack Rat",
        "The token won't copy counters on Pack Rat, nor will it copy other effects that have changed Pack Rat's power, toughness, types, color, or so on. Normally, this means the token will simply be a Pack Rat. But if any copy effects have affected that Pack Rat, they're taken into account."
    );
    let mut t = TestGame::new(2);
    let rat = t.battlefield(P0, "Pack Rat");
    dress(&mut t, rat, false);
    // Pack Rat is tapped; its ability has no {T} in its cost.
    let tok = pack_rat_copy(&mut t, rat);
    // Two Rats: a plain 2/2 black Pack Rat.
    assert_plain_copy(&t, tok, "Pack Rat", Some((2, 2)));
    // A Pack Rat that's a copy of Pack Rat via a copy effect (Clone): the token copies the
    // copiable values, Pack Rat's.
    let mut t = TestGame::new(2);
    let real = t.battlefield(P0, "Pack Rat");
    let clone = t.battlefield(P0, "Clone");
    become_copy(&mut t, clone, real);
    assert_eq!(t.obj_now(clone).chars.name, "Pack Rat");
    let tok = pack_rat_copy(&mut t, clone);
    assert_plain_copy(&t, tok, "Pack Rat", Some((3, 3)));
    assert_eq!(abilities_with(&t, tok, "Create a token"), 1);
}

/// Deals 1 damage to `id` from a Shock-like source P1 controls, then resolves the
/// triggers. Returns the new tokens of P0.
fn hurt(t: &mut TestGame, id: ObjectId) -> Vec<ObjectId> {
    let src = t.battlefield(P1, "Grizzly Bears");
    let before = battlefield_now(t);
    damage(t, src, 1, id);
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn polyraptor_token_has_the_enrage_ability() {
    cr!("707.2", "603.2");
    ruling!(
        "Polyraptor",
        "The token will have Polyraptor's ability. It will also be able to create copies of itself."
    );
    supported("Polyraptor");
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Polyraptor");
    let tok = hurt(&mut t, raptor);
    assert_eq!(tok.len(), 1);
    assert_eq!(t.obj_now(tok[0]).chars.name, "Polyraptor");
    // The token is dealt damage: it creates a copy of itself.
    let more = hurt(&mut t, tok[0]);
    assert_eq!(more.len(), 1);
    assert_eq!(abilities_with(&t, more[0], "dealt damage"), 1);
}

#[test]
fn polyraptor_token_doesnt_copy_counters_or_damage() {
    cr!("707.2", "707.3");
    ruling!(
        "Polyraptor",
        "The token won't copy counters or damage marked on Polyraptor, nor will it copy other effects that have changed Polyraptor's power, toughness, types, color, or so on. Normally, this means the token will simply be a Polyraptor. But if any copy effects have affected that Polyraptor, they're taken into account."
    );
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Polyraptor");
    dress(&mut t, raptor, true);
    let tok = hurt(&mut t, raptor);
    assert_eq!(tok.len(), 1);
    assert_plain_copy(&t, tok[0], "Polyraptor", Some((5, 5)));
    // A Clone copying Polyraptor: the token is a Polyraptor too.
    let mut t = TestGame::new(2);
    let real = t.battlefield(P0, "Polyraptor");
    let clone = t.battlefield(P0, "Clone");
    become_copy(&mut t, clone, real);
    let tok = hurt(&mut t, clone);
    assert_eq!(tok.len(), 1);
    assert_plain_copy(&t, tok[0], "Polyraptor", Some((5, 5)));
}

/// P0 casts Ornithopter (a historic spell) and answers each "you may pay {1}" with `pay`;
/// returns P0's new tokens once everything resolved.
fn cast_historic(t: &mut TestGame, pays: &[bool]) -> Vec<ObjectId> {
    for p in pays {
        t.answer_yes(P0, *p);
    }
    let before = battlefield_now(t);
    let thopter = t.hand(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve_all();
    new_tokens(t, P0, &before)
        .into_iter()
        .filter(|id| t.obj_now(*id).chars.name != "Ornithopter")
        .collect()
}

#[test]
fn mishras_self_replicator_token_has_its_ability() {
    cr!("707.2", "603.2");
    ruling!(
        "Mishra's Self-Replicator",
        "The token will have Mishra’s Self-Replicator’s ability. It will also be able to create copies of itself."
    );
    supported("Mishra's Self-Replicator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mishra's Self-Replicator");
    t.lands(P0, "Wastes", 3);
    let tok = cast_historic(&mut t, &[true]);
    assert_eq!(tok.len(), 1);
    assert_eq!(t.obj_now(tok[0]).chars.name, "Mishra's Self-Replicator");
    // Both trigger on the next historic spell: two more tokens.
    let more = cast_historic(&mut t, &[true, true]);
    assert_eq!(more.len(), 2);
}

#[test]
fn mishras_self_replicator_pays_once_per_ability() {
    cr!("603.5", "118.1");
    ruling!(
        "Mishra's Self-Replicator",
        "While resolving the triggered ability of Mishra’s Self-Replicator, you can’t pay {1} multiple times to create more than one token. However, if you control more than one Mishra’s Self-Replicator, you can pay {1} for each of their abilities."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mishra's Self-Replicator");
    t.lands(P0, "Wastes", 5);
    let tok = cast_historic(&mut t, &[true, true, true]);
    assert_eq!(tok.len(), 1, "one payment, one token");
    assert_eq!(tapped_count(&t), 1);
    // Two Replicators: two triggers, each can be paid for.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mishra's Self-Replicator");
    t.battlefield(P0, "Mishra's Self-Replicator");
    t.lands(P0, "Wastes", 5);
    let tok = cast_historic(&mut t, &[true, true]);
    assert_eq!(tok.len(), 2);
    assert_eq!(tapped_count(&t), 2);
}

fn tapped_count(t: &TestGame) -> usize {
    crate::r_s01_common::tapped_lands(t, P0)
}

#[test]
fn mishras_self_replicator_token_doesnt_copy_counters_or_damage() {
    cr!("707.2");
    ruling!(
        "Mishra's Self-Replicator",
        "The token won’t copy counters or damage marked on Mishra’s Self-Replicator, nor will it copy other effects that have changed Mishra’s Self-Replicator’s power, toughness, types, color, and so on. Normally, this means the token will simply be a Mishra’s Self-Replicator, but if any copy effects have affected that Mishra’s Self-Replicator, they’re taken into account."
    );
    let mut t = TestGame::new(2);
    let rep = t.battlefield(P0, "Mishra's Self-Replicator");
    dress(&mut t, rep, true);
    t.lands(P0, "Wastes", 1);
    let tok = cast_historic(&mut t, &[true]);
    assert_eq!(tok.len(), 1);
    assert_plain_copy(&t, tok[0], "Mishra's Self-Replicator", Some((2, 2)));
    assert_eq!(t.obj_now(tok[0]).chars.colors, ColorSet::default());
}

/// P0 sacrifices `what` and answers each "you may pay {3}" with the given answers;
/// returns the new tokens once everything resolved.
fn sacrifice_with(t: &mut TestGame, what: ObjectId, pays: &[bool]) -> Vec<ObjectId> {
    for p in pays {
        t.answer_yes(P0, *p);
    }
    let before = battlefield_now(t);
    let what = t.g.current(what);
    t.g.sacrifice(what, P0);
    t.g.flush_events();
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn sorcerers_broom_token_has_its_ability() {
    cr!("707.2", "603.2");
    ruling!(
        "Sorcerer's Broom",
        "The token will have Sorcerer's Broom's ability. It will also be able to create copies of itself."
    );
    supported("Sorcerer's Broom");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sorcerer's Broom");
    t.lands(P0, "Wastes", 9);
    let food = t.battlefield(P0, "Ornithopter");
    let tok = sacrifice_with(&mut t, food, &[true]);
    assert_eq!(tok.len(), 1);
    assert_eq!(t.obj_now(tok[0]).chars.name, "Sorcerer's Broom");
    // Another sacrifice: both Brooms trigger.
    let food = t.battlefield(P0, "Ornithopter");
    let more = sacrifice_with(&mut t, food, &[true, true]);
    assert_eq!(more.len(), 2);
}

#[test]
fn sorcerers_broom_pays_once_per_ability() {
    cr!("603.5");
    ruling!(
        "Sorcerer's Broom",
        "While resolving the triggered ability of Sorcerer's Broom, you can't pay {3} multiple times to create more than one token. However, if you control more than one Sorcerer's Broom, you can pay {3} for each of their abilities."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sorcerer's Broom");
    t.lands(P0, "Wastes", 9);
    let food = t.battlefield(P0, "Ornithopter");
    let tok = sacrifice_with(&mut t, food, &[true, true, true]);
    assert_eq!(tok.len(), 1);
    assert_eq!(tapped_count(&t), 3);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sorcerer's Broom");
    t.battlefield(P0, "Sorcerer's Broom");
    t.lands(P0, "Wastes", 9);
    let food = t.battlefield(P0, "Ornithopter");
    let tok = sacrifice_with(&mut t, food, &[true, true]);
    assert_eq!(tok.len(), 2);
    assert_eq!(tapped_count(&t), 6);
}

#[test]
fn sorcerers_broom_token_doesnt_copy_counters_auras_or_equipment() {
    cr!("707.2");
    ruling!(
        "Sorcerer's Broom",
        "The token won't copy counters, Auras, or Equipment on Sorcerer's Broom, nor will it copy other effects that have changed Sorcerer's Broom's power, toughness, types, color, and so on. Normally, this means the token will simply be a Sorcerer's Broom, but if any copy effects have affected that Sorcerer's Broom, they're taken into account."
    );
    let mut t = TestGame::new(2);
    let broom = t.battlefield(P0, "Sorcerer's Broom");
    attach_new(&mut t, P0, "Bonesplitter", broom);
    attach_new(&mut t, P0, "Holy Strength", broom);
    dress(&mut t, broom, false);
    t.lands(P0, "Wastes", 3);
    let food = t.battlefield(P0, "Ornithopter");
    let tok = sacrifice_with(&mut t, food, &[true]);
    assert_eq!(tok.len(), 1);
    assert_plain_copy(&t, tok[0], "Sorcerer's Broom", Some((2, 1)));
}

/// Mist-Syndicate Naga (`naga`) attacks P1 unblocked; returns P0's new tokens.
fn naga_connects(t: &mut TestGame, attackers: &[ObjectId]) -> Vec<ObjectId> {
    let before = battlefield_now(t);
    t.set_step(P0, Step::PrecombatMain);
    let decl: Vec<_> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    t.attack(&decl, &[]);
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn mist_syndicate_naga_token_has_its_ability() {
    cr!("707.2", "510.3a");
    ruling!(
        "Mist-Syndicate Naga",
        "The token will have Mist-Syndicate Naga’s ability. It will also be able to create copies of itself."
    );
    supported("Mist-Syndicate Naga");
    let mut t = TestGame::new(2);
    let naga = t.battlefield(P0, "Mist-Syndicate Naga");
    let tok = naga_connects(&mut t, &[naga]);
    assert_eq!(tok.len(), 1);
    // Next turn both connect: two more.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    let more = naga_connects(&mut t, &[naga, tok[0]]);
    assert_eq!(more.len(), 2);
    for id in more {
        assert_eq!(t.obj_now(id).chars.name, "Mist-Syndicate Naga");
    }
}

#[test]
fn mist_syndicate_naga_token_doesnt_copy_counters_auras_or_equipment() {
    cr!("707.2");
    ruling!(
        "Mist-Syndicate Naga",
        "The token won’t copy counters, Auras, or Equipment on Mist-Syndicate Naga, nor will it copy other effects that have changed its power, toughness, types, color, or so on. Normally, this means the token will simply be a Mist-Syndicate Naga. But if any copy effects have affected Mist-Syndicate Naga, they’re taken into account."
    );
    let mut t = TestGame::new(2);
    let naga = t.battlefield(P0, "Mist-Syndicate Naga");
    attach_new(&mut t, P0, "Bonesplitter", naga);
    attach_new(&mut t, P0, "Holy Strength", naga);
    dress(&mut t, naga, false);
    crate::r_p021_common::untap(&mut t, naga);
    let tok = naga_connects(&mut t, &[naga]);
    assert_eq!(tok.len(), 1);
    // The token entered after combat damage; it's untapped (it didn't attack).
    assert_plain_copy(&t, tok[0], "Mist-Syndicate Naga", Some((3, 1)));
}

/// Advances P0 to their end step after sacrificing a permanent this turn, resolves the
/// Airship triggers; returns P0's new tokens.
fn airship_end_step(t: &mut TestGame) -> Vec<ObjectId> {
    t.set_step(P0, Step::PrecombatMain);
    let food = t.battlefield(P0, "Ornithopter");
    let before = battlefield_now(t);
    t.g.sacrifice(food, P0);
    t.g.flush_events();
    t.settle();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn phoenix_fleet_airship_token_makes_copies_on_future_turns() {
    cr!("707.2", "603.4");
    ruling!(
        "Phoenix Fleet Airship",
        "The token will have Phoenix Fleet Airship's abilities. It will also be able to create copies of itself on future turns."
    );
    supported("Phoenix Fleet Airship");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phoenix Fleet Airship");
    let tok = airship_end_step(&mut t);
    assert_eq!(tok.len(), 1);
    assert!(abilities_with(&t, tok[0], "Crew") > 0);
    assert_eq!(t.named_on_battlefield("Phoenix Fleet Airship").len(), 2);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    let more = airship_end_step(&mut t);
    assert_eq!(more.len(), 2);
    assert_eq!(t.named_on_battlefield("Phoenix Fleet Airship").len(), 4);
}

#[test]
fn phoenix_fleet_airship_token_doesnt_copy_counters_or_damage() {
    cr!("707.2");
    ruling!(
        "Phoenix Fleet Airship",
        "The token won't copy counters or damage marked on Phoenix Fleet Airship, nor will it copy other effects that have changed Phoenix Fleet Airship's power, toughness, types, color, and so on. Normally, this means the token will simply be a Phoenix Fleet Airship, but if any copy effects have affected that Phoenix Fleet Airship, they're taken into account."
    );
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Phoenix Fleet Airship");
    // Crewed (a creature until end of turn), damaged, with counters.
    crate::r_p021_common::modify(
        &mut t,
        ship,
        vec![mtg_engine::ability::Modification::AddTypes(vec![
            CardType::Creature,
        ])],
    );
    dress(&mut t, ship, true);
    let tok = airship_end_step(&mut t);
    assert_eq!(tok.len(), 1);
    // The ship is still a creature (until the cleanup step); the token isn't.
    assert!(t.obj_now(ship).is(CardType::Creature));
    assert!(!t.obj_now(tok[0]).is(CardType::Creature));
    assert_plain_copy(&t, tok[0], "Phoenix Fleet Airship", None);
}

#[test]
fn vaultborn_tyrant_token_triggers_its_own_enters_ability() {
    cr!("707.2", "603.6a");
    ruling!(
        "Vaultborn Tyrant",
        "When the token copy of Vaultborn Tyrant enters the battlefield, its second ability will trigger."
    );
    supported("Vaultborn Tyrant");
    let mut t = TestGame::new(2);
    let tyrant = t.battlefield(P0, "Vaultborn Tyrant");
    let life = t.life(P0);
    let hand = t.hand_size(P0);
    let before = battlefield_now(&t);
    crate::r_s02_common::destroy(&mut t, tyrant);
    t.resolve_all();
    let tok = new_tokens(&t, P0, &before);
    assert_eq!(tok.len(), 1);
    assert!(t.obj_now(tok[0]).is(CardType::Artifact));
    assert_eq!(t.life(P0), life + 3);
    assert_eq!(t.hand_size(P0), hand + 1);
}
