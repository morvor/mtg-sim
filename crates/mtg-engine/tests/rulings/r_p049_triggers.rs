//! Rulings batch P049 — triggered abilities of "drain" cards: intervening "if" clauses
//! about the turn's history (Arrogant Outlaw, Bloodchief Ascension, Denethor), "one or
//! more" and simultaneous events (Anje, Wayward Servant, Sludge Strider, Ark of Hunger,
//! Soul Enervation, Circle of Affliction), the look-back rule (Bloodchief Ascension's
//! second ability doesn't; Grandmother Ravi Sengir's does), triggers from costs and from
//! resolving effects (Fumulus), Gollum's exhausted modes, Night Market Lookout, Doom
//! Foretold, and Arterial Flow.

use crate::r_p049_common::*;
use crate::r_p212_common::back_face;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` loses `n` life (as an effect would) and the events are settled.
fn lose(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.lose_life(p, n);
    t.g.flush_events();
    t.settle();
}

/// `p` gains `n` life (as an effect would) and the events are settled.
fn gain(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.gain_life(p, n);
    t.g.flush_events();
    t.settle();
}

#[test]
fn arrogant_outlaw_cares_that_an_opponent_lost_life_not_how_their_total_changed() {
    cr!("603.4", "119.3");
    ruling!(
        "Arrogant Outlaw",
        "an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    supported("Arrogant Outlaw");
    let mut t = TestGame::new(2);
    gain(&mut t, P1, 2);
    lose(&mut t, P1, 1);
    assert_eq!(t.life(P1), 21);
    t.enter(P0, "Arrogant Outlaw");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 19));
    // Only a gain: no trigger.
    let mut t = TestGame::new(2);
    gain(&mut t, P1, 2);
    t.enter(P0, "Arrogant Outlaw");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 22));
}

#[test]
fn arterial_flow_drains_even_opponents_who_couldnt_discard() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Arterial Flow",
        "Arterial Flow causes each opponent to lose 2 life if you control a Vampire, even if some or all of those players were unable to discard any cards."
    );
    supported("Arterial Flow");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Vampire Nighthawk");
    // P1 has no cards in hand; P2 has one.
    t.hand(P2, "Grizzly Bears");
    cast_new(&mut t, P0, "Arterial Flow", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P2), 0);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (22, 18, 18));
}

#[test]
fn bloodchief_ascensions_end_step_trigger_checks_the_whole_turn() {
    cr!("603.4", "119.3");
    ruling!(
        "Bloodchief Ascension",
        "Bloodchief Ascension's first ability checks how much life was lost by each opponent over the course of the entire turn, even if Bloodchief Ascension wasn't on the battlefield the whole time."
    );
    supported("Bloodchief Ascension");
    let mut t = TestGame::new(2);
    lose(&mut t, P1, 2);
    let asc = t.battlefield(P0, "Bloodchief Ascension");
    t.answer_yes(P0, true);
    end_step(&mut t, P0);
    assert_eq!(t.counters(asc, "quest"), 1);
    ruling!(
        "Bloodchief Ascension",
        "It doesn't care whether life was also gained. For example, if an opponent lost 4 life and gained 6 life during the turn"
    );
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Bloodchief Ascension");
    lose(&mut t, P1, 4);
    gain(&mut t, P1, 6);
    assert_eq!(t.life(P1), 22);
    t.answer_yes(P0, true);
    end_step(&mut t, P0);
    assert_eq!(t.counters(asc, "quest"), 1);
    ruling!(
        "Bloodchief Ascension",
        "It won't trigger at all unless an opponent has already lost 2 or more life by the time the end step begins."
    );
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Bloodchief Ascension");
    lose(&mut t, P1, 1);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "quest counter"), 0);
    // Losing more life during the end step is too late.
    lose(&mut t, P1, 1);
    assert_eq!(triggers_on_stack(&t, "quest counter"), 0);
    t.resolve_all();
    assert_eq!(t.counters(asc, "quest"), 0);
}

#[test]
fn bloodchief_ascensions_second_ability_doesnt_look_back_in_time() {
    cr!("603.10", "603.10a", "603.6c");
    ruling!(
        "Bloodchief Ascension",
        "If a Bloodchief Ascension with three quest counters on it and a permanent an opponent owns are destroyed at the same time, for example, the game will not \"look back in time\" at the game state, and Bloodchief Ascension's second ability won't trigger."
    );
    supported("Bloodchief Ascension");
    supported("Akroma's Vengeance");
    let mut t = TestGame::new(2);
    let asc = t.battlefield(P0, "Bloodchief Ascension");
    put_counters(&mut t, asc, "quest", 3);
    t.battlefield(P1, "Grizzly Bears");
    // It works while it's there: a card into P1's graveyard, P1 may lose 2 (P0 says yes).
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, shock).target(P0).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    // Destroyed together with P1's creature: no trigger.
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Akroma's Vengeance", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(asc));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
}

#[test]
fn denethors_end_step_trigger_checks_the_turn_once_and_needs_a_death_before_it() {
    cr!("603.4", "603.2");
    ruling!(
        "Denethor, Ruling Steward",
        "Denethor, Ruling Steward doesn't need to have been on the battlefield when the creature died."
    );
    supported("Denethor, Ruling Steward");
    let mut t = TestGame::new(2);
    a_creature_dies(&mut t, P0);
    t.battlefield(P0, "Denethor, Ruling Steward");
    end_step(&mut t, P0);
    assert_eq!(tokens_of(&t, P0, "Soldier"), 1);
    ruling!(
        "Denethor, Ruling Steward",
        "Denethor, Ruling Steward's first ability will trigger only once during your end step, no matter how many creatures died under your control this turn."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Denethor, Ruling Steward");
    a_creature_dies(&mut t, P0);
    a_creature_dies(&mut t, P0);
    end_step(&mut t, P0);
    assert_eq!(tokens_of(&t, P0, "Soldier"), 1);
    // No creature died: no trigger; a death during the end step is too late.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Denethor, Ruling Steward");
    // A creature of P1's dying isn't one dying under P0's control.
    a_creature_dies(&mut t, P1);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Soldier"), 0);
    a_creature_dies(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "Soldier"), 0);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Soldier"), 0);
}

#[test]
fn doom_foretold_can_be_sacrificed_to_its_own_ability() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Doom Foretold",
        "During your upkeep, you can sacrifice Doom Foretold for its own ability."
    );
    supported("Doom Foretold");
    let mut t = TestGame::new(2);
    let doom = t.battlefield(P0, "Doom Foretold");
    t.hand(P0, "Grizzly Bears");
    // P0's upkeep: Doom Foretold is P0's only nonland, nontoken permanent.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(doom));
    assert!(t.in_graveyard(P0, "Doom Foretold"));
    // Having sacrificed, P0 discards nothing and loses no life.
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(tokens_of(&t, P0, "Knight"), 0);
}

#[test]
fn doom_foretolds_effect_happens_after_it_has_left_the_battlefield() {
    cr!("113.7a", "608.2c");
    ruling!(
        "Doom Foretold",
        "If Doom Foretold leaves the battlefield or changes control while its ability is on the stack, its effect still happens. You just won't be able to sacrifice it if instructed to do so."
    );
    supported("Doom Foretold");
    let mut t = TestGame::new(2);
    let doom = t.battlefield(P0, "Doom Foretold");
    t.hand(P1, "Grizzly Bears");
    // P1's upkeep: P1 controls no nonland, nontoken permanent. The trigger goes on the
    // stack; Doom Foretold is bounced in response.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifices a nonland"), 1);
    bounce(&mut t, doom);
    assert!(t.in_hand(P0, "Doom Foretold"));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(tokens_of(&t, P0, "Knight"), 1);
    // Doom Foretold stayed in P0's hand: there was nothing to sacrifice.
    assert!(t.in_hand(P0, "Doom Foretold"));
}

#[test]
fn circle_of_affliction_triggers_once_per_source_and_pays_one_each() {
    cr!("603.2c", "603.2", "118.12");
    ruling!(
        "Circle of Affliction",
        "Each time a source deals damage to you, you may pay {1}. You can’t pay more than {1}, even if more than 1 damage was dealt."
    );
    ruling!(
        "Circle of Affliction",
        "If several sources deal damage to you simultaneously (for example, several attacking creatures), this ability triggers once for each source."
    );
    supported("Circle of Affliction");
    let mut t = TestGame::new(2);
    // Red chosen as it enters.
    crate::r_p022_common::choose_color(&mut t, P0, Color::Red);
    t.enter(P0, "Circle of Affliction");
    t.resolve_all();
    t.lands(P0, "Swamp", 5);
    // Two red attackers deal 3 and 2 damage to P0 at once: two triggers, each paying {1}
    // for 1 life.
    t.set_step(P1, Step::PrecombatMain);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Goblin Piker");
    attack_with(&mut t, &[(a, Entity::Player(P0)), (b, Entity::Player(P0))]);
    t.advance_to(P1, Step::CombatDamage);
    // The triggers target as they're put on the stack; the payment is chosen as each
    // resolves.
    for _ in 0..2 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_yes(P0, true);
    }
    t.settle();
    assert_eq!(triggers_on_stack(&t, "pay {1}"), 2);
    assert_eq!(t.life(P0), 15);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (17, 18));
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is(CardType::Land) && o.tapped)
            .count(),
        2
    );
}

#[test]
fn night_market_lookout_triggers_only_when_it_actually_becomes_tapped() {
    cr!("603.2e", "701.26a");
    ruling!(
        "Night Market Lookout",
        "If an effect attempts to tap it, but it was already tapped at the time, this ability won't trigger."
    );
    supported("Night Market Lookout");
    supported("Twiddle");
    let mut t = TestGame::new(2);
    let lookout = t.battlefield(P0, "Night Market Lookout");
    tap(&mut t, lookout);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    // Twiddle: "You may tap or untap target artifact, creature, or land." Tapping it
    // again does nothing.
    cast_new(&mut t, P0, "Twiddle", &[obj(lookout)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert!(tapped(&t, lookout));
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn anje_triggers_once_when_she_and_other_vampires_enter_together() {
    cr!("603.2c");
    ruling!(
        "Anje, Maid of Dishonor",
        "If Anje, Maid of Dishonor and some number of other Vampires enter the battlefield under your control at the same time, you create only one Blood token."
    );
    supported("Anje, Maid of Dishonor");
    let mut t = TestGame::new(2);
    enter_together(
        &mut t,
        &[
            (P0, "Anje, Maid of Dishonor"),
            (P0, "Vampire Nighthawk"),
            (P0, "Vampire Nighthawk"),
        ],
    );
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Blood"), 1);
}

#[test]
fn wayward_servant_triggers_for_a_zombie_entering_with_it() {
    cr!("603.2c", "603.6a");
    ruling!(
        "Wayward Servant",
        "If Wayward Servant enters the battlefield at the same time as another Zombie creature, its ability triggers for that Zombie."
    );
    supported("Wayward Servant");
    let mut t = TestGame::new(2);
    enter_together(&mut t, &[(P0, "Wayward Servant"), (P0, "Walking Corpse")]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn sludge_strider_triggers_for_artifacts_entering_or_leaving_with_it() {
    cr!("603.6a", "603.10a", "603.2c");
    ruling!(
        "Sludge Strider",
        "If Sludge Strider and another artifact enter under your control at the same time, Sludge Strider's triggered ability will trigger. Similarly, if Sludge Strider and another artifact you control leave the battlefield at the same time, Sludge Strider's triggered ability will trigger."
    );
    supported("Sludge Strider");
    supported("Akroma's Vengeance");
    let mut t = TestGame::new(2);
    enter_together(&mut t, &[(P0, "Sludge Strider"), (P0, "Sol Ring")]);
    assert_eq!(triggers_on_stack(&t, "pay {1}"), 1);
    t.resolve_all();
    // Both leave at once.
    cast_new(&mut t, P0, "Akroma's Vengeance", &[]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Sludge Strider"));
    assert!(t.in_graveyard(P0, "Sol Ring"));
    assert_eq!(triggers_on_stack(&t, "pay {1}"), 1);
}

#[test]
fn sludge_strider_triggers_twice_for_a_zero_toughness_artifact_creature() {
    cr!("603.6a", "603.10a", "704.3", "704.5f"); // CR 603.6c
    ruling!(
        "Sludge Strider",
        "If an artifact creature with 0 toughness enters under your control, Sludge Strider's ability will trigger a total of twice: Once when that creature enters, and once when it's put into a graveyard. Both abilities will be put on the stack after the state-based action check that puts the 0-toughness creature into the graveyard is finished."
    );
    supported("Sludge Strider");
    supported("Hangarback Walker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sludge Strider");
    // Hangarback Walker put onto the battlefield enters with no counters: a 0/0.
    let w = t.enter(P0, "Hangarback Walker");
    assert!(t.on_battlefield(w));
    assert!(stack(&t).is_empty());
    t.settle();
    assert!(t.in_graveyard(P0, "Hangarback Walker"));
    assert_eq!(triggers_on_stack(&t, "pay {1}"), 2);
}

#[test]
fn ark_of_hunger_and_soul_enervation_trigger_once_for_cards_leaving_together() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Ark of Hunger",
        "If multiple cards leave your graveyard at the same time, Ark of Hunger's first ability will trigger only once."
    );
    ruling!(
        "Soul Enervation",
        "If multiple creature cards leave your graveyard at the same time, Soul Enervation's last ability will trigger only once."
    );
    supported("Ark of Hunger");
    supported("Soul Enervation");
    supported("Bojuka Bog");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ark of Hunger");
    t.battlefield(P0, "Soul Enervation");
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    // Bojuka Bog: "When this land enters, exile target player's graveyard."
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.enter(P0, "Bojuka Bog");
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(triggers_on_stack(&t, "1 damage to each opponent"), 1);
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn grandmother_ravi_sengir_triggers_for_creatures_dying_with_her() {
    cr!("603.10a", "603.6c", "608.2h");
    ruling!(
        "Invasion of Ulgrotha // Grandmother Ravi Sengir",
        "If creatures controlled by opponents die at the same time as Grandmother Ravi Sengir, the last ability of Grandmother Ravi Sengir will trigger once for each of those creatures. You won’t put +1/+1 counters on it, but you will gain some life."
    );
    supported("Invasion of Ulgrotha // Grandmother Ravi Sengir");
    supported("Wrath of God");
    let mut t = TestGame::new(2);
    let ravi = back_face(&mut t, P0, "Invasion of Ulgrotha // Grandmother Ravi Sengir");
    assert_eq!(name_now(&t, ravi), "Grandmother Ravi Sengir");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert!(!t.on_battlefield(ravi));
    assert_eq!(triggers_on_stack(&t, "gain 1 life"), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.obj_now(ravi).counter(counters::PLUS1), 0);
}

#[test]
fn fumulus_triggers_from_a_cost_resolve_before_and_from_an_effect_after() {
    cr!("603.3", "603.2", "601.2h", "608.2"); // CR 603.3b.
    ruling!(
        "Fumulus, the Infestation",
        "If a nontoken creature is sacrificed to pay a cost of a spell or ability, Fumulus's thirdsecond ability will resolve before that spell or ability."
    );
    supported("Fumulus, the Infestation");
    supported("Bontu the Glorified");
    supported("Innocent Blood");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fumulus, the Infestation");
    let bontu = t.battlefield(P0, "Bontu the Glorified");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    // Bontu: "{1}{B}, Sacrifice another creature: Scry 1. Each opponent loses 1 life and
    // you gain 1 life." The sacrifice is a cost: the Insect trigger goes on the stack
    // above the ability.
    t.answer_choose(P0, &[obj(bears)]);
    let ability = activate_containing(&mut t, P0, bontu, "Sacrifice another creature")
        .unwrap()
        .unwrap();
    t.settle();
    let st = stack(&t);
    assert_eq!(st.len(), 2);
    assert_eq!(st[0], ability);
    assert_eq!(triggers_on_stack(&t, "Insect"), 1);
    t.resolve();
    assert_eq!(tokens_of(&t, P0, "Insect"), 1);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Innocent Blood: "Each player sacrifices a creature." The sacrifice happens as it
    // resolves: it finishes resolving before the trigger is put on the stack.
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giant)]);
    let blood = cast_new(&mut t, P0, "Innocent Blood", &[]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Innocent Blood"));
    assert!(resolved(&t, blood));
    assert_eq!(triggers_on_stack(&t, "Insect"), 2);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Insect"), 3);
}

/// Gollum, Riddle Master enters under P0's control choosing even (zero is even).
fn gollum_even(t: &mut TestGame) -> ObjectId {
    supported("Gollum, Riddle Master");
    let from = t.asked().len();
    let g = t.enter(P0, "Gollum, Riddle Master");
    t.resolve_all();
    // Find what was chosen: the option decision asked as it entered.
    let chosen_even = t.asked()[from..].iter().any(|(p, d)| {
        *p == P0
            && matches!(d, Decision::ChooseOption { options, .. }
                if options.first().is_some_and(|o| o.to_lowercase().contains("even")))
    });
    if !chosen_even {
        // The first option was odd: re-enter choosing the second.
        let mut t2 = TestGame::new(2);
        std::mem::swap(t, &mut t2);
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        let g = t.enter(P0, "Gollum, Riddle Master");
        t.resolve_all();
        return g;
    }
    g
}

#[test]
fn gollums_trigger_does_nothing_once_all_three_modes_have_been_chosen() {
    cr!("603.3c", "700.2");
    ruling!(
        "Gollum, Riddle Master",
        "If all three modes have been chosen, Gollum's triggered ability is removed from the stack with no effect, but Gollum remains on the battlefield."
    );
    let mut t = TestGame::new(2);
    let gollum = gollum_even(&mut t);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 8);
    let (life0, life1) = (t.life(P0), t.life(P1));
    let hand = t.hand_size(P0);
    // P1 casts four spells with even mana value: three triggers each choose a different
    // mode; the fourth has none left, so it's removed from the stack as it would be put
    // there (CR 603.3c).
    for i in 0..4 {
        let bears = t.hand(P1, "Grizzly Bears");
        t.cast(P1, bears).go();
        t.settle();
        assert_eq!(
            triggers_on_stack(&t, "chosen"),
            if i < 3 { 1 } else { 0 },
            "spell {i}"
        );
        t.resolve_all();
        for l in t
            .g
            .permanents()
            .filter(|o| o.controller == P1 && o.is(CardType::Land))
            .map(|o| o.id)
            .collect::<Vec<_>>()
        {
            t.g.objects[l.0 as usize].tapped = false;
        }
    }
    // Exactly the three modes' effects happened once each: a counter, a drain of 2, a
    // card.
    assert!(t.on_battlefield(gollum));
    assert_eq!(t.counters(gollum, counters::PLUS1), 1);
    assert_eq!((t.life(P0), t.life(P1)), (life0 + 2, life1 - 2));
    assert_eq!(t.hand_size(P0), hand + 1);
}
