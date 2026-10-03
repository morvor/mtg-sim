//! Rulings batch P203 — connive (CR 701.50): conniving creatures that left the
//! battlefield, "when it connives this way" reflexive triggers, several creatures
//! conniving at once, and resolution without interruption.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::*;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Whether any permanent has a +1/+1 counter.
fn any_counters(t: &TestGame) -> bool {
    t.g.permanents().any(|o| o.counter(counters::PLUS1) > 0)
}

#[test]
fn a_creature_that_left_still_connives_and_its_connive_this_way_trigger_triggers() {
    cr!("701.50a", "701.50b", "701.50f", "603.12");
    ruling!(
        "Psychic Pickpocket",
        "If a resolving spell or ability instructs a specific creature to connive but that creature has left the battlefield, the creature still connives. Abilities that trigger “when [that creature] connives, such as that of Psychic Pickpocket, will trigger."
    );
    ruling!(
        "Spymaster's Vault",
        "If a resolving spell or ability instructs a specific creature to connive but that creature has left the battlefield, the creature still connives, although you can't put any +1/+1 counters on it. Abilities that trigger \"when [that creature] connives\" will trigger."
    );
    supported("Psychic Pickpocket");
    // Psychic Pickpocket: "When this creature enters, it connives. When it connives this
    // way, return up to one target nonland permanent to its owner's hand." It's destroyed
    // with its enters trigger on the stack; P0 then draws Shock and discards it (a nonland
    // card: no counter can be put on the Pickpocket).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    stack_library(&mut t, P0, &["Shock"]);
    let pp = t.enter(P0, "Psychic Pickpocket");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, pp);
    assert!(t.in_graveyard(P0, "Psychic Pickpocket"));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shock"));
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(!any_counters(&t));
}

#[test]
fn obscura_interceptor_that_left_still_connives_and_returns_a_spell() {
    cr!("701.50a", "701.50f", "603.12");
    ruling!(
        "Obscura Interceptor",
        "If a resolving spell or ability instructs a specific creature to connive but that creature has left the battlefield, the creature still connives. Abilities that trigger “when [that creature] connives,” such as that of Obscura Interceptor, will trigger."
    );
    supported("Obscura Interceptor");
    // P1 casts Opt; P0 flashes in Obscura Interceptor, which is destroyed with its enters
    // trigger on the stack. It still connives, and its reflexive trigger returns Opt.
    let mut t = TestGame::new(2);
    let opt = in_hand_with_mana(&mut t, P1, "Opt");
    let spell = t.cast(P1, opt).go();
    let oi = t.enter(P0, "Obscura Interceptor");
    t.settle();
    destroy(&mut t, oi);
    t.answer_targets(P0, &[Entity::Object(spell)]);
    let hand = t.hand_size(P0);
    t.resolve();
    // The connive: a card drawn and one discarded.
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.graveyard_size(P0), 2);
    t.resolve();
    assert!(t.in_hand(P1, "Opt"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn psychic_pickpockets_target_is_chosen_after_it_connives() {
    cr!("603.12", "701.50a");
    ruling!(
        "Psychic Pickpocket",
        "The target permanent to return is chosen after Psychic Pickpocket connives."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    stack_library(&mut t, P0, &["Shock"]);
    t.hand(P0, "Grizzly Bears");
    let pp = t.enter(P0, "Psychic Pickpocket");
    t.settle();
    let from = t.asked().len();
    // The enters trigger has no target ...
    assert!(target_candidates_since(&t, 0).is_empty());
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve();
    // ... it connived (discarding Shock: a counter), then the reflexive trigger went on
    // the stack with its target, and players get priority with it there.
    assert_eq!(t.counters(pp, counters::PLUS1), 1);
    assert_eq!(t.stack_len(), 1);
    let asked = t.asked()[from..].to_vec();
    let discard = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("discard")))
        .unwrap();
    let target = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .unwrap();
    assert!(discard < target);
    t.resolve();
    assert!(t.in_hand(P1, "Hill Giant"));
}

fn target_candidates_since(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn nobody_can_act_while_spymasters_vault_has_a_creature_connive() {
    cr!("701.50a", "701.50d", "608.2c", "117.4");
    ruling!(
        "Spymaster's Vault",
        "Once an ability that causes a creature to connive begins to resolve, no player may take any other actions until it's done. Notably, opponents can't try to remove the conniving creature after you discard cards but before it receives +1/+1 counters, if any."
    );
    supported("Spymaster's Vault");
    // Two creatures died this turn: the Hill Giant connives 2, drawing and discarding two
    // Shocks (two counters).
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        let b = t.battlefield(P1, "Grizzly Bears");
        destroy(&mut t, b);
    }
    t.battlefield(P0, "Swamp");
    let vault = t.battlefield(P0, "Spymaster's Vault");
    let giant = t.battlefield(P0, "Hill Giant");
    stack_library(&mut t, P0, &["Shock", "Shock"]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, vault, "connives").unwrap();
    let seen = watch(
        &mut t,
        P1,
        |_| true,
        |g| {
            let giant = g.find_in_zone(Zone::Battlefield, "Hill Giant")[0];
            (
                g.obj(giant).counter(counters::PLUS1),
                g.find_in_zone(Zone::Graveyard(P0), "Shock").len(),
            )
        },
    );
    let ok = t.g.run_until(1_000, |g| g.stack.is_empty());
    assert!(ok);
    assert_eq!(t.counters(giant, counters::PLUS1), 2);
    let seen = seen.lock().unwrap().clone();
    assert!(seen.contains(&(0, 0)));
    assert!(seen.iter().all(|s| *s == (0, 0) || *s == (2, 2)), "{seen:?}");
}

#[test]
fn creatures_conniving_at_once_connive_one_at_a_time_in_their_controllers_order() {
    cr!("701.50a", "701.50c");
    ruling!(
        "Lethal Scheme",
        "When multiple creatures are instructed to connive simultaneously, they connive one at a time in the order of their controller's choice."
    );
    supported("Lethal Scheme");
    // Two Bears convoke Lethal Scheme ({2}{B}{B}, two Swamps pay {B}{B}). P0 has the second
    // one connive first: it draws Shock (a nonland card: a counter), then the first one
    // draws a Plains (no counter).
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Hill Giant");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    stack_library(&mut t, P0, &["Shock", "Plains"]);
    let card = t.hand(P0, "Lethal Scheme");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Object(a), Entity::Object(b)]),
    );
    t.cast(P0, card).target(target).go();
    assert!(t.obj(a).tapped && t.obj(b).tapped);
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(choice_candidates(&t, from, "connives next").len(), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    assert_eq!(t.counters(a, counters::PLUS1), 0);
    assert!(t.in_graveyard(P0, "Shock") && t.in_graveyard(P0, "Plains"));
}

#[test]
fn copycrook_copying_nothing_is_a_0_0_without_the_connive_ability() {
    cr!("707.2", "704.5f");
    ruling!(
        "Copycrook",
        "You can choose not to copy anything. In that case, Copycrook simply enters the battlefield as a 0/0 creature and is probably put into your graveyard immediately, unless something else is increasing its toughness to keep it alive. It won't have \"Whenever this creature attacks, it connives.\""
    );
    supported("Copycrook");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[]);
    t.answer_yes(P0, false);
    let c = t.enter(P0, "Copycrook");
    t.settle();
    assert!(!t.on_battlefield(c));
    assert!(t.in_graveyard(P0, "Copycrook"));
    // With Glorious Anthem it survives as a 1/1 Copycrook without the attack trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    t.answer_choose(P0, &[]);
    t.answer_yes(P0, false);
    let c = t.enter(P0, "Copycrook");
    t.settle();
    assert!(t.on_battlefield(c));
    assert_eq!(t.pt(c), (1, 1));
    // (Only its own enter-as-a-copy replacement ability mentions conniving.)
    assert!(!t.obj_now(c).chars.abilities.iter().any(|a| matches!(
        a.kind,
        mtg_engine::ability::AbilityKind::Triggered(_)
    )));
}
