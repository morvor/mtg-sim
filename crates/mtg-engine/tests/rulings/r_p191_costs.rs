//! Rulings batch P191 — costs: monocolored hybrid symbols paid a different way each (CR
//! 107.4e, 601.2f), cost reductions that only reduce generic mana (CR 118.7, 601.2f),
//! mana cost and mana value unchanged by them (CR 202.3), {X} in a cost (CR 107.3),
//! casting without paying the mana cost (CR 118.9), costs of activated abilities (CR
//! 602.2), and costs to attack (CR 508.1d, 508.1h).

use crate::r_p003_common::kill;
use crate::r_p076_common::mana;
use crate::r_p191_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported, with_subtype};
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Answers the next "How will you pay {..}?" choice (1 = the colored half, 2 = the
/// generic half).
fn pay_way(t: &mut TestGame, p: PlayerId, how: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(how));
}

// ---------------------------------------------------------------------------------------
// Monocolored hybrid: Advice from the Fae, Beseech the Queen, Tower Above
// ---------------------------------------------------------------------------------------

#[test]
fn monocolored_hybrid_symbols_are_each_paid_their_own_way() {
    cr!("107.4e", "601.2f", "601.2h");
    ruling!(
        "Advice from the Fae",
        "If a cost includes more than one monocolored hybrid mana symbol, you can choose a different way to pay for each symbol. For example, you can pay for Advice from the Fae by spending {U}{U}{U}, {2}{U}{U}, {4}{U}, or {6}."
    );
    ruling!(
        "Beseech the Queen",
        "If a cost includes more than one monocolored hybrid mana symbol, you can choose a different way to pay for each symbol. For example, you can pay for Beseech the Queen by spending {B}{B}{B}, {2}{B}{B}, {4}{B}, or {6}."
    );
    ruling!(
        "Tower Above",
        "If a cost includes more than one monocolored hybrid mana symbol, you can choose a different way to pay for each symbol. For example, you can pay for Tower Above by spending {G}{G}{G}, {2}{G}{G}, {4}{G}, or {6}."
    );
    for (name, color) in [
        ("Advice from the Fae", ManaType::U),
        ("Beseech the Queen", ManaType::B),
        ("Tower Above", ManaType::G),
    ] {
        supported(name);
        // `generic` of the three symbols paid as {2}.
        for generic in 0..=3u32 {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            let c = t.hand(P0, name);
            mana(&mut t, P0, color, 3 - generic);
            mana(&mut t, P0, ManaType::C, 2 * generic);
            for i in 0..3 {
                pay_way(&mut t, P0, if i < generic { 2 } else { 1 });
            }
            if name == "Tower Above" {
                t.cast(P0, c).target(bears).go();
            } else {
                t.cast(P0, c).go();
            }
            assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{name}: {generic}");
            assert_eq!(t.stack_len(), 1, "{name}: {generic}");
        }
        // One mana short of the way chosen: it can't be cast.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let c = t.hand(P0, name);
        mana(&mut t, P0, color, 1);
        mana(&mut t, P0, ManaType::C, 3);
        for how in [2, 2, 1] {
            pay_way(&mut t, P0, how);
        }
        let r = if name == "Tower Above" {
            t.cast(P0, c).target(bears).try_go()
        } else {
            t.cast(P0, c).try_go()
        };
        assert!(r.is_err(), "{name}");
    }
}

#[test]
fn generic_reductions_apply_to_hybrid_only_if_paying_generic() {
    cr!("601.2f", "107.4e", "118.7");
    ruling!(
        "Beseech the Queen",
        "If an effect reduces the cost to cast a spell by an amount of generic mana, it applies to a monocolored hybrid spell only if you’ve chosen a method of paying for it that includes generic mana."
    );
    supported("Baral, Chief of Compliance");
    // The {1} reduction applies to a {2/B} paid with generic mana: {1}{B}{B} for black,
    // black and one other mana. It never reduces a {2/B} paid with {B}: two black mana
    // alone can't pay.
    // (Baral, black, colorless, castable)
    for (baral, black, colorless, ok) in [
        (true, 2, 1, true),
        (true, 3, 0, true),
        (true, 2, 0, false),
        (false, 2, 1, false),
    ] {
        let mut t = TestGame::new(2);
        if baral {
            t.battlefield(P0, "Baral, Chief of Compliance");
        }
        let c = t.hand(P0, "Beseech the Queen");
        mana(&mut t, P0, ManaType::B, black);
        mana(&mut t, P0, ManaType::C, colorless);
        let case = (baral, black, colorless);
        assert_eq!(t.cast(P0, c).try_go().is_ok(), ok, "{case:?}");
        if ok {
            assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{case:?}");
        }
    }
}

// ---------------------------------------------------------------------------------------
// Cost reductions: Blinkmoth Infusion, Khalni Hydra, Maelstrom Muse, Blood for the Blood
// God!, Emrakul, the Promised End
// ---------------------------------------------------------------------------------------

#[test]
fn blinkmoth_infusion_without_artifacts_costs_full_price() {
    cr!("702.41a", "601.2f");
    ruling!(
        "Blinkmoth Infusion",
        "If you control no artifacts, Blinkmoth Infusion costs twelve generic mana and two blue mana to cast."
    );
    for (generic, ok) in [(11, false), (12, true)] {
        let mut t = TestGame::new(2);
        let c = t.hand(P0, "Blinkmoth Infusion");
        mana(&mut t, P0, ManaType::U, 2);
        mana(&mut t, P0, ManaType::C, generic);
        assert_eq!(t.cast(P0, c).try_go().is_ok(), ok, "{generic}");
    }
}

#[test]
fn khalni_hydra_reduction_doesnt_change_its_mana_cost_or_value() {
    cr!("202.3", "601.2f", "118.7");
    ruling!(
        "Khalni Hydra",
        "Khalni Hydra's cost reduction effect doesn't change its mana cost or mana value."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Llanowar Elves", 5);
    let hydra = t.hand(P0, "Khalni Hydra");
    mana(&mut t, P0, ManaType::G, 3);
    let spell = t.cast(P0, hydra).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert_eq!(t.g.mana_value_of(spell), 8);
    assert_eq!(
        t.obj(spell).chars.mana_cost,
        mtg_engine::mana::ManaCost::parse("{G}{G}{G}{G}{G}{G}{G}{G}")
    );
    t.resolve_all();
    assert_eq!(t.g.mana_value_of(t.g.current(hydra)), 8);
}

#[test]
fn maelstrom_muse_reduces_only_generic_mana() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Maelstrom Muse",
        "Maelstrom Muse’s cost reduction can reduce only generic mana in a spell’s total cost. It can’t reduce colored mana requirements."
    );
    supported("Maelstrom Muse");
    // (spell, its target, the mana available, castable)
    for (spell, mana_ty, ok) in [
        ("Lightning Bolt", None, false),
        ("Divination", Some(ManaType::U), true),
    ] {
        let mut t = TestGame::new(2);
        let muse = t.battlefield(P0, "Maelstrom Muse");
        attack_with(&mut t, &[(muse, pl(P1))]);
        t.resolve_all();
        t.advance_to(P0, Step::PostcombatMain);
        let c = t.hand(P0, spell);
        if let Some(ty) = mana_ty {
            mana(&mut t, P0, ty, 1);
        }
        let r = if spell == "Lightning Bolt" {
            t.cast(P0, c).target(pl(P1)).try_go()
        } else {
            t.cast(P0, c).try_go()
        };
        assert_eq!(r.is_ok(), ok, "{spell}");
    }
}

#[test]
fn blood_for_the_blood_god_still_costs_its_colored_mana() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Blood for the Blood God!",
        "The ability that reduces the mana cost only reduces its generic mana cost. It will cost at least {B}{B}{R}, even if more than eight creatures have died this turn."
    );
    supported("Blood for the Blood God!");
    for (pool, ok) in [
        (vec![(ManaType::B, 2), (ManaType::R, 1)], true),
        (vec![(ManaType::B, 2), (ManaType::C, 1)], false),
        (vec![(ManaType::B, 1), (ManaType::R, 1)], false),
    ] {
        let mut t = TestGame::new(2);
        for _ in 0..10 {
            let b = t.battlefield(P1, "Grizzly Bears");
            kill(&mut t, b);
        }
        let c = t.hand(P0, "Blood for the Blood God!");
        for (ty, n) in &pool {
            mana(&mut t, P0, *ty, *n);
        }
        assert_eq!(t.cast(P0, c).try_go().is_ok(), ok, "{pool:?}");
        if ok {
            t.resolve_all();
            assert_eq!(t.life(P1), 12);
        }
    }
}

#[test]
fn emrakul_counts_at_most_nine_card_types() {
    cr!("205.2a", "601.2f", "205.4a", "205.3a");
    ruling!(
        "Emrakul, the Promised End",
        "The card types that could appear in your graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Supertypes (such as legendary and basic) and subtypes (such as Human and Equipment) are not counted. The maximum discount that Emrakul’s own ability can provide is {9}."
    );
    supported("Emrakul, the Promised End");
    for (generic, ok) in [(3, false), (4, true)] {
        let mut t = TestGame::new(2);
        // Kindred enchantment, battle, artifact creature, basic land, instant, sorcery,
        // planeswalker; plus a legendary creature and a second Faerie (nothing new).
        for name in [
            "Bitterblossom",
            "Invasion of Alara",
            "Ornithopter",
            "Forest",
            "Shock",
            "Divination",
            "Jace Beleren",
            "Shivan Dragon",
            "Bitterblossom",
        ] {
            t.graveyard(P0, name);
        }
        let c = t.hand(P0, "Emrakul, the Promised End");
        mana(&mut t, P0, ManaType::C, generic);
        t.answer_targets(P0, &[pl(P1)]);
        assert_eq!(t.cast(P0, c).try_go().is_ok(), ok, "{generic}");
    }
}

// ---------------------------------------------------------------------------------------
// X: Crackle with Power, Rocco, Cabaretti Caterer
// ---------------------------------------------------------------------------------------

#[test]
fn crackle_with_power_costs_three_per_x_and_hits_x_targets() {
    cr!("107.3", "107.3a", "601.2b", "601.2f", "115.1d");
    ruling!(
        "Crackle with Power",
        "If you choose 1 for the value of X, Crackle with Power will cost {3}{R}{R} to cast and deal five damage to a single target. If you choose 2 for the value of X, it will cost {6}{R}{R} to cast and deal ten damage to each of up to two targets, and so on."
    );
    supported("Crackle with Power");
    for x in [1u32, 2] {
        // One generic short: it can't be cast.
        let mut t = TestGame::new(2);
        let c = t.hand(P0, "Crackle with Power");
        mana(&mut t, P0, ManaType::R, 2);
        mana(&mut t, P0, ManaType::C, 3 * x - 1);
        t.answer_targets(P0, &[pl(P1)]);
        assert!(t.cast(P0, c).x(x as i64).try_go().is_err(), "{x}");
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let c = t.hand(P0, "Crackle with Power");
        mana(&mut t, P0, ManaType::R, 2);
        mana(&mut t, P0, ManaType::C, 3 * x);
        let targets: Vec<Entity> = [pl(P1), obj(giant)][..x as usize].to_vec();
        t.answer_targets(P0, &targets);
        t.cast(P0, c).x(x as i64).go();
        assert_eq!(t.g.player(P0).mana_pool.total(), 0);
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - 5 * x as i32, "{x}");
        assert_eq!(t.on_battlefield(giant), x == 1, "{x}");
    }
    // X = 1 can't have two targets.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let c = t.hand(P0, "Crackle with Power");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::C, 3);
    t.answer_targets(P0, &[pl(P1), obj(giant)]);
    if t.cast(P0, c).x(1).try_go().is_ok() {
        let spell = t.g.stack[0];
        assert!(targets_of(&t, spell).len() <= 1);
    }
}

#[test]
fn rocco_cast_without_paying_its_mana_cost_has_x_zero() {
    cr!("107.3b", "118.9a", "601.2b");
    ruling!(
        "Rocco, Cabaretti Caterer",
        "If an effect allows you to cast Rocco without paying its mana cost, X is 0."
    );
    supported("Rocco, Cabaretti Caterer");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Memnite"]);
    t.battlefield(P0, "Omniscience");
    let rocco = t.hand(P0, "Rocco, Cabaretti Caterer");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[1])]);
    t.cast(P0, rocco).method(CastMethod::Free).go();
    t.resolve_all();
    assert!(t.on_battlefield(rocco));
    // Only the mana value 0 creature card could be found.
    let choices = entity_choices(&t, from);
    assert!(!choices.is_empty());
    for c in &choices {
        assert!(!c.contains(&obj(lib[0])));
    }
    assert_eq!(t.named_on_battlefield("Memnite").len(), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

// ---------------------------------------------------------------------------------------
// Brilliant Ultimatum: casting without paying the mana cost
// ---------------------------------------------------------------------------------------

/// P0 may cast `card` this turn without paying its mana cost.
fn free(t: &mut TestGame, card: ObjectId) {
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![card],
        mtg_engine::ability::Duration::EndOfTurn,
        true,
        None,
    );
}

#[test]
fn without_paying_its_mana_cost_allows_kicker_but_no_alternative_cost() {
    cr!("118.9a", "118.9d", "601.2b", "702.33d");
    ruling!(
        "Brilliant Ultimatum",
        "If you cast a card “without paying its mana cost,” you can’t pay any alternative costs. You can pay additional costs, such as conspire costs and kicker costs."
    );
    // Kicked Burst Lightning, cast without paying its mana cost: the kicker is paid.
    let mut t = TestGame::new(2);
    let burst = t.hand(P0, "Burst Lightning");
    free(&mut t, burst);
    mana(&mut t, P0, ManaType::C, 4);
    t.cast(P0, burst)
        .method(CastMethod::Free)
        .kicked(true)
        .target(pl(P1))
        .go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Fireblast cast without paying its mana cost: no Mountains are sacrificed.
    let mut t = TestGame::new(2);
    let mountains = t.lands(P0, "Mountain", 2);
    let fireblast = t.hand(P0, "Fireblast");
    free(&mut t, fireblast);
    let from = t.asked().len();
    t.cast(P0, fireblast)
        .method(CastMethod::Free)
        .target(pl(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    for m in mountains {
        assert!(t.on_battlefield(m));
    }
    assert!(t.asked()[from..].iter().all(|(_, d)| !matches!(
        d,
        mtg_engine::decision::Decision::ChooseCastingMethod { .. }
    )));
}

// ---------------------------------------------------------------------------------------
// Eris, Roar of the Storm
// ---------------------------------------------------------------------------------------

#[test]
fn eris_counts_spells_cast_before_it_entered() {
    cr!("603.2", "700.14");
    ruling!(
        "Eris, Roar of the Storm",
        "Spells that were cast before Eris entered the battlefield count. If Eris was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    supported("Eris, Roar of the Storm");
    // Eris is the first spell: the next one is the second.
    let mut t = TestGame::new(2);
    let eris = t.hand(P0, "Eris, Roar of the Storm");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 8);
    t.cast(P0, eris).go();
    t.resolve_all();
    assert!(t.on_battlefield(eris));
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, shock).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Dragon").len(), 1);
    // A spell cast before Eris entered (put onto the battlefield): the next is the second.
    let mut t = TestGame::new(2);
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, shock).target(pl(P1)).go();
    t.resolve_all();
    t.battlefield(P0, "Eris, Roar of the Storm");
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, shock).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Dragon").len(), 1);
    // A third spell: no more.
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, shock).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Dragon").len(), 1);
}

// ---------------------------------------------------------------------------------------
// Deity of Scars, Karador, Klauth's Will
// ---------------------------------------------------------------------------------------

#[test]
fn deity_of_scars_needs_a_counter_to_remove() {
    cr!("602.2b", "118.3", "601.2h");
    ruling!(
        "Deity of Scars",
        "Removing a -1/-1 counter from Deity of Scars is a cost of activating its activated ability. You can't activate the ability if there are no -1/-1 counters on Deity of Scars."
    );
    let mut t = TestGame::new(2);
    let deity = t.enter(P0, "Deity of Scars");
    assert_eq!(t.counters(deity, "-1/-1"), 2);
    for left in [1, 0] {
        mana(&mut t, P0, ManaType::B, 1);
        t.activate(P0, deity, 0, &[]).unwrap();
        // Paid as the ability is activated, before it resolves.
        assert_eq!(t.counters(deity, "-1/-1"), left);
        t.resolve_all();
    }
    mana(&mut t, P0, ManaType::B, 1);
    assert!(t.activate(P0, deity, 0, &[]).is_err());
}

#[test]
fn karador_creature_dying_in_your_main_phase_can_be_cast_first() {
    cr!("117.3b", "601.3");
    ruling!(
        "Karador, Ghost Chieftain",
        "If a creature card is put into your graveyard during your main phase and the stack is empty, you have a chance to cast it before any player may attempt to remove that card from your graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karador, Ghost Chieftain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, bolt).target(bears).go();
    t.resolve();
    let card = t.g.current(bears);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    assert!(t.g.stack.is_empty());
    // P0 receives priority first and may cast it from the graveyard.
    assert_eq!(t.g.turn.priority, Some(P0));
    t.lands(P0, "Forest", 2);
    assert!(!legal_cast_methods(&mut t, P0, card).is_empty());
}

#[test]
fn klauths_will_no_one_acts_while_it_is_being_cast() {
    cr!("601.2", "117.1");
    ruling!(
        "Klauth's Will",
        "Once you've announced that you're casting Klauth's Will, players can't take any actions until you've finished doing so. Notably, opponents can't try to remove your commander to change how many modes you may choose."
    );
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: mtg_engine::game::Variant::Commander,
            ..Default::default()
        },
    );
    let dragon = t.battlefield(P0, "Shivan Dragon");
    t.g.objects[dragon.0 as usize].is_commander = true;
    t.g.players[P0.idx()]
        .commander_names
        .push("Shivan Dragon".into());
    t.g.dirty = true;
    t.g.recompute();
    t.battlefield(P1, "Grizzly Bears");
    let relic = t.battlefield(P1, "Sol Ring");
    let card = t.hand(P0, "Klauth's Will");
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    t.answer_targets(P0, &[obj(relic)]);
    let from = t.asked().len();
    t.cast(P0, card).x(2).go();
    // No player was given priority (or any other decision) while it was cast.
    assert!(t.asked()[from..]
        .iter()
        .all(|(p, d)| *p == P0 && !matches!(d, mtg_engine::decision::Decision::Priority { .. })));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(!t.on_battlefield(relic));
}

// ---------------------------------------------------------------------------------------
// Norn's Annex
// ---------------------------------------------------------------------------------------

/// P1's attack declaration in P1's combat: `attackers` attack P0 (or what they name).
fn p1_attacks(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(t, attackers);
}

fn attackers_now(t: &TestGame) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| c.attackers.iter().map(|a| a.id).collect())
        .unwrap_or_default()
}

#[test]
fn norns_annex_lets_creatures_that_must_attack_stay_home() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Norn's Annex",
        "If you control Norn's Annex, your opponents can choose not to attack with a creature with an ability that says it must attack."
    );
    supported("Bloodrock Cyclops");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Norn's Annex");
    t.battlefield(P1, "Bloodrock Cyclops");
    t.set_step(P1, Step::BeginningOfCombat);
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn norns_annex_each_attacker_and_each_annex_is_its_own_cost() {
    cr!("508.1d", "508.1h", "508.1i", "107.4f");
    ruling!(
        "Norn's Annex",
        "Multiple Norn's Annexes controlled by the same player will each impose a cost to attack. Players choose how they pay each cost individually. For example, if a creature you control is attacking a player who controls two Norn's Annexes, you may pay {W} for one cost and 2 life for the other."
    );
    ruling!(
        "Norn's Annex",
        "The controller of each creature attacking you or a planeswalker you control pays either {W} or 2 life as attackers are being declared."
    );
    // Two Annexes, one attacker: {W} for one and 2 life for the other.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Norn's Annex");
    t.battlefield(P0, "Norn's Annex");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let plains = t.lands(P1, "Plains", 1)[0];
    p1_attacks(&mut t, &[(bears, pl(P0))]);
    assert_eq!(attackers_now(&t), vec![bears]);
    assert!(t.obj(plains).tapped);
    assert_eq!(t.life(P1), 18);
    // One Annex, two attackers (one at P0's planeswalker): two costs, paid with life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Norn's Annex");
    let jace = t.battlefield(P0, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    p1_attacks(&mut t, &[(bears, pl(P0)), (giant, obj(jace))]);
    assert_eq!(attackers_now(&t).len(), 2);
    assert_eq!(t.life(P1), 16);
    // Three players: attacking P2 (no Annex) costs nothing.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Norn's Annex");
    let bears = t.battlefield(P1, "Grizzly Bears");
    p1_attacks(&mut t, &[(bears, pl(P2))]);
    assert_eq!(attackers_now(&t), vec![bears]);
    assert_eq!(t.life(P1), 20);
}
