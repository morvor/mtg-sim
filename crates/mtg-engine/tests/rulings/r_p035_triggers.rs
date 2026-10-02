//! Rulings batch P035 — triggered abilities of "counters matter" cards: cast triggers that
//! resolve before their spell, abilities whose targets become illegal, and values
//! determined only as an ability resolves.

use crate::r_p035_common::*;
use crate::r_s01_common::{attack_with, give_mana_for, supported, tokens, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::move_to;
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Cast triggers resolve first
// ---------------------------------------------------------------------------------------

#[test]
fn sphinx_bone_wand_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5");
    ruling!(
        "Sphinx-Bone Wand",
        "If you cast an instant or sorcery spell, Sphinx-Bone Wand’s ability triggers and goes on the stack on top of it."
    );
    supported("Sphinx-Bone Wand");
    let mut t = TestGame::new(2);
    let wand = t.battlefield(P0, "Sphinx-Bone Wand");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The Wand's ability resolved; the Bolt is still on the stack.
    assert_eq!(t.counters(wand, counters::CHARGE), 1);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(bolt), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn sphinx_bone_wand_illegal_target_no_counter() {
    cr!("608.2b");
    ruling!(
        "Sphinx-Bone Wand",
        "If the targeted permanent or player is an illegal target by the time the ability resolves, the ability doesn’t resolve. You won’t put a charge counter on Sphinx-Bone Wand"
    );
    let mut t = TestGame::new(2);
    let wand = t.battlefield(P0, "Sphinx-Bone Wand");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    destroy(&mut t, bears);
    t.resolve();
    assert_eq!(t.counters(wand, counters::CHARGE), 0, "no charge counter");
    assert_eq!(t.stack_len(), 1, "only the Bolt is left");
}

#[test]
fn surrakar_spellblade_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5");
    ruling!(
        "Surrakar Spellblade",
        "If you cast an instant or sorcery spell, Surrakar Spellblade’s first ability triggers and goes on the stack on top of it."
    );
    supported("Surrakar Spellblade");
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Surrakar Spellblade");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_yes(P0, true);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.counters(blade, counters::CHARGE), 1);
    assert_eq!(t.zone(bolt), Zone::Stack, "the Bolt hasn't resolved yet");
}

#[test]
fn surrakar_spellblade_draws_all_or_nothing() {
    cr!("603.5");
    ruling!(
        "Surrakar Spellblade",
        "When Surrakar Spellblade’s second ability resolves, you either draw a card for each charge counter on it or you draw no cards at all."
    );
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let blade = t.battlefield(P0, "Surrakar Spellblade");
        put(&mut t, blade, counters::CHARGE, 3);
        let hand = t.hand_size(P0);
        attack_with(&mut t, &[(blade, Entity::Player(P1))]);
        t.answer_yes(P0, yes);
        t.advance_to(P0, Step::EndOfCombat);
        t.resolve_all();
        assert_eq!(t.life(P1), 18);
        let drawn = t.hand_size(P0) - hand;
        assert_eq!(drawn, if yes { 3 } else { 0 }, "yes = {yes}");
    }
}

#[test]
fn leaders_talent_level_3_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "113.7a");
    ruling!(
        "Leader's Talent",
        "Leader's Talent's level 3 ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Leader's Talent");
    supported("Counterspell");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Leader's Talent");
    mtg_engine::classes::set_level(&mut t.g, class, 3);
    t.g.recompute();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the Bolt and the trigger above it");
    // P1 counters the Bolt in response.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Bolt"), "countered");
    t.resolve_all();
    assert_eq!(
        t.counters(bears, counters::PLUS1),
        1,
        "the trigger still resolved"
    );
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------------------
// Illegal targets
// ---------------------------------------------------------------------------------------

#[test]
fn planewide_celebration_all_targets_illegal_nothing_happens() {
    cr!("608.2b", "700.2d");
    ruling!(
        "Planewide Celebration",
        "If the second mode is chosen at least once, and every target permanent card is an illegal target by the time Planewide Celebration tries to resolve, the spell doesn't resolve."
    );
    supported("Planewide Celebration");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Planewide Celebration");
    let pc = t.hand(P0, "Planewide Celebration");
    t.cast(P0, pc).modes(&[0, 1, 3, 3]).target(card).go();
    move_to(&mut t, card, Zone::Exile);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty(), "no Citizen");
    assert_eq!(t.life(P0), 20, "no life gained");
    // Two targets, one still legal: do as much as possible.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    give_mana_for(&mut t, P0, "Planewide Celebration");
    let pc = t.hand(P0, "Planewide Celebration");
    t.cast(P0, pc).modes(&[1, 1, 3, 3]).target(a).target(b).go();
    move_to(&mut t, a, Zone::Exile);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(
        t.in_exile("Grizzly Bears"),
        "the illegal target isn't affected"
    );
    assert_eq!(t.life(P0), 28);
}

#[test]
fn planewide_celebration_repeated_modes_are_performed_in_order() {
    cr!("700.2d", "608.2c");
    ruling!(
        "Planewide Celebration",
        "If a mode is chosen more than once, you perform that mode's instruction that many times sequentially."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    give_mana_for(&mut t, P0, "Planewide Celebration");
    let pc = t.hand(P0, "Planewide Celebration");
    // Create a Citizen, proliferate twice (choosing the Bears each time), gain 4 life.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, pc).modes(&[0, 2, 2, 3]).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 3, "proliferated twice");
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.life(P0), 24);
}

/// The Triumph of Anax on the battlefield with three lore counters; adding the fourth
/// triggers chapter IV.
fn anax_at_chapter_four(t: &mut TestGame, mine: ObjectId, theirs: ObjectId) -> ObjectId {
    let saga = t.battlefield(P0, "The Triumph of Anax");
    let s = t.g.current(saga);
    t.g.objects[s.0 as usize]
        .counters
        .insert(counters::LORE.into(), 3);
    t.answer_targets(P0, &[Entity::Object(mine)]);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    put(t, saga, counters::LORE, 1);
    assert_eq!(triggers_on_stack(t, "fights"), 1);
    saga
}

#[test]
fn triumph_of_anax_final_chapter_illegal_target_no_fight() {
    cr!("701.14b", "608.2b");
    ruling!(
        "The Triumph of Anax",
        "If either target is an illegal target as the final chapter ability resolves, no creature will deal or be dealt damage."
    );
    supported("The Triumph of Anax");
    for remove_mine in [true, false] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        anax_at_chapter_four(&mut t, mine, theirs);
        // Hexproof-free way of making one target illegal: it stops being a creature
        // controlled by the right player — here it leaves the battlefield.
        if remove_mine {
            move_to(&mut t, mine, Zone::Hand(P0));
        } else {
            move_to(&mut t, theirs, Zone::Hand(P1));
        }
        t.resolve_all();
        let survivor = if remove_mine { theirs } else { mine };
        assert_eq!(
            t.obj_now(survivor).damage,
            0,
            "remove mine {remove_mine}: no damage dealt"
        );
    }
    // Both legal: they fight.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    anax_at_chapter_four(&mut t, mine, theirs);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.obj_now(mine).damage, 2);
}

#[test]
fn triumph_of_anax_x_is_locked_in_as_the_chapter_resolves() {
    cr!("608.2h", "714.2b");
    ruling!(
        "The Triumph of Anax",
        "The value of X for the first three chapters is determined only as each chapter ability resolves."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let saga = t.enter(P0, "The Triumph of Anax");
    t.settle();
    // Before chapter I resolves, a second lore counter is added (chapter II triggers).
    t.answer_targets(P0, &[Entity::Object(bears)]);
    put(&mut t, saga, counters::LORE, 1);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // Chapter II resolved first, with two lore counters: +2/+0.
    assert_eq!(t.pt(bears), (4, 2));
    t.resolve();
    // Chapter I resolves now that there are two lore counters: +2/+0 more.
    assert_eq!(t.pt(bears), (6, 2));
    // Removing the lore counters later doesn't change the bonuses.
    let s = t.g.current(saga);
    t.g.remove_counters(Entity::Object(s), counters::LORE, 2);
    t.g.recompute();
    assert_eq!(t.pt(bears), (6, 2));
}

#[test]
fn concord_with_the_kami_first_mode_illegal_target_nothing_happens() {
    cr!("608.2b", "700.2");
    ruling!(
        "Concord with the Kami",
        "If you choose the first mode, and the target creature becomes an illegal target by the time the ability tries to resolve, the ability won't resolve and none of its effects will happen, even if you chose other modes."
    );
    supported("Concord with the Kami");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Concord with the Kami");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    // An enchanted creature: the second mode would draw a card.
    attach_new(&mut t, P0, "Rancor", bears);
    let giant = t.battlefield(P0, "Hill Giant");
    put(&mut t, giant, counters::CHARGE, 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "choose one or more"), 1);
    let hand = t.hand_size(P0);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "no card drawn");
}

#[test]
fn concord_with_the_kami_mode_availability() {
    cr!("700.2a", "700.2b");
    ruling!(
        "Concord with the Kami",
        "You cannot choose the first mode unless you have a legal target for it. You may choose the second and third mode even if you don't have an enchanted or equipped creature at the time this ability triggers."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Concord with the Kami");
    t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1, 2]));
    let from = t.asked().len();
    to_end_step(&mut t, P0);
    let available: Vec<Vec<usize>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseModes { available, .. } => {
                Some(available.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(available, vec![vec![1, 2]], "no creature with a counter");
    assert_eq!(
        triggers_on_stack(&t, "choose one or more"),
        1,
        "modes two and three"
    );
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(tokens(&t, P0).is_empty());
}

// ---------------------------------------------------------------------------------------
// Values determined as the ability resolves
// ---------------------------------------------------------------------------------------

#[test]
fn qala_x_is_calculated_once() {
    cr!("608.2h");
    ruling!(
        "Qala, Ajani's Pridemate",
        "The value of X is calculated only once, as Qala's first ability resolves."
    );
    supported("Qala, Ajani's Pridemate");
    let mut t = TestGame::new(2);
    let qala = t.battlefield(P0, "Qala, Ajani's Pridemate");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, qala, counters::PLUS1, 2);
    attack_with(
        &mut t,
        &[(qala, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    put(&mut t, qala, counters::PLUS1, 1);
    assert_eq!(t.pt(bears), (4, 2), "not updated");
}

#[test]
fn qala_one_trigger_per_life_gain_event() {
    cr!("119.9", "603.2");
    ruling!(
        "Qala, Ajani's Pridemate",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Qala's second ability triggers only once."
    );
    ruling!(
        "Qala, Ajani's Pridemate",
        "Qala's second ability triggers just once for each life-gaining event"
    );
    supported("Congregate");
    let mut t = TestGame::new(2);
    let qala = t.battlefield(P0, "Qala, Ajani's Pridemate");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    // 1 life: Qala's own ability.
    t.lands(P0, "Plains", 4);
    t.activate(P0, qala, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.counters(qala, counters::PLUS1), 1);
    // 6 life "for each creature": one event.
    give_mana_for(&mut t, P0, "Congregate");
    let c = t.hand(P0, "Congregate");
    t.cast(P0, c).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 27);
    assert_eq!(t.counters(qala, counters::PLUS1), 2);
}

#[test]
fn qala_teammate_life_gain_doesnt_trigger() {
    cr!("810.9", "119.9");
    ruling!(
        "Qala, Ajani's Pridemate",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Qala's second ability to trigger"
    );
    let mut t = crate::r_p050_common::two_headed_giant();
    let qala = t.battlefield(P0, "Qala, Ajani's Pridemate");
    give_mana_for(&mut t, P1, "Congregate");
    let c = t.hand(P1, "Congregate");
    t.cast(P1, c).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 32, "the team gained 2");
    assert_eq!(t.counters(qala, counters::PLUS1), 0);
}

#[test]
fn tromell_x_is_determined_as_the_ability_resolves() {
    cr!("608.2h", "701.34a");
    ruling!(
        "Tromell, Seymour's Butler",
        "The value of X is determined only once, as Tromell's last ability resolves."
    );
    supported("Tromell, Seymour's Butler");
    let mut t = TestGame::new(2);
    let tromell = t.battlefield(P0, "Tromell, Seymour's Butler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    // One nontoken creature entered this turn under P0's control, one under P1's (then
    // P0 gained control of it), and a token (not counted).
    t.enter(P0, "Elite Vanguard");
    let stolen = t.enter(P1, "Savannah Lions");
    crate::r_s06_common::give_control(&mut t, stolen, P0);
    crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    t.lands(P0, "Plains", 1);
    t.activate(P0, tromell, 0, &[]).unwrap();
    // In response, another creature enters: X counts it.
    t.enter(P0, "Hill Giant");
    for _ in 0..3 {
        t.answer_choose(P0, &[Entity::Object(bears)]);
    }
    t.resolve_all();
    assert_eq!(
        t.counters(bears, counters::PLUS1),
        4,
        "proliferated three times"
    );
}

#[test]
fn hydra_trainer_x_is_determined_once() {
    cr!("608.2h", "603.12");
    ruling!(
        "Hydra Trainer",
        "The value of X is determined only once, as the reflexive triggered ability resolves."
    );
    supported("Hydra Trainer");
    let mut t = TestGame::new(2);
    let trainer = t.battlefield(P0, "Hydra Trainer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    put(&mut t, trainer, counters::CHARGE, 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(&mut t, &[(trainer, Entity::Player(P1))]);
    t.resolve_all();
    // Two counters among permanents P0 controls: +2/+2.
    assert_eq!(t.pt(bears), (5, 5));
    put(&mut t, bears, counters::CHARGE, 3);
    assert_eq!(t.pt(bears), (5, 5), "not updated");
}

#[test]
fn magmatic_core_damage_is_divided_as_it_triggers() {
    cr!("601.2d", "603.3d", "608.2h");
    ruling!(
        "Magmatic Core",
        "The damage is divided when Magmatic Core’s ability triggers. Increasing or decreasing the number of age counters on Magmatic Core after the ability triggers but before it resolves has no effect on the amount of damage that’s dealt."
    );
    supported("Magmatic Core");
    let mut t = TestGame::new(2);
    let core = t.battlefield(P0, "Magmatic Core");
    put(&mut t, core, counters::AGE, 2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "divided"), 1);
    put(&mut t, core, counters::AGE, 3);
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.obj_now(b).damage, 1);
}

#[test]
fn magmatic_core_may_choose_zero_targets() {
    cr!("601.2c", "603.3d");
    ruling!(
        "Magmatic Core",
        "You can choose zero targets. In this case, Magmatic Core deals no damage."
    );
    let mut t = TestGame::new(2);
    let core = t.battlefield(P0, "Magmatic Core");
    put(&mut t, core, counters::AGE, 2);
    let a = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[]);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 0);
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------------------
// Compiler fixes found by these rulings
// ---------------------------------------------------------------------------------------

#[test]
fn hallar_deals_damage_equal_to_its_own_counters() {
    // "Hallar deals damage equal to the number of +1/+1 counters on it": "it" is Hallar
    // (as for Sphinx-Bone Wand), not the kicked spell that triggered the ability.
    cr!("603.2", "702.33d");
    supported("Hallar, the Firefletcher");
    let mut t = TestGame::new(2);
    let hallar = t.battlefield(P0, "Hallar, the Firefletcher");
    put(&mut t, hallar, counters::PLUS1, 1);
    t.lands(P0, "Mountain", 5);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst).kicked(true).target(P1).go();
    t.settle();
    t.resolve();
    assert_eq!(t.counters(hallar, counters::PLUS1), 2);
    assert_eq!(t.life(P1), 18, "two +1/+1 counters on Hallar");
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn divided_damage_among_any_number_of_targets_may_have_zero_targets() {
    // "Any number" includes zero, also for divided damage and prevention (Magmatic Core
    // above, and these enters triggers).
    cr!("107.1c", "603.3d");
    supported("Bogardan Hellkite");
    supported("Angel of Salvation");
    for name in ["Bogardan Hellkite", "Angel of Salvation"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[]);
        t.enter(P0, name);
        t.settle();
        let mins: Vec<u32> = t
            .asked()
            .iter()
            .filter_map(|(_, d)| match d {
                mtg_engine::decision::Decision::ChooseTargets { min, .. } => Some(*min),
                _ => None,
            })
            .collect();
        assert_eq!(mins, vec![0], "{name}: zero targets allowed");
        t.resolve_all();
        assert_eq!(t.obj_now(bears).damage, 0);
        assert_eq!(t.life(P1), 20);
        assert_eq!(t.life(P0), 20);
    }
}

#[test]
fn spell_dividing_damage_among_zero_targets_still_resolves() {
    // A spell cast with zero of its "any number of" targets has no targets to become
    // illegal, so it resolves (it isn't removed for lack of legal targets, CR 608.2b);
    // a fixed number of targets ("X target creatures") still needs at least one, each
    // getting at least 1 damage (CR 601.2d).
    cr!("107.1c", "608.2b");
    supported("Rolling Thunder");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let thunder = t.hand(P0, "Rolling Thunder");
    let spell = t.cast(P0, thunder).x(3).targets(&[]).go();
    let mins: Vec<u32> = t
        .asked()
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { min, .. } => Some(*min),
            _ => None,
        })
        .collect();
    assert_eq!(mins, vec![0]);
    t.resolve_all();
    assert!(crate::r_s07_common::resolved(&t, spell));
    assert!(t.in_graveyard(P0, "Rolling Thunder"));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.life(P1), 20);

    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let swarm = t.hand(P0, "Meteor Swarm");
    t.cast(P0, swarm).x(1).targets(&[]).go();
    let min = t.asked().iter().find_map(|(_, d)| match d {
        mtg_engine::decision::Decision::ChooseTargets { min, .. } => Some(*min),
        _ => None,
    });
    assert_eq!(min, Some(1), "X target creatures: at least one");
}
