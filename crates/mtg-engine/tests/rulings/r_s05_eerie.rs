//! Rulings batch S05 — eerie (an ability word): "Whenever an enchantment you control
//! enters and whenever you fully unlock a Room, ...".

use crate::r_s01_common::*;
use crate::r_s05_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn an_eerie_permanent_entering_with_enchantments_triggers_for_each_of_them() {
    cr!("603.6a", "603.2");
    ruling!(
        "Balemurk Leech",
        "If a permanent with an eerie ability enters at the same time as one or more enchantments, its ability will trigger for each of those enchantments."
    );
    supported("Balemurk Leech");
    // Balemurk Leech: "Eerie — Whenever an enchantment you control enters and whenever
    // you fully unlock a Room, each opponent loses 1 life."
    let mut t = TestGame::new(2);
    let leech = t.hand(P0, "Balemurk Leech");
    let a = t.hand(P0, "Glorious Anthem");
    let b = t.hand(P0, "Glorious Anthem");
    run_from(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::AllTargets,
            to: Destination::battlefield(),
        },
        &[
            Entity::Object(leech),
            Entity::Object(a),
            Entity::Object(b),
        ],
    );
    assert_eq!(t.named_on_battlefield("Glorious Anthem").len(), 2);
    // Two triggers: one for each enchantment.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}
