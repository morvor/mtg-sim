//! Rulings batch P001 — paying costs: sacrifices as additional costs and targets chosen
//! before costs are paid (CR 601.2c, 601.2h), mana abilities activated while casting
//! (CR 601.2g), mana with a rider on what it's spent on (CR 106.6), "tapped for mana"
//! (CR 106.12), and abilities activated from a graveyard (CR 602.1).

use crate::r_p001_common::*;
use crate::r_p006_common::in_graveyard;
use crate::r_s01_common::{attack_with, give_mana_for, supported};
use crate::r_s02_common::{create_token, target_candidates};
use crate::r_s04_common::add_mana;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn trash_for_treasures_sacrificed_artifact_can_reduce_its_cost() {
    cr!("601.2f", "601.2h");
    ruling!(
        "Trash for Treasure",
        "The artifact that you sacrifice is sacrificed after determining the total cost to cast Trash for Treasure. Its abilities may affect that cost."
    );
    supported("Trash for Treasure");
    supported("Ruby Medallion");
    // Ruby Medallion ("Red spells you cast cost {1} less to cast.") is the artifact
    // sacrificed: Trash for Treasure still costs {1}{R}.
    let mut t = TestGame::new(2);
    let medallion = t.battlefield(P0, "Ruby Medallion");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Trash for Treasure");
    assert!(castable(&mut t, P0, card));
    t.answer_choose(P0, &[obj(medallion)]);
    t.cast(P0, card).target(thopter).go();
    assert!(in_graveyard(&t, medallion));
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.on_battlefield(thopter));
}

#[test]
fn trash_for_treasure_cant_target_the_artifact_it_sacrifices() {
    cr!("601.2c", "601.2h");
    ruling!(
        "Trash for Treasure",
        "Trash for Treasure can't target the artifact that you sacrifice to pay its additional cost."
    );
    // No artifact card in the graveyard: it can't be cast, though sacrificing the
    // Ornithopter would put one there.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    give_mana_for(&mut t, P0, "Trash for Treasure");
    let card = t.hand(P0, "Trash for Treasure");
    assert!(!try_cast(&mut t, P0, card));
    assert!(t.on_battlefield(thopter));
    // With Juggernaut there, it's the only target offered.
    let jugg = t.graveyard(P0, "Juggernaut");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(thopter)]);
    t.cast(P0, card).target(jugg).go();
    assert_eq!(target_candidates(&t, P0, from), vec![vec![obj(jugg)]]);
    t.resolve_all();
    assert!(t.on_battlefield(jugg));
    assert!(in_graveyard(&t, thopter));
}

#[test]
fn trash_for_treasure_cant_use_one_artifact_for_mana_and_the_sacrifice() {
    cr!("601.2g", "601.2h");
    ruling!(
        "Trash for Treasure",
        "You can't sacrifice an artifact to generate mana to pay towards Trash for Treasure's cost and also to pay its additional cost."
    );
    // Two Mountains and a Treasure: the Treasure can be sacrificed for mana or for the
    // additional cost, not both, so {2}{R} can't be paid.
    for mountains in [2, 3] {
        let mut t = TestGame::new(2);
        let treasure = create_token(&mut t, P0, "Treasure");
        t.graveyard(P0, "Juggernaut");
        t.lands(P0, "Mountain", mountains);
        let card = t.hand(P0, "Trash for Treasure");
        t.answer_choose(P0, &[obj(treasure)]);
        let r = t.cast(P0, card).try_go();
        assert_eq!(r.is_ok(), mountains == 3, "{mountains} Mountains");
        if mountains == 3 {
            assert!(!t.on_battlefield(treasure));
            assert_eq!(untapped_lands(&t, P0), 0);
            t.resolve_all();
            assert!(t.in_graveyard(P0, "Trash for Treasure"));
            assert_eq!(t.named_on_battlefield("Juggernaut").len(), 1);
        } else {
            // The attempt is undone: nothing was paid.
            assert!(t.on_battlefield(treasure));
            assert!(t.in_hand(P0, "Trash for Treasure"));
            assert_eq!(untapped_lands(&t, P0), 2);
        }
    }
}

#[test]
fn launch_party_does_nothing_if_its_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Launch Party",
        "If the target creature is an illegal target when Launch Party tries to resolve, it won’t resolve and none of its effects will happen. No player will lose 2 life."
    );
    supported("Launch Party");
    supported("Unsummon");
    let mut t = TestGame::new(2);
    let fodder = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P0, "Launch Party");
    let card = t.hand(P0, "Launch Party");
    t.answer_choose(P0, &[obj(fodder)]);
    t.cast(P0, card).target(giant).go();
    // In response, P1 returns the Giant to hand.
    let unsummon = t.hand(P1, "Unsummon");
    add_mana(&mut t, P1, ManaType::U, 1);
    t.cast(P1, unsummon).target(giant).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
    // The sacrifice (a cost) isn't undone.
    assert!(in_graveyard(&t, fodder));
}

#[test]
fn champion_of_stray_souls_returns_itself_only_from_the_graveyard() {
    cr!("602.1", "113.6m");
    ruling!(
        "Champion of Stray Souls",
        "The last ability can be activated only if Champion of Stray Souls is in your graveyard."
    );
    supported("Champion of Stray Souls");
    for zone in [Zone::Battlefield, Zone::Hand(P0), Zone::Graveyard(P0)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 7);
        let champ = crate::r_s01_common::place(&mut t, P0, "Champion of Stray Souls", zone);
        // On the battlefield its abilities are numbered from the first; elsewhere only the
        // last one is listed as activatable.
        let last = 1;
        let ok = can_activate_nth(&mut t, P0, champ, last);
        assert_eq!(ok, zone == Zone::Graveyard(P0), "{zone:?}");
        if zone == Zone::Graveyard(P0) {
            let lib = t.library_size(P0);
            t.activate(P0, champ, last, &[]).unwrap();
            t.resolve_all();
            assert_eq!(t.library_size(P0), lib + 1);
            assert_eq!(t.g.player(P0).library.last().copied(), Some(t.g.current(champ)));
        }
    }
}

#[test]
fn champion_of_stray_souls_cant_target_the_creatures_it_sacrifices() {
    cr!("602.2b", "601.2c", "601.2h");
    ruling!(
        "Champion of Stray Souls",
        "You choose the targets of the first ability as you activate that ability, before you pay any costs. You can't target any of the creatures you sacrifice."
    );
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Champion of Stray Souls");
    t.lands(P0, "Swamp", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, champ, 0, &[obj(giant)]).unwrap();
    // Only the Giant was offered: the Bears were still on the battlefield.
    assert_eq!(target_candidates(&t, P0, from), vec![vec![obj(giant)]]);
    assert!(in_graveyard(&t, bears));
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert!(in_graveyard(&t, bears));
}

#[test]
fn bookwurm_isnt_put_into_the_library_if_it_left_the_graveyard() {
    cr!("400.7");
    ruling!(
        "Bookwurm",
        "If Bookwurm is removed from your graveyard in response to its last ability, it won't be put into your library."
    );
    supported("Bookwurm");
    supported("Tormod's Crypt");
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P0, "Bookwurm");
    t.lands(P0, "Forest", 3);
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    let lib = t.library_size(P0);
    t.activate(P0, wurm, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    // In response, P1 exiles P0's graveyard.
    t.activate(P1, crypt, 0, &[pl(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(wurm), Zone::Exile);
    assert_eq!(t.library_size(P0), lib);
}

#[test]
fn screaming_swarm_can_target_its_controller() {
    cr!("115.1", "701.17a");
    ruling!(
        "Screaming Swarm",
        "You may target yourself with Screaming Swarm's triggered ability."
    );
    supported("Screaming Swarm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Screaming Swarm");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let lib = (t.library_size(P0), t.library_size(P1));
    t.answer_targets(P0, &[pl(P0)]);
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib.0 - 2);
    assert_eq!(t.library_size(P1), lib.1);
}

#[test]
fn biophagus_mana_doubled_gives_each_creature_a_counter() {
    cr!("106.6", "106.6a", "106.12");
    ruling!(
        "Biophagus",
        "If Biophagus's ability produces more than one mana, perhaps due to an effect such as that of Mana Reflection, each creature cast with that mana will enter the battlefield with an additional +1/+1 counter."
    );
    supported("Biophagus");
    // Two Llanowar Elves, each cast with one of the two green mana.
    let mut t = TestGame::new(2);
    let bio = t.battlefield(P0, "Biophagus");
    t.battlefield(P0, "Mana Reflection");
    let e1 = t.hand(P0, "Llanowar Elves");
    let e2 = t.hand(P0, "Llanowar Elves");
    let e1 = t.cast(P0, e1).go();
    assert!(t.obj_now(bio).tapped);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 1);
    t.resolve_all();
    let e2 = t.cast(P0, e2).go();
    t.resolve_all();
    assert_eq!(t.counters(e1, counters::PLUS1), 1);
    assert_eq!(t.counters(e2, counters::PLUS1), 1);
    // Both mana spent on Grizzly Bears: two counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Biophagus");
    t.battlefield(P0, "Mana Reflection");
    let bears = t.hand(P0, "Grizzly Bears");
    let bears = t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn a_storage_land_is_tapped_for_mana_only_if_it_makes_mana() {
    cr!("106.12", "106.12a");
    ruling!(
        "Bottomless Vault",
        "It is only considered “tapped for mana” if you activate its mana ability and generate mana from it."
    );
    ruling!(
        "Dwarven Hold",
        "It is only considered “tapped for mana” if you activate its mana ability and generate mana from it."
    );
    ruling!(
        "Hollow Trees",
        "It is only considered “tapped for mana” if you activate its mana ability and generate mana from it."
    );
    ruling!(
        "Icatian Store",
        "It is only considered “tapped for mana” if you activate its mana ability and generate mana from it."
    );
    ruling!(
        "Sand Silos",
        "It is only considered “tapped for mana” if you activate its mana ability and generate mana from it."
    );
    supported("Manabarbs");
    // Manabarbs: "Whenever a player taps a land for mana, Manabarbs deals 1 damage to
    // that player."
    for land in [
        "Bottomless Vault",
        "Dwarven Hold",
        "Hollow Trees",
        "Icatian Store",
        "Sand Silos",
    ] {
        supported(land);
        for removed in [0usize, 2] {
            let mut t = TestGame::new(2);
            t.battlefield(P1, "Manabarbs");
            let l = t.battlefield(P0, land);
            t.g.add_counters(obj(l), "storage", 2, None);
            t.answer(P0, DecisionKind::X, Answer::Number(removed as i64));
            t.activate(P0, l, 0, &[]).unwrap();
            t.settle();
            t.resolve_all();
            assert!(t.obj_now(l).tapped, "{land}");
            assert_eq!(t.g.player(P0).mana_pool.total(), removed, "{land}");
            let expected = if removed == 0 { 20 } else { 19 };
            assert_eq!(t.life(P0), expected, "{land}, {removed} removed");
        }
    }
}
