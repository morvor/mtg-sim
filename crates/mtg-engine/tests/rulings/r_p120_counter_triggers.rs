//! Rulings batch P120 — triggered abilities around +1/+1 counters: abilities that trigger
//! on drawing, gaining life, becoming a target, entering or counters being put (CR 603.2,
//! 603.2c, 603.6a), intervening "if" clauses checked on triggering and again on resolution
//! (CR 603.4), and abilities that look back at their source (CR 603.10, 608.2h).

use crate::r_p120_common::*;
use crate::r_s24_common::enter_together;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Removes `n` counters of `kind` from the permanent and settles.
fn remove(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.remove_counters(Entity::Object(id), kind, n);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Advances to P0's next upkeep and puts its triggered abilities on the stack.
fn to_upkeep(t: &mut TestGame) {
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

// --- Simic Ascendancy ----------------------------------------------------------------------

/// P0's Simic Ascendancy with `n` growth counters.
fn ascendancy(t: &mut TestGame, n: u32) -> ObjectId {
    let a = t.battlefield(P0, "Simic Ascendancy");
    put_counters(t, a, "growth", n);
    a
}

#[test]
fn simic_ascendancy_with_nineteen_counters_doesnt_trigger() {
    cr!("603.4");
    ruling!(
        "Simic Ascendancy",
        "If Simic Ascendancy doesn't have twenty or more growth counters on it as your upkeep begins, its last ability won't trigger."
    );
    supported("Simic Ascendancy");
    let mut t = TestGame::new(2);
    ascendancy(&mut t, 19);
    to_upkeep(&mut t);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(!t.has_lost(P1));
}

#[test]
fn simic_ascendancy_that_left_uses_its_last_counters() {
    cr!("603.4", "608.2h");
    ruling!(
        "Simic Ascendancy",
        "If the last ability does trigger, but Simic Ascendancy leaves the battlefield, use the number of counters it had on it immediately before it left the battlefield to determine whether you win the game."
    );
    let mut t = TestGame::new(2);
    let a = ascendancy(&mut t, 20);
    to_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, a);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn simic_ascendancy_with_counters_removed_doesnt_win() {
    cr!("603.4");
    ruling!(
        "Simic Ascendancy",
        "If the last ability does trigger, but counters are removed from Simic Ascendancy so it has fewer than twenty remaining on it, you won't win the game."
    );
    let mut t = TestGame::new(2);
    let a = ascendancy(&mut t, 20);
    to_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    remove(&mut t, a, "growth", 1);
    t.resolve_all();
    assert!(!t.has_lost(P1));
}

// --- entering ------------------------------------------------------------------------------

#[test]
fn skyknight_squire_triggers_for_creatures_entering_with_it() {
    cr!("603.6a");
    ruling!(
        "Skyknight Squire",
        "If Skyknight Squire enters at the same time as one or more other creatures you control, its first ability will trigger for each of those other creatures."
    );
    supported("Skyknight Squire");
    let mut t = TestGame::new(2);
    let ids = enter_together(
        &mut t,
        &[
            (P0, "Skyknight Squire"),
            (P0, "Grizzly Bears"),
            (P0, "Hill Giant"),
        ],
    );
    t.resolve_all();
    assert_eq!(plus1(&t, ids[0]), 2);
}

#[test]
fn the_flesh_is_weak_kills_a_one_toughness_creature_before_its_counter() {
    cr!("704.5f", "603.3", "613.4c");
    ruling!(
        "The Flesh Is Weak",
        "If The Flesh is Weak enters the battlefield while you control a nonartifact creature with 1 toughness and no +1/+1 counters on it, the last ability will cause its toughness to become 0 and it will die before the first ability can put a counter on it."
    );
    supported("The Flesh Is Weak");
    let mut t = TestGame::new(2);
    let vanguard = t.battlefield(P0, "Elite Vanguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "The Flesh Is Weak");
    t.settle();
    assert!(t.in_graveyard(P0, "Elite Vanguard"));
    assert!(!t.on_battlefield(vanguard));
    t.resolve_all();
    // The Bears gets its counter and becomes an artifact: no longer -1/-1.
    assert_eq!(plus1(&t, bears), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn foundry_hornet_entering_with_a_counter_satisfies_itself() {
    cr!("603.4", "614.1c");
    ruling!(
        "Foundry Hornet",
        "If an effect causes Foundry Hornet to enter the battlefield with a +1/+1 counter on it, it satisfies its own triggered ability."
    );
    supported("Foundry Hornet");
    supported("Grumgully, the Generous");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grumgully, the Generous");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let hornet = t.enter(P0, "Foundry Hornet");
    assert_eq!(plus1(&t, hornet), 1);
    t.resolve_all();
    assert_eq!(t.pt(theirs), (1, 1));
}

#[test]
fn foundry_hornet_checks_on_entering_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Foundry Hornet",
        "If you don’t control a creature with a +1/+1 counter as Foundry Hornet enters the battlefield, its ability doesn’t trigger, even if you can give a +1/+1 counter to a creature right away. If you control no creatures with a +1/+1 counter as the ability resolves, nothing happens."
    );
    // No creature with a counter as it enters: no trigger.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Foundry Hornet");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.pt(theirs), (2, 2));
    // A counter as it enters, gone as the ability resolves: nothing happens.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, mine, 1);
    t.enter(P0, "Foundry Hornet");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    remove(&mut t, mine, PLUS1, 1);
    t.resolve_all();
    assert_eq!(t.pt(theirs), (2, 2));
}

// --- drawing -------------------------------------------------------------------------------

#[test]
fn toothy_doesnt_trigger_for_cards_put_into_hand() {
    cr!("121.2");
    ruling!(
        "Toothy, Imaginary Friend",
        "If a spell or ability causes you to put cards in your hand without specifically using the word \"draw,\" Toothy's middle ability won't trigger."
    );
    supported("Impulse");
    let mut t = TestGame::new(2);
    let toothy = t.battlefield(P0, "Toothy, Imaginary Friend");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Impulse", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(plus1(&t, toothy), 0);
}

#[test]
fn toothy_triggers_per_card_drawn_and_pir_adds_one_to_each() {
    cr!("121.2", "603.2c", "614.1a");
    ruling!(
        "Toothy, Imaginary Friend",
        "If you draw multiple cards, Toothy's middle ability triggers that many times. If your team also controls Pir, each of those resolving triggered abilities puts two +1/+1 counters on Toothy."
    );
    supported("Divination");
    supported("Pir, Imaginative Rascal");
    for pir in [false, true] {
        let mut t = TestGame::new(2);
        let toothy = t.battlefield(P0, "Toothy, Imaginary Friend");
        if pir {
            t.battlefield(P0, "Pir, Imaginative Rascal");
        }
        cast_new(&mut t, P0, "Divination", &[]);
        t.resolve();
        assert_eq!(triggers_on_stack(&t, "Whenever you draw a card"), 2);
        t.resolve_all();
        assert_eq!(plus1(&t, toothy), if pir { 4 } else { 2 });
    }
}

#[test]
fn chasm_skulker_triggers_per_card_drawn() {
    cr!("121.2", "603.2c");
    ruling!(
        "Chasm Skulker",
        "If you draw multiple cards, the first ability will trigger that many times. Each of these abilities will cause a +1/+1 counter to be put on Chasm Skulker."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Chasm Skulker");
    cast_new(&mut t, P0, "Divination", &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(plus1(&t, s), 2);
}

#[test]
fn thought_gorger_counts_and_discards_the_hand_on_resolution() {
    cr!("608.2", "608.2c");
    ruling!(
        "Thought Gorger",
        "Once an ability starts to resolve, it's too late to respond to it. For example, you can't count a card in your hand while determining how many +1/+1 counters Thought Gorger gets and then cast it in response. If a card is counted, it'll be discarded."
    );
    supported("Thought Gorger");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P0, "Lightning Bolt");
    }
    let hand = t.hand_size(P0);
    let gorger = t.enter(P0, "Thought Gorger");
    t.resolve_all();
    assert_eq!(plus1(&t, gorger), hand as u32);
    assert_eq!(t.hand_size(P0), 0);
}

// --- gaining life --------------------------------------------------------------------------

#[test]
fn life_gained_for_each_is_one_event() {
    cr!("119.9", "603.2c");
    ruling!(
        "Aerith Gainsborough",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Aerith Gainsborough's second ability triggers only once."
    );
    ruling!(
        "Voice of the Blessed",
        "If you gain an amount of life “for each” of something, that life is gained as one event and the ability of Voice of the Blessed triggers only once."
    );
    supported("Congregate");
    let mut t = TestGame::new(2);
    let aerith = t.battlefield(P0, "Aerith Gainsborough");
    let voice = t.battlefield(P0, "Voice of the Blessed");
    t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Congregate", &[Entity::Player(P0)]);
    t.resolve();
    assert_eq!(t.life(P0), 26);
    t.resolve_all();
    assert_eq!(plus1(&t, aerith), 1);
    assert_eq!(plus1(&t, voice), 1);
}

#[test]
fn aerith_doesnt_trigger_for_a_teammates_life_gain() {
    cr!("810.9", "603.2");
    ruling!(
        "Aerith Gainsborough",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Aerith Gainsborough's second ability to trigger, even though it caused your team's life total to increase."
    );
    let mut t = two_headed_giant();
    let aerith = t.battlefield(P0, "Aerith Gainsborough");
    let life = t.life(P0);
    gain_life(&mut t, P1, 3);
    assert_eq!(t.life(P0), life + 3);
    t.resolve_all();
    assert_eq!(plus1(&t, aerith), 0);
    gain_life(&mut t, P0, 3);
    t.resolve_all();
    assert_eq!(plus1(&t, aerith), 1);
}

// --- becoming a target ---------------------------------------------------------------------

#[test]
fn angelic_cub_triggers_when_a_target_is_changed_to_it() {
    cr!("603.2", "115.7");
    ruling!(
        "Angelic Cub",
        "If a spell or ability has one or more of its targets changed to Angelic Cub, Angelic Cub's first ability will trigger if it hasn't yet been the target of a spell or ability this turn."
    );
    supported("Angelic Cub");
    supported("Redirect");
    let mut t = TestGame::new(2);
    let cub = t.battlefield(P0, "Angelic Cub");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    cast_new(&mut t, P0, "Redirect", &[obj(growth)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(cub)]);
    t.resolve();
    assert_eq!(crate::r_s25_common::targets_of(&t, growth), vec![obj(cub)]);
    t.resolve_all();
    assert_eq!(plus1(&t, cub), 1);
}

#[test]
fn angelic_cub_triggers_when_a_copy_targets_it() {
    cr!("603.2", "707.10c");
    ruling!(
        "Angelic Cub",
        "If you create a copy of a spell on the stack and target Angelic Cub, Angelic Cub's first ability will trigger as long as it hasn't yet been the target of a spell or ability this turn."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    let cub = t.battlefield(P0, "Angelic Cub");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    cast_new(&mut t, P0, "Twincast", &[obj(growth)]);
    crate::r_s25_common::change_copy_targets(&mut t, P0, &[Some(obj(cub))]);
    t.resolve();
    t.resolve_all();
    assert_eq!(plus1(&t, cub), 1);
}

#[test]
fn swarm_shambler_doesnt_trigger_for_a_counter_put_after_targeting() {
    cr!("603.2");
    ruling!(
        "Swarm Shambler",
        "If a creature without a +1/+1 counter on it becomes the target of a spell an opponent controls, putting a +1/+1 counter on it after that won’t cause Swarm Shambler’s middle ability to trigger."
    );
    supported("Swarm Shambler");
    let mut t = TestGame::new(2);
    t.enter(P0, "Swarm Shambler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
    give_plus1(&mut t, bears, 1);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Insect"), 0);
}

#[test]
fn swarm_shambler_triggers_once_for_a_spell_targeting_a_creature_twice() {
    cr!("603.2");
    ruling!(
        "Swarm Shambler",
        "If a spell targets a creature you control with a +1/+1 counter on it more than once, Swarm Shambler’s ability triggers only once."
    );
    supported("Seeds of Strength");
    let mut t = TestGame::new(2);
    t.enter(P0, "Swarm Shambler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    cast_new(&mut t, P1, "Seeds of Strength", &[obj(bears), obj(bears), obj(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Insect"), 1);
    assert_eq!(t.pt(bears), (6, 6));
}

// --- intervening "if" and loops ------------------------------------------------------------

#[test]
fn ayaras_oathsworn_rechecks_its_counters_on_resolution() {
    cr!("603.4");
    ruling!(
        "Ayara's Oathsworn",
        "If the triggered ability does trigger, that ability will check Ayara's Oathsworn again as it tries to resolve. If Ayara's Oathsworn has four or more +1/+1 counters on it at that time, the ability won't resolve."
    );
    supported("Ayara's Oathsworn");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ayara's Oathsworn");
    give_plus1(&mut t, a, 3);
    let library = t.library_size(P0);
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to_step(Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.stack_len(), 1);
    give_plus1(&mut t, a, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, a), 4);
    assert_eq!(t.library_size(P0), library);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn two_enduring_scalelords_loop_until_you_decline() {
    cr!("603.2", "603.5");
    ruling!(
        "Enduring Scalelord",
        "If you control two Enduring Scalelords, putting a +1/+1 counter on one of them will cause the ability of the other one to trigger."
    );
    supported("Enduring Scalelord");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Enduring Scalelord");
    let b = t.battlefield(P0, "Enduring Scalelord");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    give_plus1(&mut t, a, 1);
    t.resolve_all();
    // A: 1 + 1 (from B's trigger's trigger); B: 1; then the loop stops.
    assert_eq!(plus1(&t, a), 2);
    assert_eq!(plus1(&t, b), 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn noosegraf_mob_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5");
    ruling!(
        "Noosegraf Mob",
        "Noosegraf Mob’s triggered ability resolves before the spell that caused it to trigger."
    );
    supported("Noosegraf Mob");
    let mut t = TestGame::new(2);
    let mob = t.enter(P0, "Noosegraf Mob");
    assert_eq!(plus1(&t, mob), 5);
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(tokens_named(&t, P0, "Zombie"), 1);
    assert_eq!(plus1(&t, mob), 4);
    assert_eq!(t.stack_len(), 1, "the Bears spell is still on the stack");
}

// --- Moss-Pit Skeleton ---------------------------------------------------------------------

#[test]
fn moss_pit_skeleton_triggers_for_entering_with_counters_and_for_counters_put() {
    cr!("603.2", "122.6");
    ruling!(
        "Moss-Pit Skeleton",
        "Moss-Pit Skeleton's last ability triggers if a creature enters the battlefield under your control with +1/+1 counters on it, as well as when +1/+1 counters are put on a creature you already control."
    );
    supported("Moss-Pit Skeleton");
    for entering in [false, true] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Moss-Pit Skeleton");
        t.answer_yes(P0, true);
        if entering {
            t.enter(P0, "Grakmaw, Skyclave Ravager");
        } else {
            let bears = t.battlefield(P0, "Grizzly Bears");
            give_plus1(&mut t, bears, 1);
        }
        t.resolve_all();
        assert!(!t.in_graveyard(P0, "Moss-Pit Skeleton"), "entering = {entering}");
        let top = *t.g.player(P0).library.last().unwrap();
        assert_eq!(t.g.obj(top).chars.name, "Moss-Pit Skeleton");
    }
}

#[test]
fn moss_pit_skeleton_must_already_be_in_the_graveyard() {
    cr!("603.2", "603.10");
    ruling!(
        "Moss-Pit Skeleton",
        "Moss-Pit Skeleton's last ability triggers only if it's already in your graveyard as +1/+1 counters are put onto a creature you control."
    );
    let mut t = TestGame::new(2);
    let skeleton = t.battlefield(P0, "Moss-Pit Skeleton");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    give_plus1(&mut t, bears, 1);
    assert_eq!(t.stack_len(), 0);
    destroy(&mut t, skeleton);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Moss-Pit Skeleton"));
}

