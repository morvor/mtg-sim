//! Rulings batch S33 — "whenever you gain life" and "life you gained" in Two-Headed
//! Giant: life gain happens to each player individually and the result is applied to the
//! team's shared life total (CR 810.9), so life gained by a teammate isn't life "you"
//! gained, even though the team's life total increased.

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s05_common::enter;
use crate::r_s25_common::cast_new;
use crate::r_s33_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// In a Two-Headed Giant game, P0 controls `name`; P0's teammate P1 casts Sacred Nectar
/// ("You gain 4 life."), then P0 does. Returns the number of triggered abilities whose
/// text contains `trigger` that were put on the stack after each, and leaves the game
/// after P0's has resolved.
fn teammate_then_you_gain(t: &mut TestGame, name: &str, trigger: &str) -> (usize, usize) {
    supported(name);
    t.battlefield(P0, name);
    let life = t.life(P0);
    cast_new(t, P1, "Sacred Nectar", &[]);
    t.resolve();
    t.settle();
    // The team's life total went up.
    assert_eq!((t.life(P0), t.life(P1)), (life + 4, life + 4));
    let teammate = triggers_on_stack(t, trigger);
    t.resolve_all();
    cast_new(t, P0, "Sacred Nectar", &[]);
    t.resolve();
    t.settle();
    let you = triggers_on_stack(t, trigger);
    t.resolve_all();
    (teammate, you)
}

#[test]
fn bloodbond_vampire_life_gained_by_a_teammate_doesnt_trigger_it() {
    cr!("810.9", "119.9");
    ruling!(
        "Bloodbond Vampire",
        "In a Two-Headed Giant game, life gained by your teammate won’t cause the ability to trigger, even though it causes your team’s life total to increase."
    );
    let mut t = two_headed_giant();
    assert_eq!(
        teammate_then_you_gain(&mut t, "Bloodbond Vampire", "+1/+1 counter"),
        (0, 1)
    );
    let vampire = t.named_on_battlefield("Bloodbond Vampire")[0];
    assert_eq!(t.counters(vampire, "+1/+1"), 1);
}

#[test]
fn cliffhaven_vampire_life_gained_by_a_teammate_doesnt_trigger_it() {
    cr!("810.9", "119.9");
    ruling!(
        "Cliffhaven Vampire",
        "In a Two-Headed Giant game, life gained by your teammate won’t cause the ability to trigger, even though it causes your team’s life total to increase."
    );
    // "Whenever you gain life, each opponent loses 1 life."
    let mut t = two_headed_giant();
    assert_eq!(
        teammate_then_you_gain(&mut t, "Cliffhaven Vampire", "each opponent loses 1 life"),
        (0, 1)
    );
    // Each opponent lost 1 life, once: the opposing team lost 2.
    assert_eq!(t.life(P2), 28);
}

#[test]
fn scion_of_the_swarm_life_gained_by_a_teammate_doesnt_trigger_it() {
    cr!("810.9", "119.9");
    ruling!(
        "Scion of the Swarm",
        "In a Two-Headed Giant game, life gained by your teammate won't cause the ability to trigger, even though it caused your team's life total to increase."
    );
    let mut t = two_headed_giant();
    assert_eq!(
        teammate_then_you_gain(&mut t, "Scion of the Swarm", "+1/+1 counter"),
        (0, 1)
    );
}

#[test]
fn vito_life_gained_by_a_teammate_doesnt_trigger_it() {
    cr!("810.9", "119.9");
    ruling!(
        "Vito, Thorn of the Dusk Rose",
        "In a Two-Headed Giant game, life gained by your teammate won't cause the ability to trigger, even though it caused your team's life total to increase."
    );
    // "Whenever you gain life, target opponent loses that much life."
    let mut t = two_headed_giant();
    t.answer_targets(P0, &[Entity::Player(P2)]);
    assert_eq!(
        teammate_then_you_gain(&mut t, "Vito, Thorn of the Dusk Rose", "loses that much life"),
        (0, 1)
    );
    assert_eq!(t.life(P2), 26);
}

#[test]
fn gideons_company_life_gained_by_a_teammate_doesnt_trigger_it() {
    cr!("810.9", "119.9");
    ruling!(
        "Gideon's Company",
        "In a Two-Headed Giant game, life gained by your teammate won’t cause the ability to trigger, even though it caused your team’s life total to increase."
    );
    // "Whenever you gain life, put two +1/+1 counters on this creature."
    let mut t = two_headed_giant();
    assert_eq!(
        teammate_then_you_gain(&mut t, "Gideon's Company", "two +1/+1 counters"),
        (0, 1)
    );
}

#[test]
fn cleric_of_lifes_bond_life_gained_by_a_teammate_isnt_your_first() {
    cr!("810.9", "119.9");
    ruling!(
        "Cleric of Life's Bond",
        "In a Two-Headed Giant game, life gained by your teammate won’t cause the ability to trigger, even though it caused your team’s life total to increase."
    );
    // "Whenever you gain life for the first time each turn, put a +1/+1 counter on this
    // creature." The teammate's gain isn't P0's first: P0's later gain still triggers it.
    let mut t = two_headed_giant();
    assert_eq!(
        teammate_then_you_gain(&mut t, "Cleric of Life's Bond", "+1/+1 counter"),
        (0, 1)
    );
}

#[test]
fn voice_of_the_blessed_life_gained_by_a_teammate_doesnt_trigger_it() {
    cr!("810.9", "119.9");
    ruling!(
        "Voice of the Blessed",
        "In a Two-Headed Giant game, life gained by your teammate won't cause the first ability to trigger, even though it caused your team's life total to increase."
    );
    // "Whenever you gain life, put a +1/+1 counter on this creature."
    let mut t = two_headed_giant();
    assert_eq!(
        teammate_then_you_gain(&mut t, "Voice of the Blessed", "+1/+1 counter"),
        (0, 1)
    );
}

#[test]
fn karlov_life_gained_by_a_teammate_doesnt_trigger_it() {
    cr!("810.9", "119.9");
    ruling!(
        "Karlov of the Ghost Council",
        "In a Two-Headed Giant game, life gained by your teammate won't cause the ability to trigger, even though it causes your team's life total to increase."
    );
    // "Whenever you gain life, put two +1/+1 counters on Karlov."
    let mut t = two_headed_giant();
    assert_eq!(
        teammate_then_you_gain(&mut t, "Karlov of the Ghost Council", "two +1/+1 counters"),
        (0, 1)
    );
    let karlov = t.named_on_battlefield("Karlov of the Ghost Council")[0];
    assert_eq!(t.counters(karlov, "+1/+1"), 2);
}

#[test]
fn angelic_accord_life_gained_by_a_teammate_isnt_considered() {
    cr!("810.9", "603.4");
    ruling!(
        "Angelic Accord",
        "In a Two-Headed Giant game, life gained by your teammate isn’t considered, even though it causes your team’s life total to increase."
    );
    supported("Angelic Accord");
    // "At the beginning of each end step, if you gained 4 or more life this turn, create a
    // 4/4 white Angel creature token with flying." P1 gains 4 life; P0 gains none.
    let mut t = two_headed_giant();
    t.battlefield(P0, "Angelic Accord");
    cast_new(&mut t, P1, "Sacred Nectar", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 34);
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.settle();
    t.resolve_all();
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
    // P0 gaining 4 life themself: the next end step, an Angel.
    let mut t = two_headed_giant();
    t.battlefield(P0, "Angelic Accord");
    cast_new(&mut t, P0, "Sacred Nectar", &[]);
    t.resolve_all();
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}

#[test]
fn voracious_wurm_life_gained_by_a_teammate_isnt_considered() {
    cr!("810.9", "614.1c");
    ruling!(
        "Voracious Wurm",
        "In a Two-Headed Giant game, life gained by your teammate isn’t considered, even though it causes your team’s life total to increase."
    );
    supported("Voracious Wurm");
    // "This creature enters with X +1/+1 counters on it, where X is the amount of life
    // you've gained this turn."
    let mut t = two_headed_giant();
    cast_new(&mut t, P1, "Sacred Nectar", &[]);
    t.resolve_all();
    let wurm = enter(&mut t, P0, "Voracious Wurm");
    assert_eq!(t.counters(wurm, "+1/+1"), 0);
    cast_new(&mut t, P0, "Sacred Nectar", &[]);
    t.resolve_all();
    let wurm = enter(&mut t, P0, "Voracious Wurm");
    assert_eq!(t.counters(wurm, "+1/+1"), 4);
}
