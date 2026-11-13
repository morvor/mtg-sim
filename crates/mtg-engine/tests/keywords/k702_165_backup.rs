//! CR 702.165 Backup.

use crate::common_k702_153_167::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts Clone copying `what` and resolves the Clone spell (its enters abilities wait).
fn clone_of(t: &mut TestGame, what: ObjectId) -> ObjectId {
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(what)]);
    t.resolve();
    t.g.current(clone)
}

#[test]
fn backup_puts_counters_and_grants_the_abilities_printed_below_it() {
    cr!("702.165", "702.165a");
    ruling!(
        "Gloomfang Mauler",
        "If a backup ability causes another creature to gain abilities, the creature with backup will still have those abilities."
    );
    assert_supported("Boon-Bringer Valkyrie");
    // Boon-Bringer Valkyrie: backup 1, then "Flying, first strike, lifelink".
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let v = t.enter(P0, "Boon-Bringer Valkyrie");
    t.settle();
    assert_eq!(triggers_starting(&t, "Backup ").len(), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
    assert_eq!(t.pt(bears), (3, 3));
    for k in [
        KeywordKind::Flying,
        KeywordKind::FirstStrike,
        KeywordKind::Lifelink,
    ] {
        assert!(has_kw(&t, bears, k), "{k:?}");
        assert!(has_kw(&t, v, k));
    }
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!has_kw(&t, bears, KeywordKind::Flying));
    assert_eq!(plus1(&t, bears), 1);
}

#[test]
fn backup_targeting_itself_only_puts_counters() {
    cr!("702.165a");
    ruling!(
        "Gloomfang Mauler",
        "If a backup ability targets the creature with backup, that creature will get +1/+1 counters, but it won’t gain additional abilities."
    );
    let mut t = TestGame::new(2);
    // The target is chosen as the trigger is put on the stack: the Valkyrie itself.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let valk = t.enter(P0, "Boon-Bringer Valkyrie");
    t.answer_targets(P0, &[Entity::Object(valk)]);
    t.settle();
    t.resolve_all();
    assert_eq!(plus1(&t, valk), 1);
    assert_eq!(t.pt(valk), (5, 5));
    assert!(!has_kw(&t, bears, KeywordKind::Flying));
    // The Valkyrie doesn't get a second instance of its abilities.
    assert_eq!(kw_count(&t, valk, KeywordKind::Flying), 1);
}

#[test]
fn abilities_printed_above_backup_arent_granted() {
    cr!("702.165a");
    assert_supported("Death-Greeter's Champion");
    // Death-Greeter's Champion: dash, then backup 1, then double strike.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Death-Greeter's Champion");
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::DoubleStrike));
    assert!(!has_kw(&t, bears, KeywordKind::Dash));
    assert_eq!(plus1(&t, bears), 1);
}

#[test]
fn a_copy_of_a_backup_creature_keeps_the_order_of_its_printed_abilities() {
    cr!("702.165b");
    ruling!(
        "Gloomfang Mauler",
        "If a permanent enters the battlefield as a copy of a card with a backup ability or a token is created that is a copy of that card, the order of the printed abilities is maintained."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let valk = t.battlefield(P0, "Boon-Bringer Valkyrie");
    let clone = clone_of(&mut t, valk);
    assert_eq!(t.obj(clone).chars.name, "Boon-Bringer Valkyrie");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.settle();
    assert_eq!(triggers_starting(&t, "Backup ").len(), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert!(has_kw(&t, bears, KeywordKind::Lifelink));
}

#[test]
fn only_abilities_printed_on_the_creature_are_granted() {
    cr!("702.165c");
    ruling!(
        "Gloomfang Mauler",
        "Backup confers only abilities that are actually printed below it."
    );
    assert_supported("Fervor");
    // Fervor gives P0's creatures haste; an opponent's creature targeted by backup gets
    // flying from it, but not haste.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fervor");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    let valk = t.enter(P0, "Boon-Bringer Valkyrie");
    // The Valkyrie has haste (Fervor) and an ability a resolving effect gave it.
    gain(
        &mut t,
        P0,
        valk,
        mtg_engine::keywords::Keyword::new(KeywordKind::Vigilance),
    );
    assert!(has_kw(&t, valk, KeywordKind::Haste));
    t.resolve_all();
    assert!(has_kw(&t, theirs, KeywordKind::Flying));
    assert!(!has_kw(&t, theirs, KeywordKind::Haste));
    assert!(!has_kw(&t, theirs, KeywordKind::Vigilance));
}

#[test]
fn the_granted_abilities_are_determined_as_the_ability_is_put_on_the_stack() {
    cr!("702.165d");
    ruling!(
        "Gloomfang Mauler",
        "They won’t change if the permanent with backup loses any abilities before the backup ability resolves."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let valk = t.enter(P0, "Boon-Bringer Valkyrie");
    t.settle();
    assert_eq!(triggers_starting(&t, "Backup ").len(), 1);
    // The Valkyrie loses all its abilities with its backup ability on the stack.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(valk)],
    );
    assert!(!has_kw(&t, valk, KeywordKind::Flying));
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    assert!(has_kw(&t, bears, KeywordKind::Lifelink));
}

#[test]
fn each_backup_ability_triggers_separately() {
    cr!("702.165a");
    ruling!(
        "Conclave Sledge-Captain",
        "If the same creature is the target of more than one of Conclave Sledge-Captain’s backup abilities, it will get more than one instance of its triggered ability."
    );
    assert_supported("Conclave Sledge-Captain");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Object(bears)]);
    }
    t.enter(P0, "Conclave Sledge-Captain");
    t.settle();
    assert_eq!(triggers_starting(&t, "Backup ").len(), 3);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 3);
    assert_eq!(kw_count(&t, bears, KeywordKind::Trample), 3);
    // Three instances of "Whenever this creature deals combat damage to a player, put
    // that many +1/+1 counters on it."
    let n = t
        .obj_now(bears)
        .chars
        .abilities
        .iter()
        .filter(|a| {
            a.text
                .starts_with("Whenever ~ deals combat damage to a player")
        })
        .count();
    assert_eq!(n, 3);
}
