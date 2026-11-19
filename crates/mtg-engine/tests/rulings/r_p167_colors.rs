//! Rulings batch P167 — cards that care about green (and other colors): Xenagos's
//! devotion as it enters (CR 614.12, 700.5) and its type change (CR 613.1d, 506.4),
//! Kruphix's mana (CR 106.4), cost reductions for multicolored spells (CR 601.2f),
//! permission checked as casting begins (CR 601.3), triggers resolving before the spell
//! (CR 603.3), and last known information (CR 608.2h).

use crate::r_p194_common::{assert_total_cost, mana};
use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s04_common::{is_creature, next_upkeep, top_of_stack};
use crate::r_s05_common::move_to;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------------
// Xenagos, God of Revels
// ---------------------------------------------------------------------------------------

/// Permanents giving P0 `n` devotion to red and green (5 or 7), before Xenagos.
fn devotion(t: &mut TestGame, n: u32) {
    // Renata (GG), Kird Ape (R), Llanowar Elves (G), Raging Goblin (R): 5.
    t.battlefield(P0, "Renata, Called to the Hunt");
    for c in ["Kird Ape", "Llanowar Elves", "Raging Goblin"] {
        t.battlefield(P0, c);
    }
    if n >= 7 {
        t.battlefield(P0, "Kird Ape");
        t.battlefield(P0, "Llanowar Elves");
    }
}

#[test]
fn xenagos_devotion_as_it_enters_doesnt_count_its_own_symbols() {
    cr!("614.12", "700.5", "614.1d");
    ruling!(
        "Xenagos, God of Revels",
        "As Xenagos enters the battlefield, your devotion to red and green will determine whether any replacement effects that affect creatures entering the battlefield apply. Because replacement effects are considered before Xenagos is on the battlefield, the mana symbols in its mana cost won't be counted when determining this."
    );
    // Renata: "Each other creature you control enters with an additional +1/+1 counter on
    // it." Devotion 5 before Xenagos (7 with it): not a creature as it enters.
    for (others, counter) in [(5, 0), (7, 1)] {
        let mut t = TestGame::new(2);
        devotion(&mut t, others);
        let x = t.enter(P0, "Xenagos, God of Revels");
        t.settle();
        assert!(
            is_creature(&t, x),
            "{others}: devotion 7 or more on the battlefield"
        );
        assert_eq!(t.counters(x, counters::PLUS1), counter, "{others}");
    }
}

#[test]
fn xenagos_keeps_its_counters_and_is_a_legendary_enchantment_otherwise() {
    cr!("613.1d", "205.1a", "205.4b", "122.1");
    ruling!(
        "Xenagos, God of Revels",
        "Counters put on Xenagos remain on it while it's not a creature, even if they have no effect."
    );
    ruling!(
        "Xenagos, God of Revels",
        "If Xenagos stops being a creature, it loses the type creature and the creature type God. It continues to be a legendary enchantment."
    );
    let mut t = TestGame::new(2);
    devotion(&mut t, 5);
    let x = t.battlefield(P0, "Xenagos, God of Revels");
    assert!(is_creature(&t, x));
    t.g.add_counters(obj(x), counters::PLUS1, 2, None);
    t.g.recompute();
    assert_eq!(t.pt(x), (8, 7));
    let elves = t.named_on_battlefield("Llanowar Elves")[0];
    destroy(&mut t, elves);
    let o = t.obj_now(x);
    assert!(!o.is(CardType::Creature));
    assert!(o.is(CardType::Enchantment));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert!(!o.chars.has_subtype("God"));
    assert_eq!(t.counters(x, counters::PLUS1), 2);
    // A creature again: the counters count again.
    t.battlefield(P0, "Llanowar Elves");
    assert_eq!(t.pt(x), (8, 7));
}

#[test]
fn xenagos_leaves_combat_when_it_stops_being_a_creature() {
    cr!("506.4");
    ruling!(
        "Xenagos, God of Revels",
        "If Xenagos is attacking or blocking and it stops being a creature, it will be removed from combat. It won't rejoin combat if it resumes being a creature later during that combat."
    );
    let mut t = TestGame::new(2);
    devotion(&mut t, 5);
    let x = t.battlefield(P0, "Xenagos, God of Revels");
    attack_with(&mut t, &[(x, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(crate::r_s09_common::attacks_now(&t).len(), 1);
    let elves = t.named_on_battlefield("Llanowar Elves")[0];
    destroy(&mut t, elves);
    assert!(crate::r_s09_common::attacks_now(&t).is_empty());
    t.battlefield(P0, "Llanowar Elves");
    assert!(is_creature(&t, x));
    assert!(crate::r_s09_common::attacks_now(&t).is_empty());
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn xenagos_losing_all_abilities_still_stops_being_a_creature() {
    cr!("613.1d", "613.1f");
    ruling!(
        "Xenagos, God of Revels",
        "If an effect causes Xenagos to lose all abilities, its ability that causes it to stop being a creature still applies if appropriate."
    );
    // Dress Down: "Creatures lose all abilities." With devotion 7, Xenagos is a creature
    // and loses its abilities; at devotion 5 it isn't a creature anyway.
    let mut t = TestGame::new(2);
    devotion(&mut t, 5);
    let x = t.battlefield(P0, "Xenagos, God of Revels");
    t.battlefield(P1, "Dress Down");
    assert!(is_creature(&t, x));
    assert!(!t.obj_now(x).has_keyword(KeywordKind::Indestructible));
    let elves = t.named_on_battlefield("Llanowar Elves")[0];
    destroy(&mut t, elves);
    assert!(!is_creature(&t, x));
}

// ---------------------------------------------------------------------------------------
// Kruphix, God of Horizons
// ---------------------------------------------------------------------------------------

#[test]
fn kruphix_keeps_unspent_mana_as_colorless() {
    cr!("106.4", "500.5", "614.1a");
    ruling!(
        "Kruphix, God of Horizons",
        "As long as Kruphix is on the battlefield, you'll keep unspent mana as steps and phases end (although it will become colorless). This means you can add mana and spend it during a future step, phase, or turn. Once Kruphix leaves the battlefield, you have until the end of the current step or phase to use the mana before it disappears."
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kruphix, God of Horizons");
    mana(&mut t, P0, ManaType::G, 2);
    t.advance_to(P0, Step::BeginningOfCombat);
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.total(), 2);
    assert_eq!(pool.count(ManaType::C), 2);
    // Into the next turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    // Gone: the mana lasts until the end of this step.
    move_to(&mut t, k, Zone::Graveyard(P0));
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

// ---------------------------------------------------------------------------------------
// Cost reductions and permissions
// ---------------------------------------------------------------------------------------

#[test]
fn multicolored_spells_are_reduced_once() {
    cr!("601.2f", "105.2");
    ruling!(
        "Goblin Anarchomancer",
        "A spell that's both red and green costs {1} less to cast."
    );
    ruling!(
        "Sunscape Familiar",
        "If a spell is both green and blue, you pay {1} less, not {2} less."
    );
    // Bloodbraid Elf ({2}{R}{G}) costs {1}{R}{G}.
    assert_total_cost(
        &|t| {
            t.battlefield(P0, "Goblin Anarchomancer");
        },
        P0,
        "Bloodbraid Elf",
        &[(ManaType::R, 1), (ManaType::G, 1)],
        1,
    );
    // Edric, Spymaster of Trest ({1}{G}{U}) costs {G}{U}.
    assert_total_cost(
        &|t| {
            t.battlefield(P0, "Sunscape Familiar");
        },
        P0,
        "Edric, Spymaster of Trest",
        &[(ManaType::G, 1), (ManaType::U, 1)],
        0,
    );
    // And Sunscape Familiar still reduces a mono-green spell (Grizzly Bears: {G}).
    assert_total_cost(
        &|t| {
            t.battlefield(P0, "Sunscape Familiar");
        },
        P0,
        "Grizzly Bears",
        &[(ManaType::G, 1)],
        0,
    );
}

#[test]
fn hungering_yeti_checks_the_permanent_only_as_casting_begins() {
    cr!("601.3", "702.8a");
    ruling!(
        "Hungering Yeti",
        "Hungering Yeti checks if you control a green or blue permanent only as you begin to cast it. Once you begin to cast Hungering Yeti as though it had flash, it doesn't matter what happens to the green or blue permanents you control."
    );
    // On the opponent's turn, the only green permanent is a Wild Cantor ({R/G}), sacrificed
    // for mana to cast it.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let cantor = t.battlefield(P0, "Wild Cantor");
    let yeti = t.hand(P0, "Hungering Yeti");
    mana(&mut t, P0, ManaType::C, 4);
    assert!(can_cast(&mut t, P0, yeti, CastMethod::Normal));
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(3),
    );
    t.cast(P0, yeti).go();
    assert!(
        !t.on_battlefield(cantor),
        "sacrificed for mana while casting"
    );
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hungering Yeti").len(), 1);
    // Without a green or blue permanent, not at instant speed.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let yeti = t.hand(P0, "Hungering Yeti");
    mana(&mut t, P0, ManaType::C, 4);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!can_cast(&mut t, P0, yeti, CastMethod::Normal));
}

#[test]
fn lure_of_prey_sees_an_artifact_creature_spell() {
    cr!("601.3", "300.2");
    ruling!(
        "Lure of Prey",
        "Does work for artifact creatures cast by your opponent."
    );
    // During the opponent's turn.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let lure = t.hand(P0, "Lure of Prey");
    let bears = t.hand(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 2);
    assert!(!can_cast(&mut t, P0, lure, CastMethod::Normal));
    let thopter = t.hand(P1, "Ornithopter");
    t.cast(P1, thopter).go();
    t.resolve_all();
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 2);
    assert!(can_cast(&mut t, P0, lure, CastMethod::Normal));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, lure).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn march_of_burgeoning_life_x_and_exiled_cards() {
    cr!("601.2b", "601.2f", "107.3a", "118.7");
    ruling!(
        "March of Burgeoning Life",
        "For example, say you wanted to choose a target with mana value 3. You need X to be at least 4. You could choose to cast March of Burgeoning Life by exiling two green cards from your hand and paying {G}, by exiling one green card and paying {2}{G}, or just by paying {4}{G}. If you had a reason to, you could even choose a greater value for X."
    );
    // A target with mana value 3 (Centaur Courser), and another in the library.
    for (exiled, generic) in [(2, 0), (1, 2), (0, 4)] {
        let mut t = TestGame::new(2);
        let courser = t.battlefield(P0, "Centaur Courser");
        t.library_top(P0, "Centaur Courser");
        let greens: Vec<ObjectId> = (0..2).map(|_| t.hand(P0, "Llanowar Elves")).collect();
        let march = t.hand(P0, "March of Burgeoning Life");
        mana(&mut t, P0, ManaType::G, 1);
        mana(&mut t, P0, ManaType::C, generic);
        t.answer(
            P0,
            DecisionKind::OptionalCost,
            mtg_engine::decision::Answer::Number(exiled as i64),
        );
        let from = t.asked().len();
        t.cast(P0, march).x(4).target(obj(courser)).go();
        // X = 4 is among the values offered.
        let offered = t.asked()[from..].iter().find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseX { max, .. } => Some(*max),
            _ => None,
        });
        assert!(offered.is_some_and(|m| m >= 4), "{exiled}: {offered:?}");
        for g in &greens[..exiled] {
            assert_eq!(t.zone(*g), Zone::Exile);
        }
        assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{exiled}");
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Centaur Courser").len(),
            2,
            "{exiled}"
        );
        assert_eq!(t.hand_size(P0), 2 - exiled, "{exiled}");
    }
    // X = 3 isn't enough for a mana value 3 target.
    let mut t = TestGame::new(2);
    let courser = t.battlefield(P0, "Centaur Courser");
    let march = t.hand(P0, "March of Burgeoning Life");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 3);
    assert!(t
        .cast(P0, march)
        .x(3)
        .target(obj(courser))
        .try_go()
        .is_err());
}

#[test]
fn march_of_burgeoning_life_default_x_counts_the_reduction() {
    cr!("601.2b", "601.2f", "107.3a");
    interp!("default-numbers");
    // Two green cards exiled ({4} less) and {G} available: the greatest X the spell can
    // be cast with, the default, is 4 ({4}{G} reduced to {G}); no greater X is offered.
    let mut t = TestGame::new(2);
    let courser = t.battlefield(P0, "Centaur Courser");
    t.library_top(P0, "Centaur Courser");
    for _ in 0..2 {
        t.hand(P0, "Llanowar Elves");
    }
    let march = t.hand(P0, "March of Burgeoning Life");
    mana(&mut t, P0, ManaType::G, 1);
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Number(2),
    );
    let from = t.asked().len();
    let spell = t.cast(P0, march).target(obj(courser)).go();
    let offered = t.asked()[from..].iter().find_map(|(_, d)| match d {
        mtg_engine::decision::Decision::ChooseX { max, .. } => Some(*max),
        _ => None,
    });
    assert_eq!(offered, Some(4));
    assert_eq!(t.g.obj(spell).stack.as_ref().and_then(|s| s.x), Some(4));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Centaur Courser").len(), 2);
}

// ---------------------------------------------------------------------------------------
// Triggers
// ---------------------------------------------------------------------------------------

#[test]
fn bloodhall_ooze_triggers_twice_for_a_black_green_permanent() {
    cr!("603.4", "105.2");
    ruling!(
        "Bloodhall Ooze",
        "A permanent that’s both black and green will allow both abilities to trigger (as will separate black permanents and green permanents, of course)."
    );
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Bloodhall Ooze");
    t.battlefield(P0, "Shambling Shell");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(ooze, counters::PLUS1), 2);
}

#[test]
fn ulasht_counts_a_red_green_creature_twice() {
    cr!("614.1c", "105.2");
    ruling!(
        "Ulasht, the Hate Seed",
        "Creatures that are both red and green provide two +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bloodbraid Elf");
    t.battlefield(P0, "Raging Goblin");
    let u = t.enter(P0, "Ulasht, the Hate Seed");
    assert_eq!(t.counters(u, counters::PLUS1), 3);
}

#[test]
fn murkfiend_liege_untaps_with_the_active_players_permanents() {
    cr!("502.3");
    ruling!(
        "Murkfiend Liege",
        "All your green and/or blue creatures untap during each other player's untap step. You have no choice about what untaps. Those creatures untap at the same time as the active player's permanents."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Murkfiend Liege");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let oracle = t.battlefield(P0, "Coiling Oracle");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    for c in [elves, oracle, goblin, theirs] {
        t.g.tap(c);
    }
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(elves).tapped);
    assert!(!t.obj_now(oracle).tapped);
    assert!(t.obj_now(goblin).tapped, "red");
    assert!(!t.obj_now(theirs).tapped);
    // Untapped in the untap step's one untap action, with the active player's permanent:
    // three untap events in a row, nothing between them.
    let untaps: Vec<usize> =
        t.g.turn_events
            .iter()
            .enumerate()
            .filter_map(|(i, e)| match e {
                mtg_engine::events::Event::Untapped { .. } => Some(i),
                _ => None,
            })
            .collect();
    assert_eq!(untaps.len(), 3);
    assert_eq!(untaps[2] - untaps[0], 2, "{untaps:?}");
}

#[test]
fn wurms_tooth_resolves_before_the_green_spell() {
    cr!("603.3", "405.5");
    ruling!(
        "Wurm's Tooth",
        "If a player casts a green spell, Wurm’s Tooth’s ability triggers and is put on the stack on top of that spell. Wurm’s Tooth’s ability will resolve (causing you to gain 1 life) before the spell does."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wurm's Tooth");
    let bears = t.hand(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    mana(&mut t, P1, ManaType::G, 1);
    mana(&mut t, P1, ManaType::C, 1);
    let spell = t.cast(P1, bears).go();
    t.answer_yes(P0, true);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(top_of_stack(&t), spell);
    t.resolve();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.g.stack.last(), Some(&spell));
}

#[test]
fn green_suns_zenith_countered_goes_to_the_graveyard() {
    cr!("701.6a");
    ruling!(
        "Green Sun's Zenith",
        "If Green Sun's Zenith is countered, none of its effects will happen. Notably, it will be put into its owner's graveyard rather than shuffled into its owner's library."
    );
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    let gsz = t.hand(P0, "Green Sun's Zenith");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let spell = t.cast(P0, gsz).x(2).go();
    let cs = t.hand(P1, "Counterspell");
    mana(&mut t, P1, ManaType::U, 2);
    t.cast(P1, cs).target(obj(spell)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Green Sun's Zenith"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn yorvo_compares_with_the_creatures_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Yorvo, Lord of Garenbrig",
        "If the entering green creature dies before Yorvo's triggered ability resolves, use its power as it last existed on the battlefield to determine whether Yorvo gets a second +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    let yorvo = t.battlefield(P0, "Yorvo, Lord of Garenbrig");
    t.g.add_counters(obj(yorvo), counters::PLUS1, 4, None);
    // Craw Wurm (6/4) enters; it dies before the trigger resolves.
    let wurm = crate::r_s05_common::enter(&mut t, P0, "Craw Wurm");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, wurm);
    t.resolve_all();
    assert_eq!(t.counters(yorvo, counters::PLUS1), 6);
    // A 1/1 doesn't give the second counter.
    let elves = crate::r_s05_common::enter(&mut t, P0, "Llanowar Elves");
    destroy(&mut t, elves);
    t.resolve_all();
    assert_eq!(t.counters(yorvo, counters::PLUS1), 7);
}

#[test]
fn verdant_succession_finds_an_animated_still_life() {
    cr!("603.10a", "700.4");
    ruling!(
        "Verdant Succession",
        "If the creature was only a creature as the result of an animation effect, its controller still gets to search for a card with the same name. For example, if an animated Still Life is put into a graveyard, its controller gets to search their library for a card named Still Life."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Verdant Succession");
    let life = t.battlefield(P0, "Still Life");
    let other = t.library_top(P0, "Still Life");
    mana(&mut t, P0, ManaType::G, 2);
    t.activate(P0, life, 0, &[]).unwrap();
    t.resolve_all();
    assert!(is_creature(&t, life));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(other)]);
    destroy(&mut t, life);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Still Life").len(), 1);
    assert!(t.in_graveyard(P0, "Still Life"));
}

// ---------------------------------------------------------------------------------------
// Color-checking statics
// ---------------------------------------------------------------------------------------

#[test]
fn color_statics_table() {
    cr!("105.2", "611.3a", "613.1e");
    ruling!(
        "Toxic Iguanar",
        "As long as at least one permanent you control is the specified color, the ability will “work” and grant this creature the bonus. Otherwise, it won’t have the bonus."
    );
    ruling!(
        "Juniper Order Advocate",
        "If Juniper Order Advocate somehow becomes green, its ability will affect itself."
    );
    // Toxic Iguanar: deathtouch with a green permanent (any kind), not without.
    for (green, deathtouch) in [
        (None, false),
        (Some("Llanowar Elves"), true),
        (Some("Rancor"), true),
    ] {
        let mut t = TestGame::new(2);
        let iguanar = t.battlefield(P0, "Toxic Iguanar");
        if let Some(g) = green {
            if g == "Rancor" {
                crate::r_s06_common::attach_new(&mut t, P0, g, iguanar);
            } else {
                t.battlefield(P0, g);
            }
        }
        assert_eq!(
            t.obj_now(iguanar).has_keyword(KeywordKind::Deathtouch),
            deathtouch,
            "{green:?}"
        );
    }
    // Juniper Order Advocate made green by Lifelace ("Target spell or permanent becomes
    // green.").
    let mut t = TestGame::new(2);
    let jo = t.battlefield(P0, "Juniper Order Advocate");
    assert_eq!(t.pt(jo), (1, 2));
    let lace = t.hand(P0, "Lifelace");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, lace).target(obj(jo)).go();
    t.resolve_all();
    assert_eq!(t.pt(jo), (2, 3));
}

#[test]
fn magus_of_the_order_made_nongreen_can_still_sacrifice_itself() {
    cr!("602.2b", "118.3");
    ruling!(
        "Magus of the Order",
        "If Magus of the Order becomes nongreen, you can still activate its ability by sacrificing it and a green creature."
    );
    let mut t = TestGame::new(2);
    let magus = t.battlefield(P0, "Magus of the Order");
    let elves = t.battlefield(P0, "Llanowar Elves");
    // Cerulean Wisps: "Target creature becomes blue until end of turn. Untap that
    // creature. Draw a card."
    let wisps = t.hand(P0, "Cerulean Wisps");
    mana(&mut t, P0, ManaType::U, 1);
    t.cast(P0, wisps).target(obj(magus)).go();
    t.resolve_all();
    assert!(!crate::r_s05_common::colors(&t, magus).contains(Color::Green));
    t.library_top(P0, "Craw Wurm");
    mana(&mut t, P0, ManaType::G, 1);
    t.answer_choose(P0, &[obj(elves)]);
    t.activate(P0, magus, 0, &[]).expect("activated");
    assert!(!t.on_battlefield(magus));
    assert!(!t.on_battlefield(elves));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}
