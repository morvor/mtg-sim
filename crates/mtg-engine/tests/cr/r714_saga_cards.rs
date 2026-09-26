//! CR 714: Saga cards.

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::saga;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// History of Benalia: "I, II — Create a 2/2 white Knight creature token with vigilance.
/// III — Knights you control get +2/+1 until end of turn."
const BENALIA: &str = "History of Benalia";

fn knights(t: &TestGame) -> usize {
    t.named_on_battlefield("Knight Token").len()
}

fn lore(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::LORE)
}

fn add_lore(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.add_counters(Entity::Object(id), counters::LORE, n, None);
    t.g.flush_events();
}

/// A Saga with the given chapter text, compiled by the real compiler.
fn saga_card(name: &str, text: &str) -> CardDef {
    oracle_card(name, "Enchantment — Saga", "{0}", None, text)
}

#[test]
fn a_saga_has_chapter_abilities() {
    cr!("714.1", "714.2", "714.2a");
    supported(BENALIA);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, BENALIA);
    let o = t.obj(s);
    assert!(o.chars.has_subtype("Saga"));
    // Its chapter symbols are triggered abilities: I, II and III.
    assert!(o
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Triggered(_)))
        .count()
        >= 2);
    let mut ns = saga::chapter_numbers(o);
    ns.sort();
    assert_eq!(ns, vec![1, 2, 3]);
    // IV represents 4.
    supported("Summon: G.F. Ifrit");
    let ifrit = t.battlefield(P0, "Summon: G.F. Ifrit");
    let mut ns = saga::chapter_numbers(t.obj(ifrit));
    ns.sort();
    assert_eq!(ns, vec![1, 2, 3, 4]);
    let custom = t.custom(
        P0,
        saga_card("Long Tale", "I — You gain 1 life.\nIV — You gain 4 life.\nVI — You gain 6 life."),
        Zone::Battlefield,
    );
    let mut ns = saga::chapter_numbers(t.obj(custom));
    ns.sort();
    assert_eq!(ns, vec![1, 4, 6]);
}

#[test]
fn a_saga_creatures_other_abilities_are_independent_of_its_chapters() {
    cr!("714.1a");
    supported("Summon: Titan");
    // Summon: Titan, an Enchantment Creature — Saga Giant 7/7 with "Reach, trample" in
    // the text box below its type line.
    let mut t = TestGame::new(2);
    let titan = t.battlefield(P0, "Summon: Titan");
    let c = &t.obj(titan).chars;
    assert!(c.is_creature() && c.is(CardType::Enchantment) && c.has_subtype("Saga"));
    assert_eq!(t.pt(titan), (7, 7));
    assert!(c.has_keyword(KeywordKind::Reach) && c.has_keyword(KeywordKind::Trample));
    // They apply at any number of lore counters.
    add_lore(&mut t, titan, 1);
    t.g.recompute();
    assert!(t.obj(titan).chars.has_keyword(KeywordKind::Trample));
}

#[test]
fn a_chapter_triggers_as_the_lore_count_reaches_its_number() {
    cr!("714.2b");
    ruling!(
        "History of Benalia",
        "the third lore counter put on a Saga causes the III chapter ability to trigger"
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, BENALIA);
    t.g.objects[s.0 as usize].counters.clear();
    // From 0 to 2 at once: chapters I and II trigger (one "when one or more lore counters
    // are put" event).
    add_lore(&mut t, s, 2);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(knights(&t), 2);
    // The third counter triggers III; I and II don't trigger again.
    add_lore(&mut t, s, 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(knights(&t), 2);
    let k = t.named_on_battlefield("Knight Token")[0];
    assert_eq!(t.pt(k), (4, 3));
    // Other kinds of counters don't trigger chapters.
    let s2 = t.battlefield(P0, BENALIA);
    t.g.objects[s2.0 as usize].counters.clear();
    t.g.add_counters(Entity::Object(s2), counters::PLUS1, 3, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn two_numerals_are_two_chapter_abilities() {
    cr!("714.2c");
    let mut t = TestGame::new(2);
    // "I, II — Create a Knight": it triggers at 1 and again at 2.
    let s = t.enter(P0, BENALIA);
    t.resolve_all();
    assert_eq!(lore(&t, s), 1);
    assert_eq!(knights(&t), 1);
    add_lore(&mut t, s, 1);
    t.resolve_all();
    assert_eq!(knights(&t), 2);
}

#[test]
fn the_final_chapter_number_is_the_greatest_chapter_number() {
    cr!("714.2d", "714.4");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, BENALIA);
    assert_eq!(saga::final_chapter(t.obj(s)), Some(3));
    let ifrit = t.battlefield(P0, "Summon: G.F. Ifrit");
    assert_eq!(saga::final_chapter(t.obj(ifrit)), Some(4));
    // A Saga that somehow has no chapter abilities has final chapter number 0, and isn't
    // sacrificed however many lore counters it has.
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(s)],
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
    );
    assert_eq!(saga::final_chapter(t.obj(s)), None);
    add_lore(&mut t, s, 5);
    t.resolve_all();
    assert!(t.on_battlefield(s));
    assert_eq!(knights(&t), 0);
}

#[test]
fn the_final_chapter_ability() {
    cr!("714.2e");
    // "Whenever the final chapter ability of a Saga you control resolves, you gain 5
    // life."
    let watcher = CB::new("Chronicler")
        .enchantment()
        .ability(super::r600_common::trig(
            TriggerCond::AbilityResolved {
                source: Filter::and(vec![
                    Filter::Subtype("Saga".into()),
                    Filter::ControlledBy(PlayerRel::You),
                ]),
                final_chapter: true,
            },
            Body::effect(super::r600_common::gain(5)),
        ))
        .build();
    let mut t = TestGame::new(2);
    t.custom(P0, watcher, Zone::Battlefield);
    let s = t.enter(P0, BENALIA);
    t.resolve_all();
    add_lore(&mut t, s, 1);
    t.resolve_all();
    // Chapters I and II aren't its final chapter ability.
    assert_eq!(t.life(P0), 20);
    add_lore(&mut t, s, 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

#[test]
fn a_saga_enters_with_a_lore_counter() {
    cr!("714.3", "714.3a");
    ruling!(
        "History of Benalia",
        "As a Saga enters the battlefield, its controller puts a lore counter on it"
    );
    let mut t = TestGame::new(2);
    let s = t.enter(P0, BENALIA);
    // The counter is there as it enters: chapter I triggers from it.
    assert_eq!(lore(&t, s), 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(knights(&t), 1);
    // Every Saga without read ahead has that ability, even one without chapter
    // abilities.
    let blank = t.custom(P0, saga_card("Blank Page", ""), Zone::Hand(P0));
    let blank = t
        .g
        .move_object(blank, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, None)
        .unwrap();
    assert_eq!(lore(&t, blank), 1);
    // A non-Saga enchantment doesn't.
    let e = t.enter(P0, "Glorious Anthem");
    assert_eq!(lore(&t, e), 0);
}

#[test]
fn a_saga_with_read_ahead_enters_with_the_chosen_number_of_lore_counters() {
    cr!("714.3b");
    ruling!(
        "The Cruelty of Gix",
        "its controller chooses a number from one to that Saga’s greatest chapter number"
    );
    supported("The Cruelty of Gix");
    // The Cruelty of Gix (read ahead): "III — Put target creature card from a graveyard
    // onto the battlefield under your control."
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let s = t.enter(P0, "The Cruelty of Gix");
    assert_eq!(lore(&t, s), 3);
    // Only chapter III triggers (skipped chapters don't).
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let hg = t.named_on_battlefield("Hill Giant");
    assert_eq!(hg.len(), 1);
    assert_eq!(t.obj(hg[0]).controller, P0);
    assert_eq!(t.life(P0), 20, "chapter II didn't trigger");
    // Choosing one: chapter I triggers.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    let s = t.enter(P0, "The Cruelty of Gix");
    assert_eq!(lore(&t, s), 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn a_lore_counter_is_added_as_the_precombat_main_phase_begins() {
    cr!("714.3c");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, BENALIA);
    let theirs = t.battlefield(P1, BENALIA);
    let blank = t.custom(P0, saga_card("Blank Page", ""), Zone::Battlefield);
    let (before, before_theirs, before_blank) = (lore(&t, s), lore(&t, theirs), lore(&t, blank));
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    // Chapter abilities triggered by the turn-based action are waiting; P0's Saga got a
    // counter, P1's and the one without chapter abilities didn't.
    assert_eq!(lore(&t, s), before + 1);
    assert_eq!(lore(&t, theirs), before_theirs);
    assert_eq!(lore(&t, blank), before_blank);
}

#[test]
fn a_saga_is_sacrificed_after_its_final_chapter_ability_leaves_the_stack() {
    cr!("714.4");
    ruling!(
        "History of Benalia",
        "the Saga’s controller sacrifices it as soon as its chapter ability has left the stack"
    );
    let mut t = TestGame::new(2);
    let s = t.enter(P0, BENALIA);
    t.resolve_all();
    add_lore(&mut t, s, 2);
    t.settle();
    // Chapter III is on the stack: the Saga stays.
    assert_eq!(t.stack_len(), 2);
    assert!(t.on_battlefield(s));
    t.resolve();
    t.settle();
    assert!(t.on_battlefield(s));
    t.resolve();
    t.settle();
    assert!(!t.on_battlefield(s));
    assert!(t.in_graveyard(P0, BENALIA));
}
