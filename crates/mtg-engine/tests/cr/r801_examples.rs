//! The worked examples of CR 801 (the limited range of influence option), played out as
//! tests. Seat `i` is `PlayerId(i)`; a player's left is the next seat in turn order.

use super::r600_examples_common::{custom_costed, supported};
use super::r800_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn ids(v: &[usize]) -> Vec<PlayerId> {
    v.iter().map(|i| PlayerId(*i as u8)).collect()
}

fn sorted(mut v: Vec<PlayerId>) -> Vec<PlayerId> {
    v.sort();
    v
}

#[test]
fn a_range_of_one_is_you_and_the_players_next_to_you() {
    cr!("801.2", "801.2a", "801.4");
    cr_example!("801.2a", "A range of influence of 1 means");
    let mut t = ranged(7, 1);
    assert_eq!(sorted(t.g.players_in_range(P3)), ids(&[2, 3, 4]));
    assert_eq!(sorted(t.g.players_in_range(P0)), ids(&[0, 1, 6]));
    // What P3 can target follows.
    let c = bolt_candidates(&mut t, P3);
    for p in 0..7u8 {
        let p = PlayerId(p);
        assert_eq!(
            c.contains(&Entity::Player(p)),
            [P2, P3, P4].contains(&p),
            "{p}"
        );
    }
}

#[test]
fn a_range_of_two_is_you_and_two_players_on_each_side() {
    cr!("801.2", "801.2a", "801.4");
    cr_example!("801.2a", "A range of influence of 2 means");
    let mut t = ranged(7, 2);
    assert_eq!(sorted(t.g.players_in_range(P3)), ids(&[1, 2, 3, 4, 5]));
    assert_eq!(sorted(t.g.players_in_range(P0)), ids(&[0, 1, 2, 5, 6]));
    let c = bolt_candidates(&mut t, P3);
    for p in 0..7u8 {
        let p = PlayerId(p);
        assert_eq!(
            c.contains(&Entity::Player(p)),
            (1..=5).contains(&p.0),
            "{p}"
        );
    }
}

#[test]
fn carissa_enters_alexs_range_at_the_start_of_the_next_turn_after_rob_leaves() {
    cr!("801.2c", "801.4");
    cr_example!("801.2c", "Carissa will enter Alex’s range of influence");
    // Carissa (P0) sits to Rob's (P1's) right and Alex (P2) to his left.
    let (carissa, rob, alex) = (P0, P1, P2);
    let mut t = ranged(5, 1);
    to_turn_of(&mut t, P4);
    assert!(!t.g.players_in_range(alex).contains(&carissa));
    concede(&mut t, rob);
    // Not yet: the players within range were determined as this turn began.
    t.answer_targets(alex, &[Entity::Player(P3)]);
    let c = bolt_candidates(&mut t, alex);
    assert!(!c.contains(&Entity::Player(carissa)), "{c:?}");
    t.resolve_all();
    // Carissa's turn is the next one.
    to_turn_of(&mut t, carissa);
    assert!(t.g.players_in_range(alex).contains(&carissa));
    let c = bolt_candidates(&mut t, alex);
    assert!(c.contains(&Entity::Player(carissa)), "{c:?}");
}

#[test]
fn cuombajj_witches_rob_chooses_a_target_within_both_ranges() {
    cr!("801.5a", "801.4", "601.7");
    cr_example!("801.5a", "Cuombajj Witches");
    supported("Cuombajj Witches");
    // Rob is P1 and Alex, to his left, is P2.
    let (rob, alex) = (P1, P2);
    let mut t = ranged(5, 1);
    let witches = t.battlefield(alex, "Cuombajj Witches");
    let creatures: Vec<ObjectId> = (0..5).map(|i| bear(&mut t, PlayerId(i))).collect();
    let from = t.asked().len();
    t.answer_targets(alex, &[Entity::Player(rob)]);
    t.answer_choose(alex, &[Entity::Player(rob)]);
    t.answer_targets(rob, &[Entity::Player(alex)]);
    t.activate(alex, witches, 0, &[]).expect("activate");
    let robs: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, .. } if *p == rob => Some(candidates.clone()),
            _ => None,
        })
        .next_back()
        .expect("Rob chooses the second target");
    let mut expected = vec![
        Entity::Player(rob),
        Entity::Player(alex),
        Entity::Object(witches),
        Entity::Object(creatures[1]),
        Entity::Object(creatures[2]),
    ];
    let mut got = robs.clone();
    expected.sort();
    got.sort();
    assert_eq!(got, expected, "himself, Alex, or their creatures");
    t.resolve_all();
    assert_eq!(t.life(rob), 19);
    assert_eq!(t.life(alex), 19);
}

#[test]
fn carissa_chooses_alexs_mode_although_he_is_out_of_her_range() {
    cr!("801.5b", "601.7");
    cr_example!("801.5b", "Carissa can choose the mode even though");
    // Carissa (P0, range 1) sits to Rob's (P1's) right, Alex (P2, range 2) to his left.
    let (carissa, alex) = (P0, P2);
    let mut t = TestGame::with_config(
        5,
        GameConfig {
            range_of_influence: Some(1),
            player_ranges: vec![(alex, 2)],
            ..GameConfig::free_for_all()
        },
    );
    assert!(!t.g.players_in_range(carissa).contains(&alex));
    let spell = custom_costed(
        "Opponent's Gift",
        "Sorcery",
        "{1}",
        "An opponent chooses one —\n• You draw two cards.\n• Each creature you control gets +2/+2 until end of turn.",
    );
    let card = t.custom(alex, spell, Zone::Hand(alex));
    t.set_step(alex, Step::PrecombatMain);
    t.lands(alex, "Wastes", 1);
    t.answer_choose(alex, &[Entity::Player(carissa)]);
    t.answer(carissa, DecisionKind::Modes, Answer::Indices(vec![0]));
    let hand = t.hand_size(alex);
    t.cast(alex, card).go();
    let asked: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![carissa]);
    t.resolve_all();
    assert_eq!(t.hand_size(alex), hand - 1 + 2, "Carissa chose \"you draw two cards\"");
}

#[test]
fn the_nearest_opponent_to_the_emperors_left_separates_fact_or_fiction() {
    cr!("801.5c", "809.3a");
    cr_example!("801.5c", "Fact or Fiction");
    supported("Fact or Fiction");
    // Two teams of three, everyone with range of influence 1. P1 is team 0's emperor:
    // both players next to them are their generals.
    let mut t = TestGame::with_config(
        6,
        GameConfig {
            range_of_influence: Some(1),
            ..GameConfig::emperor(vec![0, 0, 0, 1, 1, 1])
        },
    );
    let emperor = P1;
    assert_eq!(t.g.emperor_of(P0), Some(emperor));
    for _ in 0..5 {
        t.library_top(emperor, "Island");
    }
    t.set_step(emperor, Step::PrecombatMain);
    t.lands(emperor, "Island", 4);
    let fof = t.hand(emperor, "Fact or Fiction");
    t.cast(emperor, fof).go();
    t.resolve();
    let separators: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| {
            matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.starts_with("Separate"))
        })
        .map(|(p, _)| *p)
        .collect();
    // P3 is the closest opponent to the emperor's left (P5 is closer on the right).
    assert_eq!(separators, vec![P3]);
    assert!(t.in_graveyard(emperor, "Fact or Fiction"));
    assert_eq!(t.hand_size(emperor) + t.graveyard_size(emperor), 5 + 1);
}

/// An Aura Rob controls that's attached to Alex's creature.
fn rob_aura(t: &mut TestGame, rob: PlayerId, name: &str, trigger: &str, to: ObjectId) -> ObjectId {
    let def = custom_card(
        name,
        "Enchantment — Aura",
        None,
        &format!("Enchant creature\n{trigger}"),
    );
    let aura = t.custom(rob, def, Zone::Battlefield);
    assert!(t.g.attach(aura, Entity::Object(to)));
    t.g.recompute();
    aura
}

#[test]
fn becomes_blocked_triggers_but_becomes_blocked_by_a_creature_out_of_range_doesnt() {
    cr!("801.7", "509.3c", "509.3d");
    cr_example!("801.7", "Whenever enchanted creature becomes blocked by a creature");
    supported("Runeclaw Bear");
    // Rob is P0, Alex (to his left) P1; Alex attacks P2, two seats from Rob.
    let (rob, alex, left) = (P0, P1, P2);
    let mut t = ranged(5, 1);
    let bear = t.battlefield(alex, "Runeclaw Bear");
    let blocker = t.battlefield(left, "Grizzly Bears");
    rob_aura(
        &mut t,
        rob,
        "Blocked Aura",
        "Whenever enchanted creature becomes blocked, you gain 1 life.",
        bear,
    );
    rob_aura(
        &mut t,
        rob,
        "Blocked By Aura",
        "Whenever enchanted creature becomes blocked by a creature, you gain 2 life.",
        bear,
    );
    t.set_step(alex, Step::BeginningOfCombat);
    declare(&mut t, &[(bear, Entity::Player(left))]);
    block(&mut t, left, &[(blocker, bear)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(is_blocked(&t, bear));
    t.resolve_all();
    // Only the first Aura's ability triggered.
    assert_eq!(t.life(rob), 21, "{}", t.dump_log());
    // Without the option, both would have.
    let mut t = TestGame::with_config(5, GameConfig::free_for_all());
    let bear = t.battlefield(alex, "Runeclaw Bear");
    let blocker = t.battlefield(left, "Grizzly Bears");
    rob_aura(&mut t, rob, "Blocked Aura", "Whenever enchanted creature becomes blocked, you gain 1 life.", bear);
    rob_aura(&mut t, rob, "Blocked By Aura", "Whenever enchanted creature becomes blocked by a creature, you gain 2 life.", bear);
    t.set_step(alex, Step::BeginningOfCombat);
    declare(&mut t, &[(bear, Entity::Player(left))]);
    block(&mut t, left, &[(blocker, bear)]);
    go_to(&mut t, Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(t.life(rob), 23);
}

#[test]
fn extractor_demon_carissas_triggers_and_alexs_doesnt() {
    cr!("801.7", "801.7a", "603.10a");
    cr_example!("801.7a", "Extractor Demon");
    supported("Extractor Demon");
    // Alex (P0) and Carissa (P2) are outside each other's range.
    let (alex, carissa) = (P0, P2);
    let mut t = ranged(5, 1);
    let alex_demon = t.battlefield(alex, "Extractor Demon");
    let carissa_demon = t.battlefield(carissa, "Extractor Demon");
    // Carissa controls a Runeclaw Bear that Alex owns: Rob (P1), who sits between them,
    // gave it to her (an effect of Alex's can't affect Carissa, CR 801.10).
    let bear = t.battlefield(alex, "Runeclaw Bear");
    apply(
        &mut t,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::Player(carissa),
            duration: Duration::Permanent,
        },
        &[bear],
    );
    assert_eq!(t.obj_now(bear).controller, carissa);
    t.answer_yes(carissa, true);
    t.answer_targets(carissa, &[Entity::Player(P3)]);
    t.g.destroy(bear, None);
    t.settle();
    assert!(t.in_graveyard(alex, "Runeclaw Bear"));
    let sources: Vec<ObjectId> = t
        .g
        .stack
        .iter()
        .map(|s| t.g.ability_source_of(*s))
        .collect();
    assert_eq!(sources, vec![carissa_demon], "only Carissa's Demon triggered");
    assert!(!sources.contains(&alex_demon));
    let lib = t.library_size(P3);
    t.resolve_all();
    assert_eq!(t.library_size(P3), lib - 2);
}

#[test]
fn pyroclasm_damages_only_creatures_within_alexs_range() {
    cr!("801.10");
    cr_example!("801.10", "Pyroclasm");
    supported("Pyroclasm");
    let alex = P0;
    let mut t = ranged(6, 1);
    let creatures: Vec<ObjectId> = (0..6).map(|i| bear(&mut t, PlayerId(i))).collect();
    t.set_step(alex, Step::PrecombatMain);
    t.lands(alex, "Mountain", 2);
    let pyro = t.hand(alex, "Pyroclasm");
    t.cast(alex, pyro).go();
    t.resolve();
    for (i, c) in creatures.iter().enumerate() {
        let in_range = [0, 1, 5].contains(&i);
        assert_eq!(!t.on_battlefield(*c), in_range, "P{i}'s creature");
    }
}

#[test]
fn coat_of_arms_counts_only_creatures_within_alexs_range() {
    cr!("801.11");
    cr_example!("801.11", "Coat of Arms will boost Alex’s creatures");
    cr_example!("801.11", "Coat of Arms will boost Rob’s creatures");
    supported("Coat of Arms");
    // Alex is P0; Rob (P5) sits to his right, and the player to Alex's left is P1.
    let (alex, rob) = (P0, P5);
    let mut t = ranged(6, 1);
    t.battlefield(alex, "Coat of Arms");
    // A Bear for each player.
    let bears: Vec<ObjectId> = (0..6).map(|i| bear(&mut t, PlayerId(i))).collect();
    t.g.recompute();
    // Alex's Bear counts the Bears of Alex's left and right neighbors only.
    assert_eq!(t.pt(bears[alex.idx()]), (4, 4));
    // Rob's Bear counts Alex's and the one of the player to Alex's left (out of Rob's
    // range), but not the one of the player to Rob's right (out of Alex's).
    assert_eq!(t.pt(bears[rob.idx()]), (4, 4));
    assert_eq!(t.pt(bears[1]), (4, 4));
    // Creatures outside Alex's range aren't affected at all.
    for i in 2..5 {
        assert_eq!(t.pt(bears[i]), (2, 2), "P{i}'s Bear");
    }
}

#[test]
fn captains_maneuver_cant_redirect_lava_axe_to_carissa_out_of_alexs_range() {
    cr!("801.13", "801.13a", "614.9");
    cr_example!("801.13a", "Captain’s Maneuver");
    supported("Lava Axe");
    supported("Captain's Maneuver");
    // Alex (P0), Rob (P1), Carissa (P2): Carissa is in Rob's range but not Alex's.
    let (alex, rob, carissa) = (P0, P1, P2);
    let mut t = ranged(5, 1);
    t.set_step(alex, Step::PrecombatMain);
    t.lands(alex, "Mountain", 5);
    let axe = t.hand(alex, "Lava Axe");
    t.cast(alex, axe).target(Entity::Player(rob)).go();
    t.lands(rob, "Mountain", 4);
    t.lands(rob, "Plains", 1);
    let maneuver = t.hand(rob, "Captain's Maneuver");
    t.cast(rob, maneuver)
        .x(3)
        .target(Entity::Player(rob))
        .target(Entity::Player(carissa))
        .go();
    t.resolve();
    assert_eq!(t.life(rob), 20);
    t.resolve_all();
    assert_eq!(t.life(rob), 18, "{}", t.dump_log());
    assert_eq!(t.life(carissa), 20);
}

fn prevent_creature_damage() -> mtg_engine::card::CardDef {
    custom_card(
        "Creature Peace",
        "Enchantment",
        None,
        "Prevent all damage that would be dealt by creatures.",
    )
}

#[test]
fn alexs_creature_damage_prevention_doesnt_stop_carissas_creature() {
    cr!("801.13b");
    cr_example!("801.13b", "Prevent all damage that would be dealt by creatures");
    // Alex (P0), Rob (P1), Carissa (P2).
    let (alex, rob, carissa) = (P0, P1, P2);
    let mut t = ranged(5, 1);
    t.custom(alex, prevent_creature_damage(), Zone::Battlefield);
    let attacker = bear(&mut t, carissa);
    t.set_step(carissa, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(rob))], &[]);
    assert_eq!(t.life(rob), 18);
    // Rob's own creature attacking Alex is within range: its damage is prevented.
    let robs = bear(&mut t, rob);
    to_turn_of(&mut t, rob);
    t.set_step(rob, Step::BeginningOfCombat);
    t.attack(&[(robs, Entity::Player(alex))], &[]);
    assert_eq!(t.life(alex), 20);
}

#[test]
fn mending_hands_prevents_carissas_lightning_blast_to_rob() {
    cr!("801.13b");
    cr_example!("801.13b", "Mending Hands");
    supported("Lightning Blast");
    supported("Mending Hands");
    let (alex, rob, carissa) = (P0, P1, P2);
    let mut t = ranged(5, 1);
    t.set_step(carissa, Step::PrecombatMain);
    t.lands(carissa, "Mountain", 4);
    let blast = t.hand(carissa, "Lightning Blast");
    t.cast(carissa, blast).target(Entity::Player(rob)).go();
    t.lands(alex, "Plains", 1);
    let hands = t.hand(alex, "Mending Hands");
    t.cast(alex, hands).target(Entity::Player(rob)).go();
    t.resolve_all();
    assert_eq!(t.life(rob), 20);
}

#[test]
fn alexs_fog_doesnt_prevent_combat_damage_between_carissa_and_rob() {
    cr!("801.13b");
    cr_example!("801.13b", "Fog");
    supported("Fog");
    let (alex, rob, carissa) = (P0, P1, P2);
    let mut t = ranged(5, 1);
    let attacker = bear(&mut t, carissa);
    let blocker = bear(&mut t, rob);
    t.set_step(carissa, Step::BeginningOfCombat);
    declare(&mut t, &[(attacker, Entity::Player(rob))]);
    block(&mut t, rob, &[(blocker, attacker)]);
    go_to(&mut t, Step::DeclareBlockers);
    t.lands(alex, "Forest", 1);
    let fog = t.hand(alex, "Fog");
    t.cast(alex, fog).go();
    t.resolve();
    assert!(t.in_graveyard(alex, "Fog"));
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(attacker), "Rob's creature dealt it damage");
    assert!(!t.on_battlefield(blocker), "Carissa's creature dealt it damage");
}
