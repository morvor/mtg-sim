//! Aggregates over players (CR 608.2h, 101.4): Balance and its relatives.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn count(t: &TestGame, p: PlayerId, ty: types::CardType) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.chars.card_types.contains(ty)
        })
        .count()
}

#[test]
fn balance_evens_out_lands_hands_and_creatures() {
    cr!("101.4", "608.2h");
    ruling!(
        "Balance",
        "Each type of object is counted during the corresponding part of the process."
    );
    ruling!(
        "Balance",
        "Balance doesn't have targets, so permanents that can't be targeted"
    );
    for c in [
        "Balance",
        "Restore Balance",
        "Balancing Act",
        "Magus of the Balance",
    ] {
        compiles(c);
    }
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    t.lands(P1, "Forest", 2);
    t.battlefield(P0, "Grizzly Bears");
    for _ in 0..3 {
        t.battlefield(P1, "Llanowar Elves");
    }
    // Hexproof doesn't matter.
    t.battlefield(P1, "Gladecover Scout");
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    for _ in 0..5 {
        t.hand(P1, "Island");
    }
    let balance = t.hand(P0, "Balance");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, balance).go();
    t.resolve_all();
    use types::CardType::{Creature, Land};
    assert_eq!((count(&t, P0, Land), count(&t, P1, Land)), (2, 2));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (3, 3));
    assert_eq!((count(&t, P0, Creature), count(&t, P1, Creature)), (1, 1));
}

#[test]
fn balancing_act_evens_out_permanents_then_hands() {
    cr!("101.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    t.battlefield(P1, "Forest");
    t.battlefield(P1, "Llanowar Elves");
    t.hand(P0, "Island");
    for _ in 0..4 {
        t.hand(P1, "Island");
    }
    let act = t.hand(P0, "Balancing Act");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, act).go();
    t.resolve_all();
    let perms = |p: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).controller == p)
            .count()
    };
    assert_eq!((perms(P0), perms(P1)), (2, 2));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (1, 1));
}
