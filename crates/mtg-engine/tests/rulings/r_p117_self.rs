//! Rulings batch P117 — abilities of creatures that care about their own power: negative
//! power counts as 0 (CR 107.1b), mana abilities (CR 605), simultaneous leaves-the-
//! battlefield events (CR 603.10a), life-gain events (CR 119.10), costs locked in before
//! mana abilities are activated (CR 601.2f–h), and miscellaneous triggers.

use crate::r_p117_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- negative power -------------------------------------------------------------------

#[test]
fn attendant_of_vraska_negative_power_changes_no_life() {
    cr!("107.1b", "603.10a");
    ruling!(
        "Attendant of Vraska",
        "If Attendant of Vraska’s power was negative, you don’t lose or gain life."
    );
    supported("Attendant of Vraska");
    for (minus, life) in [(0, 23), (5, 20)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Vraska, Golgari Queen");
        let a = t.battlefield(P0, "Attendant of Vraska");
        if minus > 0 {
            minus_power(&mut t, a, minus);
        }
        destroy(&mut t, a);
        t.resolve_all();
        assert_eq!(t.life(P0), life);
    }
}

#[test]
fn brambleguard_captain_negative_power_gives_plus_zero() {
    cr!("107.1b", "608.2h");
    ruling!(
        "Brambleguard Captain",
        "If Brambleguard Captain’s power is negative as its ability resolves, the target creature will get +0/+0 until end of turn."
    );
    supported("Brambleguard Captain");
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Brambleguard Captain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    beginning_of_combat(&mut t);
    assert_eq!(stack(&t), 1);
    minus_power(&mut t, captain, 4);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn halana_and_alena_negative_power_puts_no_counters() {
    cr!("107.1b", "122.1");
    ruling!(
        "Halana and Alena, Partners",
        "If Halana and Alena's power is somehow negative at the time its triggered ability resolves, you will put no counters on that creature. The effect doesn't remove counters from that creature or give it -1/-1 counters."
    );
    supported("Halana and Alena, Partners");
    let mut t = TestGame::new(2);
    let ha = t.battlefield(P0, "Halana and Alena, Partners");
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 1);
    t.answer_targets(P0, &[obj(bears)]);
    beginning_of_combat(&mut t);
    minus_power(&mut t, ha, 4);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 1);
    assert_eq!(t.counters(bears, "-1/-1"), 0);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn marionette_master_negative_power_changes_no_life() {
    cr!("107.1b");
    ruling!(
        "Marionette Master",
        "If Marionette Master's power is negative, the target opponent doesn't lose (or gain) life."
    );
    supported("Marionette Master");
    let mut t = TestGame::new(2);
    let mm = t.battlefield(P0, "Marionette Master");
    let thopter = t.battlefield(P0, "Ornithopter");
    minus_power(&mut t, mm, 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, thopter);
    assert_eq!(stack(&t), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn esper_sentinel_negative_power_x_is_zero_and_may_still_decline() {
    cr!("107.1b", "118.12");
    ruling!(
        "Esper Sentinel",
        "If Esper Sentinel's has negative power when this ability resolves, then {X} is {0}. The opponent may still choose not to pay the cost if they want you to draw a card."
    );
    supported("Esper Sentinel");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let sentinel = t.battlefield(P0, "Esper Sentinel");
        minus_power(&mut t, sentinel, 3);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P1, pay);
        cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
        t.settle();
        assert_eq!(stack(&t), 2);
        let hand = t.hand_size(P0);
        t.resolve();
        assert_eq!(t.hand_size(P0), hand + usize::from(!pay), "pay: {pay}");
    }
}

#[test]
fn esper_sentinel_doesnt_trigger_if_the_first_spell_was_already_cast() {
    cr!("603.2", "603.4");
    ruling!(
        "Esper Sentinel",
        "If a noncreature spell was already cast by an opponent the turn Esper Sentinel enters the battlefield, that opponent already cast their first noncreature spell this turn, and Esper Sentinel's ability won't trigger for that opponent that turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    t.battlefield(P0, "Esper Sentinel");
    cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.settle();
    assert_eq!(stack(&t), 1, "only the second Giant Growth");
}

// --- mana abilities ---------------------------------------------------------------------

#[test]
fn heronblade_elite_mana_ability_and_nonpositive_power() {
    cr!("605.1a", "605.3a", "107.1b");
    ruling!(
        "Heronblade Elite",
        "Heronblade Elite's last ability is a mana ability. It doesn't use the stack and can't be responded to. If Heronblade Elite's power is 0 or less at the time the ability resolves, no mana is added."
    );
    supported("Heronblade Elite");
    let mut t = TestGame::new(2);
    let elite = with_counters(&mut t, P0, "Heronblade Elite", 2);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, elite, 0, &[]).unwrap();
    assert_eq!(stack(&t), 0);
    assert_eq!(pool_total(&t, P0), 3);
    let mut t = TestGame::new(2);
    let elite = t.battlefield(P0, "Heronblade Elite");
    minus_power(&mut t, elite, 2);
    let _ = t.activate(P0, elite, 0, &[]);
    assert_eq!(stack(&t), 0);
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn helga_mana_ability_doesnt_use_the_stack() {
    cr!("605.1a", "605.3a");
    ruling!(
        "Helga, Skittish Seer",
        "Helga's last ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Helga, Skittish Seer");
    let mut t = TestGame::new(2);
    let helga = with_counters(&mut t, P0, "Helga, Skittish Seer", 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, helga, 0, &[]).unwrap();
    assert_eq!(stack(&t), 0);
    assert_eq!(pool_total(&t, P0), 2);
}

// --- costs locked in -------------------------------------------------------------------

/// `reducer` (with one +1/+1 counter: power 2) reduces `spell`'s cost by {2}; the only
/// other mana is one `land` and Ashnod's Altar, which sacrifices the reducer for {C}{C}
/// while the cost is paid. Returns whether the spell was cast.
fn reducer_sacrificed_for_mana(reducer: &str, spell: &str, land: &str, target: bool) -> bool {
    let mut t = TestGame::new(2);
    let r = with_counters(&mut t, P0, reducer, 1);
    t.battlefield(P0, "Ashnod's Altar");
    t.battlefield(P0, land);
    let card = t.hand(P0, spell);
    let target = if target {
        obj(t.battlefield(P1, "Grizzly Bears"))
    } else {
        Entity::Player(P1)
    };
    t.answer_choose(P0, &[obj(r)]);
    let ok = t.cast_with(P0, card, &[target]).is_ok();
    if ok {
        assert!(!t.on_battlefield(r), "sacrificed for mana while paying");
    }
    ok
}

#[test]
fn vadrik_cost_reduction_is_locked_in_before_mana_abilities() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Vadrik, Astral Archmage",
        "All costs to cast a spell are locked in before mana abilities are activated and those costs are paid. If Vadrik's power changes after costs are locked in, or if Vadrik leaves the battlefield while you are activating mana abilities or paying those costs, the cost reduction effect won't be affected and the total cost to cast the spell won't change."
    );
    supported("Vadrik, Astral Archmage");
    // Lava Axe ({4}{R}) costs {2}{R}: a Mountain and {C}{C} from sacrificing Vadrik.
    assert!(reducer_sacrificed_for_mana(
        "Vadrik, Astral Archmage",
        "Lava Axe",
        "Mountain",
        false
    ));
}

#[test]
fn golden_tail_trainer_cost_reduction_is_locked_in_before_mana_abilities() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Golden-Tail Trainer",
        "All costs to cast a spell are locked in before mana abilities are activated and those costs are paid. If Golden-Tail Trainer's power changes after costs are locked in, or if Golden-Tail Trainer leaves the battlefield while you are activating mana abilities or paying those costs, the cost reduction effect won't be affected and the total cost to cast the spell won't change."
    );
    supported("Golden-Tail Trainer");
    // Soaring Hope ({4}{W}) costs {2}{W}.
    assert!(reducer_sacrificed_for_mana(
        "Golden-Tail Trainer",
        "Soaring Hope",
        "Plains",
        true
    ));
}

// --- life-gain events --------------------------------------------------------------------

/// P0 attacks with two Child of Night (2/1 lifelink) unblocked, or with one blocked by two
/// 1/1s if `double_block`; returns the +1/+1 counters on `name` afterwards.
fn lifelink_combat(name: &str, double_block: bool) -> u32 {
    let mut t = TestGame::new(2);
    let watcher = t.battlefield(P0, name);
    let c1 = t.battlefield(P0, "Child of Night");
    let c2 = t.battlefield(P0, "Child of Night");
    if double_block {
        let a = t.battlefield(P1, "Llanowar Elves");
        let b = t.battlefield(P1, "Llanowar Elves");
        attack_with(&mut t, &[(c1, Entity::Player(P1))]);
        block_and_finish(&mut t, P1, &[(a, c1), (b, c1)]);
    } else {
        attack_with(
            &mut t,
            &[(c1, Entity::Player(P1)), (c2, Entity::Player(P1))],
        );
        block_and_finish(&mut t, P1, &[]);
    }
    t.resolve_all();
    plus1(&t, watcher)
}

#[test]
fn elendas_hierophant_triggers_once_per_lifelink_source() {
    cr!("702.15b", "119.10", "510.2");
    ruling!(
        "Elenda's Hierophant",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Elenda's Hierophant's second ability will trigger twice."
    );
    supported("Elenda's Hierophant");
    assert_eq!(lifelink_combat("Elenda's Hierophant", false), 2);
    assert_eq!(lifelink_combat("Elenda's Hierophant", true), 1);
}

#[test]
fn fiendish_panda_triggers_once_per_lifelink_source() {
    cr!("702.15b", "119.10", "510.2");
    ruling!(
        "Fiendish Panda",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Fiendish Panda's first ability will trigger twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, planeswalkers, and/or battles at the same time"
    );
    supported("Fiendish Panda");
    assert_eq!(lifelink_combat("Fiendish Panda", false), 2);
    assert_eq!(lifelink_combat("Fiendish Panda", true), 1);
}

#[test]
fn fiendish_panda_triggers_once_per_event_whatever_the_amount() {
    cr!("119.10");
    ruling!(
        "Fiendish Panda",
        "Fiendish Panda's first ability triggers just once for each life-gaining event, whether it's 1 life from Dazzling Angel or 4 life from Apothecary Stomper."
    );
    let mut t = TestGame::new(2);
    let panda = t.battlefield(P0, "Fiendish Panda");
    gain_life(&mut t, P0, 4);
    t.resolve_all();
    assert_eq!(plus1(&t, panda), 1);
    gain_life(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, panda), 2);
}

/// P1 attacks with a Hill Giant and Grizzly Bears; P0 blocks the Giant with `name` and the
/// Bears with a Child of Night (lifelink). Returns whether `name` survived.
fn dies_while_gaining_life(name: &str) -> bool {
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let w = t.battlefield(P0, name);
    let child = t.battlefield(P0, "Child of Night");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(giant, Entity::Player(P0)), (bears, Entity::Player(P0))],
    );
    block_and_finish(&mut t, P0, &[(w, giant), (child, bears)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22, "lifelink");
    t.on_battlefield(w)
}

#[test]
fn elendas_hierophant_doesnt_survive_lethal_damage_dealt_with_the_gain() {
    cr!("704.5g", "704.3", "603.3");
    ruling!(
        "Elenda's Hierophant",
        "If Elenda's Hierophant is dealt lethal damage at the same time that you gain life, it won't receive a counter from its second ability in time to save it."
    );
    assert!(!dies_while_gaining_life("Elenda's Hierophant"));
}

#[test]
fn fiendish_panda_doesnt_survive_lethal_damage_dealt_with_the_gain() {
    cr!("704.5g", "704.3", "603.3");
    ruling!(
        "Fiendish Panda",
        "If Fiendish Panda is dealt lethal damage at the same time that you gain life, it won't receive a counter from its first ability in time to save it."
    );
    assert!(!dies_while_gaining_life("Fiendish Panda"));
}

// --- simultaneous leaves-the-battlefield events -----------------------------------------

#[test]
fn elenda_dying_with_another_creature_triggers_both_abilities() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Elenda, the Dusk Rose",
        "If Elenda dies at the same time as another creature, both of its triggered abilities trigger. However, the first one won't do anything since you can't put a +1/+1 counter on Elenda."
    );
    supported("Elenda, the Dusk Rose");
    let mut t = TestGame::new(2);
    let elenda = t.battlefield(P0, "Elenda, the Dusk Rose");
    t.battlefield(P0, "Grizzly Bears");
    wrath(&mut t);
    assert_eq!(stack(&t), 2);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Vampire"), 1, "Elenda died as a 1/1");
    assert_eq!(plus1(&t, elenda), 0);
}

#[test]
fn jenova_dying_with_mutants_triggers_for_each() {
    cr!("603.10a");
    ruling!(
        "Jenova, Ancient Calamity",
        "If Jenova dies at the same time as one or more Mutants you control, its last ability will trigger for each of those Mutants"
    );
    supported("Jenova, Ancient Calamity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jenova, Ancient Calamity");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    beginning_of_combat(&mut t);
    t.resolve_all();
    assert!(t.obj_now(bears).chars.has_subtype("Mutant"));
    assert_eq!(t.pt(bears), (3, 3));
    stack_library(&mut t, P0, &["Forest"; 5]);
    let hand = t.hand_size(P0);
    let jenova = t.named_on_battlefield("Jenova, Ancient Calamity")[0];
    t.g.destroy_all(vec![jenova, bears], None, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(stack(&t), 1, "Jenova isn't a Mutant");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn syr_vondam_leaving_with_others_triggers_for_each() {
    cr!("603.10a");
    ruling!(
        "Syr Vondam, Sunstar Exemplar",
        "If Syr Vondam, Sunstar Exemplar leaves the battlefield at the same time as one or more other creatures you control die or are put into exile, its first triggered ability will trigger for each of those other creatures."
    );
    supported("Syr Vondam, Sunstar Exemplar");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syr Vondam, Sunstar Exemplar");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    wrath(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn szarel_leaving_with_other_sacrificed_permanents_triggers_for_each() {
    cr!("603.10a", "701.21a");
    ruling!(
        "Szarel, Genesis Shepherd",
        "If Szarel leaves the battlefield at the same time as you sacrifice one or more other nontoken permanents during your turn (probably because you also sacrificed Szarel), its last ability triggers for each of those other nontoken permanents."
    );
    supported("Szarel, Genesis Shepherd");
    let mut t = TestGame::new(2);
    let szarel = t.battlefield(P0, "Szarel, Genesis Shepherd");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.g.sacrifice_simultaneously(&[(szarel, P0), (bears, P0), (elves, P0)]);
    t.g.flush_events();
    t.settle();
    assert_eq!(stack(&t), 2);
    t.resolve_all();
    assert_eq!(plus1(&t, giant), 4);
}

#[test]
fn szarel_uses_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Szarel, Genesis Shepherd",
        "If Szarel leaves the battlefield before its last ability resolves, use its power as it last existed on the battlefield to determine how many +1/+1 counters to put on the target creature."
    );
    let mut t = TestGame::new(2);
    let szarel = t.battlefield(P0, "Szarel, Genesis Shepherd");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.g.sacrifice(bears, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(stack(&t), 1);
    grow_then_exile(&mut t, szarel, 2);
    t.resolve_all();
    assert_eq!(plus1(&t, giant), 4);
}

#[test]
fn marionette_master_sees_artifact_tokens_go_to_the_graveyard() {
    cr!("111.7", "603.10a");
    ruling!(
        "Marionette Master",
        "Artifact tokens that are sacrificed or destroyed are put into their owner's graveyard before ceasing to exist. If you controlled the token, Marionette Master's last ability will trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Marionette Master");
    let treasure = crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    assert!(t.obj_now(treasure).is(mtg_engine::types::CardType::Artifact));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.sacrifice(treasure, P0);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn marionette_master_dying_with_an_artifact_triggers() {
    cr!("603.10a");
    ruling!(
        "Marionette Master",
        "If Marionette Master and an artifact you control are put into a graveyard at the same time, Marionette Master's ability triggers."
    );
    let mut t = TestGame::new(2);
    let mm = with_counters(&mut t, P0, "Marionette Master", 2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.destroy_all(vec![mm, thopter], None, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(stack(&t), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "its last known power");
}

#[test]
fn moorland_rescuer_returns_cards_even_if_exiled_in_response() {
    cr!("603.10a", "608.2b");
    ruling!(
        "Moorland Rescuer",
        "If Moorland Rescuer is exiled from its owner's graveyard in response to its triggered ability, the creature cards will still be returned to the battlefield."
    );
    supported("Moorland Rescuer");
    let mut t = TestGame::new(2);
    let rescuer = t.battlefield(P0, "Moorland Rescuer");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    destroy(&mut t, rescuer);
    assert_eq!(stack(&t), 1);
    exile(&mut t, rescuer);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn orca_keeps_its_damage_division_for_remaining_legal_targets() {
    cr!("608.2b", "601.2d", "603.3d");
    ruling!(
        "Orca, Siege Demon",
        "If Orca, Siege Demon's last ability has multiple targets, and some but not all of them are illegal targets when the ability resolves, Orca, Siege Demon will still deal damage to the remaining legal targets according to the original damage division."
    );
    supported("Orca, Siege Demon");
    let mut t = TestGame::new(2);
    let orca = t.battlefield(P0, "Orca, Siege Demon");
    let air = t.battlefield(P1, "Air Elemental");
    t.answer_targets(P0, &[obj(air), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 2]));
    destroy(&mut t, orca);
    assert_eq!(stack(&t), 1);
    exile(&mut t, air);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// --- spells cast ------------------------------------------------------------------------

/// P0 casts Giant Growth on `target` (with lands for it), leaving it on the stack.
fn growth(t: &mut TestGame, target: ObjectId) -> ObjectId {
    cast_new(t, P0, "Giant Growth", &[obj(target)])
}

#[test]
fn captain_ripley_vance_counts_countered_spells() {
    cr!("601.2i", "603.2");
    ruling!(
        "Captain Ripley Vance",
        "Believe it or not, Ripley counts spells that were cast even if they didn't resolve. This means it still counts spells that were countered."
    );
    supported("Captain Ripley Vance");
    let mut t = TestGame::new(2);
    let ripley = t.battlefield(P0, "Captain Ripley Vance");
    for _ in 0..2 {
        let s = growth(&mut t, ripley);
        counter_spell(&mut t, s);
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    growth(&mut t, ripley);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "third spell"), 1);
}

#[test]
fn captain_ripley_vance_need_not_see_the_first_two_spells() {
    cr!("603.2");
    ruling!(
        "Captain Ripley Vance",
        "Captain Ripley Vance need not be on the battlefield to witness the first two spells being cast. As long as it's already on the battlefield when you cast the third spell of a turn, its ability triggers."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        growth(&mut t, bears);
        t.resolve_all();
    }
    t.battlefield(P0, "Captain Ripley Vance");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    growth(&mut t, bears);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "third spell"), 1);
}

#[test]
fn captain_ripley_vance_doesnt_count_copies() {
    cr!("707.10", "707.12");
    ruling!(
        "Captain Ripley Vance",
        "Copying a spell is not the same as casting a spell."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Captain Ripley Vance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let s = growth(&mut t, bears);
    mtg_engine::copy::copy_spell(&mut t.g, s, P0, false).expect("copied");
    t.g.flush_events();
    t.settle();
    let s = growth(&mut t, bears);
    mtg_engine::copy::copy_spell(&mut t.g, s, P0, false).expect("copied");
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "third spell"), 0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    growth(&mut t, bears);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "third spell"), 1);
}

#[test]
fn carmen_sacrifice_for_a_cost_resolves_first_during_resolution_after() {
    cr!("603.3", "601.2h", "608.2");
    ruling!(
        "Carmen, Cruel Skymarcher",
        "If a permanent is sacrificed to pay a cost of a spell or ability, Carmen, Cruel Skymarcher's second ability will resolve before that spell or ability. Conversely, if a permanent is sacrificed during the resolution of a spell or ability, that spell or ability will finish resolving before Carmen, Cruel Skymarcher's second ability is put onto the stack."
    );
    supported("Carmen, Cruel Skymarcher");
    // Sacrificed as Fling's cost: Carmen's trigger is above Fling and resolves first.
    let mut t = TestGame::new(2);
    let carmen = t.battlefield(P0, "Carmen, Cruel Skymarcher");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let fling = t.hand(P0, "Fling");
    t.answer_choose(P0, &[obj(bears)]);
    let fling = t.cast(P0, fling).target(P1).go();
    t.settle();
    assert_eq!(stack(&t), 2);
    t.resolve();
    assert!(t.g.stack.contains(&fling), "Fling hasn't resolved yet");
    assert_eq!(plus1(&t, carmen), 1);
    // Sacrificed while Diabolic Edict resolves: it finishes resolving first.
    let mut t = TestGame::new(2);
    let carmen = t.battlefield(P0, "Carmen, Cruel Skymarcher");
    t.battlefield(P1, "Grizzly Bears");
    let edict = cast_new(&mut t, P0, "Diabolic Edict", &[Entity::Player(P1)]);
    t.resolve();
    assert!(resolved(&t, edict));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(stack(&t), 1);
    assert_eq!(plus1(&t, carmen), 0);
    t.resolve_all();
    assert_eq!(plus1(&t, carmen), 1);
}

#[test]
fn narset_multiple_prowess_instances_trigger_separately() {
    cr!("702.108b");
    ruling!(
        "Narset, Enlightened Exile",
        "If a creature has multiple instances of prowess, each triggers separately."
    );
    supported("Narset, Enlightened Exile");
    let mut t = TestGame::new(2);
    let narset = t.battlefield(P0, "Narset, Enlightened Exile");
    let spear = t.battlefield(P0, "Monastery Swiftspear");
    growth(&mut t, narset);
    t.resolve_all();
    assert_eq!(t.pt(spear), (1 + 2, 2 + 2));
}

#[test]
fn myriad_construct_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "603.2");
    ruling!(
        "Myriad Construct",
        "An ability that triggers when a permanent becomes the target of a spell resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Myriad Construct");
    let mut t = TestGame::new(2);
    let construct = t.battlefield(P0, "Myriad Construct");
    let s = growth(&mut t, construct);
    t.settle();
    assert_eq!(stack(&t), 2);
    counter_spell(&mut t, s);
    t.resolve_all();
    assert!(!t.on_battlefield(construct));
    assert_eq!(tokens_named(&t, P0, "Construct"), 4);
}

// --- other self-referential abilities ------------------------------------------------------

#[test]
fn duggan_can_activate_again_as_a_new_object() {
    cr!("602.5b", "400.7");
    ruling!(
        "Duggan, Private Detective",
        "Duggan, Private Detective's last ability can be activated only once during the game. However, if Duggan leaves and later returns to the battlefield, he's considered a new permanent (with a new set of abilities) and it can be activated again."
    );
    supported("Duggan, Private Detective");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P0, "Forest");
    }
    let duggan = t.battlefield(P0, "Duggan, Private Detective");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 4);
    t.activate(P0, duggan, 0, &[obj(wurm)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"), "6 damage: twice its power of 3");
    t.g.objects[duggan.0 as usize].tapped = false;
    assert!(t.activate(P0, duggan, 0, &[obj(giant)]).is_err());
    let ex = t
        .g
        .move_object(duggan, mtg_engine::object::Zone::Exile, mtg_engine::events::MoveCause::Effect, None)
        .unwrap();
    let new = t
        .g
        .move_object(ex, mtg_engine::object::Zone::Battlefield, mtg_engine::events::MoveCause::Effect, None)
        .unwrap();
    t.g.objects[new.0 as usize].summoning_sick = false;
    t.resolve_all();
    assert!(t.activate(P0, new, 0, &[obj(giant)]).is_ok());
}

#[test]
fn wojek_bodyguard_others_may_attack_or_block_elsewhere() {
    cr!("506.5", "508.1c", "509.1c");
    ruling!(
        "Wojek Bodyguard",
        "Although Wojek Bodyguard can't attack alone, other attacking creatures don't have to attack the same player, planeswalker, or battle."
    );
    supported("Wojek Bodyguard");
    let mut t = TestGame::new(3);
    let wojek = t.battlefield(P0, "Wojek Bodyguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(wojek, Entity::Player(P1)), (bears, Entity::Player(P2))],
    );
    let combat = t.g.combat.as_ref().unwrap();
    assert!(combat.attacker(wojek).is_some() && combat.attacker(bears).is_some());
}

#[test]
fn wojek_bodyguard_blockers_may_block_different_creatures() {
    cr!("509.1c");
    ruling!(
        "Wojek Bodyguard",
        "Similarly, other blocking creatures don't have to block the same creature that Wojek Bodyguard blocks."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let wojek = t.battlefield(P0, "Wojek Bodyguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(a, Entity::Player(P0)), (b, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wojek, a), (bears, b)]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    let combat = t.g.combat.as_ref().unwrap();
    assert!(combat.is_blocked(a) && combat.is_blocked(b));
}

#[test]
fn aura_gnarlid_counts_every_aura() {
    cr!("613.4c");
    ruling!(
        "Aura Gnarlid",
        "Aura Gnarlid's second ability counts each Aura regardless of who controls it."
    );
    supported("Aura Gnarlid");
    let mut t = TestGame::new(2);
    let gnarlid = t.battlefield(P0, "Aura Gnarlid");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Pacifism");
    let theirs = t.battlefield(P1, "Pacifism");
    t.g.attach(mine, obj(bears));
    t.g.attach(theirs, obj(bears));
    t.g.recompute();
    assert_eq!(t.pt(gnarlid), (4, 4));
}

#[test]
fn battering_wurm_compares_powers_as_blockers_are_declared() {
    cr!("509.1b", "506.4");
    ruling!(
        "Battering Wurm",
        "Battering Wurm compares each blocker's power against its own as blockers are being declared."
    );
    supported("Battering Wurm");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Battering Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    // A pumped blocker (5) may block; pumping the Wurm afterwards doesn't remove it.
    giant_growth(&mut t, bears);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, wurm)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(wurm));
    giant_growth(&mut t, wurm);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(wurm));
    // An unpumped Bears can't block it.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Battering Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, wurm)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(!t.g.combat.as_ref().unwrap().is_blocked(wurm));
}

#[test]
fn stone_giant_activated_in_the_end_step_destroys_at_the_next_end_step() {
    cr!("603.7a", "513.1");
    ruling!(
        "Stone Giant",
        "If Stone Giant’s ability is activated during a turn’s end step, the delayed triggered ability won’t trigger until the beginning of the next turn’s end step. The targeted creature will be destroyed at that time."
    );
    supported("Stone Giant");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Stone Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.activate(P0, giant, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(bears));
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn tracker_and_karplusan_yeti_first_strike_doesnt_matter() {
    cr!("702.7b", "120.3");
    ruling!("Tracker", "Giving either creature first strike does not affect the ability.");
    ruling!(
        "Karplusan Yeti",
        "Giving either creature first strike does not affect the ability."
    );
    for (name, cost, survives) in [("Tracker", 2, false), ("Karplusan Yeti", 0, true)] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let knight = t.battlefield(P1, "Youthful Knight");
        t.lands(P0, "Forest", cost);
        t.activate(P0, c, 0, &[obj(knight)]).unwrap();
        t.resolve_all();
        assert!(!t.on_battlefield(knight), "{name}");
        assert_eq!(t.on_battlefield(c), survives, "{name}");
        if survives {
            assert_eq!(damage(&t, c), 2);
        }
    }
}

// --- creatures put onto the battlefield attacking --------------------------------------------

#[test]
fn shadowfax_creature_put_onto_the_battlefield_attacking_didnt_attack() {
    cr!("508.4", "508.3a");
    ruling!(
        "Shadowfax, Lord of Horses",
        "Although the creature you put onto the battlefield is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won't trigger when it enters the battlefield attacking."
    );
    let mut t = TestGame::new(2);
    let shadowfax = t.battlefield(P0, "Shadowfax, Lord of Horses");
    let krenko = t.hand(P0, "Krenko, Tin Street Kingpin");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(krenko)]);
    attack_with(&mut t, &[(shadowfax, Entity::Player(P1))]);
    t.resolve_all();
    let krenko = t.g.current(krenko);
    assert!(t.g.combat.as_ref().unwrap().attacker(krenko).is_some());
    assert_eq!(tokens_named(&t, P0, "Goblin"), 0);
}

#[test]
fn paladin_elizabeth_taggerdy_creature_put_onto_the_battlefield_attacking_didnt_attack() {
    cr!("508.4", "508.3a");
    ruling!(
        "Paladin Elizabeth Taggerdy",
        "Although the creature you put onto the battlefield is attacking, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won’t trigger when it enters the battlefield attacking."
    );
    supported("Paladin Elizabeth Taggerdy");
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Paladin Elizabeth Taggerdy");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Forest"]);
    let krenko = t.hand(P0, "Krenko, Tin Street Kingpin");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(krenko)]);
    attack_with(
        &mut t,
        &[
            (paladin, Entity::Player(P1)),
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    let krenko = t.g.current(krenko);
    assert!(t.on_battlefield(krenko));
    assert!(t.g.combat.as_ref().unwrap().attacker(krenko).is_some());
    assert_eq!(tokens_named(&t, P0, "Goblin"), 0);
}

// --- Exuberant Wolfbear ---------------------------------------------------------------------

/// Exuberant Wolfbear (with `n` +1/+1 counters) attacks; its trigger targets a Human
/// (Elite Vanguard, 2/1) that got Giant Growth earlier this turn. Returns the (Human,
/// Wolfbear) P/T afterwards.
fn wolfbear(n: u32) -> ((i32, i32), (i32, i32)) {
    let mut t = TestGame::new(2);
    let wolfbear = with_counters(&mut t, P0, "Exuberant Wolfbear", n);
    let human = t.battlefield(P0, "Elite Vanguard");
    giant_growth(&mut t, human);
    t.answer_targets(P0, &[obj(human)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(wolfbear, Entity::Player(P1))]);
    t.resolve_all();
    (t.pt(human), t.pt(wolfbear))
}

#[test]
fn exuberant_wolfbear_modifiers_apply_after_the_base_is_set() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Exuberant Wolfbear",
        "Any effects that modify the Human’s power and/or toughness without setting them to a specific number or value will apply after its base power and toughness are set, regardless of the order in which those effects were created."
    );
    assert_eq!(wolfbear(0).0, (4 + 3, 4 + 3));
}

#[test]
fn exuberant_wolfbear_is_unaffected() {
    cr!("613.4b");
    ruling!(
        "Exuberant Wolfbear",
        "Exuberant Wolfbear’s power and toughness are unaffected when the target Human takes on those values."
    );
    assert_eq!(wolfbear(1), ((5 + 3, 5 + 3), (5, 5)));
}

#[test]
fn meglonoth_blocking_two_creatures_triggers_twice() {
    cr!("509.1a", "603.2c");
    ruling!(
        "Meglonoth",
        "If Meglonoth somehow blocks more than one creature, its ability will trigger once for each creature it blocks."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    let meg = t.battlefield(P0, "Meglonoth");
    t.battlefield(P0, "High Ground");
    attack_with(&mut t, &[(a, Entity::Player(P0)), (b, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(meg, a), (meg, b)]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    t.settle();
    assert_eq!(stack(&t), 2);
    t.resolve_all();
    assert_eq!(lost(&t, P1), 12);
}
