//! Rulings batch P020 — tokens that enter the battlefield attacking were never declared as
//! attackers, so "whenever [a creature] attacks" abilities don't trigger for them (CR
//! 508.4, 508.3a), even their own (myriad); as copies, they still have the copied
//! creature's "enters" and "enters with" abilities (CR 707.2, 614.1c, 603.6a). A creature
//! that becomes a copy of an attacker after attackers are declared doesn't trigger the
//! copied "attacks" abilities either.

use crate::r_p020_common::*;
use crate::r_s01_common::{attack_with, supported, tokens};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Gleam of Battle: "Whenever a creature you control attacks, put a +1/+1 counter on it."
fn gleam(t: &mut TestGame) {
    supported("Gleam of Battle");
    t.battlefield(P0, "Gleam of Battle");
}

/// P0's new tokens are attacking but got no +1/+1 counter from Gleam of Battle, and the
/// Brazen Wolves among them ("Whenever this creature attacks, it gets +2/+0") weren't
/// pumped: none of them was declared as an attacker.
fn none_declared(t: &TestGame, toks: &[ObjectId]) {
    assert!(!toks.is_empty());
    for id in toks {
        let now = t.g.current(*id);
        assert!(t.g.is_attacking(now), "{id:?} isn't attacking");
        assert_eq!(t.counters(now, counters::PLUS1), 0, "{id:?} 'attacked'");
        if t.obj_now(now).chars.name == "Brazen Wolves" {
            assert_eq!(t.pt(now), (2, 3));
        }
    }
}

#[test]
fn flamerush_rider_token_wasnt_declared_as_an_attacker() {
    cr!("508.4", "508.3a", "603.2");
    ruling!(
        "Flamerush Rider",
        "Although the token is attacking, it was never declared as an attacking creature (for purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    // "Whenever this creature attacks, create a token that's a copy of another target
    // attacking creature and that's tapped and attacking."
    supported("Flamerush Rider");
    supported("Brazen Wolves");
    let mut t = TestGame::new(2);
    gleam(&mut t);
    let rider = t.battlefield(P0, "Flamerush Rider");
    let wolves = t.battlefield(P0, "Brazen Wolves");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(wolves)]);
    attack_with(
        &mut t,
        &[(rider, Entity::Player(P1)), (wolves, Entity::Player(P1))],
    );
    t.resolve_all();
    // The declared attackers triggered.
    assert_eq!(t.counters(rider, counters::PLUS1), 1);
    assert_eq!(t.pt(wolves), (2 + 2 + 1, 3 + 1));
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Brazen Wolves");
    none_declared(&t, &toks);
}

#[test]
fn flamerush_rider_token_has_the_copied_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a", "508.4");
    ruling!(
        "Flamerush Rider",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"As [this creature] enters the battlefield\" or \"[This creature] enters the battlefield with\" abilities of the copied creature will also work."
    );
    supported("Flamerush Rider");
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Flamerush Rider");
    let rift = t.battlefield(P0, RIFTWATCHER);
    let life = t.life(P0);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(rift)]);
    attack_with(
        &mut t,
        &[(rider, Entity::Player(P1)), (rift, Entity::Player(P1))],
    );
    t.resolve_all();
    let toks = riftwatcher_tokens_entered(&t, P0, &[], life, 1);
    assert!(t.g.is_attacking(toks[0]));
}

/// P0 attacks with Shaun, Father of Synths and the legendary creature `other`; Shaun's
/// "Whenever you attack, you may create a tapped and attacking token that's a copy of
/// target attacking legendary creature you control other than Shaun" targets `other`.
fn shaun_copies(t: &mut TestGame, other: ObjectId) -> Vec<ObjectId> {
    supported("Shaun, Father of Synths");
    let shaun = t.battlefield(P0, "Shaun, Father of Synths");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(other)]);
    attack_with(
        t,
        &[(shaun, Entity::Player(P1)), (other, Entity::Player(P1))],
    );
    t.resolve_all();
    let toks: Vec<ObjectId> = tokens(t, P0)
        .into_iter()
        .filter(|id| t.obj_now(*id).chars.name == t.obj_now(other).chars.name)
        .collect();
    assert_eq!(toks.len(), 1);
    assert!(t.obj_now(toks[0]).chars.has_subtype("Synth"));
    toks
}

#[test]
fn shaun_token_wasnt_declared_as_an_attacker() {
    cr!("508.4", "508.3a", "603.2");
    ruling!(
        "Shaun, Father of Synths",
        "Although the token is attacking, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won’t trigger when it enters the battlefield attacking."
    );
    supported("Daghatar the Adamant");
    let mut t = TestGame::new(2);
    gleam(&mut t);
    let daghatar = t.enter(P0, "Daghatar the Adamant");
    t.g.objects[daghatar.0 as usize].summoning_sick = false;
    let toks = shaun_copies(&mut t, daghatar);
    assert_eq!(t.counters(daghatar, counters::PLUS1), 4 + 1);
    // The token has only the four counters it entered with.
    assert!(t.g.is_attacking(toks[0]));
    assert_eq!(t.counters(toks[0], counters::PLUS1), 4);
}

#[test]
fn shaun_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Shaun, Father of Synths",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any “as [this creature] enters the battlefield” or “[this creature] enters the battlefield with” abilities of the copied creature will also work."
    );
    // Dori, Bearer of Friends: "When Dori enters, create a Treasure token."
    supported("Dori, Bearer of Friends");
    let mut t = TestGame::new(2);
    let dori = t.battlefield(P0, "Dori, Bearer of Friends");
    shaun_copies(&mut t, dori);
    let treasures = tokens(&t, P0)
        .into_iter()
        .filter(|id| t.obj_now(*id).chars.has_subtype("Treasure"))
        .count();
    assert_eq!(treasures, 1);
}

/// P0's Phantom Steed exiled `exiled` with its enters trigger, then attacks: "Whenever
/// this creature attacks, create a tapped and attacking token that's a copy of the exiled
/// card, except it's an Illusion in addition to its other types."
fn phantom_steed_attacks(t: &mut TestGame, exiled: ObjectId) -> Vec<ObjectId> {
    supported("Phantom Steed");
    t.answer_targets(P0, &[Entity::Object(exiled)]);
    let steed = t.enter(P0, "Phantom Steed");
    t.resolve_all();
    assert!(!t.on_battlefield(exiled));
    let steed = t.g.current(steed);
    t.g.objects[steed.0 as usize].summoning_sick = false;
    t.set_step(P0, Step::PrecombatMain);
    attack_with(t, &[(steed, Entity::Player(P1))]);
    t.resolve_all();
    let toks = tokens(t, P0);
    assert_eq!(toks.len(), 1);
    assert!(t.obj_now(toks[0]).chars.has_subtype("Illusion"));
    toks
}

#[test]
fn phantom_steed_token_wasnt_declared_as_an_attacker() {
    cr!("508.4", "508.3a", "603.2");
    ruling!(
        "Phantom Steed",
        "Although the token created by the triggered ability is attacking, it was never declared as an attacking creature (for the purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    supported("Brazen Wolves");
    let mut t = TestGame::new(2);
    gleam(&mut t);
    let wolves = t.battlefield(P0, "Brazen Wolves");
    let toks = phantom_steed_attacks(&mut t, wolves);
    none_declared(&t, &toks);
}

#[test]
fn phantom_steed_token_has_the_copied_enters_abilities() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Phantom Steed",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any “As [this creature] enters the battlefield” or “[This creature] enters the battlefield with” abilities of the copied creature will also work."
    );
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, RIFTWATCHER);
    // Exiling the Riftwatcher gains 2 life (it left the battlefield).
    let life = t.life(P0) + 2;
    phantom_steed_attacks(&mut t, rift);
    riftwatcher_tokens_entered(&t, P0, &[], life, 1);
}

#[test]
fn sandstorm_crasher_token_has_the_copied_enters_abilities() {
    cr!("707.2", "701.43a", "614.1c", "603.6a");
    ruling!(
        "Sandstorm Crasher",
        "Any enters abilities of the copied creature will trigger when the token enters. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the copied creature will also work."
    );
    // "You may exert this creature as it attacks. When you do, create a tapped and
    // attacking token that's a copy of target creature you control."
    supported("Sandstorm Crasher");
    let mut t = TestGame::new(2);
    let crasher = t.battlefield(P0, "Sandstorm Crasher");
    let rift = t.battlefield(P0, RIFTWATCHER);
    let life = t.life(P0);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(rift)]);
    attack_with(&mut t, &[(crasher, Entity::Player(P1))]);
    t.resolve_all();
    let toks = riftwatcher_tokens_entered(&t, P0, &[], life, 1);
    assert!(t.g.is_attacking(toks[0]));
}

#[test]
fn calamity_tokens_have_the_copied_enters_abilities() {
    cr!("707.2", "702.171a", "614.1c", "603.6a");
    ruling!(
        "Calamity, Galloping Inferno",
        "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield. Any \"as [this creature] enters the battlefield\" or \"[this creature] enters the battlefield with\" abilities of the copied creature will also work."
    );
    // "Whenever Calamity attacks while saddled, choose a nonlegendary creature that
    // saddled it this turn and create a tapped and attacking token that's a copy of it.
    // ... Repeat this process once. Saddle 1"
    supported("Calamity, Galloping Inferno");
    let mut t = TestGame::new(2);
    let calamity = t.battlefield(P0, "Calamity, Galloping Inferno");
    let rift = t.battlefield(P0, RIFTWATCHER);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(rift)]);
    crate::r_s06_common::activate_containing(&mut t, P0, calamity, "Saddle").expect("saddle");
    t.resolve_all();
    assert!(t.obj_now(rift).tapped);
    let life = t.life(P0);
    t.answer_choose(P0, &[Entity::Object(rift)]);
    t.answer_choose(P0, &[Entity::Object(rift)]);
    attack_with(&mut t, &[(calamity, Entity::Player(P1))]);
    t.resolve_all();
    let toks = riftwatcher_tokens_entered(&t, P0, &[], life, 2);
    assert!(toks.iter().all(|x| t.g.is_attacking(*x)));
}

#[test]
fn myriad_tokens_dont_trigger_their_own_myriad() {
    cr!("508.4", "702.116a", "603.2");
    ruling!(
        "Scurry of Squirrels",
        "Although the tokens enter attacking, they were never declared as attackers. Abilities that trigger whenever a creature attacks won't trigger, including the myriad ability of the tokens."
    );
    // Scurry of Squirrels: "Myriad, myriad". Three opponents: each myriad instance makes a
    // token attacking each of the two other opponents.
    supported("Scurry of Squirrels");
    let mut t = TestGame::new(4);
    gleam(&mut t);
    let scurry = t.battlefield(P0, "Scurry of Squirrels");
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..8 {
        t.answer_yes(P0, true);
    }
    attack_with(&mut t, &[(scurry, Entity::Player(P1))]);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 4, "only the two myriad triggers of the card");
    assert_eq!(t.counters(scurry, counters::PLUS1), 1);
    none_declared(&t, &toks);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn muddle_myriad_tokens_have_the_copied_enters_abilities() {
    cr!("707.2", "707.9a", "702.116a", "614.1c", "603.6a");
    ruling!(
        "Muddle, the Ever-Changing",
        "Any \"enters\" abilities of the copied creature will trigger when the tokens enter. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the copied creature will also work."
    );
    // "Whenever you cast an instant or sorcery spell, Muddle becomes a copy of up to one
    // target nonlegendary creature you control until end of turn, except it has myriad."
    supported("Muddle, the Ever-Changing");
    let mut t = TestGame::new(3);
    let muddle = t.battlefield(P0, "Muddle, the Ever-Changing");
    let rift = t.battlefield(P0, RIFTWATCHER);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(rift)]);
    crate::r_s25_common::cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(rift)]);
    t.resolve_all();
    assert_eq!(t.obj_now(muddle).chars.name, RIFTWATCHER);
    // Becoming a copy isn't entering: no time counters, no life.
    assert_eq!(t.counters(muddle, counters::TIME), 0);
    let life = t.life(P0);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(muddle, Entity::Player(P1))]);
    t.resolve_all();
    let toks = riftwatcher_tokens_entered(&t, P0, &[], life, 1);
    assert!(t.g.is_attacking(toks[0]));
}

#[test]
fn a_creature_becoming_a_copy_of_an_attacker_doesnt_trigger_its_attack_abilities() {
    cr!("508.3a", "707.2", "603.2");
    ruling!(
        "Tilonalli's Skinshifter",
        "Because attackers have already been declared, any abilities Tilonalli’s Skinshifter copies that trigger when it or other creatures attack won’t trigger."
    );
    // "Whenever this creature attacks, it becomes a copy of another target nonlegendary
    // attacking creature until end of turn." Brazen Wolves: "Whenever this creature
    // attacks, it gets +2/+0 until end of turn."
    supported("Tilonalli's Skinshifter");
    supported("Brazen Wolves");
    let mut t = TestGame::new(2);
    let skin = t.battlefield(P0, "Tilonalli's Skinshifter");
    let wolves = t.battlefield(P0, "Brazen Wolves");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(wolves)]);
    attack_with(
        &mut t,
        &[(skin, Entity::Player(P1)), (wolves, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(t.obj_now(skin).chars.name, "Brazen Wolves");
    assert_eq!(t.pt(wolves), (4, 3));
    assert_eq!(t.pt(skin), (2, 3));
}
