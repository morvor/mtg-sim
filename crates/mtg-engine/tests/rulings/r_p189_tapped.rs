//! Rulings batch P189 — "becomes tapped" triggers (CR 603.2e: they trigger only when a
//! permanent actually changes from untapped to tapped, not as it enters tapped), and
//! triggered mana abilities (CR 605.1b, 605.4a).

use crate::r_p076_common::mana;
use crate::r_p189_common::*;
use crate::r_s01_common::{attack_with, supported, with_subtype};
use crate::r_s02_common::can_activate;
use mtg_engine::ability::{Destination, Effect, Sel};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts the real card `name` onto the battlefield tapped under `p`'s control, from `p`'s
/// hand, as an effect would; returns the new permanent.
fn enter_tapped(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let card = t.hand(p, name);
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(card)]];
    t.g.exec(
        &Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield().under_your_control().tapped(),
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    let now = t.g.current(card);
    assert!(t.on_battlefield(now) && t.obj(now).tapped);
    now
}

/// Taps `obj` as an effect would; returns whether it became tapped.
fn tap_by_effect(t: &mut TestGame, obj: ObjectId) -> bool {
    let tapped = t.g.tap(obj);
    t.g.flush_events();
    t.settle();
    tapped
}

/// `source` (the real card `name`) triggers when an effect taps it, but not when an effect
/// "taps" it while it's already tapped.
fn only_actual_tapping_triggers(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    let me = t.battlefield(P0, name);
    assert!(tap_by_effect(&mut t, me));
    assert_eq!(triggers_of(&t, me), 1, "{name}");
    t.resolve_all();
    let me = t.g.current(me);
    assert!(!tap_by_effect(&mut t, me));
    t.resolve_all();
    assert_eq!(triggers_of(&t, me), 1, "{name}: retriggered while already tapped");
}

#[test]
fn emmara_only_actual_tapping_triggers() {
    cr!("603.2e", "701.26a");
    ruling!(
        "Emmara, Soul of the Accord",
        "For the ability to trigger, Emmara has to actually change from untapped to tapped."
    );
    only_actual_tapping_triggers("Emmara, Soul of the Accord");
}

#[test]
fn reckless_racer_only_actual_tapping_triggers() {
    cr!("603.2e", "701.26a");
    ruling!(
        "Reckless Racer",
        "For the ability to trigger, Reckless Racer has to actually change from untapped to tapped."
    );
    only_actual_tapping_triggers("Reckless Racer");
}

#[test]
fn spireside_infiltrator_only_actual_tapping_triggers() {
    cr!("603.2e", "701.26a");
    ruling!(
        "Spireside Infiltrator",
        "For the ability to trigger, Spireside Infiltrator has to actually change from untapped to tapped."
    );
    only_actual_tapping_triggers("Spireside Infiltrator");
}

#[test]
fn judge_and_veteran_only_actual_tapping_triggers() {
    cr!("603.2e", "701.26a");
    // Judge of Currents: "Whenever a Merfolk you control becomes tapped" (it's a Merfolk
    // itself); Veteran of the Depths: "Whenever this creature becomes tapped".
    ruling!(
        "Judge of Currents",
        "For the ability to trigger, the creature has to actually change from untapped to tapped."
    );
    ruling!(
        "Veteran of the Depths",
        "For the ability to trigger, the creature has to actually change from untapped to tapped."
    );
    only_actual_tapping_triggers("Judge of Currents");
    only_actual_tapping_triggers("Veteran of the Depths");
}

/// The real card `name` has no activated ability to tap it; attacking taps it and its
/// trigger resolves. Returns the game in the declare attackers step, after resolving.
fn triggered_not_activated(name: &str) -> (TestGame, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    let me = t.battlefield(P0, name);
    assert!(!can_activate(&mut t, P0, me), "{name} has an activated ability");
    t.hand(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    attack_with(&mut t, &[(me, Entity::Player(P1))]);
    assert!(t.obj(me).tapped);
    assert_eq!(triggers_of(&t, me), 1, "{name}");
    t.resolve_all();
    (t, me)
}

#[test]
fn emmara_triggered_not_activated() {
    cr!("603.2e", "508.1f");
    ruling!(
        "Emmara, Soul of the Accord",
        "Emmara's ability is a triggered ability, not an activated ability. It doesn't allow you to tap Emmara whenever you want"
    );
    let (t, _) = triggered_not_activated("Emmara, Soul of the Accord");
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 1);
}

#[test]
fn spireside_infiltrator_triggered_not_activated() {
    cr!("603.2e", "508.1f");
    ruling!(
        "Spireside Infiltrator",
        "This is a triggered ability, not an activated ability. It doesn't allow you to tap Spireside Infiltrator whenever you want"
    );
    let (t, _) = triggered_not_activated("Spireside Infiltrator");
    assert_eq!(t.life(P1), 19);
}

#[test]
fn reckless_racer_triggered_not_activated_crewing() {
    cr!("603.2e", "702.122a");
    ruling!(
        "Reckless Racer",
        "It doesn’t allow you to tap Reckless Racer whenever you want; rather, you need some other way of tapping it, such as by attacking or crewing a Vehicle."
    );
    let (t, _) = triggered_not_activated("Reckless Racer");
    assert_eq!(t.g.player(P0).graveyard.len(), 1, "discarded a card");
    // Crewing a Vehicle taps it too.
    supported("Smuggler's Copter");
    let mut t = TestGame::new(2);
    let racer = t.battlefield(P0, "Reckless Racer");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(racer)]);
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.activate(P0, copter, 0, &[]).unwrap();
    t.settle();
    assert!(t.obj(racer).tapped);
    assert_eq!(triggers_of(&t, racer), 1);
}

#[test]
fn goblin_medics_triggers_when_it_taps_to_attack() {
    cr!("603.2e", "508.1f");
    ruling!(
        "Goblin Medics",
        "The ability does trigger when it taps to attack."
    );
    supported("Goblin Medics");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Goblin Medics");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(gm, Entity::Player(P1))]);
    assert_eq!(triggers_of(&t, gm), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn vampire_envoy_attacking_but_not_entering_tapped() {
    cr!("603.2e", "110.5b");
    ruling!(
        "Vampire Envoy",
        "Vampire Envoy’s last ability will trigger if it becomes tapped for any reason, including attacking. However, if it enters the battlefield tapped for some reason, the ability won’t trigger."
    );
    supported("Vampire Envoy");
    let mut t = TestGame::new(2);
    let ve = enter_tapped(&mut t, P0, "Vampire Envoy");
    t.resolve_all();
    assert_eq!(triggers_of(&t, ve), 0);
    assert_eq!(t.life(P0), 20);
    let ve2 = t.battlefield(P0, "Vampire Envoy");
    attack_with(&mut t, &[(ve2, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

// ---------------------------------------------------------------------------
// Quest for Renewal: "Whenever a creature you control becomes tapped, you may put a quest
// counter on this enchantment. As long as there are four or more quest counters on this
// enchantment, untap all creatures you control during each other player's untap step."
// ---------------------------------------------------------------------------

#[test]
fn quest_for_renewal_creatures_entering_tapped_dont_trigger() {
    cr!("603.2e", "110.5b");
    ruling!(
        "Quest for Renewal",
        "Creatures put onto the battlefield tapped don’t cause Quest for Renewal’s first ability to trigger."
    );
    supported("Quest for Renewal");
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Quest for Renewal");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    enter_tapped(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(triggers_of(&t, q), 0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap_by_effect(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(q, "quest"), 1);
}

/// P0 controls `quests` Quests for Renewal with four quest counters, `tapped` creatures
/// (all tapped) and a tapped Forest; the game advances into P1's upkeep.
fn quest_untap_step(quests: usize, creatures: &[&str]) -> (TestGame, Vec<ObjectId>, ObjectId) {
    supported("Quest for Renewal");
    let mut t = TestGame::new(2);
    for _ in 0..quests {
        let q = t.battlefield(P0, "Quest for Renewal");
        t.g.add_counters(Entity::Object(q), "quest", 4, None);
    }
    let cs: Vec<ObjectId> = creatures.iter().map(|n| t.battlefield(P0, n)).collect();
    let forest = t.battlefield(P0, "Forest");
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    for id in cs.iter().chain([&forest, &p1_bears]) {
        t.g.objects[id.0 as usize].tapped = true;
    }
    t.g.recompute();
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj(p1_bears).tapped, "the active player's permanents untap");
    (t, cs, forest)
}

#[test]
fn quest_for_renewal_untaps_creatures_in_other_untap_steps() {
    cr!("502.3");
    ruling!(
        "Quest for Renewal",
        "As another player’s untap step begins, if there are four or more quest counters on Quest for Renewal, all your creatures untap during that untap step."
    );
    let (t, cs, forest) = quest_untap_step(1, &["Grizzly Bears", "Hill Giant"]);
    assert!(cs.iter().all(|c| !t.obj(*c).tapped));
    assert!(t.obj(forest).tapped, "only creatures");
}

#[test]
fn quest_for_renewal_ignores_your_untap_step_restrictions() {
    cr!("502.3");
    ruling!(
        "Quest for Renewal",
        "During another player’s untap step, effects that would otherwise cause your creatures to stay tapped don’t apply because they apply only during *your* untap step."
    );
    supported("Deep-Slumber Titan");
    let (t, cs, _) = quest_untap_step(1, &["Deep-Slumber Titan"]);
    assert!(!t.obj(cs[0]).tapped);
}

#[test]
fn quest_for_renewal_two_quests_are_redundant() {
    cr!("502.3");
    ruling!(
        "Quest for Renewal",
        "Controlling more than one Quest for Renewal with four or more quest counters on it is redundant. You can’t untap your permanents more than once in a single untap step."
    );
    // Servant of Tymaret: "Inspired — Whenever this creature becomes untapped, each
    // opponent loses 1 life. You gain life equal to the life lost this way."
    supported("Servant of Tymaret");
    let (mut t, cs, _) = quest_untap_step(2, &["Servant of Tymaret"]);
    t.resolve_all();
    assert!(!t.obj(cs[0]).tapped);
    assert_eq!(triggers_of(&t, cs[0]), 1);
    assert_eq!(t.life(P1), 19);
}

// ---------------------------------------------------------------------------
// Scaretiller: "Whenever this creature becomes tapped, choose one — • You may put a land
// card from your hand onto the battlefield tapped. • Return target land card from your
// graveyard to the battlefield tapped."
// ---------------------------------------------------------------------------

#[test]
fn scaretiller_triggered_not_activated() {
    cr!("603.2e", "508.1f");
    ruling!(
        "Scaretiller",
        "Scaretiller's ability is a triggered ability, not an activated ability. It doesn't allow you to tap it whenever you want"
    );
    supported("Scaretiller");
    let mut t = TestGame::new(2);
    let st = t.battlefield(P0, "Scaretiller");
    assert!(!can_activate(&mut t, P0, st));
    t.hand(P0, "Forest");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    attack_with(&mut t, &[(st, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
}

#[test]
fn scaretiller_doesnt_count_as_playing_a_land() {
    cr!("305.4", "305.2");
    ruling!(
        "Scaretiller",
        "Neither mode of Scaretiller's ability counts as playing a land. It can put a land card onto the battlefield even if you've already played your land for the turn, and even if it's not your turn."
    );
    supported("Scaretiller");
    let mut t = TestGame::new(2);
    let st = t.battlefield(P0, "Scaretiller");
    let f = t.hand(P0, "Forest");
    t.play_land(P0, f).unwrap();
    t.graveyard(P0, "Island");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    tap_by_effect(&mut t, st);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Island").len(), 1);
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1, "only the Forest");
    // A land can still be put onto the battlefield on an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    t.g.untap(t.g.current(st));
    t.hand(P0, "Mountain");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    let st = t.g.current(st);
    tap_by_effect(&mut t, st);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mountain").len(), 1);
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);
}

#[test]
fn scaretiller_tapped_for_a_cost_resolves_first() {
    cr!("603.3", "702.122a", "602.2");
    ruling!(
        "Scaretiller",
        "If Scaretiller becomes tapped while casting a spell or activating an ability, that spell or ability resolves after Scaretiller's triggered ability has resolved."
    );
    supported("Scaretiller");
    supported("Smuggler's Copter");
    let mut t = TestGame::new(2);
    let st = t.battlefield(P0, "Scaretiller");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    t.hand(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(st)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.activate(P0, copter, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(*t.g.stack.last().unwrap(), stack_triggers_from(&t, st)[0]);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert!(!t.obj(copter).is(CardType::Creature), "crew hasn't resolved yet");
    t.resolve();
    assert!(t.obj(copter).is(CardType::Creature));
}

// ---------------------------------------------------------------------------
// Triggered mana abilities (CR 605.1b, 605.4a) and Mana Web.
// ---------------------------------------------------------------------------

#[test]
fn barbflare_gremlin_is_a_mana_ability() {
    cr!("605.1b", "605.4a");
    ruling!(
        "Barbflare Gremlin",
        "Barbflare Gremlin's last ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Barbflare Gremlin");
    let mut t = TestGame::new(2);
    let bg = t.battlefield(P0, "Barbflare Gremlin");
    t.g.objects[bg.0 as usize].tapped = true;
    let forest = t.battlefield(P1, "Forest");
    t.activate(P1, forest, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P1).mana_pool.count(ManaType::G), 2);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn vernal_bloom_is_a_mana_ability() {
    cr!("605.1b", "605.4a", "106.12a");
    ruling!(
        "Vernal Bloom",
        "This is a triggered mana ability. It does not go on the stack."
    );
    supported("Vernal Bloom");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vernal Bloom");
    let forest = t.battlefield(P1, "Forest");
    t.activate(P1, forest, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P1).mana_pool.count(ManaType::G), 2);
}

#[test]
fn wild_growth_mana_isnt_something_the_land_can_produce() {
    cr!("605.1b", "106.12a");
    ruling!(
        "Wild Growth",
        "The additional mana is not an ability of the land and is not something the land can produce."
    );
    supported("Wild Growth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Web");
    let island = t.battlefield(P1, "Island");
    let forest = t.battlefield(P1, "Forest");
    let island2 = t.battlefield(P1, "Island");
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P1, ManaType::G, 1);
    let wg = t.hand(P1, "Wild Growth");
    t.cast(P1, wg).target(island).go();
    t.resolve_all();
    let island = t.g.current(island);
    assert_eq!(t.obj(island).chars.abilities.len(), 1, "no ability of the land");
    t.activate(P1, island, 0, &[]).unwrap();
    let pool = &t.g.player(P1).mana_pool;
    assert_eq!(
        (pool.count(ManaType::U), pool.count(ManaType::G)),
        (1, 1)
    );
    t.resolve_all();
    // Mana Web taps lands that could produce {U} (what the Island can produce), not {G}.
    assert!(t.obj(island2).tapped);
    assert!(!t.obj(forest).tapped);
}

#[test]
fn mana_web_checks_every_mana_ability_of_the_land() {
    cr!("106.7", "603.2");
    ruling!(
        "Mana Web",
        "this card checks which types of mana (white, blue, black, red, green, or colorless) that land could produce using any of its mana abilities, not merely the one the player just used."
    );
    supported("Brushland");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Web");
    let brush = t.battlefield(P1, "Brushland");
    let forest = t.battlefield(P1, "Forest");
    let plains = t.battlefield(P1, "Plains");
    let island = t.battlefield(P1, "Island");
    let wastes = t.battlefield(P1, "Wastes");
    // Tapped for {C}: lands that could produce {C}, {G} or {W} are tapped.
    t.activate(P1, brush, 0, &[]).unwrap();
    assert_eq!(t.g.player(P1).mana_pool.count(ManaType::C), 1);
    t.resolve_all();
    assert!(t.obj(forest).tapped && t.obj(plains).tapped && t.obj(wastes).tapped);
    assert!(!t.obj(island).tapped);
}

#[test]
fn mana_web_opponent_taps_lands_in_response() {
    cr!("605.3a", "603.3");
    ruling!(
        "Mana Web",
        "The opponent can tap their lands for mana in response to Mana Web’s ability triggering."
    );
    let mut t = TestGame::new(2);
    let web = t.battlefield(P0, "Mana Web");
    let f1 = t.battlefield(P1, "Forest");
    let f2 = t.battlefield(P1, "Forest");
    t.activate(P1, f1, 0, &[]).unwrap();
    t.settle();
    assert_eq!(stack_triggers_from(&t, web).len(), 1);
    // In response, the other Forest is tapped for mana.
    t.activate(P1, f2, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj(f2).tapped);
    assert_eq!(t.g.player(P1).mana_pool.count(ManaType::G), 2);
}
