//! Rulings batch P146 — Treasure and surveil engines: "Nth spell each turn" triggers
//! (CR 603.2), per-player and per-event combat damage triggers (CR 510.2, 603.2c),
//! end step intervening-if conditions (CR 603.4), modal triggers (CR 700.2b), fights with
//! a missing source (CR 701.14b, 608.2b), discard triggers and the stack (CR 603.3), and
//! copies of spells (CR 707.10c).

use crate::r_p146_common::*;
use crate::r_s25_common::{change_copy_targets, keep_copy_targets};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The number of surveil decisions asked of `p` since `from`.
fn surveils_since(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::Surveil { .. }))
        .count()
}

#[test]
fn glarb_x_must_make_the_mana_value_four_or_more() {
    cr!("601.2b", "601.3e", "202.3e");
    ruling!(
        "Glarb, Calamity's Augur",
        "If the top card of your library has {X} in its mana cost, you must choose a value of X that makes that spell's mana value 4 or greater in order to cast it."
    );
    supported("Glarb, Calamity's Augur");
    supported("Walking Ballista");
    // "You may play lands and cast spells with mana value 4 or greater from the top of
    // your library." Walking Ballista ({X}{X}): X=1 is mana value 2; X=2 is 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glarb, Calamity's Augur");
    t.lands(P0, "Wastes", 4);
    let wb = t.library_top(P0, "Walking Ballista");
    assert!(t.cast(P0, wb).x(1).try_go().is_err());
    assert_eq!(t.zone(wb), Zone::Library(P0));
    t.clear_answers();
    t.cast(P0, wb).x(2).go();
    t.resolve_all();
    let wb = t.named_on_battlefield("Walking Ballista");
    assert_eq!(wb.len(), 1);
    assert_eq!(t.counters(wb[0], "+1/+1"), 2);
}

#[test]
fn dimir_strandcatcher_counts_opponents_as_it_resolves() {
    cr!("608.2h", "800.4a");
    ruling!(
        "Dimir Strandcatcher",
        "The value of X is determined only once, as Dimir Strandcatcher's second ability resolves. If a player leaves the game after the ability triggers but before it resolves, they won't be counted when the ability resolves."
    );
    // "Whenever you attack, surveil X, where X is the number of opponents being
    // attacked." (Its end step ability doesn't compile; this one does.)
    let c = mtg_engine::card::card("Dimir Strandcatcher");
    assert_eq!(c.unsupported_text().len(), 1);
    assert!(c.unsupported_text()[0].starts_with("At the beginning of each end step"));
    let mut t = TestGame::new(3);
    let d = t.battlefield(P0, "Dimir Strandcatcher");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(d, Entity::Player(P1)), (b, Entity::Player(P2))]);
    assert_eq!(t.stack_len(), 1);
    // P2 leaves the game before it resolves: surveil 1, not 2.
    t.g.lose_game(P2);
    t.settle();
    let from = n_asked(&t);
    t.resolve_all();
    let sizes: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Surveil { cards } => Some(cards.len()),
            _ => None,
        })
        .collect();
    assert_eq!(sizes, vec![1]);
}

#[test]
fn wary_farmer_checks_as_the_end_step_begins() {
    cr!("603.4", "513.1");
    ruling!(
        "Wary Farmer",
        "Wary Farmer's ability will check as your end step starts to see if another creature entered the battlefield under your control this turn."
    );
    supported("Wary Farmer");
    // "At the beginning of your end step, if another creature entered the battlefield
    // under your control this turn, surveil 1."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wary Farmer");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A creature entering during the end step is too late.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // One that entered earlier in the turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wary Farmer");
    t.enter(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let from = n_asked(&t);
    t.resolve_all();
    assert_eq!(surveils_since(&t, P0, from), 1);
}

#[test]
fn relic_retriever_checks_as_the_end_step_begins() {
    cr!("603.4", "513.1");
    ruling!(
        "Relic Retriever",
        "Relic Retriever's last ability will check as your end step starts to see if any cards have left your graveyard this turn."
    );
    supported("Relic Retriever");
    // "At the beginning of each end step, if a card left your graveyard this turn, create
    // a Treasure token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Relic Retriever");
    let card = t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    move_to(&mut t, card, Zone::Exile);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 0);
    // A card that left before the end step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Relic Retriever");
    let card = t.graveyard(P0, "Grizzly Bears");
    move_to(&mut t, card, Zone::Exile);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
}

/// The mode indices chosen in modal-choice decisions asked of P0 since `from`.
fn mode_choices_since(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseModes { .. }))
        .count()
}

#[test]
fn etherwrought_page_mode_is_chosen_as_it_triggers_each_time() {
    cr!("700.2b", "603.3c");
    ruling!(
        "Etherwrought Page",
        "You choose a mode when the ability triggers."
    );
    ruling!(
        "Etherwrought Page",
        "You may choose a different mode each time it triggers."
    );
    supported("Etherwrought Page");
    // "At the beginning of your upkeep, choose one — • You gain 2 life. • Surveil 1. •
    // Each opponent loses 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Etherwrought Page");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    let from = n_asked(&t);
    into_upkeep(&mut t, P0);
    // Chosen as it was put on the stack.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(mode_choices_since(&t, from), 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 20));
    // The next upkeep: another mode.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 19));
}

#[test]
fn deadeye_plunderers_damage_can_become_lethal_later() {
    cr!("704.5g", "120.6", "613.4c");
    ruling!(
        "Deadeye Plunderers",
        "Because damage remains marked on a creature until it's removed as the turn ends, the damage Deadeye Plunderers takes during combat may become lethal if artifacts you control leave the battlefield later in the turn."
    );
    supported("Deadeye Plunderers");
    // "This creature gets +1/+1 for each artifact you control."
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Deadeye Plunderers");
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(d), (5, 5));
    let src = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(src, obj(d), 4, true);
    t.settle();
    assert!(t.on_battlefield(d));
    destroy_together(&mut t, &[a, b]);
    assert!(t.in_graveyard(P0, "Deadeye Plunderers"));
}

#[test]
fn thorin_triggers_for_each_dwarf() {
    cr!("510.2", "603.2c");
    ruling!(
        "Thorin, Company's Leader",
        "If multiple Dwarves you control deal combat damage to players and/or battles at the same time, Thorin's first ability will trigger once for each of those Dwarves."
    );
    supported("Thorin, Company's Leader");
    // "Whenever a Dwarf you control deals combat damage to a player or battle, create two
    // Treasure tokens."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thorin, Company's Leader");
    let a = t.battlefield(P0, "Dwarven Trader");
    let b = t.battlefield(P0, "Dwarven Trader");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(stacked_triggers(&t), 2);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 4);
}

/// Surly Badgersaur and a Grizzly Bears for P1; P0 discards Lightning Bolt (a
/// noncreature, nonland card) with the Bears as the fight trigger's target.
fn badgersaur_discards_bolt() -> (TestGame, ObjectId, ObjectId) {
    supported("Surly Badgersaur");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Surly Badgersaur");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_targets(P0, &[obj(bears)]);
    discard(&mut t, P0, bolt);
    assert_eq!(t.stack_len(), 1);
    (t, s, bears)
}

#[test]
fn surly_badgersaur_fight_without_badgersaur_or_target() {
    cr!("701.14b", "608.2b");
    ruling!(
        "Surly Badgersaur",
        "If the target creature is an illegal target when Surly Badgersaur's last ability tries to resolve, the ability doesn't resolve. If it's a legal target but Surly Badgersaur is no longer on the battlefield when the ability resolves, the target creature won't deal or be dealt damage."
    );
    // "Whenever you discard a noncreature, nonland card, this creature fights up to one
    // target creature you don't control."
    let (mut t, s, bears) = badgersaur_discards_bolt();
    move_to(&mut t, s, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.obj(bears).damage, 0);
    // The target gone: nothing happens to the Badgersaur.
    let (mut t, s, bears) = badgersaur_discards_bolt();
    move_to(&mut t, bears, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.obj(s).damage, 0);
    // Normally they fight.
    let (mut t, s, _) = badgersaur_discards_bolt();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.obj(s).damage, 2);
}

#[test]
fn surly_badgersaur_discard_as_cost_versus_while_resolving() {
    cr!("603.3", "601.2i", "608.2");
    ruling!(
        "Surly Badgersaur",
        "If you discard a card as a cost to cast a spell or activate an ability, Surly Badgersaur's triggered ability resolves before that spell or ability but after you've chosen targets for it."
    );
    supported("Lightning Axe");
    // Lightning Axe: "As an additional cost to cast this spell, discard a card or pay {5}.
    // Lightning Axe deals 5 damage to target creature." Discarding Hill Giant: the
    // Badgersaur's +1/+1 counter trigger is above the Axe.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Surly Badgersaur");
    let target = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let axe = t.hand(P0, "Lightning Axe");
    let giant = t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    let spell = t.cast(P0, axe).target(target).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], spell);
    t.resolve();
    assert_eq!(t.pt(s), (4, 4));
    assert!(t.on_battlefield(target));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Discarding while Haggle resolves: Haggle finishes (the draw) before the trigger.
    ruling!(
        "Surly Badgersaur",
        "If you discard a card while a spell or ability is resolving, that spell or ability finishes resolving before Surly Badgersaur's triggered ability does."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Surly Badgersaur");
    t.lands(P0, "Mountain", 1);
    let giant = t.hand(P0, "Hill Giant");
    let h = t.hand(P0, "Merchant of the Vale // Haggle");
    t.cast(P0, h).method(CastMethod::Half(1)).go();
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(giant)]);
    let hand = t.hand_size(P0);
    t.resolve();
    // Haggle resolved fully (drew a card); the trigger is waiting.
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.pt(s), (3, 3));
    t.resolve_all();
    assert_eq!(t.pt(s), (4, 4));
}

#[test]
fn surly_badgersaur_abilities_are_triggered() {
    cr!("603.1", "602.1");
    ruling!(
        "Surly Badgersaur",
        "Surly Badgersaur's abilities are triggered abilities, not activated abilities. They don't allow you to discard a card whenever you want; rather, you need some other way of discarding a card, such as a cycling ability."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Surly Badgersaur");
    t.hand(P0, "Forest");
    assert!(!can_activate(&mut t, P0, s));
    // Another way to discard: Mad Prophet ("{T}, Discard a card: Draw a card.").
    let m = t.battlefield(P0, "Mad Prophet");
    activate_resolve(&mut t, P0, m, 0, &[]);
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn monument_to_endurance_does_nothing_once_all_modes_were_chosen() {
    cr!("700.2b", "603.3c");
    ruling!(
        "Monument to Endurance",
        "If you discard a card after all three have been chosen in a turn, that instance of the ability is removed from the stack with no effect."
    );
    supported("Monument to Endurance");
    // "Whenever you discard a card, choose one that hasn't been chosen this turn — • Draw
    // a card. • Create a Treasure token. • Each opponent loses 3 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Monument to Endurance");
    let cards = fill_hand(&mut t, P0, 4);
    for c in &cards[..3] {
        discard(&mut t, P0, *c);
        t.resolve_all();
    }
    assert_eq!(treasures(&t, P0), 1);
    assert_eq!(t.life(P1), 17);
    let hand = t.hand_size(P0);
    discard(&mut t, P0, cards[3]);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand - 1);
}

#[test]
fn storm_the_vault_triggers_per_player_and_per_damage_step() {
    cr!("510.2", "510.4", "603.2c", "810.7");
    ruling!(
        "Storm the Vault // Vault of Catlacan",
        "In a Two-Headed Giant game, if you control more than one creature that can attack, you attack different opponents so that Storm the Vault's first ability triggers twice."
    );
    ruling!(
        "Storm the Vault // Vault of Catlacan",
        "Storm the Vault's first ability can trigger more than once in a turn if creatures you control deal combat damage at different times in a turn (most likely because one or more has first strike) or if creatures you control deal combat damage to more than one player at once."
    );
    supported("Storm the Vault // Vault of Catlacan");
    // "Whenever one or more creatures you control deal combat damage to a player, create a
    // Treasure token."
    // Two-Headed Giant: attacking both opponents: two Treasures.
    let mut t = two_headed_giant();
    t.battlefield(P0, "Storm the Vault // Vault of Catlacan");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P2)), (b, Entity::Player(P3))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 2);
    // Both at the same opponent: one.
    let mut t = two_headed_giant();
    t.battlefield(P0, "Storm the Vault // Vault of Catlacan");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P2)), (b, Entity::Player(P2))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    // A first striker and a regular creature: two damage steps, two Treasures.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Storm the Vault // Vault of Catlacan");
    let a = t.battlefield(P0, "Savannah Lions");
    let b = t.battlefield(P0, "Elvish Archers");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 2);
}

/// P1 casts Lightning Bolt targeting P0 (with a Mountain for it).
fn p1_bolts(t: &mut TestGame) -> ObjectId {
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go()
}

#[test]
fn monologue_tax_counts_spells_cast_before_it_and_triggers_only_on_the_second() {
    cr!("603.2", "601.2i");
    ruling!(
        "Monologue Tax",
        "It doesn't matter if Monologue Tax was on the battlefield for the first spell. It also doesn't matter if that first spell resolved or not."
    );
    ruling!(
        "Monologue Tax",
        "Nothing special happens on the third spell, fourth spell, and so on."
    );
    supported("Monologue Tax");
    // "Whenever an opponent casts their second spell each turn, you create a Treasure
    // token."
    let mut t = TestGame::new(2);
    let first = p1_bolts(&mut t);
    counter(&mut t, first);
    t.battlefield(P0, "Monologue Tax");
    p1_bolts(&mut t);
    t.settle();
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
    for _ in 0..2 {
        p1_bolts(&mut t);
        t.resolve_all();
    }
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn monologue_tax_triggers_for_each_opponent() {
    cr!("603.2");
    ruling!(
        "Monologue Tax",
        "The ability can trigger once each turn for each opponent."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Monologue Tax");
    for p in [P1, P2] {
        for _ in 0..2 {
            t.lands(p, "Mountain", 1);
            let bolt = t.hand(p, "Lightning Bolt");
            t.cast(p, bolt).target(Entity::Player(P0)).go();
            t.resolve_all();
        }
    }
    assert_eq!(treasures(&t, P0), 2);
}

#[test]
fn vault_robber_cost_is_paid_before_anyone_can_respond() {
    cr!("602.2b", "601.2h", "601.2i");
    ruling!(
        "Vault Robber",
        "Once you announce that you’re activating the activated ability, no player may take actions until the ability has been paid for."
    );
    supported("Vault Robber");
    // "{1}, {T}, Exile a creature card from your graveyard: Create a Treasure token."
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vault Robber");
    t.lands(P0, "Wastes", 1);
    let card = t.graveyard(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(card)]);
    let from = n_asked(&t);
    t.activate(P0, v, 0, &[]).unwrap();
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 1);
}

#[test]
fn pain_distributor_must_be_there_when_the_first_spell_is_cast() {
    cr!("603.2", "601.2i");
    ruling!(
        "Pain Distributor",
        "Pain Distributor has to be on the battlefield at the moment they cast their first spell."
    );
    ruling!(
        "Pain Distributor",
        "Pain Distributor’s second ability triggers whenever a player casts their first spell each turn, not just on their own turn."
    );
    supported("Pain Distributor");
    // "Whenever a player casts their first spell each turn, they create a Treasure token."
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Pain Distributor", &[]);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 0);
    // The opponent's first spell this turn (during P0's turn): P1 gets a Treasure.
    p1_bolts(&mut t);
    t.resolve_all();
    assert_eq!(treasures(&t, P1), 1);
    // P0's second spell: nothing.
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 0);
}

#[test]
fn sword_of_wealth_and_power_copy_may_keep_or_change_targets() {
    cr!("707.10c", "603.7");
    ruling!(
        "Sword of Wealth and Power",
        "The copy created by Sword of Wealth and Power’s delayed triggered ability will have the same targets as the spell it’s copying unless you choose new ones."
    );
    supported("Sword of Wealth and Power");
    // "Whenever equipped creature deals combat damage to a player, create a Treasure
    // token. When you next cast an instant or sorcery spell this turn, copy that spell.
    // You may choose new targets for the copy."
    for change in [false, true] {
        let mut t = TestGame::new(2);
        let sword = t.battlefield(P0, "Sword of Wealth and Power");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.g.attach(sword, obj(bears));
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        block_and_finish(&mut t, P1, &[]);
        t.resolve_all();
        assert_eq!(t.life(P1), 16);
        assert_eq!(treasures(&t, P0), 1);
        t.set_step(P0, Step::PostcombatMain);
        if change {
            change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
        } else {
            keep_copy_targets(&mut t, P0);
        }
        cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
        t.resolve_all();
        if change {
            assert_eq!(t.life(P1), 13);
            assert!(t.in_graveyard(P1, "Hill Giant"));
        } else {
            assert_eq!(t.life(P1), 10);
            assert!(t.on_battlefield(giant));
        }
    }
}
