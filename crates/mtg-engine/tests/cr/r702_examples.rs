//! The worked examples of CR 702 (keyword abilities), played out as tests. Examples
//! already set up by a keyword's own tests are cited there instead:
//! - 702.1a (Varolz): `keywords/k702_097_scavenge.rs`;
//! - 702.1b (Volcano Hellion): `rulings/gap_trigger_timing.rs`;
//! - 702.1c (Concerted Effort), 702.14d, 702.15e, the second 702.19b example (protection
//!   from green), 702.26b ("Destroy all creatures"), 702.44c, both 702.65b examples,
//!   702.103d (Garruk's Horde), 702.164b and 702.184c: the `keywords/k702_NNN_*.rs`
//!   test of that keyword.

use crate::r506_common::*;
use crate::r600_examples_common::{custom_costed, supported};
use mtg_engine::decision::{Action, Answer};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Advances to `p`'s next upkeep and puts the upkeep triggers on the stack.
fn next_upkeep(t: &mut TestGame, p: PlayerId) {
    let other = if p == P0 { P1 } else { P0 };
    if t.g.turn.active == p {
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

fn tapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.chars.is_land() && o.tapped
        })
        .count()
}

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, method: &CastMethod) -> bool {
    crate::r600_examples_common::can_cast(t, p, card, method)
}

// --- 702.1b: a split card's granted flashback cost --------------------------------------

#[test]
fn past_in_flames_fire_ice_flashback_costs_the_half_cast() {
    cr!("702.34a", "709.3", "709.4");
    cr_example!("702.1b", "Fire//Ice is a split card");
    supported("Past in Flames");
    supported("Fire // Ice");
    const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
    let mut t = TestGame::new(2);
    let fi = t.graveyard(P0, "Fire // Ice");
    t.lands(P0, "Mountain", 4);
    let pif = t.hand(P0, "Past in Flames");
    t.cast(P0, pif).go();
    t.resolve_all();
    // In the graveyard, Fire // Ice has flashback; its mana cost (and so its flashback
    // cost) is the combined {2}{U}{R}.
    let card = t.g.current(fi);
    let o = t.obj(card);
    assert!(o.has_keyword(KeywordKind::Flashback));
    assert_eq!(o.chars.mana_value(), 4);
    // Cast as Fire, the flashback cost is Fire's {1}{R}: two Mountains are enough.
    for l in t.g.battlefield.clone() {
        if t.obj(l).chars.is_land() {
            t.g.objects[l.0 as usize].tapped = true;
        }
    }
    t.lands(P0, "Mountain", 2);
    assert!(can_cast(&mut t, P0, card, &FLASHBACK));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
    let spell = t
        .cast(P0, card)
        .method(FLASHBACK)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(t.obj(spell).chars.name, "Fire");
    assert_eq!(tapped_lands(&t, P0), 6);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Fire // Ice"));
}

// --- 702.19b, 702.19c: trample ----------------------------------------------------------

#[test]
fn a_blocker_of_two_attackers_counts_damage_from_both_for_trample() {
    cr!("702.19b", "509.1a", "510.1c");
    cr_example!("702.19b", "can block an additional creature blocks two attackers");
    let mut t = TestGame::new(2);
    let plain = bf(&mut t, P0, vanilla("Plain Attacker", 1, 1));
    let tramp = bf(
        &mut t,
        P0,
        custom_card("Trampling Attacker", "Creature — Beast", Some((3, 3)), "Trample"),
    );
    let blocker = bf(
        &mut t,
        P1,
        custom_card(
            "Double Blocker",
            "Creature — Soldier",
            Some((2, 2)),
            "This creature can block an additional creature each combat.",
        ),
    );
    to_combat(&mut t, P0);
    declare(
        &mut t,
        &[(plain, Entity::Player(P1)), (tramp, Entity::Player(P1))],
    );
    block(&mut t, P1, &[(blocker, plain), (blocker, tramp)]);
    // 1 from the first attacker to the blocker; from the trampler, 1 to the blocker
    // (lethal together with the first attacker's 1) and 2 to the defending player.
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1]));
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 2]));
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(blocker), "{}", t.dump_log());
    assert_eq!(t.life(P1), 18);
}

#[test]
fn trample_over_planeswalkers_counts_another_attackers_damage_to_the_planeswalker() {
    cr!("702.19c", "510.1c");
    cr_example!("702.19c", "7/7 with trample over planeswalkers");
    let mut t = TestGame::new(2);
    let mut pw_def = custom_card("Test Walker", "Legendary Planeswalker — Test", None, "");
    pw_def.faces[0].chars.loyalty = Some(3);
    let pw = bf(&mut t, P1, pw_def);
    assert_eq!(t.counters(pw, "loyalty"), 3);
    let small = bf(&mut t, P0, vanilla("Small Attacker", 1, 1));
    let big = bf(
        &mut t,
        P0,
        custom_card(
            "Big Attacker",
            "Creature — Beast",
            Some((7, 7)),
            "Trample over planeswalkers",
        ),
    );
    to_combat(&mut t, P0);
    declare(
        &mut t,
        &[(small, Entity::Object(pw)), (big, Entity::Object(pw))],
    );
    // 1 from the first attacker; 2 from the second to the planeswalker (lethal with the
    // first's 1) and 5 to the defending player.
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 5]));
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(pw), "{}", t.dump_log());
    assert_eq!(t.life(P1), 15);
}

// --- 702.22h: banding -------------------------------------------------------------------

#[test]
fn blocking_the_flier_of_a_band_blocks_its_swampwalker_too() {
    cr!("702.22h", "702.14c", "702.9b");
    cr_example!("702.22h", "a creature with flying and a creature with swampwalk");
    supported("Mesa Pegasus");
    supported("Bog Wraith");
    let mut t = TestGame::new(2);
    let pegasus = t.battlefield(P0, "Mesa Pegasus"); // flying, banding
    let wraith = t.battlefield(P0, "Bog Wraith"); // swampwalk
    t.battlefield(P1, "Swamp");
    let spider = t.battlefield(P1, "Giant Spider"); // reach
    to_combat(&mut t, P0);
    // The Pegasus bands with the Wraith.
    t.answer_choose(P0, &[Entity::Object(wraith)]);
    declare(
        &mut t,
        &[(pegasus, Entity::Player(P1)), (wraith, Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    let combat = t.g.combat.as_ref().unwrap();
    assert!(combat.attacker(pegasus).unwrap().band.is_some());
    assert_eq!(
        combat.attacker(pegasus).unwrap().band,
        combat.attacker(wraith).unwrap().band
    );
    // The Wraith can't be blocked (P1 controls a Swamp); the flying Pegasus can be.
    assert!(!t.g.can_block(spider, wraith));
    assert!(t.g.can_block(spider, pegasus));
    block(&mut t, P1, &[(spider, pegasus)]);
    go_to(&mut t, Step::DeclareBlockers);
    let combat = t.g.combat.as_ref().unwrap();
    assert!(combat.is_blocked(wraith));
    assert_eq!(combat.blockers_of(wraith), vec![spider]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

// --- 702.24a, 702.24b: cumulative upkeep ------------------------------------------------

#[test]
fn cumulative_upkeep_w_or_u_is_paid_with_any_mix_for_each_age_counter() {
    cr!("702.24a");
    cr_example!("702.24a", "Cumulative upkeep {W} or {U}");
    for whites in 0..=3 {
        let mut t = TestGame::new(2);
        let c = bf(
            &mut t,
            P0,
            custom_card(
                "Hybrid Keeper",
                "Creature — Spirit",
                Some((3, 3)),
                "Cumulative upkeep {W} or {U}",
            ),
        );
        t.g.add_counters(Entity::Object(c), "age", 2, None);
        t.lands(P0, "Plains", whites);
        t.lands(P0, "Island", 3 - whites);
        next_upkeep(&mut t, P0);
        t.answer_yes(P0, true);
        t.resolve();
        // An age counter first, then three {W} or {U}: {W}{W}{W}, {W}{W}{U}, {W}{U}{U}
        // or {U}{U}{U}.
        assert_eq!(t.counters(c, "age"), 3);
        assert!(t.on_battlefield(c), "{whites} Plains: {}", t.dump_log());
        assert_eq!(tapped_lands(&t, P0), 3);
    }
    // Two lands can't pay for three age counters.
    let mut t = TestGame::new(2);
    let c = bf(
        &mut t,
        P0,
        custom_card(
            "Hybrid Keeper",
            "Creature — Spirit",
            Some((3, 3)),
            "Cumulative upkeep {W} or {U}",
        ),
    );
    t.g.add_counters(Entity::Object(c), "age", 2, None);
    t.lands(P0, "Plains", 2);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.on_battlefield(c));
}

#[test]
fn cumulative_upkeep_sacrifice_a_creature_needs_different_creatures() {
    cr!("702.24a", "701.21a");
    cr_example!("702.24a", "Cumulative upkeep—Sacrifice a creature");
    let keeper = || {
        custom_card(
            "Hungry Keeper",
            "Creature — Spirit",
            Some((3, 3)),
            "Cumulative upkeep—Sacrifice a creature.",
        )
    };
    // Two other creatures: both are sacrificed and the keeper stays.
    let mut t = TestGame::new(2);
    let c = bf(&mut t, P0, keeper());
    t.g.add_counters(Entity::Object(c), "age", 1, None);
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(b1), Entity::Object(b2)]);
    t.resolve();
    assert_eq!(t.counters(c, "age"), 2);
    assert!(t.on_battlefield(c));
    assert!(!t.on_battlefield(b1) && !t.on_battlefield(b2));
    // One other creature: the same creature can't be sacrificed twice, so paying means
    // sacrificing the keeper itself too. Either way it leaves the battlefield.
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        let c = bf(&mut t, P0, keeper());
        t.g.add_counters(Entity::Object(c), "age", 1, None);
        let b1 = t.battlefield(P0, "Grizzly Bears");
        next_upkeep(&mut t, P0);
        t.answer_yes(P0, pay);
        if pay {
            t.answer_choose(P0, &[Entity::Object(b1), Entity::Object(c)]);
        }
        t.resolve();
        assert!(!t.on_battlefield(c), "{}", t.dump_log());
        assert_eq!(t.on_battlefield(b1), !pay);
        if pay {
            // Two different creatures were to be chosen, from just the two there are.
            let asked = t.asked();
            let (cands, min) = asked
                .iter()
                .rev()
                .find_map(|(_, d)| match d {
                    mtg_engine::decision::Decision::ChooseEntities {
                        candidates, min, ..
                    } => Some((candidates.clone(), *min)),
                    _ => None,
                })
                .expect("the creatures to sacrifice were chosen");
            assert_eq!(min, 2);
            assert_eq!(cands.len(), 2);
        }
    }
    // An answer naming the same creature twice isn't a legal payment: it isn't
    // sacrificed "twice" while the keeper stays.
    let mut t = TestGame::new(2);
    let c = bf(&mut t, P0, keeper());
    t.g.add_counters(Entity::Object(c), "age", 1, None);
    let b1 = t.battlefield(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(b1), Entity::Object(b1)]);
    t.resolve();
    assert!(!t.on_battlefield(c));
}

#[test]
fn two_cumulative_upkeep_pay_1_life_instances_cost_1_then_2() {
    cr!("702.24b", "702.24a");
    cr_example!("702.24b", "two instances of “Cumulative upkeep—Pay 1 life.”");
    let mut t = TestGame::new(2);
    let c = bf(
        &mut t,
        P0,
        custom_card(
            "Twice Keeper",
            "Creature — Spirit",
            Some((3, 3)),
            "Cumulative upkeep—Pay 1 life.\nCumulative upkeep—Pay 1 life.",
        ),
    );
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.counters(c, "age"), 1);
    assert_eq!(t.life(P0), 19);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.counters(c, "age"), 2);
    assert_eq!(t.life(P0), 17);
    assert!(t.on_battlefield(c));
}

// --- 702.26b: phasing -------------------------------------------------------------------

#[test]
fn a_phased_out_creature_isnt_counted_by_draw_a_card_for_each_creature() {
    cr!("702.26b");
    cr_example!("702.26b", "Draw a card for each creature you control");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let gone = t.battlefield(P0, "Grizzly Bears");
    mtg_engine::kw::phasing::phase_out(&mut t.g, vec![gone]);
    t.g.recompute();
    let spell = t.custom(
        P0,
        custom_costed(
            "Count Them",
            "Sorcery",
            "{0}",
            "Draw a card for each creature you control.",
        ),
        Zone::Hand(P0),
    );
    let hand = t.hand_size(P0);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

// --- 702.47a, 702.47c: splice -----------------------------------------------------------

fn splice(t: &mut TestGame, p: PlayerId) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Bool(true));
}

#[test]
fn a_spliced_card_stays_in_hand_and_can_pay_the_spells_discard_cost() {
    cr!("702.47a", "601.2b", "601.2h");
    cr_example!("702.47a", "Since the card with splice remains in the player’s hand");
    supported("Everdream");
    supported("Thrill of Possibility");
    // Everdream: "Draw a card. Splice onto instant or sorcery {2}{U}". Thrill of
    // Possibility: "As an additional cost to cast this spell, discard a card. Draw two
    // cards."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Mountain", 2);
    let thrill = t.hand(P0, "Thrill of Possibility");
    let ever = t.hand(P0, "Everdream");
    // Splice Everdream, then discard it to pay Thrill's cost.
    splice(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(ever)]);
    let spell = t.cast(P0, thrill).go();
    assert!(t.in_graveyard(P0, "Everdream"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    // Thrill's two cards plus Everdream's spliced one.
    assert_eq!(t.hand_size(P0), 3, "{}", t.dump_log());
    let _ = spell;
    // Spliced without being discarded, it stays in hand and can be cast normally later.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    t.lands(P0, "Mountain", 2);
    let thrill = t.hand(P0, "Thrill of Possibility");
    let ever = t.hand(P0, "Everdream");
    let other = t.hand(P0, "Grizzly Bears");
    splice(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.cast(P0, thrill).go();
    assert!(t.in_hand(P0, "Everdream"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1 + 3);
    let hand = t.hand_size(P0);
    t.cast(P0, ever).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Everdream"));
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
}

#[test]
fn glacial_ray_spliced_onto_a_blue_spell_can_damage_a_protection_from_red_creature() {
    cr!("702.47c", "702.16b");
    cr_example!("702.47c", "Suppose Glacial Ray is spliced onto Reach Through Mists");
    supported("Glacial Ray");
    supported("Reach Through Mists");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "Silver Knight"); // 2/2, protection from red
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    let mists = t.hand(P0, "Reach Through Mists");
    t.hand(P0, "Glacial Ray");
    splice(&mut t, P0);
    let spell = t.cast(P0, mists).target(knight).go();
    let o = t.obj_now(spell);
    assert_eq!(o.chars.name, "Reach Through Mists");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Silver Knight"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_hand(P0, "Glacial Ray"));
}

// --- 702.51b: convoke -------------------------------------------------------------------

#[test]
fn heartless_summoning_reduces_siege_wurm_before_convoke() {
    cr!("702.51b", "702.51a", "601.2f", "601.2h");
    cr_example!("702.51b", "Heartless Summoning says, in part");
    supported("Heartless Summoning");
    supported("Siege Wurm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heartless Summoning");
    // {3}{G}{G}: two green creatures and three others pay it all.
    let greens: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let others: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Memnite")).collect();
    let extra = t.battlefield(P0, "Memnite");
    let all: Vec<Entity> = greens
        .iter()
        .chain(others.iter())
        .map(|c| Entity::Object(*c))
        .collect();
    t.answer_choose(P0, &all);
    let wurm = t.hand(P0, "Siege Wurm");
    t.cast(P0, wurm).go();
    for c in greens.iter().chain(others.iter()) {
        assert!(t.obj_now(*c).tapped);
    }
    assert!(!t.obj_now(extra).tapped);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Siege Wurm").len(), 1);
    // Up to two green creatures and up to three others, the rest with mana: two green
    // creatures, one other and three lands (one of them for {G}... no: the green is paid
    // by creatures) pay {3}{G}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heartless Summoning");
    let g1 = t.battlefield(P0, "Grizzly Bears");
    let m1 = t.battlefield(P0, "Memnite");
    t.lands(P0, "Forest", 3);
    t.answer_choose(P0, &[Entity::Object(g1), Entity::Object(m1)]);
    let wurm = t.hand(P0, "Siege Wurm");
    t.cast(P0, wurm).go();
    assert!(t.obj_now(g1).tapped && t.obj_now(m1).tapped);
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Siege Wurm").len(), 1);
}

// --- 702.103d: bestow -------------------------------------------------------------------

#[test]
fn aether_storm_doesnt_stop_casting_a_creature_card_bestowed() {
    cr!("702.103d", "702.103a");
    cr_example!("702.103d", "Aether Storm is an enchantment");
    supported("Aether Storm");
    supported("Nyxborn Rollicker");
    const BESTOW: CastMethod = CastMethod::Keyword(KeywordKind::Bestow);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aether Storm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Nyxborn Rollicker");
    assert!(!can_cast(&mut t, P0, c, &CastMethod::Normal));
    assert!(can_cast(&mut t, P0, c, &BESTOW));
    t.cast(P0, c).method(BESTOW).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
}

// --- 702.177b: exhaust ------------------------------------------------------------------

#[test]
fn loots_exhaust_mana_ability_cant_be_activated_while_activating_elvish_refuelers() {
    cr!("702.177b", "602.2", "605.3a");
    cr_example!("702.177b", "Elvish Refueler has an exhaust ability");
    supported("Elvish Refueler");
    supported("Loot, the Pathfinder");
    let uid = |t: &mut TestGame, src: ObjectId, text: &str| -> u64 {
        t.g.recompute();
        t.obj(src)
            .chars
            .abilities
            .iter()
            .find(|a| a.text.contains(text))
            .map(|a| a.uid)
            .unwrap_or_else(|| panic!("no ability {text}"))
    };
    let activatable = |t: &mut TestGame, src: ObjectId, uid: u64| -> bool {
        t.g.recompute();
        t.g.turn.priority = Some(P0);
        let src = t.g.current(src);
        let a = t
            .obj(src)
            .chars
            .abilities
            .iter()
            .find(|a| a.uid == uid)
            .cloned()
            .expect("ability");
        let mtg_engine::ability::AbilityKind::Activated(act) = &a.kind else {
            panic!("not an activated ability");
        };
        t.g.can_activate(P0, src, &a, act)
    };
    let add = |t: &mut TestGame, ty: ManaType, n: u32| {
        t.g.players[P0.idx()].mana_pool.add_type(ty, n);
    };
    // The Refueler's exhaust ability (and, if `loot_used`, Loot's mana ability) was
    // activated on an earlier turn; it's now P0's next turn, with one Forest.
    let setup = |loot_used: bool| {
        let mut t = TestGame::new(2);
        let elf = t.battlefield(P0, "Elvish Refueler");
        let loot = t.battlefield(P0, "Loot, the Pathfinder");
        let elf_ex = uid(&mut t, elf, "Exhaust — {1}{G}");
        let loot_mana = uid(&mut t, loot, "Add three mana");
        add(&mut t, ManaType::G, 3);
        t.g.turn.priority = Some(P0);
        t.g.activate_ability(P0, elf, elf_ex).unwrap();
        t.resolve_all();
        if loot_used {
            t.g.turn.priority = Some(P0);
            t.g.activate_ability(P0, loot, loot_mana).unwrap();
            t.resolve_all();
        }
        t.advance_to(P1, Step::PrecombatMain);
        t.advance_to(P0, Step::PrecombatMain);
        t.lands(P0, "Forest", 1);
        (t, elf, loot, elf_ex, loot_mana)
    };
    // Control: Loot's mana ability was never activated. Activating the Refueler's
    // exhaust ability again (the Refueler allows it), Loot's mana ability can pay for it.
    let (mut t, elf, loot, elf_ex, _) = setup(false);
    assert!(activatable(&mut t, elf, elf_ex));
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, elf, elf_ex).unwrap();
    assert!(t.obj_now(loot).tapped, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 2);
    // Both were activated before. Each could be activated again by itself...
    let (mut t, elf, loot, elf_ex, loot_mana) = setup(true);
    assert!(activatable(&mut t, loot, loot_mana));
    // ...but Loot's mana ability can't pay for the Refueler's: the Refueler's exhaust
    // ability began to be activated first, so Loot's can't be activated during it.
    assert!(!activatable(&mut t, elf, elf_ex));
    t.g.turn.priority = Some(P0);
    assert!(t.g.activate_ability(P0, elf, elf_ex).is_err());
    assert!(!t.obj_now(loot).tapped);
    assert_eq!(t.counters(elf, "+1/+1"), 1);
    // With a second Forest, the Refueler's ability is paid for without Loot.
    t.lands(P0, "Forest", 1);
    assert!(activatable(&mut t, elf, elf_ex));
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, elf, elf_ex).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 2);
    // Having activated an exhaust ability this turn, Loot's can't be activated now.
    assert!(!activatable(&mut t, loot, loot_mana));
}
