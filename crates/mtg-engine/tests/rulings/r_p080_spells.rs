//! Rulings batch P080 — instants for attacking creatures: when they can be cast and what
//! they find (after combat damage, outside combat), values counted as they resolve
//! (CR 608.2h), choices made by the creature's owner or controller, hexproof and spells
//! that target only a player, divided damage (CR 601.2d), and redirection to a creature
//! removed from combat.

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported, tokens, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s03_common::to_blockers;
use crate::r_s07_common::damage_on;
use crate::r_s10_common::attacking;
use crate::r_s21_common::go_to;
use mtg_engine::combat::remove_from_combat;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's creatures `names` attack P1.
fn p0_attacks(t: &mut TestGame, names: &[&str]) -> Vec<ObjectId> {
    let ids: Vec<ObjectId> = names.iter().map(|n| t.battlefield(P0, n)).collect();
    let decl: Vec<(ObjectId, Entity)> = ids.iter().map(|i| (*i, Entity::Player(P1))).collect();
    attack_with(t, &decl);
    t.resolve_all();
    ids
}

#[test]
fn sacrifice_spells_after_combat_damage_find_only_survivors() {
    cr!("701.21a", "511.1", "506.4");
    ruling!(
        "Entrapment Maneuver",
        "If you cast Entrapment Maneuver after combat damage has been dealt, only creatures that survived combat can be sacrificed."
    );
    ruling!(
        "Wing Shards",
        "If you cast Wing Shards after combat damage has been dealt, only creatures that survived combat can be sacrificed."
    );
    for name in ["Entrapment Maneuver", "Wing Shards"] {
        supported(name);
        // P0's Savannah Lions (blocked by a Hill Giant) and Grizzly Bears attack; after
        // combat damage only the Bears remain, and P0 must sacrifice them.
        let mut t = TestGame::new(2);
        let lions = t.battlefield(P0, "Savannah Lions");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        to_blockers(
            &mut t,
            &[(lions, Entity::Player(P1)), (bears, Entity::Player(P1))],
            &[(giant, lions)],
        );
        go_to(&mut t, Step::EndOfCombat);
        assert!(!t.on_battlefield(lions));
        let from = t.asked().len();
        cast_card(&mut t, P1, name, &[Entity::Player(P0)]).unwrap();
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "{name}");
        // P0 had a single choice: nothing about the dead Lions was offered.
        for (_, d) in &t.asked()[from..] {
            if let Decision::ChooseEntities { candidates, .. } = d {
                assert!(!candidates.contains(&obj(lions)), "{name}");
            }
        }
        if name == "Entrapment Maneuver" {
            assert_eq!(tokens(&t, P1).len(), 2, "the Bears' toughness");
        }
    }
}

#[test]
fn devouring_light_can_exile_an_attacker_after_it_dealt_damage() {
    cr!("511.3", "506.4", "510.2");
    ruling!(
        "Devouring Light",
        "The declare blockers step is the last chance to cast Devouring Light before creatures deal their combat damage. However, a creature remains an attacking or blocking creature during the combat damage step and end of combat step."
    );
    supported("Devouring Light");
    for step in [Step::CombatDamage, Step::EndOfCombat] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
        go_to(&mut t, step);
        assert_eq!(t.life(P1), 18, "damage was dealt");
        assert!(attacking(&t, bears));
        cast_card(&mut t, P1, "Devouring Light", &[obj(bears)]).unwrap();
        t.resolve_all();
        assert!(t.in_exile("Grizzly Bears"), "{step:?}");
    }
}

#[test]
fn lava_storm_outside_combat_has_no_effect() {
    cr!("506.4", "608.2h");
    ruling!(
        "Lava Storm",
        "Lava Storm may be cast at any time, but it will have no effect unless there are attacking or blocking creatures on the battlefield"
    );
    supported("Lava Storm");
    for mode in [0usize, 1] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let lions = t.battlefield(P1, "Savannah Lions");
        let c = t.hand(P0, "Lava Storm");
        crate::r_s25_common::lands_for_cost(&mut t, P0, "Lava Storm");
        t.cast(P0, c).modes(&[mode]).go();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Lava Storm"));
        assert_eq!((damage_on(&t, bears), damage_on(&t, lions)), (0, 0));
    }
    // In combat it damages the attackers.
    let mut t = TestGame::new(2);
    let ids = p0_attacks(&mut t, &["Grizzly Bears"]);
    let c = t.hand(P1, "Lava Storm");
    crate::r_s25_common::lands_for_cost(&mut t, P1, "Lava Storm");
    t.cast(P1, c).modes(&[0]).go();
    t.resolve_all();
    assert!(!t.on_battlefield(ids[0]));
}

#[test]
fn angel_of_the_dire_hour_cast_outside_combat_triggers_but_does_nothing() {
    cr!("603.2", "506.4");
    ruling!(
        "Angel of the Dire Hour",
        "If you cast Angel of the Dire Hour outside of combat, its triggered ability will trigger but have no effect."
    );
    supported("Angel of the Dire Hour");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    let angel = t.hand(P0, "Angel of the Dire Hour");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Angel of the Dire Hour");
    t.cast(P0, angel).go();
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "exile all attacking"), 1);
    t.resolve_all();
    assert!(t.on_battlefield(bears) && t.on_battlefield(mine));
    assert_eq!(t.named_on_battlefield("Angel of the Dire Hour").len(), 1);
}

#[test]
fn blessed_reversal_counts_attackers_as_it_resolves() {
    cr!("608.2h");
    ruling!("Blessed Reversal", "The creatures are counted on resolution.");
    supported("Blessed Reversal");
    // Two attack; one is removed from combat and another enters attacking: 2 × 3 life.
    let mut t = TestGame::new(2);
    let ids = p0_attacks(&mut t, &["Grizzly Bears", "Hill Giant"]);
    cast_card(&mut t, P1, "Blessed Reversal", &[]).unwrap();
    remove_from_combat(&mut t.g, ids[0]);
    enter_attacking(&mut t, P0, "Savannah Lions", Entity::Player(P1));
    enter_attacking(&mut t, P0, "Memnite", Entity::Player(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 29);
}

#[test]
fn focus_fire_counts_your_creatures_as_it_resolves() {
    cr!("608.2h", "107.3a");
    ruling!(
        "Focus Fire",
        "The value of X is calculated only once, as Focus Fire resolves."
    );
    supported("Focus Fire");
    // P1 controls one creature when casting and three when it resolves: 2 + 3 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let ids = p0_attacks(&mut t, &["Colossal Dreadmaw"]);
    cast_card(&mut t, P1, "Focus Fire", &[obj(ids[0])]).unwrap();
    t.battlefield(P1, "Memnite");
    t.battlefield(P1, "Memnite");
    t.resolve_all();
    assert_eq!(damage_on(&t, ids[0]), 5);
    // Losing creatures afterwards changes nothing.
    for c in crate::r_s01_common::creatures(&t, P1) {
        destroy(&mut t, c);
    }
    assert_eq!(damage_on(&t, ids[0]), 5);
}

#[test]
fn puncturing_light_checks_power_on_targeting_and_on_resolution() {
    cr!("115.1", "608.2b");
    ruling!(
        "Puncturing Light",
        "The power of the targeted creature is checked both as you target it and as Puncturing Light resolves."
    );
    supported("Puncturing Light");
    // A 4/4 attacker can't be targeted.
    let mut t = TestGame::new(2);
    let ids = p0_attacks(&mut t, &["Serra Angel", "Grizzly Bears"]);
    let candidates = crate::r_s04_common::spell_targets(&mut t, P1, "Puncturing Light");
    assert_eq!(candidates, vec![obj(ids[1])]);
    // The Bears are pumped to 5/5 in response: it doesn't resolve.
    cast_card(&mut t, P1, "Puncturing Light", &[obj(ids[1])]).unwrap();
    cast_card(&mut t, P0, "Giant Growth", &[obj(ids[1])]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(ids[1]));
    // Without the pump it's destroyed.
    let mut t = TestGame::new(2);
    let ids = p0_attacks(&mut t, &["Grizzly Bears"]);
    cast_card(&mut t, P1, "Puncturing Light", &[obj(ids[0])]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(ids[0]));
}

#[test]
fn excise_controller_chooses_to_pay_as_it_resolves() {
    cr!("608.2c", "118.12");
    ruling!(
        "Excise",
        "The creature’s controller gets the option to pay when this spell resolves."
    );
    supported("Excise");
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        let ids = p0_attacks(&mut t, &["Grizzly Bears"]);
        t.lands(P0, "Wastes", 2);
        let c = t.hand(P1, "Excise");
        t.lands(P1, "Plains", 1);
        t.lands(P1, "Wastes", 2);
        let from = t.asked().len();
        t.cast(P1, c).x(2).target(obj(ids[0])).go();
        // Nothing asked of P0 while casting.
        assert!(t.asked()[from..]
            .iter()
            .all(|(p, d)| *p != P0 || matches!(d, Decision::Priority { .. })));
        t.answer_yes(P0, pay);
        t.resolve_all();
        assert_eq!(t.on_battlefield(ids[0]), pay, "pay: {pay}");
        assert_eq!(
            crate::r_s04_common::untapped_lands(&t, P0),
            if pay { 0 } else { 2 }
        );
    }
}

#[test]
fn blessed_alliance_the_opponent_chooses_the_attacker() {
    cr!("701.21a", "115.10");
    ruling!(
        "Blessed Alliance",
        "The targeted opponent chooses which of their attacking creatures to sacrifice, not you."
    );
    supported("Blessed Alliance");
    let mut t = TestGame::new(2);
    let ids = p0_attacks(&mut t, &["Grizzly Bears", "Hill Giant"]);
    t.answer_choose(P0, &[obj(ids[0])]);
    let c = t.hand(P1, "Blessed Alliance");
    t.lands(P1, "Plains", 2);
    let from = t.asked().len();
    t.cast(P1, c).modes(&[2]).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(ids[0]));
    assert!(t.on_battlefield(ids[1]));
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(d, Decision::ChooseEntities { candidates, .. } if candidates.len() == 2)));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. })));
}

#[test]
fn settle_the_wreckage_exiles_hexproof_attackers_and_counts_them() {
    cr!("115.10", "608.2h", "701.23a");
    ruling!(
        "Settle the Wreckage",
        "Settle the Wreckage targets only the player. Attacking creatures with hexproof that player controls will be exiled as this spell resolves."
    );
    ruling!(
        "Settle the Wreckage",
        "Settle the Wreckage targets only the player. Creatures with hexproof that player controls will be exiled as this spell resolves."
    );
    ruling!(
        "Settle the Wreckage",
        "The number of lands that player may find is the number of attacking creatures that were exiled, even if some of those creatures were tokens, weren't creature cards"
    );
    ruling!(
        "Settle the Wreckage",
        "That player can find fewer basic land cards than the number of exiled creatures, whether because they want to or because they don't have that many basic land cards left."
    );
    supported("Settle the Wreckage");
    // P0 attacks with a hexproof Gladecover Scout and a Soldier token; P0's library has
    // three Plains. With no choice made P0 finds two; choosing one finds one.
    for choose_one in [false, true] {
        let mut t = TestGame::new(2);
        let scout = t.battlefield(P0, "Gladecover Scout");
        let soldier = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
        attack_with(
            &mut t,
            &[(scout, Entity::Player(P1)), (soldier, Entity::Player(P1))],
        );
        t.resolve_all();
        let plains: Vec<ObjectId> = (0..3).map(|_| t.library_top(P0, "Plains")).collect();
        if choose_one {
            t.answer_choose(P0, &[obj(plains[0])]);
        }
        cast_card(&mut t, P1, "Settle the Wreckage", &[Entity::Player(P0)]).unwrap();
        t.resolve_all();
        assert!(t.in_exile("Gladecover Scout"));
        assert!(!t.on_battlefield(soldier));
        let lands = t
            .g
            .permanents()
            .filter(|o| o.controller == P0 && o.chars.name == "Plains")
            .count();
        assert_eq!(lands, if choose_one { 1 } else { 2 }, "one: {choose_one}");
    }
    // With a single basic land left, only one is found.
    let mut t = TestGame::new(2);
    crate::r_s11_common::empty_library(&mut t, P0);
    t.library_top(P0, "Plains");
    p0_attacks(&mut t, &["Grizzly Bears", "Hill Giant"]);
    cast_card(&mut t, P1, "Settle the Wreckage", &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    let lands = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.chars.name == "Plains")
        .count();
    assert_eq!(lands, 1);
    assert_eq!(t.library_size(P0), 0);
}

#[test]
fn fire_at_will_divides_its_damage_as_it_is_cast() {
    cr!("601.2d", "115.1d");
    ruling!(
        "Fire at Will",
        "The number of targets must be at least 1 and at most 3. You divide the damage as you cast Fire at Will, not as it resolves. Each target must be assigned at least 1 damage."
    );
    supported("Fire at Will");
    for division in [vec![3i64], vec![2, 1], vec![1, 1, 1]] {
        let mut t = TestGame::new(2);
        let names = ["Craw Wurm", "Craw Wurm", "Craw Wurm"];
        let ids = p0_attacks(&mut t, &names[..division.len()]);
        let c = t.hand(P1, "Fire at Will");
        t.lands(P1, "Plains", 3);
        t.answer(P1, DecisionKind::Divide, Answer::Numbers(division.clone()));
        let from = t.asked().len();
        let spell = t
            .cast(P1, c)
            .targets(&ids.iter().map(|i| obj(*i)).collect::<Vec<_>>())
            .go();
        // The division was asked while casting, with each target getting at least 1.
        assert!(t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::Divide { min_each: 1, .. })));
        assert_eq!(t.zone(spell), Zone::Stack);
        t.resolve_all();
        let dealt: Vec<i64> = ids.iter().map(|i| damage_on(&t, *i) as i64).collect();
        assert_eq!(dealt, division);
    }
}

#[test]
fn turn_the_tables_redirects_to_a_creature_removed_from_combat() {
    cr!("614.1a", "506.4");
    ruling!(
        "Turn the Tables",
        "If the targeted creature has been removed from combat but is still on the battlefield, the damage will be redirected."
    );
    supported("Turn the Tables");
    // P0's Craw Wurm (6/4) and Grizzly Bears attack; P1 targets the Wurm, which is then
    // removed from combat: the Bears' 2 damage to P1 is dealt to the Wurm.
    let mut t = TestGame::new(2);
    let ids = p0_attacks(&mut t, &["Craw Wurm", "Grizzly Bears"]);
    cast_card(&mut t, P1, "Turn the Tables", &[obj(ids[0])]).unwrap();
    t.resolve_all();
    remove_from_combat(&mut t.g, ids[0]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage_on(&t, ids[0]), 2);
}

#[test]
fn odds_coin_flip_has_no_call_winner_or_loser() {
    cr!("705.2");
    ruling!(
        "Odds // Ends",
        "The Odds coin flip has no winner or loser, and no player calls \"heads\" or \"tails.\""
    );
    supported("Odds // Ends");
    let mut t = TestGame::new(2);
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    let from = t.asked().len();
    let ev = t.g.turn_events.len();
    let shock_spell = t.g.stack[0];
    cast_card(&mut t, P0, "Odds // Ends", &[obj(shock_spell)]).unwrap();
    t.resolve();
    assert!(!t.asked()[from..].iter().any(
        |(_, d)| matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("Call"))
    ));
    let flips: Vec<(bool, bool)> = t.g.turn_events[ev..]
        .iter()
        .filter_map(|e| match e {
            Event::CoinFlipped { won, lost, .. } => Some((*won, *lost)),
            _ => None,
        })
        .collect();
    assert_eq!(flips, vec![(false, false)]);
}

#[test]
fn path_of_mettle_deals_1_damage_to_each_creature_without_the_abilities() {
    cr!("120.3", "603.2");
    ruling!(
        "Path of Mettle // Metzali, Tower of Triumph",
        "Path of Mettle's first ability deals 1 damage to each creature that doesn't have any of the four listed abilities. It doesn't deal 1 damage to each creature for each ability that creature doesn't have."
    );
    supported("Path of Mettle // Metzali, Tower of Triumph");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let angel = t.battlefield(P1, "Serra Angel");
    t.enter(P0, "Path of Mettle // Metzali, Tower of Triumph");
    t.resolve_all();
    assert_eq!(damage_on(&t, wurm), 1);
    assert_eq!(damage_on(&t, angel), 0, "vigilance");
}
