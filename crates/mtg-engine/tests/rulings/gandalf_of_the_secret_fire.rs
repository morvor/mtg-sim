//! Gandalf of the Secret Fire: "Whenever you cast an instant or sorcery spell from your
//! hand during an opponent's turn, exile that card with three time counters on it instead
//! of putting it into your graveyard as it resolves. Then if the exiled card doesn't have
//! suspend, it gains suspend." (CR 608.2n, 614.1a, 702.62).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn setup() -> TestGame {
    let c = mtg_engine::card::card("Gandalf of the Secret Fire");
    assert!(c.unsupported_text().is_empty(), "{:?}", c.unsupported_text());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gandalf of the Secret Fire");
    t
}

/// P0 casts Lightning Bolt from their hand at P1 and everything resolves.
fn bolt(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    t.g.turn.priority = Some(P0);
    t.cast(P0, b).target(Entity::Player(P1)).go();
    t.resolve_all();
    b
}

#[test]
fn a_spell_cast_on_an_opponents_turn_is_exiled_with_time_counters_and_gains_suspend() {
    cr!("608.2n", "614.1a", "702.62a", "702.62b");
    ruling!(
        "Gandalf of the Secret Fire",
        "If the spell requires any targets, those targets are chosen when the spell is finally cast, not when it's exiled."
    );
    let mut t = setup();
    t.set_step(P1, Step::PrecombatMain);
    let b = bolt(&mut t);
    assert_eq!(t.life(P1), 17);
    let exiled = t.g.current(b);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, counters::TIME), 3);
    assert!(t.obj_now(exiled).chars.has_keyword(KeywordKind::Suspend));
    // It counts down at P0's upkeeps; with the last counter it's cast again, with a new
    // target.
    for _ in 0..3 {
        t.advance_to(P0, Step::Upkeep);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve_all();
        t.advance_to(P1, Step::Upkeep);
    }
    assert_eq!(t.life(P1), 14);
    // This time it was cast from exile, not from a hand: it goes to the graveyard.
    assert!(t.in_graveyard(P0, "Lightning Bolt"), "{}", t.dump_log());
}

#[test]
fn a_spell_cast_on_your_own_turn_goes_to_the_graveyard() {
    cr!("608.2n");
    let mut t = setup();
    bolt(&mut t);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn a_countered_spell_goes_to_the_graveyard() {
    cr!("701.6a", "608.2n");
    let mut t = setup();
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    t.g.turn.priority = Some(P0);
    let spell = t.cast(P0, b).target(Entity::Player(P1)).go();
    // Gandalf's ability resolves; then P1 counters the spell.
    t.resolve();
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}
