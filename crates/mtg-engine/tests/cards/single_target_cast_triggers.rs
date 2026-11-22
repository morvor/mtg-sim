//! Cast triggers for a spell that targets only a single creature (CR 115.9c) whose effect
//! names that creature: Loki, God of Lies ("Whenever you cast a spell that targets only a
//! single creature, gain control of that creature until end of turn. If it's your turn,
//! untap that creature and it gains haste until end of turn."; pattern in
//! `src/oracle/patterns/triggers_single_target_referent.rs`).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn loki(t: &mut TestGame) {
    let def = card("Loki, God of Lies");
    assert!(
        def.unsupported_text().is_empty(),
        "Loki: unsupported {:?}",
        def.unsupported_text()
    );
    t.battlefield(P0, "Loki, God of Lies");
}

#[test]
fn loki_gains_control_of_the_single_target_on_your_turn() {
    cr!("115.9c", "603.2");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    loki(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    let growth = t.cast(P0, growth).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The trigger took the Bears (not the spell): untapped, with haste, while the Giant
    // Growth is still on the stack.
    assert!(t.g.stack.contains(&growth));
    assert_eq!(t.obj(growth).controller, P0);
    let o = t.obj_now(bears);
    assert_eq!(o.controller, P0);
    assert!(!o.tapped);
    assert!(o.has_keyword(KeywordKind::Haste));
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // Until end of turn only.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn loki_on_an_opponents_turn_takes_the_creature_without_untapping_it() {
    cr!("115.9c");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    loki(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.settle();
    t.resolve();
    let o = t.obj_now(bears);
    assert_eq!(o.controller, P0);
    assert!(o.tapped);
    assert!(!o.has_keyword(KeywordKind::Haste));
}

#[test]
fn loki_ignores_a_spell_with_more_than_one_target() {
    cr!("115.9c");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    loki(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    // Arc Trail: 2 damage to the Bears and 1 to P1 — two different targets.
    let trail = t.hand(P0, "Arc Trail");
    t.cast(P0, trail).target(bears).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.obj_now(bears).controller, P1);
}
