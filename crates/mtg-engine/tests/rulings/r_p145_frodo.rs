//! Rulings batch P145 — Frodo, Adventurous Hobbit: "Whenever Frodo attacks, if you gained
//! 3 or more life this turn, the Ring tempts you. Then if Frodo is your Ring-bearer and the
//! Ring has tempted you two or more times this game, draw a card." (The second condition
//! is two conditions joined by "and", `oracle/patterns/conditions_conjunction.rs`.)

use crate::r_p120_common::gain_life;
use crate::r_p145_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

const FRODO: &str = "Frodo, Adventurous Hobbit";

fn tempted(t: &TestGame) -> u32 {
    t.g.player(P0).ring_level
}

/// Frodo attacks P1; returns whether his trigger was put on the stack.
fn frodo_attacks(t: &mut TestGame, frodo: ObjectId) -> bool {
    attack_with(t, &[(frodo, Entity::Player(P1))]);
    triggers_on_stack(t, "the Ring tempts you") == 1
}

#[test]
fn frodo_needs_three_life_gained_this_turn() {
    cr!("603.4", "701.54a");
    ruling!(
        "Frodo, Adventurous Hobbit",
        "If you haven't gained 3 or more life by the time Frodo, Adventurous Hobbit attacks, the ability won't trigger at all."
    );
    ruling!(
        "Frodo, Adventurous Hobbit",
        "Frodo, Adventurous Hobbit's ability looks at how much life you've gained in the turn, even if it wasn't on the battlefield when you gained life. It doesn't care if you also lost life, even if you lost more life than you gained."
    );
    ruling!(
        "Frodo, Adventurous Hobbit",
        "The Ring tempts you just once, no matter how much life you've gained past 3 life."
    );
    supported(FRODO);
    // Two life: no trigger.
    let mut t = TestGame::new(2);
    let frodo = t.battlefield(P0, FRODO);
    gain_life(&mut t, P0, 2);
    assert!(!frodo_attacks(&mut t, frodo));
    // Three life gained before Frodo was on the battlefield, then six lost: it triggers.
    let mut t = TestGame::new(2);
    gain_life(&mut t, P0, 3);
    crate::r_p146_common::lose_life(&mut t, P0, 6);
    let frodo = t.battlefield(P0, FRODO);
    assert!(frodo_attacks(&mut t, frodo));
    t.resolve_all();
    assert_eq!(tempted(&t), 1);
    // Ten life: tempted once.
    let mut t = TestGame::new(2);
    let frodo = t.battlefield(P0, FRODO);
    gain_life(&mut t, P0, 10);
    assert!(frodo_attacks(&mut t, frodo));
    t.resolve_all();
    assert_eq!(tempted(&t), 1);
    assert_eq!(t.hand_size(P0), 0, "the first temptation: no draw");
}

#[test]
fn frodo_draws_as_ring_bearer_tempted_twice() {
    cr!("701.54a", "701.54e", "608.2c");
    supported(FRODO);
    for (frodo_bearer, draws) in [(true, 1), (false, 0)] {
        let mut t = TestGame::new(2);
        let frodo = t.battlefield(P0, FRODO);
        let bears = t.battlefield(P0, "Grizzly Bears");
        mtg_engine::kwa::ring::tempt(&mut t.g, P0, None);
        gain_life(&mut t, P0, 3);
        assert!(frodo_attacks(&mut t, frodo));
        let bearer = if frodo_bearer { frodo } else { bears };
        t.answer_choose(P0, &[obj(bearer)]);
        t.resolve_all();
        assert_eq!(tempted(&t), 2);
        assert_eq!(
            t.hand_size(P0),
            draws,
            "Frodo the Ring-bearer: {frodo_bearer}"
        );
    }
}

/// The other cards whose conditions are now read as two conditions joined by "and": the
/// Judgment incarnations ("As long as ~ is in your graveyard and you control a Mountain,
/// creatures you control have haste.") and Deathbringer Regent.
#[test]
fn conjunctions_incarnations_from_the_graveyard() {
    cr!("113.6", "611.3a", "604.1");
    use mtg_engine::keywords::KeywordKind as K;
    for (card, land, kw) in [
        ("Anger", "Mountain", K::Haste),
        ("Brawn", "Forest", K::Trample),
        ("Filth", "Swamp", K::Landwalk),
        ("Valor", "Plains", K::FirstStrike),
        ("Wonder", "Island", K::Flying),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        // In P0's graveyard without the land: no.
        let mine = t.graveyard(P0, card);
        t.g.recompute();
        assert!(!t.obj_now(bears).has_keyword(kw), "{card}");
        // The land, but the card is in P1's graveyard: no.
        move_to(&mut t, mine, mtg_engine::object::Zone::Exile);
        t.graveyard(P1, card);
        t.battlefield(P0, land);
        t.g.recompute();
        assert!(!t.obj_now(bears).has_keyword(kw), "{card}");
        // Both: P0's creatures have it (P1's don't, P1 controls no such land).
        t.graveyard(P0, card);
        t.g.recompute();
        assert!(t.obj_now(bears).has_keyword(kw), "{card}");
        assert!(!t.obj_now(theirs).has_keyword(kw), "{card}");
    }
}

#[test]
fn conjunctions_deathbringer_regent_cast_from_hand_with_five_others() {
    cr!("603.4", "601.2");
    supported("Deathbringer Regent");
    for (others, cast, destroyed) in [(5, true, true), (4, true, false), (5, false, false)] {
        let mut t = TestGame::new(2);
        let bears: Vec<ObjectId> = (0..others)
            .map(|_| t.battlefield(P1, "Grizzly Bears"))
            .collect();
        if cast {
            cast_resolve(&mut t, P0, "Deathbringer Regent", &[]);
        } else {
            enter(&mut t, P0, "Deathbringer Regent");
            t.resolve_all();
        }
        assert_eq!(
            !t.on_battlefield(bears[0]),
            destroyed,
            "{others} others, cast: {cast}"
        );
        assert_eq!(t.named_on_battlefield("Deathbringer Regent").len(), 1);
    }
}
