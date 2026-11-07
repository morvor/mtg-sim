//! Rulings batch P063 — costs and mana: the order of cost increases and reductions and
//! mana value (CR 601.2f, 202.3), hybrid activation costs (CR 107.4e), "full refund"
//! cards that give mana back (Apex of Power, Rewind, sacrifice-for-mana creatures), and
//! counting permanents by type.

use crate::r_p046_common::{cost_to_cast, pool as pool_size, tap_lands};
use crate::r_p062_common::{is_tapped, tap};
use crate::r_p063_common::*;
use crate::r_s01_common::{stack_library, supported, tokens};
use crate::r_s05_common::{enter, move_to};
use crate::r_s08_common::mana_value;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::PLUS1;
use mtg_engine::*;

#[test]
fn icebreaker_kraken_cost_increases_apply_before_its_reduction() {
    cr!("601.2f", "202.3", "702.41a");
    ruling!(
        "Icebreaker Kraken",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Icebreaker Kraken). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Icebreaker Kraken");
    // {10}{U}{U}, eleven snow lands, Sphere of Resistance (+{1}): {11}{U}{U} reduced by
    // eleven is {U}{U}. (Reducing first would leave {1}{U}{U}.)
    let mut t = TestGame::new(2);
    t.lands(P0, "Snow-Covered Island", 11);
    t.battlefield(P1, "Sphere of Resistance");
    let kraken = t.hand(P0, "Icebreaker Kraken");
    assert_eq!(cost_to_cast(&mut t, P0, kraken, &[]), Some(2));
    assert_eq!(mana_value(&t, kraken), 12, "the mana value of the spell");
    // Ten snow lands: {11}{U}{U} reduced by ten is {1}{U}{U}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Snow-Covered Island", 10);
    t.battlefield(P1, "Sphere of Resistance");
    let kraken = t.hand(P0, "Icebreaker Kraken");
    assert_eq!(cost_to_cast(&mut t, P0, kraken, &[]), Some(3));
}

#[test]
fn boreal_elemental_taxes_a_spell_only_once_however_many_times_it_targets_it() {
    cr!("601.2f", "115.3");
    ruling!(
        "Boreal Elemental",
        "An opponent’s spell that targets Boreal Elemental more than once costs only {2} more to cast."
    );
    supported("Boreal Elemental");
    // Seeds of Strength ({G}{W}): three "target creature" words, all aimed at it.
    let mut t = TestGame::new(2);
    let boreal = t.battlefield(P0, "Boreal Elemental");
    t.set_step(P1, Step::PrecombatMain);
    let seeds = t.hand(P1, "Seeds of Strength");
    let b = obj(boreal);
    t.answer_targets(P1, &[b]);
    t.answer_targets(P1, &[b]);
    assert_eq!(cost_to_cast(&mut t, P1, seeds, &[b]), Some(4));
    t.resolve_all();
    assert_eq!(t.pt(boreal), (6, 7), "it targeted the Elemental three times");
    // Control: aimed at another creature, it costs {G}{W}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boreal Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let seeds = t.hand(P1, "Seeds of Strength");
    let o = obj(bears);
    t.answer_targets(P1, &[o]);
    t.answer_targets(P1, &[o]);
    assert_eq!(cost_to_cast(&mut t, P1, seeds, &[o]), Some(2));
}

#[test]
fn boreal_elemental_increase_applies_before_reductions() {
    cr!("601.2f", "202.3");
    ruling!(
        "Boreal Elemental",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying, add any cost increases (such as that of Boreal Elemental), then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    // P1's Shock ({R}) with Goblin Electromancer ({1} less): {R} + {2} - {1} = {1}{R}.
    // (Reducing first would reduce nothing, leaving {2}{R}.)
    let mut t = TestGame::new(2);
    let boreal = t.battlefield(P0, "Boreal Elemental");
    t.battlefield(P1, "Goblin Electromancer");
    t.set_step(P1, Step::PrecombatMain);
    let shock = t.hand(P1, "Shock");
    assert_eq!(cost_to_cast(&mut t, P1, shock, &[obj(boreal)]), Some(2));
    assert_eq!(mana_value(&t, shock), 1);
}

#[test]
fn graven_cairns_hybrid_cost_is_paid_with_either_color() {
    cr!("107.4e", "602.2b");
    ruling!(
        "Graven Cairns",
        "{(b/r)}, {T}\" is the same as saying \"{B}, {T} or {R}, {T}."
    );
    supported("Graven Cairns");
    for ty in [ManaType::B, ManaType::R] {
        let mut t = TestGame::new(2);
        let cairns = t.battlefield(P0, "Graven Cairns");
        pool(&mut t, P0, ty, 1);
        t.activate(P0, cairns, 1, &[])
            .unwrap_or_else(|e| panic!("{ty:?}: {e:?}"));
        assert!(is_tapped(&t, cairns));
        assert_eq!(pool_size(&t, P0), 2, "{ty:?} paid for two mana");
    }
    // Neither color: it can't be paid.
    let mut t = TestGame::new(2);
    let cairns = t.battlefield(P0, "Graven Cairns");
    pool(&mut t, P0, ManaType::G, 1);
    assert!(t.activate(P0, cairns, 1, &[]).is_err());
}

/// P0 casts Apex of Power from hand (paying from the pool) with `top` on top of the
/// library; it resolves. Returns the exiled cards.
fn apex(t: &mut TestGame, top: &[&str]) -> Vec<ObjectId> {
    supported("Apex of Power");
    let cards = stack_library(t, P0, top);
    pool(t, P0, ManaType::R, 10);
    let apex = t.hand(P0, "Apex of Power");
    t.cast(P0, apex).go();
    t.resolve_all();
    cards
}

#[test]
fn apex_of_power_leaves_the_cards_not_cast_in_exile() {
    cr!("610.3", "400.7");
    ruling!(
        "Apex of Power",
        "Any cards not cast, including land cards, remain in exile."
    );
    let mut t = TestGame::new(2);
    let cards = apex(&mut t, &["Mountain", "Grizzly Bears"]);
    assert_eq!(pool_size(&t, P0), 10, "it added ten mana");
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    // The next turn: still in exile (and no longer castable).
    t.advance_to(P1, Step::Upkeep);
    for c in &cards {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    assert!(t.in_exile("Mountain") && t.in_exile("Grizzly Bears"));
}

#[test]
fn apex_of_power_doesnt_change_when_you_can_cast_the_exiled_cards() {
    cr!("307.1", "601.3");
    ruling!(
        "Apex of Power",
        "Apex of Power doesn't change when you can cast the exiled cards. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    let mut t = TestGame::new(2);
    let cards = apex(&mut t, &["Lava Axe", "Lightning Bolt"]);
    let (axe, bolt) = (cards[0], cards[1]);
    // (Apex's ten mana may be of another color: a Mountain for the red spells.)
    t.lands(P0, "Mountain", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s21_common::castable(&mut t, P0, axe), "a sorcery");
    assert!(crate::r_s21_common::castable(&mut t, P0, bolt), "an instant");
    t.set_step(P0, Step::PostcombatMain);
    assert!(crate::r_s21_common::castable(&mut t, P0, axe));
    let axe = t.g.current(axe);
    t.cast(P0, axe).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn a_copy_of_apex_of_power_wasnt_cast_so_it_adds_no_mana() {
    cr!("707.10", "601.2", "608.2c");
    ruling!(
        "Apex of Power",
        "If an effect copies Apex of Power, the copy wasn't cast at all, so you won't add ten mana."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Grizzly Bears"; 14]);
    pool(&mut t, P0, ManaType::R, 10);
    let apex = t.hand(P0, "Apex of Power");
    let spell = t.cast(P0, apex).go();
    pool(&mut t, P0, ManaType::U, 2);
    let twin = t.hand(P0, "Twincast");
    t.cast(P0, twin).target(obj(spell)).go();
    // Twincast, then the copy resolve.
    t.resolve();
    assert_eq!(t.stack_len(), 2, "the copy is on the stack");
    let exiled = t.g.exile.len();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.exile.len(), exiled + 7, "the copy exiled seven cards");
    assert_eq!(pool_size(&t, P0), 0, "but added no mana");
    // The original was cast from hand.
    t.resolve();
    assert_eq!(pool_size(&t, P0), 10);
}

#[test]
fn blood_pet_cant_be_sacrificed_between_assigning_and_dealing_combat_damage() {
    cr!("510.2", "510.3", "117.3");
    ruling!(
        "Blood Pet",
        "Because combat damage resolves immediately after it is assigned, you cannot sacrifice Blood Pet in between assigning and dealing combat damage. The last chance you have to sacrifice it before combat damage is dealt is during the Declare Blockers step."
    );
    supported("Blood Pet");
    // P1's Blood Pet blocks P0's Bears. The first time anyone gets priority after the
    // declare blockers step, the damage has been dealt: the Pet is already dead.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pet = t.battlefield(P1, "Blood Pet");
    crate::r_s03_common::to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(pet, bears)]);
    assert!(t.on_battlefield(pet), "it can still be sacrificed in this step");
    let from = t.asked().len();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    let priority_in_damage_step = t.asked()[from..].iter().any(|(_, d)| {
        matches!(d, mtg_engine::decision::Decision::Priority { .. })
    });
    assert!(priority_in_damage_step);
    assert!(t.in_graveyard(P1, "Blood Pet"));
    assert_eq!(t.obj_now(bears).damage, 1, "the Pet dealt its damage");
    // Sacrificed in the declare blockers step: it adds {B} and deals no damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pet = t.battlefield(P1, "Blood Pet");
    crate::r_s03_common::to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(pet, bears)]);
    t.activate(P1, pet, 0, &[]).expect("sacrifice it");
    assert!(t.in_graveyard(P1, "Blood Pet"));
    assert_eq!(t.g.players[1].mana_pool.total(), 1);
    finish_combat(&mut t);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.life(P1), 20, "the Bears stay blocked");
}

#[test]
fn dockside_extortionist_counts_an_artifact_enchantment_once() {
    cr!("608.2h", "205.2a");
    ruling!(
        "Dockside Extortionist",
        "If an opponent controls an artifact that's also an enchantment, that permanent is counted only once for Dockside Extortionist's ability."
    );
    supported("Dockside Extortionist");
    let mut t = TestGame::new(2);
    // An enchantment artifact, an artifact, and an enchantment: three permanents.
    t.battlefield(P1, "Bident of Thassa");
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P1, "Rite of Passage");
    // P0's own don't count.
    t.battlefield(P0, "Sphere of Resistance");
    enter(&mut t, P0, "Dockside Extortionist");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 3);
}

/// P1 casts Shock at P0; P0 responds with Rewind (four Islands, tapped to pay it).
fn rewind_a_shock(t: &mut TestGame) -> (ObjectId, ObjectId, Vec<ObjectId>) {
    supported("Rewind");
    let lands = t.lands(P0, "Island", 4);
    pool(t, P1, ManaType::R, 1);
    let shock = t.hand(P1, "Shock");
    let shock = t.cast(P1, shock).target(Entity::Player(P0)).go();
    let rewind = t.hand(P0, "Rewind");
    let rewind = t.cast(P0, rewind).target(obj(shock)).go();
    assert!(lands.iter().all(|l| is_tapped(t, *l)));
    (shock, rewind, lands)
}

#[test]
fn rewind_with_an_illegal_target_doesnt_untap_lands() {
    cr!("608.2b", "701.6a");
    ruling!(
        "Rewind",
        "If the spell is an illegal target when Rewind tries to resolve, perhaps because it was countered by another spell or ability, Rewind won't resolve and none of its effects will happen. You won't untap any lands."
    );
    let mut t = TestGame::new(2);
    let (shock, _, lands) = rewind_a_shock(&mut t);
    // The Shock leaves the stack before Rewind resolves.
    move_to(&mut t, shock, Zone::Graveyard(P1));
    t.answer_choose(P0, &lands.iter().map(|l| obj(*l)).collect::<Vec<_>>());
    t.resolve_all();
    assert!(lands.iter().all(|l| is_tapped(&t, *l)), "no lands untapped");
    assert!(t.in_graveyard(P0, "Rewind"));
}

#[test]
fn rewind_targets_only_the_spell_and_chooses_lands_as_it_resolves() {
    cr!("115.1", "608.2c", "701.26b");
    ruling!(
        "Rewind",
        "Rewind targets only a spell. It doesn't target any lands. The lands are chosen as Rewind resolves."
    );
    let mut t = TestGame::new(2);
    let (_, rewind, lands) = rewind_a_shock(&mut t);
    assert_eq!(
        crate::r_s25_common::targets_of(&t, rewind).len(),
        1,
        "only the spell is targeted"
    );
    // A land that entered (tapped) after Rewind was cast can be chosen.
    let late = t.battlefield(P0, "Forest");
    tap(&mut t, late);
    t.answer_choose(P0, &[obj(late), obj(lands[0]), obj(lands[1]), obj(lands[2])]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "the Shock was countered");
    assert!(!is_tapped(&t, late));
    assert!(!is_tapped(&t, lands[0]) && !is_tapped(&t, lands[1]) && !is_tapped(&t, lands[2]));
    assert!(is_tapped(&t, lands[3]), "only four lands untap");
}

#[test]
fn rewind_untaps_up_to_four_lands_anyone_controls_once_each() {
    cr!("608.2c", "701.26b");
    ruling!(
        "Rewind",
        "You can choose up to four lands, no matter who controls any of them. Those lands untap once. You can't choose one land and have it untap four times, for example."
    );
    let mut t = TestGame::new(2);
    let (_, _, lands) = rewind_a_shock(&mut t);
    let theirs = t.battlefield(P1, "Mountain");
    tap(&mut t, theirs);
    t.answer_choose(P0, &[obj(theirs), obj(lands[0])]);
    t.resolve_all();
    assert!(!is_tapped(&t, theirs), "an opponent's land can be chosen");
    assert!(!is_tapped(&t, lands[0]));
    assert!(lands[1..].iter().all(|l| is_tapped(&t, *l)));
    // Each land is chosen at most once: naming one land four times isn't a legal choice
    // (the default, untapping P0's own lands, is used instead).
    let mut t = TestGame::new(2);
    let (_, _, _lands) = rewind_a_shock(&mut t);
    let theirs = t.battlefield(P1, "Mountain");
    tap(&mut t, theirs);
    t.answer_choose(P0, &[obj(theirs); 4]);
    t.resolve_all();
    assert!(is_tapped(&t, theirs), "it didn't untap four times (or once)");
    // Choosing exactly the four of P0's: each untaps.
    let mut t = TestGame::new(2);
    let (_, _, lands) = rewind_a_shock(&mut t);
    t.answer_choose(P0, &lands.iter().map(|l| obj(*l)).collect::<Vec<_>>());
    t.resolve_all();
    assert!(lands.iter().all(|l| !is_tapped(&t, *l)));
}

#[test]
fn crystalline_crawler_colorless_mana_isnt_a_color() {
    cr!("105.1", "105.2c", "207.2c");
    ruling!(
        "Crystalline Crawler",
        "The colors of mana are white, blue, black, red, and green. Colorless is not a color."
    );
    supported("Crystalline Crawler");
    let mut t = TestGame::new(2);
    tap_lands(&mut t, P0);
    pool(&mut t, P0, ManaType::C, 2);
    pool(&mut t, P0, ManaType::W, 1);
    pool(&mut t, P0, ManaType::U, 1);
    let crawler = t.hand(P0, "Crystalline Crawler");
    t.cast(P0, crawler).go();
    t.resolve_all();
    assert_eq!(pool_size(&t, P0), 0);
    assert_eq!(t.counters(crawler, PLUS1), 2, "white and blue; colorless isn't a color");
}

#[test]
fn wild_cantor_sacrificed_for_mana_cant_also_convoke() {
    cr!("702.51a", "601.2g", "601.2h");
    ruling!(
        "Wild Cantor",
        "While casting a spell with convoke, if you sacrifice Wild Cantor to add mana, Wild Cantor won't be on the battlefield when you pay that spell's costs. It can't be tapped to help cast that spell using convoke."
    );
    supported("Wild Cantor");
    // Stoke the Flames ({2}{R}{R}, convoke) with Wild Cantor and two Mountains: three
    // payments at most, since the Cantor either makes mana or convokes.
    let mut t = TestGame::new(2);
    let cantor = t.battlefield(P0, "Wild Cantor");
    t.lands(P0, "Mountain", 2);
    let stoke = t.hand(P0, "Stoke the Flames");
    assert!(t.cast(P0, stoke).target(Entity::Player(P1)).try_go().is_err());
    assert_eq!(t.life(P1), 20);
    assert!(t.on_battlefield(cantor), "the failed cast was undone");
    // Control: three Mountains and the Cantor are enough.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wild Cantor");
    t.lands(P0, "Mountain", 3);
    let stoke = t.hand(P0, "Stoke the Flames");
    t.cast(P0, stoke).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn an_artifact_sacrificed_for_mana_cant_also_improvise() {
    // The same holds for improvise: the artifacts are tapped after mana abilities are
    // activated, so one sacrificed for mana isn't there to be tapped.
    cr!("702.126a", "601.2g", "601.2h");
    // Reverse Engineer ({3}{U}{U}, improvise) with two Islands and two artifacts that
    // sacrifice themselves for {C}: four payments at most.
    let setup = |n: usize| {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let arts: Vec<ObjectId> = (0..n)
            .map(|_| {
                t.custom(
                    P0,
                    compiled_card("Mana Shard", "Artifact", "Sacrifice this artifact: Add {C}."),
                    Zone::Battlefield,
                )
            })
            .collect();
        t.lands(P0, "Island", 2);
        let spell = t.hand(P0, "Reverse Engineer");
        (t, arts, spell)
    };
    let (mut t, arts, spell) = setup(2);
    assert!(t.cast(P0, spell).try_go().is_err());
    assert!(arts.iter().all(|a| t.on_battlefield(*a)), "the failed cast was undone");
    // Control: a third artifact is enough.
    let (mut t, _, spell) = setup(3);
    let hand = t.hand_size(P0);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
}
