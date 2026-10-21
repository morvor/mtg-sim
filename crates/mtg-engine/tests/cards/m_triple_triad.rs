//! Triple Triad (hand-written, `src/cards/triple_triad.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Casts `card` with the first way a permission allows.
fn cast_free(t: &mut TestGame, card: ObjectId) -> Result<ObjectId, mtg_engine::casting::Illegal> {
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    let Some(m) = t
        .g
        .cast_options(P0, card)
        .into_iter()
        .find(|o| t.g.can_begin_cast(P0, card, o))
        .map(|o| o.method)
    else {
        return Err(mtg_engine::casting::Illegal("no way to cast it".into()));
    };
    t.cast(P0, card).method(m).try_go()
}

#[test]
fn play_your_card_and_lesser_ones_free() {
    cr!("601.2", "118.9");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Triple Triad");
    let mine = t.library_top(P0, "Hill Giant");
    let lesser = t.library_top(P1, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_exile("Grizzly Bears"));
    t.advance_to(P0, Step::PrecombatMain);
    let mine = t.g.current(mine);
    let lesser = t.g.current(lesser);
    cast_free(&mut t, mine).unwrap();
    t.resolve();
    cast_free(&mut t, lesser).unwrap();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn not_a_card_with_greater_mana_value() {
    cr!("601.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Triple Triad");
    t.library_top(P0, "Grizzly Bears");
    let greater = t.library_top(P1, "Hill Giant");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    let greater = t.g.current(greater);
    assert!(cast_free(&mut t, greater).is_err());
}
