//! Rulings batch S33 — when life changes: a triggered ability gains its life only as it
//! resolves, so a player reduced to 0 life first loses before gaining it (CR 603.3,
//! 704.5a, 117.5); Reanimate's life loss happens after the creature is on the
//! battlefield, before abilities that triggered on its entering are put on the stack
//! (CR 608.2c, 603.3), and the creature's own abilities apply to that loss (CR 119.8).

use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s04_common::top_of_stack;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s33_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 attacks P0 with Hill Giant (3/3) and Grizzly Bears (2/2); P0's Essence Sliver
/// (3/3, "Whenever a Sliver deals damage, its controller gains that much life.") blocks
/// the Bears. Runs until combat damage has been dealt and state-based actions and
/// triggers have been handled.
fn sliver_blocks(t: &mut TestGame) -> ObjectId {
    let sliver = t.battlefield(P0, "Essence Sliver");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(t, &[(giant, Entity::Player(P0)), (bears, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(sliver, bears)]),
    );
    t.g.run_until(10_000, |g| {
        g.result.is_some() || (g.turn.step == Step::CombatDamage && !g.stack.is_empty())
    });
    sliver
}

#[test]
fn essence_sliver_you_lose_before_the_life_is_gained() {
    cr!("603.3", "704.5a", "117.5", "510.2");
    ruling!(
        "Essence Sliver",
        "You only gain the life when the triggered ability resolves. If you are reduced to zero life before the ability resolves, you will lose before gaining the life."
    );
    supported("Essence Sliver");
    // At 10 life: the trigger is on the stack after combat damage, and P0 hasn't gained
    // the life yet.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 10);
    sliver_blocks(&mut t);
    assert_eq!(t.life(P0), 7);
    assert_eq!(triggers_on_stack(&t, "gains that much life"), 1);
    t.resolve();
    assert_eq!(t.life(P0), 10);
    // At 3 life: the Hill Giant's damage reduces P0 to 0, and P0 loses before the
    // trigger resolves.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 3);
    sliver_blocks(&mut t);
    assert!(t.has_lost(P0));
    assert_eq!(t.life(P0), 0);
}

/// P0 casts Reanimate ("Put target creature card from a graveyard onto the battlefield
/// under your control. You lose life equal to that card's mana value.") targeting `name`
/// in P1's graveyard, and it resolves (no triggers resolve).
fn reanimate(t: &mut TestGame, name: &str) -> ObjectId {
    supported("Reanimate");
    let card = t.graveyard(P1, name);
    crate::r_s25_common::cast_new(t, P0, "Reanimate", &[Entity::Object(card)]);
    t.resolve();
    card
}

#[test]
fn reanimate_enters_triggers_resolve_after_you_lose_life() {
    cr!("608.2c", "603.3", "704.5a", "800.4a");
    ruling!(
        "Reanimate",
        "If any abilities trigger on the creature entering the battlefield, those abilities resolve after you lose life. If losing life results in you losing the game, those abilities won't resolve."
    );
    // Siege-Gang Commander (mana value 5): "When this creature enters, create three 1/1
    // red Goblin creature tokens."
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 10);
    let card = reanimate(&mut t, "Siege-Gang Commander");
    assert!(t.on_battlefield(card));
    assert_eq!(t.life(P0), 5);
    assert_eq!(triggers_on_stack(&t, "Goblin"), 1);
    assert!(tokens_of(&t, P0).is_empty());
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 3);
    // At 5 life, P0 goes to 0 and loses the game (here with three players, so the game
    // goes on) before the trigger is put on the stack: it never resolves.
    let mut t = TestGame::new(3);
    set_life(&mut t, P0, 5);
    reanimate(&mut t, "Siege-Gang Commander");
    assert!(t.has_lost(P0));
    t.resolve_all();
    assert!(t.g.permanents().all(|o| !o.is_token()));
}

#[test]
fn grave_researcher_reanimate_the_creatures_abilities_apply_to_the_life_loss() {
    cr!("608.2c", "119.8");
    ruling!(
        "Reanimate",
        "You lose life after the creature is already on the battlefield. Any abilities it has that interact with loss of life, such as that of Platinum Emperion, apply to that loss of life."
    );
    // Platinum Emperion (mana value 8): "Your life total can't change."
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    let card = reanimate(&mut t, "Platinum Emperion");
    assert!(t.on_battlefield(card));
    assert_eq!(t.obj_now(card).controller, P0);
    assert_eq!(t.life(P0), 5);
    assert!(!t.has_lost(P0));
    // Another creature (Hill Giant, mana value 4) costs P0 4 life.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    reanimate(&mut t, "Hill Giant");
    assert_eq!(t.life(P0), 1);
}

/// The tokens `p` controls.
fn tokens_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token())
        .map(|o| o.id)
        .collect()
}

#[test]
fn voidslime_counters_triggered_abilities_such_as_afterlife() {
    cr!("603.1", "702.135a", "701.6a", "115.1");
    ruling!(
        "Voidslime",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities, such as afterlife, are triggered abilities and will have \"when,\" \"whenever,\" or \"at\" in their reminder text."
    );
    supported("Voidslime");
    // Ministrant of Obligation: "Afterlife 2". P1's Ministrant dies; P0 counters the
    // afterlife trigger with Voidslime ("Counter target spell, activated ability, or
    // triggered ability.").
    let mut t = TestGame::new(2);
    let ministrant = t.battlefield(P1, "Ministrant of Obligation");
    crate::r_s02_common::destroy(&mut t, ministrant);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trigger = top_of_stack(&t);
    crate::r_s25_common::cast_new(&mut t, P0, "Voidslime", &[Entity::Object(trigger)]);
    t.resolve_all();
    assert!(tokens_of(&t, P1).is_empty());
    assert!(t.in_graveyard(P0, "Voidslime"));
}
