//! "Until your next turn, whenever …" delayed triggered abilities (CR 603.7b, 611.2b): they
//! trigger through the other players' turns and end as their controller's next turn
//! begins.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn dont_move_destroys_creatures_that_become_tapped_until_your_next_turn() {
    cr!("603.7b", "611.2b");
    assert_supported("Don't Move");
    // "Destroy all tapped creatures. Until your next turn, whenever a creature becomes
    // tapped, destroy it."
    let mut t = TestGame::new(2);
    let tapped = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[tapped.0 as usize].tapped = true;
    let attacker = t.battlefield(P1, "Hill Giant");
    let later = t.battlefield(P1, "Savannah Lions");
    t.lands(P0, "Plains", 5);
    let dm = t.hand(P0, "Don't Move");
    t.cast(P0, dm).go();
    t.resolve_all();
    assert!(!t.on_battlefield(tapped));
    assert!(t.on_battlefield(attacker));
    // In P1's turn, attacking taps Hill Giant: it's destroyed.
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P0))], &[]);
    assert!(!t.on_battlefield(attacker));
    assert_eq!(t.life(P0), 20);
    // From P0's next turn on, it no longer triggers.
    t.advance_to(P0, Step::Upkeep);
    t.g.tap(later);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(later));
}
