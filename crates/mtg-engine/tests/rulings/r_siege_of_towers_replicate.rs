//! Siege of Towers' replicate ruling, which names Shattering Spree (a template error,
//! docs/RULING_DISCREPANCIES.md "Siege of Towers: "copy Shattering Spree" for "copy Siege
//! of Towers""): "As the replicate triggered ability resolves, you'll copy Shattering
//! Spree for each time you paid its replicate cost, even if the original spell is no
//! longer on the stack at that time (perhaps because it was countered)." (2024-01-12).
//! Siege of Towers' replicate copies Siege of Towers (CR 702.56a); the rest of the ruling
//! holds for it.

use crate::r_s01_common::supported;
use crate::r_s11_common::spells_copied;
use crate::r_s14_common::*;
use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The names of the copies of spells on the stack.
fn copies_on_stack(t: &TestGame) -> Vec<String> {
    t.g.stack
        .iter()
        .filter(|s| t.g.obj(**s).kind == ObjKind::SpellCopy)
        .map(|s| t.g.obj(*s).chars.name.to_string())
        .collect()
}

#[test]
fn siege_of_towers_replicate_copies_it_even_after_it_was_countered() {
    cr!("702.56a", "707.10");
    ruling!(
        "Siege of Towers",
        "for each time you paid its replicate cost, even if the original spell is no longer on the stack at that time (perhaps because it was countered)."
    );
    interp!("ruling-contradicting-oracle-gives-way");
    supported("Siege of Towers");
    // Siege of Towers: "Replicate {1}{R} ... Target Mountain becomes a 3/1 creature. It's
    // still a land." P0 casts it at Mountain `a`, paying its replicate cost once (a
    // Shattering Spree in their hand, which the ruling names, stays there). P1 counters
    // Siege of Towers while the replicate trigger is on the stack: the trigger still makes
    // one copy, of Siege of Towers (not of Shattering Spree), which animates Mountain `b`.
    let mut t = TestGame::new(2);
    let spree = t.hand(P0, "Shattering Spree");
    let mountains = t.lands(P0, "Mountain", 2);
    let (a, b) = (mountains[0], mountains[1]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    let siege = cast_from_hand(&mut t, P0, "Siege of Towers", &[Entity::Object(a)]);
    t.settle();
    assert_eq!(triggers_on_stack_now(&t), 1);
    cast_from_hand(&mut t, P1, "Counterspell", &[Entity::Object(siege)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Siege of Towers"));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve();
    assert_eq!(copies_on_stack(&t), vec!["Siege of Towers".to_string()]);
    assert_eq!(spells_copied(&t), 1);
    t.resolve();
    let creature = |t: &TestGame, m: ObjectId| t.g.obj(m).is(types::CardType::Creature);
    assert!(creature(&t, b));
    assert_eq!(t.pt(b), (3, 1));
    assert!(!creature(&t, a));
    assert!(t.g.stack.is_empty());
    assert_eq!(t.zone(spree), mtg_engine::object::Zone::Hand(P0));
}
