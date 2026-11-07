//! Grip of Chaos (hand-written, `src/cards/grip_of_chaos.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;
use rand::SeedableRng;

#[test]
fn a_single_target_is_reselected_at_random() {
    cr!("115.7");
    ruling!(
        "Grip of Chaos",
        "If there are multiple legal targets, it may still choose the original one"
    );
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
    ruling!(
        "Grip of Chaos",
        "If there are no other legal targets, it keeps its target"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grip of Chaos");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let blade = t.hand(P0, "Doom Blade");
    t.cast(P0, blade).target(a).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
}

/// The single target of the stack object `id`.
fn target_of(t: &TestGame, id: ObjectId) -> Entity {
    t.g.obj(id).stack.as_ref().unwrap().chosen[0].targets[0][0]
}

#[test]
fn the_new_target_is_chosen_as_the_ability_resolves() {
    cr!("603.2", "603.3");
    ruling!(
        "Grip of Chaos",
        "the spell or ability keeps its original target until this ability resolves"
    );
    // The other legal target leaves while Grip of Chaos's ability is on the stack: as it
    // resolves, the only legal target is the original one, whatever the random choice.
    for seed in 0..12 {
        let mut t = TestGame::new(2);
        t.g.rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        t.battlefield(P0, "Grip of Chaos");
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 2);
        let blade = t.hand(P0, "Doom Blade");
        let spell = t.cast(P0, blade).target(a).go();
        t.settle();
        assert_eq!(t.stack_len(), 2, "Doom Blade and Grip of Chaos's ability");
        assert_eq!(target_of(&t, spell), Entity::Object(a));
        t.g.destroy(b, None);
        t.g.flush_events();
        t.resolve_all();
        assert!(!t.on_battlefield(a), "seed {seed}");
    }
}

#[test]
fn reselecting_the_only_legal_target_doesnt_target_it_again() {
    cr!("601.2c", "603.2");
    interp!("random-reselection-same-target");
    // Tethered Skirge: "Whenever this creature becomes the target of a spell or ability,
    // you lose 1 life." It became the target of Giant Growth as it was cast; Grip of
    // Chaos's ability reselects Giant Growth's target among the creatures: only the
    // Skirge, which stays its target without becoming it again.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grip of Chaos");
    let skirge = t.battlefield(P1, "Tethered Skirge");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    let spell = t.cast(P0, growth).target(skirge).go();
    t.settle();
    assert_eq!(
        t.stack_len(),
        3,
        "Giant Growth, the Skirge's and Grip of Chaos's abilities"
    );
    t.resolve_all();
    assert_eq!(t.pt(skirge), (5, 5), "Giant Growth resolved: {spell:?}");
    assert_eq!(t.life(P1), 19);
}
