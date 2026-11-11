//! Rulings batch P144 — Ral, Monsoon Mage // Ral, Leyline Prodigy and Stella Lee, Wild
//! Card: coin flips, loyalty counters for spells cast this turn, divided damage, casting
//! exiled cards without paying their mana costs (X is 0), and copies of spells.

use crate::r_p107_common::{act, mana};
use crate::r_p119_common::counter_spell;
use crate::r_p144_common::*;
use crate::r_s02_common::{can_activate, can_cast, destroy};
use crate::r_s17_common::enter_transformed;
use crate::r_s25_common::{change_copy_targets, keep_copy_targets, spell_copies};
use crate::r_s29_common::divide;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const RAL: &str = "Ral, Monsoon Mage // Ral, Leyline Prodigy";

/// Ral, Leyline Prodigy entering transformed (no spells cast this turn: 2 loyalty), with
/// loyalty counters added to make `loyalty`.
fn prodigy(t: &mut TestGame, loyalty: u32) -> ObjectId {
    let r = enter_transformed(t, P0, RAL);
    let have = t.counters(r, counters::LOYALTY);
    if loyalty > have {
        t.g.add_counters(obj(r), counters::LOYALTY, loyalty - have, None);
    }
    r
}

/// The −8: exiles the top eight cards of P0's library.
fn ultimate(t: &mut TestGame, r: ObjectId) {
    act(t, P0, r, "top eight", &[]).expect("−8");
    t.resolve_all();
}

#[test]
fn ral_flips_as_the_trigger_resolves_and_enters_with_loyalty_per_spell_cast() {
    cr!("705.2", "608.2c", "122.6", "601.2i");
    supported(RAL);
    supported("Edgar, King of Figaro");
    ruling!(
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
        "The first ability of Ral, Leyline Prodigy will count any instant and sorcery spells you've cast that turn, including ones that were countered, didn't resolve for other reasons, or are still on the stack."
    );
    let mut t = TestGame::new(2);
    // Before Ral is around: Opt (resolves), Shock (countered).
    let opt = t.hand(P0, "Opt");
    mana(&mut t, P0, ManaType::U, 1);
    t.cast(P0, opt).go();
    t.resolve_all();
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    let s = t.cast(P0, shock).target(P1).go();
    counter_spell(&mut t, s);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Ral and Edgar (the first coin flipped each turn is won); Lightning Bolt: Ral's
    // trigger, won flip, Ral exiled and returned transformed while the Bolt is on the
    // stack. 2 + 3 loyalty.
    let ral = t.battlefield(P0, RAL);
    t.battlefield(P0, "Edgar, King of Figaro");
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);

    ruling!(
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
        "You flip a coin as Ral, Monsoon Mage's triggered ability resolves. No player may take actions between seeing the result of the flip and the effects that occur based on the result of the flip."
    );
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve();
    // Nobody got priority during the resolution: the flip and its result together.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    let r = t.g.current(ral);
    assert!(t.on_battlefield(r));
    assert_eq!(t.obj_now(r).chars.name.as_str(), "Ral, Leyline Prodigy");
    assert_eq!(t.counters(r, counters::LOYALTY), 5);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn ral_leyline_prodigy_activates_only_at_sorcery_speed_the_turn_he_enters() {
    cr!("606.3", "307.1");
    supported(RAL);
    ruling!(
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
        "You can activate one of Ral, Leyline Prodigy's loyalty abilities the turn he enters the battlefield. However, you may do so only during one of your main phases when the stack is empty. For example, if Ral, Leyline Prodigy enters the battlefield during combat, there will be an opportunity for your opponent to remove him before you can activate one of his abilities."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let r = enter_transformed(&mut t, P0, RAL);
    assert!(t.on_battlefield(r));
    assert!(!can_activate(&mut t, P0, r));
    // Back in a main phase of the same turn (stack empty): yes; with a spell on the
    // stack: no.
    t.g.combat = None;
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_activate(&mut t, P0, r));
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bolt).target(P1).go();
    assert!(!can_activate(&mut t, P0, r));
}

#[test]
fn ral_divided_damage_is_chosen_on_activation_and_kept() {
    cr!("601.2d", "602.2b", "608.2b");
    supported(RAL);
    supported("Wind Drake");
    ruling!(
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
        "You choose how many targets Ral, Leyline Prodigy's second loyalty ability has and how the damage is divided as you activate the ability. Each target must receive at least 1 damage."
    );
    ruling!(
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
        "If some of the targets are illegal as Ral, Leyline Prodigy's second loyalty ability tries to resolve, the original division of damage still applies and the damage that would have been dealt to the illegal targets is lost. It won't be dealt instead to a legal target. If all of the targets are illegal, the ability won't resolve and none of its effects will happen. You won't draw a card."
    );
    // Two targets, 1 each; the division is asked as the ability is activated, with at
    // least 1 for each. One target becomes illegal: the other still gets only 1.
    let mut t = TestGame::new(2);
    let r = prodigy(&mut t, 3);
    t.battlefield(P0, "Wind Drake");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, P0, &[1, 1]);
    act(&mut t, P0, r, "divided", &[]).expect("−2");
    let divs: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide { min_each, .. } => Some(*min_each),
            _ => None,
        })
        .collect();
    assert_eq!(divs, vec![1]);
    let hand = t.hand_size(P0);
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    // One target, illegal: nothing happens, no card drawn (with a blue permanent).
    let mut t = TestGame::new(2);
    let r = prodigy(&mut t, 3);
    t.battlefield(P0, "Wind Drake");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b)]);
    act(&mut t, P0, r, "divided", &[]).expect("−2");
    let hand = t.hand_size(P0);
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn ral_ultimate_casts_free_x_spells_with_x_0_and_normal_timing() {
    cr!("107.3b", "118.9", "601.3", "307.1");
    supported(RAL);
    ruling!(
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
        "If one of the exiled cards has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &[
            "Blaze",
            "Lava Spike",
            "Lightning Bolt",
            "Wastes",
            "Wastes",
            "Wastes",
            "Wastes",
            "Wastes",
            "Wastes",
        ],
    );
    let r = prodigy(&mut t, 8);
    ultimate(&mut t, r);
    assert!(lib[..8].iter().all(|c| t.zone(*c) == Zone::Exile));
    assert_ne!(t.zone(lib[8]), Zone::Exile);
    let blaze = t.g.current(lib[0]);
    t.cast(P0, blaze)
        .method(CastMethod::Free)
        .x(5)
        .target(P1)
        .go();
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 20);

    ruling!(
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
        "You follow all normal timing rules for spells cast with the permission granted by Ral, Leyline Prodigy's last ability. For example, if one of the exiled cards is a sorcery, you can cast it only during your main phase while the stack is empty."
    );
    to_combat_step(&mut t);
    assert!(!can_cast(&mut t, P0, lib[1], CastMethod::Free));
    assert!(can_cast(&mut t, P0, lib[2], CastMethod::Free));
    to_main(&mut t);
    assert!(can_cast(&mut t, P0, lib[1], CastMethod::Free));
}

#[test]
fn oracles_vault_free_casts_pay_additional_costs_only() {
    cr!("107.3b", "118.9", "118.9a", "601.2b");
    supported("Oracle's Vault");
    ruling!(
        "Oracle's Vault",
        "If the card has X in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    ruling!(
        "Oracle's Vault",
        "If you cast a card “without paying its mana cost,” you can't choose to cast it for any alternative costs, such as emerge costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Tormenting Voice, those must be paid to cast the card."
    );
    /// Oracle's Vault with three brick counters exiles the top card with its second
    /// ability: it may be cast without paying its mana cost.
    fn free_exile(t: &mut TestGame) {
        let v = t.battlefield(P0, "Oracle's Vault");
        t.g.add_counters(obj(v), "brick", 3, None);
        act(t, P0, v, "without paying", &[]).expect("activate");
        t.resolve_all();
    }
    // X is 0.
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Blaze", "Wastes"]);
    free_exile(&mut t);
    let blaze = t.g.current(lib[0]);
    t.cast(P0, blaze)
        .method(CastMethod::Free)
        .x(4)
        .target(P1)
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Tormenting Voice: without a card to discard, it can't be cast; with one, it's cast
    // by discarding it.
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &["Tormenting Voice", "Wastes", "Wastes", "Wastes"],
    );
    free_exile(&mut t);
    assert!(!can_cast(&mut t, P0, lib[0], CastMethod::Free));
    let fodder = t.hand(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, lib[0], CastMethod::Free));
    t.answer_choose(P0, &[obj(fodder)]);
    let voice = t.g.current(lib[0]);
    t.cast(P0, voice).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.zone(fodder), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 2);
    // An alternative cost (emerge) can't be used for it.
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Elder Deep-Fiend", "Wastes"]);
    free_exile(&mut t);
    t.battlefield(P0, "Craw Wurm");
    mana(&mut t, P0, ManaType::U, 7);
    let emerge = CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Emerge);
    assert!(can_cast(&mut t, P0, lib[0], CastMethod::Free));
    assert!(!can_cast(&mut t, P0, lib[0], emerge));
}

#[test]
fn stella_lee_counts_spells_cast_before_she_entered() {
    cr!("601.2i", "603.2");
    supported("Stella Lee, Wild Card");
    ruling!(
        "Stella Lee, Wild Card",
        "Spells that were cast before Stella Lee entered the battlefield count. If Stella Lee was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    // Stella Lee cast as the first spell: the next spell is the second (trigger).
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Island", "Wastes"]);
    let s = t.hand(P0, "Stella Lee, Wild Card");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 2);
    t.cast(P0, s).go();
    t.resolve_all();
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    // Two spells before she's put onto the battlefield: a third spell triggers nothing,
    // and her copy ability may be activated.
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Island", "Wastes"]);
    for _ in 0..2 {
        let o = t.hand(P0, "Ornithopter");
        t.cast(P0, o).go();
        t.resolve_all();
    }
    let stella = t.battlefield(P0, "Stella Lee, Wild Card");
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_ne!(t.zone(lib[0]), Zone::Exile);
    assert!(can_activate(&mut t, P0, stella));
}

/// P0 has cast two Ornithopters this turn, with Stella Lee on the battlefield (her
/// second-spell trigger resolved). Returns Stella Lee.
fn stella_after_two_spells(t: &mut TestGame) -> ObjectId {
    let s = t.battlefield(P0, "Stella Lee, Wild Card");
    for _ in 0..2 {
        let o = t.hand(P0, "Ornithopter");
        t.cast(P0, o).go();
        t.resolve_all();
    }
    s
}

#[test]
fn stella_lee_copies_keep_x_modes_and_targets_and_arent_cast() {
    cr!("707.10", "707.10c", "707.12", "115.7");
    supported("Stella Lee, Wild Card");
    ruling!(
        "Stella Lee, Wild Card",
        "If a spell that’s copied has an X whose value was determined as it was cast, the copy will have the same value of X."
    );
    // Earthquake with X = 1, copied: each player takes 1 from each.
    let mut t = TestGame::new(2);
    let s = stella_after_two_spells(&mut t);
    let eq = t.hand(P0, "Earthquake");
    mana(&mut t, P0, ManaType::R, 2);
    let spell = t.cast(P0, eq).x(1).go();
    act(&mut t, P0, s, "Copy target", &[obj(spell)]).expect("copy");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (18, 18));

    ruling!(
        "Stella Lee, Wild Card",
        "If the copied spell is modal (that is, it says “Choose one —” or the like), the copy will have the same mode or modes. You can’t choose different ones."
    );
    ruling!(
        "Stella Lee, Wild Card",
        "The copy created by Stella Lee’s last ability will have the same targets as the spell it’s copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    // Izzet Charm, second mode (2 damage to target creature) at the Wurm: the copy has
    // the same mode, retargeted to the Bears.
    let mut t = TestGame::new(2);
    let s = stella_after_two_spells(&mut t);
    let w = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Grizzly Bears");
    let charm = t.hand(P0, "Izzet Charm");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 1);
    let spell = t.cast(P0, charm).modes(&[1]).target(w).go();
    act(&mut t, P0, s, "Copy target", &[obj(spell)]).expect("copy");
    change_copy_targets(&mut t, P0, &[Some(obj(b))]);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    let modes = |id: ObjectId| -> Vec<Option<usize>> {
        t.g.obj(id)
            .stack
            .as_ref()
            .unwrap()
            .chosen
            .iter()
            .map(|m| m.mode)
            .collect()
    };
    assert_eq!(modes(copies[0]), modes(spell));
    assert_eq!(modes(spell), vec![Some(1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(b));
    assert_eq!(t.obj_now(w).damage, 2);
    // Shock at P1, copy keeping its target: 4 damage to P1.
    let mut t = TestGame::new(2);
    let s = stella_after_two_spells(&mut t);
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    let spell = t.cast(P0, shock).target(P1).go();
    keep_copy_targets(&mut t, P0);

    ruling!(
        "Stella Lee, Wild Card",
        "The copy made by Stella Lee’s last ability is created on the stack, so it’s not “cast.”"
    );
    let cast_before =
        t.g.history
            .spells_cast
            .iter()
            .filter(|(p, _)| *p == P0)
            .count();
    act(&mut t, P0, s, "Copy target", &[obj(spell)]).expect("copy");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(
        t.g.history
            .spells_cast
            .iter()
            .filter(|(p, _)| *p == P0)
            .count(),
        cast_before
    );
}
