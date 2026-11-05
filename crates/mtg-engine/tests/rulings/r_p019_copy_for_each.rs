//! Rulings batch P019 — Radiant Performer ("choose target spell or ability that targets
//! only a single permanent or player. Copy that spell or ability for each other permanent
//! or player the spell or ability could target.", CR 707.10d), Mirror-Shield Hoplite
//! ("Whenever a creature you control becomes the target of a backup ability, copy that
//! ability.", CR 702.165), and copies of loyalty abilities: Jaya's Phoenix ("copy the next
//! loyalty ability you activate this turn when you activate it") and Leori, Sparktouched
//! Hunter ("Until end of turn, whenever you activate an ability of a planeswalker of that
//! type, copy that ability.").

use crate::r_p019_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported, triggers_on_stack};
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{abilities_from, cast_new, change_copy_targets, targets_of, x_of};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::object::{ObjKind, StackKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The copies of spells on the stack, bottom first.
fn spell_copies(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).kind == ObjKind::SpellCopy)
        .collect()
}

/// P0 casts Radiant Performer from hand (Flash); its enters trigger targets `target`, a
/// spell on the stack. Returns the Performer's permanent once its trigger has resolved.
fn perform(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let performer = cast_new(t, P0, "Radiant Performer", &[]);
    t.resolve();
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.settle();
    let trigger = *t.g.stack.last().unwrap();
    assert!(matches!(
        t.g.obj(trigger).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { .. })
    ));
    t.resolve();
    t.g.current(performer)
}

#[test]
fn radiant_performers_copies_trigger_magecraft() {
    cr!("707.10d", "707.10");
    ruling!(
        "Radiant Performer",
        "Each copy you create will cause your magecraft abilities to trigger."
    );
    supported("Radiant Performer");
    supported("Archmage Emeritus");
    // P0's Shock targets P1's Bears; it could also target Archmage Emeritus, the
    // Performer, P0 and P1: four copies, four magecraft triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archmage Emeritus");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shock = cast_new(&mut t, P0, "Shock", &[Entity::Object(bears)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "cast or copy"), 1);
    t.resolve();
    perform(&mut t, shock);
    assert_eq!(spell_copies(&t).len(), 4);
    assert_eq!(triggers_on_stack(&t, "cast or copy"), 4);
}

#[test]
fn radiant_performer_copies_an_opponents_spell_for_every_other_legal_target() {
    cr!("707.10d", "707.10", "405.5");
    ruling!(
        "Radiant Performer",
        "You control all the copies, and you'll choose which copy targets which permanent or player. For example, if the original spell says, \"Destroy target creature,\" you'll ultimately choose the order in which each creature targeted by a copy is destroyed. Remember that the original spell or ability will resolve after all of the copies."
    );
    ruling!(
        "Radiant Performer",
        "You copy the spell or ability for every permanent or player it could target. That is, you don't choose either permanents or players and then make copies based on that."
    );
    // P1's Lightning Bolt targets P0's Bears. P0 flashes in the Performer: a copy for each
    // of P0, P1 and the Performer, all controlled by P0, with no targets to choose.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt_card = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.g.turn.priority = Some(P1);
    let bolt = t.g.cast_spell(P1, bolt_card, CastMethod::Normal).unwrap();
    t.g.flush_events();
    let performer = cast_new(&mut t, P0, "Radiant Performer", &[]);
    t.resolve();
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.settle();
    let from = t.asked().len();
    t.resolve();
    let performer = t.g.current(performer);
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 3);
    assert!(copies.iter().all(|c| t.g.obj(*c).controller == P0));
    let mut targets: Vec<Entity> = copies.iter().flat_map(|c| targets_of(&t, *c)).collect();
    targets.sort_by_key(|e| format!("{e:?}"));
    let mut expected = vec![
        Entity::Player(P0),
        Entity::Player(P1),
        Entity::Object(performer),
    ];
    expected.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(targets, expected);
    assert_eq!(
        asked_count(&t, from, |d| matches!(d, Decision::ChooseTargets { .. })),
        0
    );
    // The original is under the copies.
    assert_eq!(t.g.stack[0], bolt);
    t.resolve();
    t.resolve();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 17);
    assert!(!t.on_battlefield(performer));
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn radiant_performer_copies_a_spell_with_several_targets_all_the_same_creature() {
    cr!("707.10d", "115.3");
    ruling!(
        "Radiant Performer",
        "You can choose a spell or ability with multiple targets as long as all of those targets are the same permanent or player."
    );
    supported("Seeds of Strength");
    // Seeds of Strength: "Target creature gets +1/+1 until end of turn. Target creature
    // gets +1/+1 until end of turn. Target creature gets +1/+1 until end of turn." All
    // three on the Bears; the copy gives the Performer +3/+3.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let seeds = cast_new(
        &mut t,
        P0,
        "Seeds of Strength",
        &[
            Entity::Object(bears),
            Entity::Object(bears),
            Entity::Object(bears),
        ],
    );
    assert_eq!(targets_of(&t, seeds), vec![Entity::Object(bears); 3]);
    let performer = perform(&mut t, seeds);
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(
        targets_of(&t, copies[0]),
        vec![Entity::Object(performer); 3]
    );
    t.resolve_all();
    assert_eq!(t.pt(performer), (5, 5));
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn mirror_shield_hoplite_copy_on_the_same_other_creature_doubles_counters_and_abilities() {
    cr!("702.165a", "707.10", "603.2c");
    ruling!(
        "Mirror-Shield Hoplite",
        "If a creature other than the creature with backup ends up as the target of both the backup ability and the copy, it will end up with two +1/+1 counters and it will gain the non-backup abilities twice. In some cases, the duplicate abilities will be redundant (two instances of flying, for example). However, if it picks up two triggered abilities, for example, those abilities trigger independently."
    );
    supported("Mirror-Shield Hoplite");
    supported("Hangar Scrounger");
    // Hangar Scrounger: "Backup 1 ... Whenever this creature becomes tapped, you may
    // discard a card. If you do, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror-Shield Hoplite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let scrounger = t.enter(P0, "Hangar Scrounger");
    t.settle();
    let backup = abilities_from(&t, scrounger);
    assert_eq!(backup.len(), 1);
    // Hoplite's trigger is on top; the copy keeps the Bears as its target.
    crate::r_s25_common::keep_copy_targets(&mut t, P0);
    t.resolve();
    let both = abilities_from(&t, scrounger);
    assert_eq!(both.len(), 2);
    assert_eq!(targets_of(&t, both[1]), vec![Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    // Tapping the Bears triggers both instances of the gained ability.
    t.g.tap(bears);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "becomes tapped"), 2);
}

#[test]
fn mirror_shield_hoplite_copy_has_the_backup_creature_as_its_source() {
    cr!("702.165a", "707.10b");
    ruling!(
        "Mirror-Shield Hoplite",
        "The source of the copy is the same as the source of the original backup ability. If the creature that ends up as the target of the copy is that original creature with backup, it will get a +1/+1 counter, but it won’t gain any abilities. If it’s a creature other than that original creature, it will get a +1/+1 counter and gain abilities."
    );
    // The backup ability targets the Bears; the copy targets the Scrounger itself: it
    // gets a counter but no second "becomes tapped" ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror-Shield Hoplite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let scrounger = t.enter(P0, "Hangar Scrounger");
    t.settle();
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(scrounger))]);
    t.resolve();
    let both = abilities_from(&t, scrounger);
    assert_eq!(both.len(), 2);
    assert_eq!(targets_of(&t, both[1]), vec![Entity::Object(scrounger)]);
    t.resolve_all();
    assert_eq!(t.counters(scrounger, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    t.g.tap(scrounger);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "becomes tapped"), 1);
    t.resolve_all();
    t.g.tap(bears);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "becomes tapped"), 1);
}

/// P0's `attacker` (flying, haste or not summoning sick) attacks P1 and deals combat
/// damage; the game moves on to P0's postcombat main phase.
fn hit_with(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(t, P1, &[]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
}

/// P0 activates Chandra Nalaar's "−X: Chandra Nalaar deals X damage to target creature."
/// with X = 2, targeting `target`.
fn chandra_minus_x(t: &mut TestGame, chandra: ObjectId, target: ObjectId) -> ObjectId {
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[Entity::Object(target)]);
    activate_containing(t, P0, chandra, "X damage to target creature")
        .unwrap()
        .unwrap()
}

#[test]
fn jayas_phoenix_copy_of_a_minus_x_loyalty_ability_has_the_same_x() {
    cr!("707.10", "606.4", "107.3");
    ruling!(
        "Jaya's Phoenix",
        "If the loyalty ability has -X in its cost, the copy uses the same value of X."
    );
    supported("Jaya's Phoenix");
    supported("Chandra Nalaar");
    let mut t = TestGame::new(2);
    let phoenix = t.battlefield(P0, "Jaya's Phoenix");
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    hit_with(&mut t, phoenix);
    assert_eq!(t.life(P1), 17);
    let ability = chandra_minus_x(&mut t, chandra, a);
    assert_eq!(t.counters(chandra, counters::LOYALTY), 4);
    t.settle();
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(b))]);
    t.resolve();
    let both = abilities_from(&t, chandra);
    assert_eq!(both.len(), 2);
    assert!(both.contains(&ability));
    assert!(both.iter().all(|c| x_of(&t, *c) == Some(2)));
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert_eq!(t.counters(chandra, counters::LOYALTY), 4);
}

/// Leori deals combat damage to P1 (P0 chooses the planeswalker type Chandra), then the
/// game moves on to P0's postcombat main phase.
fn leori_hits(t: &mut TestGame) {
    let leori = t.battlefield(P0, "Leori, Sparktouched Hunter");
    let chandra = mtg_engine::types::planeswalker_types()
        .iter()
        .position(|w| *w == "Chandra")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(chandra));
    hit_with(t, leori);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn leoris_copy_of_a_minus_x_loyalty_ability_has_the_same_x() {
    cr!("707.10", "606.4", "107.3");
    ruling!(
        "Leori, Sparktouched Hunter",
        "If the loyalty ability has -X in its cost, the copy uses the same value of X."
    );
    supported("Leori, Sparktouched Hunter");
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    leori_hits(&mut t);
    chandra_minus_x(&mut t, chandra, a);
    t.settle();
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(b))]);
    t.resolve();
    let both = abilities_from(&t, chandra);
    assert_eq!(both.len(), 2);
    assert!(both.iter().all(|c| x_of(&t, *c) == Some(2)));
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert_eq!(t.counters(chandra, counters::LOYALTY), 4);
}

#[test]
fn leoris_copy_resolves_even_if_the_original_ability_is_countered() {
    cr!("707.10", "603.7", "608.2h");
    ruling!(
        "Leori, Sparktouched Hunter",
        "The delayed triggered ability created by Leori's last ability, as well as the copy it creates, resolve before the ability that caused it to trigger. They resolve even if that ability is countered."
    );
    // Chandra's "+1: Chandra Nalaar deals 1 damage to target player or planeswalker." at
    // P1; P0 Stifles the original in response to Leori's trigger: the copy still deals 1.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    leori_hits(&mut t);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let original = activate_containing(&mut t, P0, chandra, "1 damage")
        .unwrap()
        .unwrap();
    t.settle();
    let trigger = *t.g.stack.last().unwrap();
    assert_ne!(trigger, original);
    // The trigger is above the ability.
    assert_eq!(t.g.stack, vec![original, trigger]);
    cast_new(&mut t, P0, "Stifle", &[Entity::Object(original)]);
    t.resolve();
    assert!(!t.g.stack.contains(&original));
    crate::r_s25_common::keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}
