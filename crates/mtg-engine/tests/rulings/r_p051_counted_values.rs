//! Rulings batch P051 — values a drain counts as it resolves (CR 608.2h): the number of
//! artifacts, colors, Dogs, Cats, Zombies or creatures, the cards in hands, the greatest
//! power; once counted, a value doesn't change. Activation restrictions are checked only
//! on activation (CR 602.5b). A creature's power as it last existed (CR 608.2h, 603.10a).

use crate::r_p051_common::*;
use crate::r_p057_common::pump;
use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

const BEARS: &str = "Grizzly Bears";

fn loyalty(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.objects[id.0 as usize]
        .counters
        .insert("loyalty".into(), n);
}

#[test]
fn rin_and_seri_counts_dogs_and_cats_on_resolution_including_itself() {
    cr!("608.2h");
    ruling!(
        "Rin and Seri, Inseparable",
        "The amount of damage Rin and Seri's last ability deals is determined only as that ability resolves. If Rin and Seri is still on the battlefield, it'll count itself."
    );
    supported("Rin and Seri, Inseparable");
    let mut t = TestGame::new(2);
    let rs = t.battlefield(P0, "Rin and Seri, Inseparable");
    assert!(activate_text(&mut t, P0, rs, "Dogs you control", &[pl(P1)]));
    // A Dog (Spirited Companion) in response.
    t.battlefield(P0, "Spirited Companion");
    t.resolve_all();
    // Two Dogs (Rin and Seri and the Companion); one Cat (Rin and Seri).
    assert_eq!(lives(&t, 2), vec![21, 18]);
}

#[test]
fn tezzeret_agent_of_bolas_counts_artifacts_on_resolution() {
    cr!("608.2h", "606.4");
    ruling!(
        "Tezzeret, Agent of Bolas",
        "The number of artifacts you control is counted when the last ability resolves, not when it is activated."
    );
    supported("Tezzeret, Agent of Bolas");
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret, Agent of Bolas");
    loyalty(&mut t, tez, 4);
    t.battlefield(P0, "Ornithopter");
    t.activate(P0, tez, 2, &[pl(P1)]).unwrap();
    t.battlefield(P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![24, 16]);
}

#[test]
fn underhanded_designs_checks_artifacts_only_on_activation() {
    cr!("602.5b");
    ruling!(
        "Underhanded Designs",
        "The number of artifacts you control is checked only as you activate Underhanded Designs's last ability. It's not checked again as it resolves."
    );
    supported("Underhanded Designs");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Underhanded Designs");
    let target = t.battlefield(P1, BEARS);
    // One artifact: it can't be activated.
    let a = t.battlefield(P0, "Ornithopter");
    assert!(!activate_text(&mut t, P0, d, "Destroy", &[obj(target)]));
    t.clear_answers();
    t.battlefield(P0, "Ornithopter");
    assert!(activate_text(&mut t, P0, d, "Destroy", &[obj(target)]));
    // One of the artifacts leaves in response: it still resolves.
    move_to(&mut t, a, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.in_graveyard(P1, BEARS));
}

#[test]
fn shimmercreep_counts_colors_on_resolution() {
    cr!("608.2h", "105.2");
    ruling!(
        "Shimmercreep",
        "The value of X is calculated only once, as Shimmercreep's last ability resolves."
    );
    supported("Shimmercreep");
    let mut t = TestGame::new(2);
    t.enter(P0, "Shimmercreep");
    t.settle();
    // A red permanent in response.
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![22, 18]);
    // Counted once: losing the red permanent later changes nothing.
    move_to(&mut t, goblin, Zone::Graveyard(P0));
    assert_eq!(lives(&t, 2), vec![22, 18]);
}

#[test]
fn niv_mizzet_guildpact_counts_color_pairs_on_resolution() {
    cr!("608.2h", "510.3a");
    ruling!(
        "Niv-Mizzet, Guildpact",
        "The value of X is checked only once, as Niv-Mizzet, Guildpact's triggered ability resolves."
    );
    supported("Niv-Mizzet, Guildpact");
    let mut t = TestGame::new(2);
    let niv = t.battlefield(P0, "Niv-Mizzet, Guildpact");
    // Green-white.
    t.battlefield(P0, "Watchwolf");
    attack_with(&mut t, &[(niv, pl(P1))]);
    // The trigger's targets: 2 damage to P1, P0 draws.
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P0)]);
    let hand = t.hand_size(P0);
    crate::r_p130_common::to_combat_damage_triggers(&mut t);
    assert_eq!(triggers_on_stack(&t, "color pairs"), 1);
    // Blue-red, in response: two color pairs.
    t.battlefield(P0, "Wee Dragonauts");
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 6 - 2);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn attack_drains_count_on_resolution() {
    cr!("608.2h", "508.1m");
    // Dai Li Agents: "Whenever this creature attacks, each opponent loses X life and you
    // gain X life, where X is the number of creatures you control with +1/+1 counters on
    // them."
    ruling!(
        "Dai Li Agents",
        "The value of X is determined as Dai Li Agents's second ability resolves, not when it is put on the stack."
    );
    supported("Dai Li Agents");
    let mut t = TestGame::new(2);
    let agents = t.battlefield(P0, "Dai Li Agents");
    let bears = t.battlefield(P0, BEARS);
    attack_with(&mut t, &[(agents, pl(P1))]);
    t.g.add_counters(obj(bears), "+1/+1", 1, None);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 19]);

    // Valley Rotcaller: "... where X is the number of other Squirrels, Bats, Lizards, and
    // Rats you control."
    ruling!(
        "Valley Rotcaller",
        "The value of X is determined as Valley Rotcaller’s triggered ability resolves."
    );
    supported("Valley Rotcaller");
    let mut t = TestGame::new(2);
    let rot = t.battlefield(P0, "Valley Rotcaller");
    t.battlefield(P0, "Typhoid Rats");
    attack_with(&mut t, &[(rot, pl(P1))]);
    // A Bat in response.
    t.battlefield(P0, "Vampire Bats");
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![22, 18]);

    // Arabella: "... it deals X damage to each opponent and you gain X life, where X is
    // the number of creatures you control with power 2 or less."
    ruling!(
        "Arabella, Abandoned Doll",
        "The value of X is determined only once, as Arabella's ability resolves."
    );
    supported("Arabella, Abandoned Doll");
    let mut t = TestGame::new(2);
    let arabella = t.battlefield(P0, "Arabella, Abandoned Doll");
    let bears = t.battlefield(P0, BEARS);
    attack_with(&mut t, &[(arabella, pl(P1))]);
    // The Bears get +1/+0 in response: power 3, not counted.
    pump(&mut t, bears, 1, 0);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 19]);
}

#[test]
fn liliana_untouched_by_death_locks_in_x() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Liliana, Untouched by Death",
        "The value of X is determined only as Liliana’s second ability resolves. If the number of Zombies you control changes later, the target creature’s power and toughness won’t change."
    );
    supported("Liliana, Untouched by Death");
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana, Untouched by Death");
    t.battlefield(P0, "Walking Corpse");
    let ogre = t.battlefield(P1, "Craw Wurm");
    t.activate(P0, lili, 1, &[obj(ogre)]).unwrap();
    // A second Zombie in response.
    let z = t.battlefield(P0, "Walking Corpse");
    t.resolve_all();
    assert_eq!(t.pt(ogre), (4, 2));
    // A third one later, and one leaving: no change.
    t.battlefield(P0, "Walking Corpse");
    move_to(&mut t, z, Zone::Graveyard(P0));
    assert_eq!(t.pt(ogre), (4, 2));
}

#[test]
fn essence_harvest_with_no_creatures_on_resolution_drains_nothing() {
    cr!("608.2h");
    ruling!(
        "Essence Harvest",
        "The value of X is determined when Essence Harvest resolves. If you control no creatures that time, no life will be lost or gained."
    );
    supported("Essence Harvest");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    cast_pool(&mut t, P0, "Essence Harvest", &[pl(P1)]);
    move_to(&mut t, wurm, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![20, 20]);
    // The greatest power as it resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, BEARS);
    cast_pool(&mut t, P0, "Essence Harvest", &[pl(P1)]);
    t.battlefield(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![26, 14]);
}

#[test]
fn arterial_flow_checks_for_a_vampire_on_resolution() {
    cr!("608.2h", "608.2c");
    ruling!(
        "Arterial Flow",
        "Whether you control a Vampire is checked as Arterial Flow resolves."
    );
    for (before, after) in [(true, false), (false, true)] {
        let mut t = TestGame::new(2);
        let v = before.then(|| t.battlefield(P0, "Vampire Interloper"));
        cast_pool(&mut t, P0, "Arterial Flow", &[]);
        if let Some(v) = v {
            move_to(&mut t, v, Zone::Graveyard(P0));
        }
        if after {
            t.battlefield(P0, "Vampire Interloper");
        }
        t.resolve_all();
        let drained = if after { vec![22, 18] } else { vec![20, 20] };
        assert_eq!(
            lives(&t, 2),
            drained,
            "vampire before {before}, after {after}"
        );
    }
}

#[test]
fn sword_of_war_and_peace_counts_hands_on_resolution() {
    cr!("608.2h", "510.3a");
    ruling!(
        "Sword of War and Peace",
        "You count the cards in the each player's hand when the triggered ability resolves."
    );
    supported("Sword of War and Peace");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, BEARS);
    let sword = t.battlefield(P0, "Sword of War and Peace");
    t.g.attach(sword, obj(bears));
    t.g.recompute();
    for _ in 0..2 {
        t.hand(P1, BEARS);
    }
    for _ in 0..3 {
        t.hand(P0, BEARS);
    }
    attack_with(&mut t, &[(bears, pl(P1))]);
    crate::r_p130_common::to_combat_damage_triggers(&mut t);
    assert_eq!(triggers_on_stack(&t, "cards in their hand"), 1);
    // In response, P1 gets another card and P0 one more.
    t.hand(P1, BEARS);
    t.hand(P0, BEARS);
    t.resolve_all();
    // 4 combat damage, then 3 for P1's three cards; 4 life for P0's four cards.
    assert_eq!(lives(&t, 2), vec![24, 13]);
}

#[test]
fn power_of_a_creature_that_left_is_its_last_known_power() {
    cr!("608.2h", "603.10a");
    // Dying Wish: "When enchanted creature dies, target player loses X life and you gain X
    // life, where X is its power."
    ruling!(
        "Dying Wish",
        "Use the creature’s power the last time it was on the battlefield to determine how much life the target player loses and how much life you gain."
    );
    supported("Dying Wish");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, BEARS);
    let wish = t.battlefield(P0, "Dying Wish");
    t.g.attach(wish, obj(bears));
    pump(&mut t, bears, 3, 0);
    t.answer_targets(P0, &[pl(P1)]);
    move_to(&mut t, bears, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![25, 15]);

    // Faerie Bladecrafter: "When this creature dies, each opponent loses X life and you
    // gain X life, where X is its power."
    ruling!(
        "Faerie Bladecrafter",
        "Use Faerie Bladecrafter's power from when it was last on the battlefield to determine the value of X."
    );
    supported("Faerie Bladecrafter");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Faerie Bladecrafter");
    t.g.add_counters(obj(f), "+1/+1", 2, None);
    move_to(&mut t, f, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![24, 16]);

    // Ukkima: "When Ukkima leaves the battlefield, it deals X damage to target player and
    // you gain X life, where X is its power."
    ruling!(
        "Ukkima, Stalking Shadow",
        "Use Ukkima's power as it last existed on the battlefield to determine how much damage it deals and how much life you gain. If that number is negative, it deals no damage and you gain no life."
    );
    supported("Ukkima, Stalking Shadow");
    for (boost, change) in [(2, 4), (-5, 0)] {
        let mut t = TestGame::new(2);
        let u = t.battlefield(P0, "Ukkima, Stalking Shadow");
        pump(&mut t, u, boost, 0);
        t.answer_targets(P0, &[pl(P1)]);
        move_to(&mut t, u, Zone::Exile);
        t.resolve_all();
        assert_eq!(lives(&t, 2), vec![20 + change, 20 - change], "{boost}");
    }
}

#[test]
fn ayara_uses_the_sacrificed_permanents_last_known_mana_value() {
    cr!("608.2h", "202.3", "707.2");
    ruling!(
        "Ayara, Widow of the Realm // Ayara, Furnace Queen",
        "The first activated ability of Ayara, Widow of the Realm uses the mana value of the permanent you sacrificed as it last existed on the battlefield, not its mana value in the graveyard."
    );
    supported("Ayara, Widow of the Realm // Ayara, Furnace Queen");
    supported("Clone");
    let mut t = TestGame::new(2);
    let ayara = t.battlefield(P0, "Ayara, Widow of the Realm // Ayara, Furnace Queen");
    t.battlefield(P0, BEARS);
    // Clone (mana value 4) enters as a copy of Grizzly Bears (mana value 2).
    let bears = t.named_on_battlefield(BEARS)[0];
    t.answer_choose(P0, &[obj(bears)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, BEARS);
    t.answer_choose(P0, &[obj(clone)]);
    assert!(activate_text(&mut t, P0, ayara, "Sacrifice", &[pl(P1)]));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Clone"));
    assert_eq!(lives(&t, 2), vec![22, 18]);
}

#[test]
fn monoist_circuit_feeder_counts_artifacts_once() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Monoist Circuit-Feeder",
        "The value of X is calculated only once, as Monoist Circuit-Feeder’s last ability resolves."
    );
    supported("Monoist Circuit-Feeder");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, BEARS);
    let theirs = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(theirs)]);
    t.enter(P0, "Monoist Circuit-Feeder");
    t.settle();
    // An artifact in response: two artifacts (the Feeder is one).
    let o = t.battlefield(P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.pt(mine), (4, 2));
    assert_eq!(t.pt(theirs), (6, 2));
    // Fixed from then on.
    move_to(&mut t, o, Zone::Graveyard(P0));
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    assert_eq!((t.pt(mine), t.pt(theirs)), ((4, 2), (6, 2)));
}
