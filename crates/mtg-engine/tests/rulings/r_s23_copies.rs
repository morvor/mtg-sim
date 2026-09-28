//! Rulings batch S23 — copies of spells (CR 707.10): copies created by storm and
//! demonstrate are put onto the stack, not cast (CR 707.10, 702.40a), and a copy has the
//! value of X chosen for the original (CR 707.10).

use crate::r_s01_common::*;
use crate::r_s04_common::is_spell_copy;
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn storm_copies_arent_cast_and_dont_count_for_later_storm() {
    cr!("702.40a", "707.10");
    ruling!(
        "Prismari, the Inspiration",
        "The copies of instant and sorcery spells you cast created by the storm ability are put directly onto the stack. They aren't cast and won't be counted by other spells with storm cast later in the turn."
    );
    supported("Prismari, the Inspiration");
    supported("Young Pyromancer");
    // Prismari: "Instant and sorcery spells you cast have storm." Young Pyromancer:
    // "Whenever you cast an instant or sorcery spell, create a 1/1 red Elemental creature
    // token."
    let mut t = TestGame::new(2);
    t.g.players[1].life = 40;
    t.battlefield(P0, "Prismari, the Inspiration");
    t.battlefield(P0, "Young Pyromancer");
    t.lands(P0, "Mountain", 3);
    let mut copies = Vec::new();
    for _ in 0..3 {
        let bolt = t.hand(P0, "Lightning Bolt");
        t.cast(P0, bolt).target(Entity::Player(P1)).go();
        // The storm trigger resolves: its copies are on the stack above the original.
        t.settle();
        t.resolve();
        copies.push(t.g.stack.iter().filter(|s| is_spell_copy(&t, **s)).count());
        t.resolve_all();
    }
    // The third Bolt's storm counts the two Bolts cast before it, not the copy.
    assert_eq!(copies, vec![0, 1, 2]);
    assert_eq!(t.life(P1), 40 - 3 * 6);
    let cast = t
        .g
        .turn_events
        .iter()
        .filter(|e| matches!(e, Event::SpellCast { .. }))
        .count();
    assert_eq!(cast, 3);
    // Only the three cast spells triggered Young Pyromancer.
    assert_eq!(t.named_on_battlefield("Elemental Token").len(), 3);
}

#[test]
fn demonstrate_copies_of_a_spell_have_its_value_of_x() {
    cr!("707.10", "702.144a", "107.3");
    ruling!(
        "Silverquill Lecturer",
        "If the spell had an X value chosen for it as it was cast, the copies will have the same value of X."
    );
    supported("Silverquill Lecturer");
    supported("Endless One");
    // Silverquill Lecturer: "Creature spells you cast have demonstrate." Endless One
    // ({X}): "This creature enters with X +1/+1 counters on it." Cast with X = 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverquill Lecturer");
    t.lands(P0, "Wastes", 3);
    let one = t.hand(P0, "Endless One");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.cast(P0, one).x(3).go();
    t.resolve_all();
    let ones = t.named_on_battlefield("Endless One");
    assert_eq!(ones.len(), 3);
    for id in ones {
        assert_eq!(t.counters(id, "+1/+1"), 3);
    }
    let p1 = t
        .named_on_battlefield("Endless One")
        .into_iter()
        .filter(|id| t.obj(*id).controller == P1)
        .count();
    assert_eq!(p1, 1);
}

#[test]
fn show_of_confidence_counts_spells_cast_before_its_trigger_resolves() {
    cr!("707.10", "603.3", "608.2h");
    ruling!(
        "Show of Confidence",
        "The triggered ability counts all instants and sorceries that were cast before it resolves. If you cast instant spells in response to the ability, those spells will count."
    );
    supported("Show of Confidence");
    // "When you cast this spell, copy it for each other instant and sorcery spell you've
    // cast this turn. You may choose new targets for the copies. / Put a +1/+1 counter on
    // target creature. It gains vigilance until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Plains", 2);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    let show = t.hand(P0, "Show of Confidence");
    t.cast(P0, show).target(Entity::Object(bears)).go();
    t.settle();
    // In response to the trigger, P0 casts another instant: it counts too.
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::U, 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    // Two copies and the original: three counters.
    assert_eq!(t.counters(bears, "+1/+1"), 3);
    // Without the response: one copy.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Plains", 2);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    let show = t.hand(P0, "Show of Confidence");
    t.cast(P0, show).target(Entity::Object(bears)).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
}

#[test]
fn thousand_year_storm_copies_have_the_value_of_x() {
    cr!("707.10", "107.3", "603.3");
    ruling!(
        "Thousand-Year Storm",
        "If the spell that's copied has an X whose value was determined as it was cast, the copies will have the same value of X."
    );
    supported("Thousand-Year Storm");
    // "Whenever you cast an instant or sorcery spell, copy it for each other instant and
    // sorcery spell you've cast before it this turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thousand-Year Storm");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 3);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    // Blaze with X = 2: one copy, also with X = 2.
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(2).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn thousand_year_storm_counts_spells_cast_before_the_triggering_spell() {
    cr!("707.10", "603.3");
    ruling!(
        "Thousand-Year Storm",
        "The copies that Thousand-Year Storm's ability creates are created on the stack, so they're not \"cast.\" Abilities that trigger when a player casts a spell (such as that of Thousand-Year Storm itself) won't trigger."
    );
    // Lightning Bolt, then in response to its trigger, Opt. The Bolt's trigger counts only
    // the spells cast before the Bolt (none), so it makes no copy; Opt's counts the Bolt.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thousand-Year Storm");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.settle();
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.settle();
    // The Opt's trigger resolves: one copy of Opt, which doesn't trigger the Storm.
    t.resolve();
    assert_eq!(crate::r_s11_common::spells_copied(&t), 1);
    assert_eq!(crate::r_s14_common::triggers_on_stack_now(&t), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P1), 17);
}
