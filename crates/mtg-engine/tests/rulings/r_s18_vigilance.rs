//! Rulings batch S18 — vigilance: Aplan Mortarium's Alien Angel tokens ("2/2 black Alien
//! Angel artifact creature tokens with first strike, vigilance, and 'Whenever an opponent
//! casts a creature spell, this token isn't a creature until end of turn.'").

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s10_common::{attacking, blocking};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::planechase;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// A planechase game where Aplan Mortarium is P0's plane and chaos ensued: P0 controls two
/// Alien Angel tokens (able to attack).
fn alien_angels() -> (TestGame, Vec<ObjectId>) {
    supported("Aplan Mortarium");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Planechase,
            ..Default::default()
        },
    );
    let plane = t.command(P0, "Aplan Mortarium");
    t.g.objects[plane.0 as usize].face_down = true;
    t.g.recompute();
    planechase::set_starting_plane(&mut t.g);
    planechase::chaos_ensues(&mut t.g, P0);
    t.g.flush_events();
    t.resolve_all();
    let angels = with_subtype(&t, P0, "Angel");
    assert_eq!(angels.len(), 2);
    for a in &angels {
        t.g.objects[a.0 as usize].summoning_sick = false;
    }
    (t, angels)
}

/// P1 casts Brazen Borrower (a creature spell with flash) and it resolves, with the
/// tokens' abilities.
fn opponent_casts_a_creature(t: &mut TestGame) {
    give_mana_for(t, P1, "Brazen Borrower // Petty Theft");
    let b = t.hand(P1, "Brazen Borrower // Petty Theft");
    t.cast(P1, b).go();
    t.resolve_all();
}

#[test]
fn the_token_stays_an_artifact_but_isnt_an_alien_angel_while_not_a_creature() {
    cr!("205.3d", "611.2", "613.1d");
    ruling!(
        "Aplan Mortarium",
        "As the triggered ability of the Alien Angel token resolves, it continues to be an artifact but will stop being both an Alien and an Angel until it becomes a creature again."
    );
    let (mut t, angels) = alien_angels();
    let a = angels[0];
    let c = t.obj_now(a).chars.clone();
    assert!(c.is(CardType::Artifact) && c.is_creature());
    assert!(c.has_subtype("Alien") && c.has_subtype("Angel"));
    assert_eq!(t.pt(a), (2, 2));
    // P1 casts a creature spell: each token's ability triggers.
    t.set_step(P1, Step::PrecombatMain);
    opponent_casts_a_creature(&mut t);
    for a in &angels {
        let c = t.obj_now(*a).chars.clone();
        assert!(c.is(CardType::Artifact));
        assert!(!c.is_creature());
        assert!(!c.has_subtype("Alien") && !c.has_subtype("Angel"));
    }
    // At end of turn it's an Alien Angel creature again.
    t.advance_to(P0, Step::Upkeep);
    let c = t.obj_now(a).chars.clone();
    assert!(c.is_creature() && c.has_subtype("Alien") && c.has_subtype("Angel"));
}

#[test]
fn an_attacking_or_blocking_token_that_stops_being_a_creature_leaves_combat() {
    cr!("506.4", "509.1h", "510.1c");
    ruling!(
        "Aplan Mortarium",
        "If an opponent casts a creature spell with flash while the token is attacking or blocking, the token is removed from combat because it stops being a creature. However, any creatures that it was blocking are still considered blocked."
    );
    // Attacking: removed from combat, it deals no combat damage.
    let (mut t, angels) = alien_angels();
    let a = angels[0];
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    assert!(attacking(&t, a));
    opponent_casts_a_creature(&mut t);
    assert!(!attacking(&t, a));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    // Blocking: removed from combat, and the Grizzly Bears it blocked stay blocked.
    let (mut t, angels) = alien_angels();
    let a = angels[0];
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(bears, Entity::Player(P0))], &[(a, bears)]);
    assert!(blocking(&t, a));
    opponent_casts_a_creature(&mut t);
    assert!(!blocking(&t, a));
    assert!(t.g.combat.as_ref().is_some_and(|c| c.is_blocked(bears)));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(a).damage, 0);
}
