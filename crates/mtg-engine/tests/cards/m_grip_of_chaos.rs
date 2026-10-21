//! Grip of Chaos (hand-written, `src/cards/grip_of_chaos.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;
use rand::SeedableRng;

#[test]
fn a_single_target_is_reselected_at_random() {
    cr!("115.7");
    ruling!("Grip of Chaos", "If there are multiple legal targets, it may still choose the original one");
    let mut outcomes = [0, 0];
    for seed in 0..24 {
        let mut t = TestGame::new(2);
        t.g.rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        t.battlefield(P0, "Grip of Chaos");
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 2);
        let blade = t.hand(P0, "Doom Blade");
        t.cast(P0, blade).target(a).go();
        t.resolve_all();
        let a_died = !t.on_battlefield(a);
        let b_died = !t.on_battlefield(b);
        assert!(a_died != b_died, "exactly one died");
        outcomes[usize::from(b_died)] += 1;
    }
    assert!(outcomes[0] > 0 && outcomes[1] > 0, "{outcomes:?}");
}

#[test]
fn no_other_legal_target_keeps_it() {
    cr!("115.7");
    ruling!("Grip of Chaos", "If there are no other legal targets, it keeps its target");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grip of Chaos");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let blade = t.hand(P0, "Doom Blade");
    t.cast(P0, blade).target(a).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
}
