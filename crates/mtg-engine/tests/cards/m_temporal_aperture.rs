//! Temporal Aperture (hand-written, `src/cards/temporal_aperture.rs`).

use mtg_engine::testing::*;
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
fn the_top_card_may_be_played_free_while_it_stays_on_top() {
    cr!("601.2", "118.9");
    ruling!("Temporal Aperture", "If your library is shuffled or that card otherwise leaves the top of your library, the effect ends");
    let mut t = TestGame::new(2);
    let ap = t.battlefield(P0, "Temporal Aperture");
    // A library of one known card, so the shuffle keeps it on top.
    let lib = t.g.players[0].library.clone();
    for c in lib {
        t.g.objects[c.0 as usize].zone = mtg_engine::object::Zone::Nowhere;
    }
    t.g.players[0].library.clear();
    let giant = t.library_top(P0, "Hill Giant");
    t.lands(P0, "Plains", 5);
    t.activate(P0, ap, 0, &[]).unwrap();
    t.resolve_all();
    cast_free(&mut t, giant).unwrap();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn not_once_another_card_is_on_top() {
    cr!("611.2b");
    let mut t = TestGame::new(2);
    let ap = t.battlefield(P0, "Temporal Aperture");
    let lib = t.g.players[0].library.clone();
    for c in lib {
        t.g.objects[c.0 as usize].zone = mtg_engine::object::Zone::Nowhere;
    }
    t.g.players[0].library.clear();
    let giant = t.library_top(P0, "Hill Giant");
    t.lands(P0, "Plains", 5);
    t.activate(P0, ap, 0, &[]).unwrap();
    t.resolve_all();
    t.library_top(P0, "Island");
    t.g.recompute();
    assert!(cast_free(&mut t, giant).is_err());
}
