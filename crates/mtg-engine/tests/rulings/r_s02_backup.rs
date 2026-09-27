//! Rulings batch S02 — backup (CR 702.165): "Backup N" means "When this creature enters,
//! put N +1/+1 counters on target creature. If that's another creature, it also gains the
//! non-backup abilities of this creature printed below this one until end of turn."
//!
//! Two of the shared rulings exist in two wordings (straight and curly apostrophes) on
//! different cards; those tests cite one card of each group.

use crate::r_s01_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts the real card `name` onto the battlefield under P0's control (a zone change, so
/// its enters abilities trigger) with its backup ability targeting `target`, and puts that
/// ability on the stack. Returns the creature.
fn enter_with_backup(t: &mut TestGame, name: &str, target: ObjectId) -> ObjectId {
    supported(name);
    let before = triggers_on_stack(t, "Backup");
    t.answer_targets(P0, &[Entity::Object(target)]);
    let id = t.enter(P0, name);
    t.settle();
    assert_eq!(triggers_on_stack(t, "Backup"), before + 1);
    id
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).chars.has_keyword(k)
}

/// How many instances of keyword `k` the object has.
fn instances(t: &TestGame, id: ObjectId, k: KeywordKind) -> usize {
    t.obj_now(id).chars.keywords().filter(|x| x.kind == k).count()
}

/// How many of the object's abilities have text containing `text`.
fn abilities_with(t: &TestGame, id: ObjectId, text: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains(text))
        .count()
}

/// Archpriest of Shadows' ability printed below its backup ability (besides deathtouch).
const RETURN: &str = "deals combat damage to a player or battle, return target creature card";

#[test]
fn backup_grants_only_the_abilities_printed_below_it() {
    cr!("702.165a", "702.165c", "707.9b");
    ruling!(
        "Archpriest of Shadows",
        "Backup confers only abilities that are actually printed below it. Any abilities that are gained by the permanent are ignored, including abilities gained due to a resolving spell or ability or copy effects."
    );
    // Archpriest of Shadows: "Backup 1 / Deathtouch / Whenever this creature deals combat
    // damage to a player or battle, return target creature card from your graveyard to the
    // battlefield." It gains flying (Jump) before its backup ability resolves: the Bears
    // get what's printed, not flying.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let archpriest = enter_with_backup(&mut t, "Archpriest of Shadows", bears);
    t.lands(P0, "Island", 1);
    let jump = t.hand(P0, "Jump");
    t.cast(P0, jump).target(archpriest).go();
    t.resolve();
    assert!(has(&t, archpriest, KeywordKind::Flying));
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert!(has(&t, bears, KeywordKind::Deathtouch));
    assert_eq!(abilities_with(&t, bears, RETURN), 1);
    assert!(!has(&t, bears, KeywordKind::Flying));

    // Heat Shimmer: "Create a token that's a copy of target creature, except it has haste
    // and 'At the beginning of the end step, exile this token.'" The token's backup ability
    // doesn't grant the abilities the copy effect gave it.
    supported("Heat Shimmer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let archpriest = t.battlefield(P0, "Archpriest of Shadows");
    t.lands(P0, "Mountain", 3);
    let shimmer = t.hand(P0, "Heat Shimmer");
    t.cast(P0, shimmer).target(archpriest).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    let token = *t
        .named_on_battlefield("Archpriest of Shadows")
        .iter()
        .find(|id| t.obj(**id).is_token())
        .expect("token copy");
    assert!(has(&t, token, KeywordKind::Haste));
    assert_eq!(abilities_with(&t, token, "At the beginning of the end step"), 1);
    assert_eq!(triggers_on_stack(&t, "Backup"), 1);
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert!(has(&t, bears, KeywordKind::Deathtouch));
    assert_eq!(abilities_with(&t, bears, RETURN), 1);
    assert!(!has(&t, bears, KeywordKind::Haste));
    assert_eq!(abilities_with(&t, bears, "At the beginning of the end step"), 0);
}

#[test]
fn the_creature_with_backup_keeps_the_abilities_it_grants() {
    cr!("702.165a", "514.2");
    ruling!(
        "Boon-Bringer Valkyrie",
        "If a backup ability causes another creature to gain abilities, the creature with backup will still have those abilities."
    );
    // Boon-Bringer Valkyrie: "Backup 1 / Flying, first strike, lifelink".
    let kws = [
        KeywordKind::Flying,
        KeywordKind::FirstStrike,
        KeywordKind::Lifelink,
    ];
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let valkyrie = enter_with_backup(&mut t, "Boon-Bringer Valkyrie", bears);
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    for k in kws {
        assert!(has(&t, bears, k), "Bears have {k:?}");
        assert!(has(&t, valkyrie, k), "Valkyrie has {k:?}");
    }
    // Until end of turn: the Bears keep only the counter.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    for k in kws {
        assert!(!has(&t, bears, k), "Bears lost {k:?}");
        assert!(has(&t, valkyrie, k), "Valkyrie has {k:?}");
    }
}

#[test]
fn a_copy_of_a_creature_with_backup_grants_the_abilities_printed_below_backup() {
    cr!("702.165b", "707.2", "707.5");
    ruling!(
        "Death-Greeter's Champion",
        "If a permanent enters the battlefield as a copy of a card with a backup ability or a token is created that is a copy of that card, the order of the printed abilities is maintained."
    );
    // Death-Greeter's Champion: "Dash {3}{R} / Backup 1 / Double strike". A Clone of it
    // grants double strike (printed below backup), not dash (printed above it).
    supported("Clone");
    let mut t = TestGame::new(2);
    let champion = t.battlefield(P0, "Death-Greeter's Champion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Clone");
    let clone = t.hand(P0, "Clone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(champion)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, clone).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Death-Greeter's Champion").len(), 2);
    assert_eq!(triggers_on_stack(&t, "Backup"), 1);
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert!(has(&t, bears, KeywordKind::DoubleStrike));
    assert!(!has(&t, bears, KeywordKind::Dash));

    // Gloomfang Mauler: "Swampcycling {2} / Backup 2 / Menace". A token copy of it
    // (Cackling Counterpart) grants menace, not swampcycling.
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    let mauler = t.battlefield(P0, "Gloomfang Mauler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let counterpart = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, counterpart).target(mauler).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "Backup"), 1);
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert!(has(&t, bears, KeywordKind::Menace));
    assert!(!has(&t, bears, KeywordKind::Cycling));
    assert_eq!(abilities_with(&t, bears, "cycling"), 0);
}

#[test]
fn a_backup_ability_targeting_its_own_creature_only_puts_counters_on_it() {
    cr!("702.165a");
    ruling!(
        "Archpriest of Shadows",
        "If a backup ability targets the creature with backup, that creature will get +1/+1 counters, but it won’t gain additional abilities."
    );
    ruling!(
        "Death-Greeter's Champion",
        "If a backup ability targets the creature with backup, that creature will get +1/+1 counters, but it won't gain additional abilities."
    );
    // The Archpriest targets itself as its backup ability is put on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    supported("Archpriest of Shadows");
    let archpriest = t.enter(P0, "Archpriest of Shadows");
    t.answer_targets(P0, &[Entity::Object(archpriest)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Backup"), 1);
    t.resolve();
    assert_eq!(t.counters(archpriest, counters::PLUS1), 1);
    assert_eq!(t.pt(archpriest), (5, 5));
    assert_eq!(instances(&t, archpriest, KeywordKind::Deathtouch), 1);
    assert_eq!(abilities_with(&t, archpriest, RETURN), 1);

    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    supported("Death-Greeter's Champion");
    let champion = t.enter(P0, "Death-Greeter's Champion");
    t.answer_targets(P0, &[Entity::Object(champion)]);
    t.settle();
    t.resolve();
    assert_eq!(t.counters(champion, counters::PLUS1), 1);
    assert_eq!(instances(&t, champion, KeywordKind::DoubleStrike), 1);
}

#[test]
fn the_abilities_backup_grants_are_fixed_when_it_triggers() {
    cr!("702.165d", "113.7a");
    ruling!(
        "Archpriest of Shadows",
        "The abilities that backup grants to the target creature are determined only once, at the time the ability triggers. They won’t change if the permanent with backup loses any abilities before the backup ability resolves."
    );
    ruling!(
        "Death-Greeter's Champion",
        "The abilities that backup grants to the target creature are determined only once, at the time the ability triggers. They won't change if the permanent with backup loses any abilities before the backup ability resolves."
    );
    // In response to the Archpriest's backup ability, it loses all abilities (Ovinize):
    // the Bears still gain deathtouch and its other ability.
    supported("Ovinize");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let archpriest = enter_with_backup(&mut t, "Archpriest of Shadows", bears);
    t.lands(P0, "Island", 2);
    let ovinize = t.hand(P0, "Ovinize");
    t.cast(P0, ovinize).target(archpriest).go();
    t.resolve();
    assert!(!has(&t, archpriest, KeywordKind::Deathtouch));
    assert_eq!(abilities_with(&t, archpriest, RETURN), 0);
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert!(has(&t, bears, KeywordKind::Deathtouch));
    assert_eq!(abilities_with(&t, bears, RETURN), 1);

    // Death-Greeter's Champion loses its abilities to an opponent's Merfolk Trickster
    // ("When this creature enters, tap target creature an opponent controls. It loses all
    // abilities until end of turn."): the Bears still gain double strike.
    supported("Merfolk Trickster");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let champion = enter_with_backup(&mut t, "Death-Greeter's Champion", bears);
    t.lands(P1, "Island", 2);
    let trickster = t.hand(P1, "Merfolk Trickster");
    t.cast(P1, trickster).go();
    t.answer_targets(P1, &[Entity::Object(champion)]);
    t.resolve();
    t.resolve();
    assert!(t.obj_now(champion).tapped);
    assert!(!has(&t, champion, KeywordKind::DoubleStrike));
    assert_eq!(triggers_on_stack(&t, "Backup"), 1);
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert!(has(&t, bears, KeywordKind::DoubleStrike));
}

#[test]
fn each_backup_instance_triggers_separately() {
    cr!("702.165a", "603.2c");
    // Conclave Sledge-Captain: "Backup 1, backup 1, backup 1 / Trample / Whenever this
    // creature deals combat damage to a player, put that many +1/+1 counters on it."
    supported("Conclave Sledge-Captain");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let captain = t.enter(P0, "Conclave Sledge-Captain");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Backup"), 3);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.counters(elves, counters::PLUS1), 1);
    assert_eq!(t.counters(captain, counters::PLUS1), 0);
    for c in [bears, elves] {
        assert!(has(&t, c, KeywordKind::Trample));
        assert!(abilities_with(&t, c, "put that many +1/+1 counters on it") >= 1);
    }
}
