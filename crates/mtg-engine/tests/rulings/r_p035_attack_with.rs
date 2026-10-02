//! Rulings batch P035 — "whenever you attack with one or more [creatures]" (Metropolis
//! Angel): one trigger for the attack, however many such creatures attack (CR 508.1,
//! 603.2c). Tests for each card the phrase compiles for.

use crate::r_p035_common::*;
use crate::r_s01_common::{attack_with, give_mana_for, stack_library, supported, tokens};
use crate::r_s05_common::tokens_with_subtype;
use crate::r_s06_common::{activate_containing, has_kw};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn attackers(ids: &[ObjectId]) -> Vec<(ObjectId, Entity)> {
    ids.iter().map(|id| (*id, Entity::Player(P1))).collect()
}

#[test]
fn you_attack_with_cards_compile() {
    for n in [
        "Amazing Alliance",
        "Anim Pakal, Thousandth Moon",
        "Hermes, Overseer of Elpis",
        "Hired Claw",
        "Interceptor, Shadow's Hound",
        "Jolene, Plundering Pugilist",
        "Lulu, Curious Hollyphant",
        "Lulu, Inspiring Hollyphant",
        "Lulu, Vengeful Hollyphant",
        "Lulu, Wild Hollyphant",
        "Metropolis Angel",
        "Mordor Trebuchet",
        "Persistent Marshstalker",
        "Sidar Jabari of Zhalfir",
        "Talion's Messenger",
        "Vrestin, Menoptra Leader",
    ] {
        supported(n);
    }
}

#[test]
fn metropolis_angel_draws_exactly_one_card() {
    cr!("603.2c", "508.1m");
    ruling!(
        "Metropolis Angel",
        "Metropolis Angel's last ability triggers whenever you attack with at least one creature that has at least one counter on it. You draw exactly one card as it resolves, no matter how many attacking creatures had counters on them."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Metropolis Angel");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Savannah Lions");
    put(&mut t, a, counters::PLUS1, 1);
    put(&mut t, b, "charge", 2);
    stack_library(&mut t, P0, &["Island", "Island", "Island"]);
    let hand = t.hand_size(P0);
    attack_with(&mut t, &attackers(&[a, b, c]));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Attacking only with creatures without counters: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Metropolis Angel");
    let c = t.battlefield(P0, "Savannah Lions");
    attack_with(&mut t, &attackers(&[c]));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn hired_claw_pings_and_grows() {
    cr!("508.1m", "602.5b");
    let mut t = TestGame::new(2);
    let claw = t.battlefield(P0, "Hired Claw");
    t.lands(P0, "Mountain", 4);
    // Can't grow before an opponent lost life this turn.
    assert!(activate_containing(&mut t, P0, claw, "+1/+1").is_err());
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &attackers(&[claw]));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    activate_containing(&mut t, P0, claw, "+1/+1").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(claw, counters::PLUS1), 1);
    assert!(
        activate_containing(&mut t, P0, claw, "+1/+1").is_err(),
        "once each turn"
    );
}

#[test]
fn amazing_alliance_gains_life_for_each_legendary_attacker() {
    cr!("603.2c", "613.4c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Amazing Alliance");
    let a = t.battlefield(P0, "Isamaru, Hound of Konda");
    let b = t.battlefield(P0, "Anim Pakal, Thousandth Moon");
    let c = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(c), (3, 3));
    attack_with(&mut t, &attackers(&[a, b, c]));
    t.resolve_all();
    assert_eq!(t.life(P0), 22, "two legendary attackers");
}

#[test]
fn anim_pakal_counter_and_attacking_gnomes() {
    cr!("508.4", "107.3c");
    let mut t = TestGame::new(2);
    let anim = t.battlefield(P0, "Anim Pakal, Thousandth Moon");
    put(&mut t, anim, counters::PLUS1, 1);
    attack_with(&mut t, &attackers(&[anim]));
    t.resolve_all();
    assert_eq!(t.counters(anim, counters::PLUS1), 2);
    let gnomes = tokens_with_subtype(&t, P0, "Gnome");
    assert_eq!(gnomes.len(), 2);
    for g in &gnomes {
        assert!(t.obj_now(*g).tapped && t.g.is_attacking(*g));
        assert!(t.obj_now(*g).is(mtg_engine::types::CardType::Artifact));
    }
    // Attacking with only Gnomes doesn't trigger it.
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t, &attackers(&gnomes));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn hermes_birds_and_scry() {
    cr!("701.22a", "508.1m");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hermes, Overseer of Elpis");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    let birds = tokens_with_subtype(&t, P0, "Bird");
    assert_eq!(birds.len(), 1);
    assert!(has_kw(&t, birds[0], KeywordKind::Flying));
    assert!(has_kw(&t, birds[0], KeywordKind::Vigilance));
    let bird = t.battlefield(P0, "Birds of Paradise");
    let from = t.asked().len();
    attack_with(&mut t, &attackers(&[bird]));
    t.resolve_all();
    let scries = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::Scry { cards } if cards.len() == 2))
        .count();
    assert_eq!(scries, 1);
}

#[test]
fn interceptor_returns_from_the_graveyard_attacking() {
    cr!("508.4", "702.111a");
    let mut t = TestGame::new(2);
    let hound = t.graveyard(P0, "Interceptor, Shadow's Hound");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P0, true);
    attack_with(&mut t, &attackers(&[isamaru]));
    t.resolve_all();
    assert!(t.on_battlefield(hound));
    assert!(t.obj_now(hound).tapped && t.g.is_attacking(t.g.current(hound)));
    assert!(has_kw(&t, hound, KeywordKind::Menace));
    // Assassins you control have menace.
    let assassin = t.battlefield(P0, "Royal Assassin");
    assert!(has_kw(&t, assassin, KeywordKind::Menace));
}

#[test]
fn jolene_treasure_and_ping() {
    cr!("111.10a", "508.1m");
    let mut t = TestGame::new(2);
    let jolene = t.battlefield(P0, "Jolene, Plundering Pugilist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &attackers(&[bears]));
    assert_eq!(t.stack_len(), 0, "power 2: no trigger");
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t, &attackers(&[jolene, bears]));
    t.resolve_all();
    let treasures = tokens_with_subtype(&t, P0, "Treasure");
    assert_eq!(treasures.len(), 1);
    t.lands(P0, "Mountain", 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(treasures[0])]);
    activate_containing(&mut t, P0, jolene, "damage").unwrap();
    t.resolve_all();
    assert!(tokens_with_subtype(&t, P0, "Treasure").is_empty());
}

/// Lulu (named) and two other flyers attack.
fn lulu_attacks(name: &str) -> (TestGame, ObjectId, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    let lulu = t.battlefield(P0, name);
    let a = t.battlefield(P0, "Wind Drake");
    let b = t.battlefield(P0, "Serra Angel");
    let c = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Island", "Island", "Island"]);
    attack_with(&mut t, &attackers(&[lulu, a, b, c]));
    (t, lulu, vec![a, b])
}

#[test]
fn lulu_curious_draws_that_many_then_discards() {
    cr!("603.2c", "508.1m");
    let (mut t, _, _) = lulu_attacks("Lulu, Curious Hollyphant");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2 - 1);
}

#[test]
fn lulu_inspiring_creates_that_many_attacking_soldiers() {
    cr!("508.4");
    let (mut t, _, _) = lulu_attacks("Lulu, Inspiring Hollyphant");
    t.resolve_all();
    let soldiers = tokens_with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 2);
    assert!(soldiers.iter().all(|s| t.g.is_attacking(*s)));
}

#[test]
fn lulu_vengeful_drains_that_much() {
    cr!("119.3");
    let (mut t, _, _) = lulu_attacks("Lulu, Vengeful Hollyphant");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn lulu_wild_pumps_those_creatures() {
    cr!("611.2c");
    let (mut t, lulu, flyers) = lulu_attacks("Lulu, Wild Hollyphant");
    t.resolve_all();
    assert_eq!(t.pt(flyers[0]), (4, 4), "Wind Drake");
    assert_eq!(t.pt(flyers[1]), (6, 6), "Serra Angel");
    assert_eq!(t.pt(lulu), (2, 4), "not Lulu herself");
}

#[test]
fn mordor_trebuchet_boulder_attacks_and_is_sacrificed() {
    cr!("508.4", "603.7a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mordor Trebuchet");
    let goblin = t.battlefield(P0, "Raging Goblin");
    attack_with(&mut t, &attackers(&[goblin]));
    t.resolve_all();
    let boulders = t.named_on_battlefield("Ballistic Boulder");
    assert_eq!(boulders.len(), 1);
    let b = boulders[0];
    assert!(t.g.is_attacking(b));
    assert!(has_kw(&t, b, KeywordKind::Flying));
    assert_eq!(t.pt(b), (2, 1));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.named_on_battlefield("Ballistic Boulder").is_empty());
    assert_eq!(t.life(P1), 20 - 1 - 2);
}

#[test]
fn persistent_marshstalker_threshold_return() {
    cr!("207.2c", "603.4");
    let mut t = TestGame::new(2);
    let stalker = t.graveyard(P0, "Persistent Marshstalker");
    let rat = t.battlefield(P0, "Typhoid Rats");
    for _ in 0..5 {
        t.graveyard(P0, "Island");
    }
    t.lands(P0, "Swamp", 3);
    // Six cards in the graveyard: no trigger.
    attack_with(&mut t, &attackers(&[rat]));
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P1, Step::End);
    t.graveyard(P0, "Island");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    attack_with(&mut t, &attackers(&[rat]));
    t.resolve_all();
    assert!(t.on_battlefield(stalker));
    assert!(t.g.is_attacking(t.g.current(stalker)));
    assert_eq!(t.pt(stalker), (4, 1), "+1/+0 for the other Rat");
}

#[test]
fn sidar_jabari_loots_and_returns_knights() {
    cr!("702.9a", "702.7b");
    let mut t = TestGame::new(2);
    let sidar = t.battlefield(P0, "Sidar Jabari of Zhalfir");
    let knight = t.graveyard(P0, "White Knight");
    stack_library(&mut t, P0, &["Island"]);
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Object(knight)]);
    attack_with(&mut t, &attackers(&[sidar]));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "drew and discarded");
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.zone(knight), Zone::Battlefield);
}

#[test]
fn talions_messenger_loot_then_counter_on_a_faerie() {
    cr!("603.12", "701.9a");
    let mut t = TestGame::new(2);
    let messenger = t.battlefield(P0, "Talion's Messenger");
    stack_library(&mut t, P0, &["Island"]);
    t.answer_targets(P0, &[Entity::Object(messenger)]);
    attack_with(&mut t, &attackers(&[messenger]));
    t.resolve_all();
    assert_eq!(t.counters(messenger, counters::PLUS1), 1);
    assert_eq!(t.g.player(P0).graveyard.len(), 1, "discarded a card");
}

#[test]
fn vrestin_counters_tokens_and_attacking_insects() {
    cr!("107.3a", "508.1m");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Vrestin, Menoptra Leader");
    t.lands(P0, "Wastes", 2);
    let vrestin = t.hand(P0, "Vrestin, Menoptra Leader");
    t.cast(P0, vrestin).x(2).go();
    t.resolve_all();
    assert_eq!(t.counters(vrestin, counters::PLUS1), 2);
    let insects = tokens(&t, P0);
    assert_eq!(insects.len(), 2);
    assert!(insects.iter().all(|i| has_kw(&t, *i, KeywordKind::Flying)));
    for i in &insects {
        t.g.objects[i.0 as usize].summoning_sick = false;
    }
    let v = t.g.current(vrestin);
    t.g.objects[v.0 as usize].summoning_sick = false;
    attack_with(&mut t, &attackers(&[insects[0], v]));
    t.resolve_all();
    assert_eq!(t.counters(insects[0], counters::PLUS1), 1);
    assert_eq!(t.counters(insects[1], counters::PLUS1), 0, "not attacking");
    assert_eq!(
        t.counters(vrestin, counters::PLUS1),
        3,
        "an attacking Insect too"
    );
}
