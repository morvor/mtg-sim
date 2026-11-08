//! Rulings batch P070 — several cards cast during one resolution (CR 608.2g, 601.2): they
//! are cast one at a time, in the order the player chooses, each with its own modes and
//! targets, and none resolves until the effect is done; the last one cast is on top of
//! the stack and resolves first (CR 405.5, 608.1).

use crate::r_p070_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Queues the order in which P0 casts the cards it chose (indices into the chosen list).
fn order(t: &mut TestGame, idx: &[usize]) {
    t.answer(P0, DecisionKind::Order, Answer::Indices(idx.to_vec()));
}

/// With Lightning Bolt and Shock (both at P1) cast in that order and still on the stack,
/// asserts the stack holds them with Shock on top, then that Shock resolves first.
fn shock_resolves_first(t: &mut TestGame) {
    let n = t.stack_len();
    assert!(n >= 2, "{}", t.dump_log());
    let names = stack_names(t);
    assert_eq!(
        &names[n - 2..],
        ["Lightning Bolt", "Shock"],
        "{}",
        t.dump_log()
    );
    let life = t.life(P1);
    t.resolve();
    assert_eq!(
        t.life(P1),
        life - 2,
        "Shock resolved first: {}",
        t.dump_log()
    );
    t.resolve();
    assert_eq!(t.life(P1), life - 5);
}

/// Answers for casting Lightning Bolt then Shock, both at P1, from among `cards` (chosen
/// in that order).
fn cast_bolt_then_shock(t: &mut TestGame, bolt: ObjectId, shock: ObjectId) {
    t.answer_choose_moved(P0, &[o(bolt), o(shock)]);
    order(t, &[0, 1]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
}

#[test]
fn primeval_spawn_casts_spells_one_after_the_other_last_resolves_first() {
    cr!("608.2g", "405.5", "601.2");
    ruling!(
        "Primeval Spawn",
        "The spells are cast one after the other during the resolution of Primeval Spawn’s last ability. The one you cast last will be the first one to resolve."
    );
    supported("Primeval Spawn");
    let mut t = TestGame::new(2);
    let spawn = t.battlefield(P0, "Primeval Spawn");
    let lib = stack_library(&mut t, P0, &["Lightning Bolt", "Shock"]);
    move_to(&mut t, spawn, Zone::Graveyard(P0));
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    cast_bolt_then_shock(&mut t, lib[0], lib[1]);
    t.resolve();
    assert_eq!(t.stack_len(), 2, "both cast during the resolution");
    shock_resolves_first(&mut t);
}

#[test]
fn jace_exiles_from_every_library_before_casting_and_casts_one_at_a_time() {
    cr!("608.2g", "405.5", "601.2");
    ruling!(
        "Jace, Architect of Thought",
        "When resolving Jace's third ability, you search each player's library (including yours) and exile the nonland cards before casting any of them."
    );
    ruling!(
        "Jace, Architect of Thought",
        "You cast the cards by putting them on the stack one at a time, choosing modes, targets, and so on. The last card you cast will be the first one to resolve."
    );
    supported("Jace, Architect of Thought");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Architect of Thought");
    t.g.obj_mut(jace).counters.insert("loyalty".into(), 8);
    let shock = t.library_top(P0, "Shock");
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.answer_choose(P0, &[o(shock)]);
    t.answer_choose(P0, &[o(bolt)]);
    // Cast P1's Bolt first, then P0's Shock.
    order(&mut t, &[1, 0]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, jace, 2, &[]).unwrap();
    t.resolve();
    let log = t.dump_log();
    // Both cards were exiled before either was cast.
    let first_cast = log.find("P0 casts").expect("cast");
    for name in ["Shock", "Lightning Bolt"] {
        let exiled_at = log
            .find(&format!("{name} #"))
            .and_then(|i| log[i..].find("-> Exile").map(|j| i + j))
            .unwrap_or_else(|| panic!("{name} exiled: {log}"));
        assert!(exiled_at < first_cast, "{name}: {log}");
    }
    shock_resolves_first(&mut t);
}

#[test]
fn epic_experiment_casts_one_at_a_time_last_resolves_first() {
    cr!("608.2g", "405.5", "601.2");
    ruling!(
        "Epic Experiment",
        "You cast the cards one at a time, choosing modes, targets, and so on. The last card you cast will be the first one to resolve."
    );
    supported("Epic Experiment");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Shock", "Lightning Bolt"]);
    let ee = t.hand(P0, "Epic Experiment");
    pool(&mut t, P0, &[U, R, C, C]);
    cast_bolt_then_shock(&mut t, lib[1], lib[0]);
    t.cast(P0, ee).x(2).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Epic Experiment"));
    shock_resolves_first(&mut t);
}

#[test]
fn kefka_casts_one_at_a_time_last_resolves_first() {
    cr!("608.2g", "405.5", "601.2");
    ruling!(
        "Kefka, Dancing Mad",
        "You cast the spells one at a time, choosing modes, targets, and so on. The last spell you cast will be the first one to resolve."
    );
    supported("Kefka, Dancing Mad");
    // Two opponents, one card in each graveyard.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Kefka, Dancing Mad");
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let shock = t.graveyard(P2, "Shock");
    cast_bolt_then_shock(&mut t, bolt, shock);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve();
    // A card was exiled from each opponent's graveyard (not one from among them all), and
    // their owners lost life equal to the mana values (1 each).
    assert!(
        is_on_stack(&t, bolt) && is_on_stack(&t, shock),
        "{}",
        t.dump_log()
    );
    assert_eq!((t.life(P1), t.life(P2)), (19, 19), "{}", t.dump_log());
    shock_resolves_first(&mut t);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn izzet_chemister_casts_in_any_order_later_spells_target_earlier_ones() {
    cr!("608.2g", "601.2c", "115.1");
    ruling!(
        "Izzet Chemister",
        "You cast the exiled cards one at a time in any order. Later spells can target ones you cast earlier. None of the spells resolve until after you're done casting them all."
    );
    supported("Izzet Chemister");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Izzet Chemister");
    // Exiled in the order Twincast, Lightning Bolt; cast Bolt first.
    let twin = t.graveyard(P0, "Twincast");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    for card in [twin, bolt] {
        mana(&mut t, P0, R, 1);
        t.activate(P0, c, 0, &[o(card)]).unwrap();
        t.resolve_all();
        t.g.untap(c);
    }
    assert!(exiled(&t, twin) && exiled(&t, bolt));
    mana(&mut t, P0, R, 2);
    t.answer_choose_moved(P0, &[o(twin), o(bolt)]);
    order(&mut t, &[1, 0]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    target_named_when_offered(&mut t, P0, "Lightning Bolt");
    t.activate(P0, c, 1, &[]).unwrap();
    t.resolve();
    // Both on the stack, neither resolved: Twincast (targeting the Bolt) on top.
    assert_eq!(
        stack_names(&t),
        ["Lightning Bolt", "Twincast"],
        "{}",
        t.dump_log()
    );
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "the Bolt and its copy: {}", t.dump_log());
}

#[test]
fn collected_conjuring_casts_the_two_sorceries_in_either_order() {
    cr!("608.2g", "405.5");
    ruling!(
        "Collected Conjuring",
        "You may cast the two sorcery cards in either order. The second is cast before the first resolves."
    );
    supported("Collected Conjuring");
    for idx in [[0, 1], [1, 0]] {
        let mut t = TestGame::new(2);
        let lib = stack_library(&mut t, P0, &["Lava Spike", "Divination"]);
        let cc = t.hand(P0, "Collected Conjuring");
        pool(&mut t, P0, &[U, R, C, C]);
        t.answer_choose_moved(P0, &[o(lib[0]), o(lib[1])]);
        order(&mut t, &idx);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.cast(P0, cc).go();
        t.resolve();
        let names = stack_names(&t);
        let expect = if idx == [0, 1] {
            ["Lava Spike", "Divination"]
        } else {
            ["Divination", "Lava Spike"]
        };
        assert_eq!(names, expect, "{}", t.dump_log());
        let hand = t.hand_size(P0);
        t.resolve();
        // The second one cast resolves first.
        if idx == [0, 1] {
            assert_eq!((t.hand_size(P0), t.life(P1)), (hand + 2, 20));
        } else {
            assert_eq!((t.hand_size(P0), t.life(P1)), (hand, 17));
        }
    }
}

// ---------------------------------------------------------------------------
// Villainous Wealth and Forger's Foundry
// ---------------------------------------------------------------------------

/// Villainous Wealth (X = 3) at P1, whose library has `top_first` on top.
fn villainous_wealth(t: &mut TestGame, top_first: &[&str]) -> Vec<ObjectId> {
    let lib = stack_library(t, P1, top_first);
    pool(t, P0, &[B, G, U, C, C, C]);
    let w = t.hand(P0, "Villainous Wealth");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, w).x(3).go();
    lib
}

#[test]
fn villainous_wealth_casts_one_at_a_time_ignoring_type_timing_only() {
    cr!("608.2g", "405.5", "307.1", "601.3");
    ruling!(
        "Villainous Wealth",
        "You cast the spells one at a time as Villainous Wealth is resolving, choosing modes, targets, and so on. The last spell you cast will be the first one to resolve. Ignore timing restrictions based on the spells' types. Other timing restrictions, such as \"Cast [this spell] only during combat,\" must be followed."
    );
    supported("Villainous Wealth");
    let mut t = TestGame::new(2);
    let lib = villainous_wealth(&mut t, &["Lightning Bolt", "Shock", "Panic"]);
    let from = t.asked().len();
    cast_bolt_then_shock(&mut t, lib[0], lib[1]);
    t.resolve();
    // Panic ("Cast this spell only during combat before blockers are declared") wasn't
    // castable in the main phase.
    assert!(exiled(&t, lib[2]), "{}", t.dump_log());
    assert!(!was_offered(&t, from, lib[2]), "{}", t.dump_log());
    shock_resolves_first(&mut t);
}

#[test]
fn villainous_wealth_casts_during_its_resolution_only() {
    cr!("608.2g");
    ruling!(
        "Villainous Wealth",
        "You choose which spells (if any) to cast as Villainous Wealth resolves. If you do, you do so as part of the resolution of Villainous Wealth. You can't wait to cast them later in the turn."
    );
    supported("Villainous Wealth");
    let mut t = TestGame::new(2);
    let lib = villainous_wealth(&mut t, &["Grizzly Bears", "Divination", "Forest"]);
    // Only the Bears (a creature spell, while Villainous Wealth resolves).
    t.answer_choose_moved(P0, &[o(lib[0])]);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(is_on_stack(&t, lib[0]), "{}", t.dump_log());
    t.resolve_all();
    not_castable_later(&mut t, lib[1]);
}

/// Casts `name` (a blue instant or sorcery of mana value 3 or less, from P0's hand) with
/// the mana of Forger's Foundry `ff` (and colorless mana for the rest), exiling it with
/// the Foundry as it resolves.
fn exile_with_foundry(t: &mut TestGame, ff: ObjectId, name: &str) -> ObjectId {
    let c = t.hand(P0, name);
    t.g.untap(ff);
    t.activate(P0, ff, 0, &[]).unwrap();
    mana(t, P0, C, 2);
    t.answer_yes(P0, true);
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(exiled(t, c), "{name}: {}", t.dump_log());
    t.g.players[0].mana_pool.empty();
    t.g.untap(ff);
    t.clear_answers();
    c
}

/// Forger's Foundry on P0's battlefield with `cards` exiled with it, in that order.
fn foundry(t: &mut TestGame, cards: &[&str]) -> (ObjectId, Vec<ObjectId>) {
    let ff = t.battlefield(P0, "Forger's Foundry");
    let ids = cards.iter().map(|n| exile_with_foundry(t, ff, n)).collect();
    (ff, ids)
}

#[test]
fn forgers_foundry_casts_one_at_a_time_ignoring_type_timing_only() {
    cr!("608.2g", "405.5", "307.1", "601.3");
    ruling!(
        "Forger's Foundry",
        "You cast the spells one at a time as Forger's Foundry's last ability is resolving, choosing modes, targets, and so on. The last spell you cast will be the first one to resolve. Ignore timing restrictions based on the cards' types. Other timing restrictions, such as \"Cast [this spell] only during combat,\" must be followed."
    );
    supported("Forger's Foundry");
    supported("Siren's Call");
    let mut t = TestGame::new(2);
    // Siren's Call ("Cast this spell only during an opponent's turn, before attackers are
    // declared") is cast on P1's turn with the Foundry's mana and exiled.
    t.set_step(P1, Step::Upkeep);
    let (ff, sc) = foundry(&mut t, &["Siren's Call"]);
    t.set_step(P0, Step::PrecombatMain);
    let div = exile_with_foundry(&mut t, ff, "Divination");
    let opt = exile_with_foundry(&mut t, ff, "Opt");
    pool(&mut t, P0, &[U, U, C, C, C]);
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[o(div), o(opt)]);
    order(&mut t, &[0, 1]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.activate(P0, ff, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(stack_names(&t), ["Divination", "Opt"], "{}", t.dump_log());
    assert!(
        exiled(&t, sc[0]) && !was_offered(&t, from, sc[0]),
        "{}",
        t.dump_log()
    );
    let hand = t.hand_size(P0);
    t.resolve();
    // Opt resolved first: one card.
    assert_eq!(t.hand_size(P0), hand + 1, "{}", t.dump_log());
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn forgers_foundry_casts_during_its_resolution_only() {
    cr!("608.2g");
    ruling!(
        "Forger's Foundry",
        "You choose which spells (if any) to cast as Forger's Foundry's last ability resolves. If you do, you do so as part of the resolution of that ability. You can't wait to cast them later in the turn."
    );
    supported("Forger's Foundry");
    let mut t = TestGame::new(2);
    let (ff, ids) = foundry(&mut t, &["Opt", "Divination"]);
    t.lands(P0, "Island", 5);
    t.answer_choose_moved(P0, &[o(ids[1])]);
    t.answer_yes(P0, true);
    t.activate(P0, ff, 1, &[]).unwrap();
    t.resolve();
    assert!(is_on_stack(&t, ids[1]), "{}", t.dump_log());
    t.resolve_all();
    not_castable_later(&mut t, ids[0]);
}

#[test]
fn forgers_foundry_trigger_uses_the_chosen_x() {
    cr!("202.3e", "107.3c", "603.2");
    ruling!(
        "Forger's Foundry",
        "When casting an instant or sorcery spell with {X} in its mana cost using the mana added by Forger's Foundry, use the value of X chosen when you cast that spell to determine whether or not the delayed triggered ability triggers."
    );
    supported("Forger's Foundry");
    supported("Prosperity");
    // Prosperity ({X}{U}): X 2 is mana value 3 (exiled); X 3 is 4 (not).
    for (x, exile) in [(2, true), (3, false)] {
        let mut t = TestGame::new(2);
        let ff = t.battlefield(P0, "Forger's Foundry");
        let p = t.hand(P0, "Prosperity");
        t.activate(P0, ff, 0, &[]).unwrap();
        mana(&mut t, P0, C, x as u32);
        t.answer_yes(P0, true);
        t.cast(P0, p).x(x).go();
        t.resolve_all();
        assert_eq!(exiled(&t, p), exile, "X {x}: {}", t.dump_log());
        assert_eq!(t.in_graveyard(P0, "Prosperity"), !exile);
    }
}
