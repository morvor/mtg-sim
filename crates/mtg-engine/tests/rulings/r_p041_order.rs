//! Rulings batch P041 — damage prevention interacting with other replacement effects:
//! halving (Ghosts of the Innocent), doubling, +1, "instead" replacements, and the
//! order the affected player or controller chooses (CR 616.1).

use crate::r_p041_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Ghosts of the Innocent -----------------------------------------------------------------

#[test]
fn ghosts_halves_one_damage_to_nothing() {
    cr!("614.1a", "120.8");
    ruling!(
        "Ghosts of the Innocent",
        "Half of 1 rounded down is 0. A source that would deal 1 damage won’t deal damage at all."
    );
    supported("Ghosts of the Innocent");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ghosts of the Innocent");
    let bears = t.battlefield(P0, "Grizzly Bears");
    damage(&mut t, bears, pl(P1), 1);
    assert_eq!(t.life(P1), 20);
    assert!(damage_events(&t).is_empty(), "no damage was dealt at all");
    // The effect does apply here: 3 damage becomes 1.
    damage(&mut t, bears, pl(P1), 3);
    assert_eq!(t.life(P1), 19);
    assert_eq!(damage_events(&t), vec![(bears, pl(P1), 1)]);
}

#[test]
fn ghosts_are_cumulative() {
    cr!("616.1e", "616.1f");
    ruling!(
        "Ghosts of the Innocent",
        "Multiple Ghosts of the Innocent effects are cumulative, and each will halve the damage, rounded down. For example, with three on the battlefield, 14 damage becomes 7, then 3, then finally 1"
    );
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Ghosts of the Innocent");
    }
    let bears = t.battlefield(P0, "Grizzly Bears");
    damage(&mut t, bears, pl(P1), 14);
    assert_eq!(t.life(P1), 19);
    assert_eq!(damage_events(&t), vec![(bears, pl(P1), 1)]);
}

#[test]
fn ghosts_and_a_prevention_shield_apply_in_the_recipients_order() {
    cr!("616.1", "616.1e", "615.7");
    ruling!(
        "Ghosts of the Innocent",
        "If a damage prevention effect and Ghosts of the Innocent’s effect would apply to the same damage, the player or the controller of the creature being dealt damage may apply the effects in either order"
    );
    supported("Benevolent Ancestor");
    // P1 has a shield preventing the next 1 damage; P0's Bears would deal 3 to P1.
    // Halving first: 3 -> 1 -> prevented. Prevention first: 3 -> 2 -> 1.
    for (ghosts_first, life) in [(true, 20), (false, 19)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ghosts of the Innocent");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let ancestor = t.battlefield(P1, "Benevolent Ancestor");
        activate_resolve(&mut t, P1, ancestor, 0, &[pl(P1)]);
        let from = t.asked().len();
        let log = if ghosts_first {
            apply_first(&mut t, P1, "Ghosts")
        } else {
            apply_first_not(&mut t, P1, "Ghosts")
        };
        damage(&mut t, bears, pl(P1), 3);
        assert_eq!(t.life(P1), life, "Ghosts first: {ghosts_first}");
        assert_eq!(replacement_choosers(&t, from), vec![P1]);
        assert_eq!(log.lock().unwrap()[0].len(), 2);
    }
}

#[test]
fn ghosts_and_furnace_of_rath_apply_in_the_affected_controllers_order() {
    cr!("616.1", "616.1e", "616.1f");
    ruling!(
        "Ghosts of the Innocent",
        "If both Ghosts of the Innocent and Furnace of Rath (which doubles damage) are on the battlefield, the controller of the permanent being dealt damage or the player being dealt damage can apply the effects in either order."
    );
    supported("Furnace of Rath");
    // 3 damage to P1's Craw Wurm (6/4) from P0's Bears: halving then doubling gives 2;
    // doubling then halving gives 3.
    for (ghosts_first, dmg) in [(true, 2), (false, 3)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ghosts of the Innocent");
        t.battlefield(P0, "Furnace of Rath");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wurm = t.battlefield(P1, "Craw Wurm");
        let from = t.asked().len();
        if ghosts_first {
            apply_first(&mut t, P1, "Ghosts");
        } else {
            apply_first_not(&mut t, P1, "Ghosts");
        }
        damage(&mut t, bears, obj(wurm), 3);
        assert_eq!(damage_marked(&t, wurm), dmg, "Ghosts first: {ghosts_first}");
        assert_eq!(replacement_choosers(&t, from), vec![P1]);
        // A player dealt damage chooses too: 3 to P1 directly.
        let from = t.asked().len();
        damage(&mut t, bears, pl(P1), 3);
        assert_eq!(t.life(P1), 20 - dmg as i32);
        assert_eq!(replacement_choosers(&t, from), vec![P1]);
    }
}

#[test]
fn ghosts_halves_redirected_damage_only_once() {
    cr!("614.5", "614.9");
    ruling!(
        "Ghosts of the Innocent",
        "If damage is redirected, it’s only halved once."
    );
    supported("Pariah");
    // Pariah: "All damage that would be dealt to you is dealt to enchanted creature
    // instead." P1's 5 damage to P0 goes to P0's Craw Wurm, halved once: 2.
    for ghosts_first in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ghosts of the Innocent");
        let wurm = t.battlefield(P0, "Craw Wurm");
        let pariah = t.battlefield(P0, "Pariah");
        assert!(t.g.attach(pariah, obj(wurm)));
        t.g.recompute();
        let giant = t.battlefield(P1, "Hill Giant");
        if ghosts_first {
            apply_first(&mut t, P0, "Ghosts");
        } else {
            apply_first_not(&mut t, P0, "Ghosts");
        }
        damage(&mut t, giant, pl(P0), 5);
        assert_eq!(t.life(P0), 20);
        assert_eq!(damage_marked(&t, wurm), 2, "Ghosts first: {ghosts_first}");
    }
}

// --- Rem Karolus, Stalwart Slayer -----------------------------------------------------------

#[test]
fn rem_karolus_and_a_prevention_shield_in_the_recipients_order() {
    cr!("616.1", "616.1e", "615.7");
    ruling!(
        "Rem Karolus, Stalwart Slayer",
        "the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Rem's effect no longer applies."
    );
    supported("Rem Karolus, Stalwart Slayer");
    supported("Remedy");
    // P1 prevents the next 3 damage to themselves (Remedy, the other 2 to a creature).
    // P0's Lightning Bolt: prevention first prevents all 3 and Rem no longer applies;
    // Rem first makes it 4, of which 3 is prevented.
    for (prevent_first, life) in [(true, 20), (false, 19)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Rem Karolus, Stalwart Slayer");
        let bears = t.battlefield(P1, "Grizzly Bears");
        lands_for_cost(&mut t, P1, "Remedy");
        let remedy = t.hand(P1, "Remedy");
        t.answer_targets(P1, &[pl(P1), obj(bears)]);
        divide(&mut t, P1, &[3, 2]);
        t.cast_with(P1, remedy, &[]).unwrap();
        t.resolve_all();
        let from = t.asked().len();
        if prevent_first {
            apply_first(&mut t, P1, "Remedy");
        } else {
            apply_first_not(&mut t, P1, "Remedy");
        }
        cast_resolve(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
        assert_eq!(t.life(P1), life, "prevention first: {prevent_first}");
        assert_eq!(replacement_choosers(&t, from), vec![P1]);
        assert_eq!(damage_events(&t).is_empty(), prevent_first);
    }
}

#[test]
fn rem_karolus_adds_one_to_each_part_of_divided_damage() {
    cr!("601.2d", "614.1a");
    ruling!(
        "Rem Karolus, Stalwart Slayer",
        "divide the original amount before adding 1."
    );
    supported("Forked Bolt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rem Karolus, Stalwart Slayer");
    let wurm = t.battlefield(P1, "Craw Wurm");
    lands_for_cost(&mut t, P0, "Forked Bolt");
    let bolt = t.hand(P0, "Forked Bolt");
    t.answer_targets(P0, &[obj(wurm), pl(P1)]);
    divide(&mut t, P0, &[1, 1]);
    t.cast_with(P0, bolt, &[]).unwrap();
    t.resolve_all();
    assert_eq!(damage_marked(&t, wurm), 2);
    assert_eq!(t.life(P1), 18);
}

// --- Phytohydra, Phyrexian Hydra ------------------------------------------------------------

#[test]
fn phytohydra_controller_orders_its_replacement_and_a_prevention_shield() {
    cr!("616.1", "616.1e", "614.1a");
    ruling!(
        "Phytohydra",
        "If another effect would prevent damage from being dealt to Phytohydra or replace it with something else, Phytohydra's controller chooses which effect to apply first."
    );
    // A shield prevents the next 1 damage to Phytohydra; P1's Giant would deal it 3.
    for (shield_first, counters) in [(true, 2), (false, 3)] {
        let mut t = TestGame::new(2);
        let hydra = t.battlefield(P0, "Phytohydra");
        let ancestor = t.battlefield(P0, "Benevolent Ancestor");
        activate_resolve(&mut t, P0, ancestor, 0, &[obj(hydra)]);
        let giant = t.battlefield(P1, "Hill Giant");
        let from = t.asked().len();
        if shield_first {
            apply_first(&mut t, P0, "Benevolent");
        } else {
            apply_first(&mut t, P0, "Phytohydra");
        }
        damage(&mut t, giant, obj(hydra), 3);
        assert_eq!(t.counters(hydra, "+1/+1"), counters, "shield first: {shield_first}");
        assert_eq!(damage_marked(&t, hydra), 0);
        assert_eq!(replacement_choosers(&t, from), vec![P0]);
    }
}

#[test]
fn phyrexian_hydra_controller_chooses_among_prevention_effects() {
    cr!("616.1", "616.1e", "615.5");
    ruling!(
        "Phyrexian Hydra",
        "If multiple damage prevention effects, including Phyrexian Hydra’s, are attempting to prevent damage that would be dealt to Phyrexian Hydra, its controller chooses which to apply."
    );
    supported("Phyrexian Hydra");
    // The shield prevents 1 of 3 first (2 counters), or the Hydra prevents all 3.
    for (shield_first, counters) in [(true, 2), (false, 3)] {
        let mut t = TestGame::new(2);
        let hydra = t.battlefield(P0, "Phyrexian Hydra");
        let ancestor = t.battlefield(P0, "Benevolent Ancestor");
        activate_resolve(&mut t, P0, ancestor, 0, &[obj(hydra)]);
        let giant = t.battlefield(P1, "Hill Giant");
        let from = t.asked().len();
        if shield_first {
            apply_first(&mut t, P0, "Benevolent");
        } else {
            apply_first(&mut t, P0, "Phyrexian Hydra");
        }
        damage(&mut t, giant, obj(hydra), 3);
        assert_eq!(t.counters(hydra, "-1/-1"), counters, "shield first: {shield_first}");
        assert_eq!(damage_marked(&t, hydra), 0);
        assert_eq!(replacement_choosers(&t, from), vec![P0]);
    }
}

// --- Ajani Steadfast's emblem, Ria Ivor -----------------------------------------------------

#[test]
fn ajani_emblem_and_dictate_of_the_twin_gods_in_the_recipients_order() {
    cr!("616.1", "616.1e", "114.1");
    ruling!(
        "Ajani Steadfast",
        "You can either (a) prevent all but 1 damage first and then let Dictate of the Twin Gods’s effect double that, for a result of being dealt 2 damage, or (b) double the damage to 6 and then prevent all but 1 of that damage."
    );
    supported("Dictate of the Twin Gods");
    for (prevent_first, life) in [(true, 18), (false, 19)] {
        let mut t = TestGame::new(2);
        let ajani = t.battlefield(P0, "Ajani Steadfast");
        set_loyalty(&mut t, ajani, 7);
        activate_resolve(&mut t, P0, ajani, 2, &[]);
        t.battlefield(P1, "Dictate of the Twin Gods");
        let giant = t.battlefield(P1, "Hill Giant");
        let from = t.asked().len();
        if prevent_first {
            apply_first_not(&mut t, P0, "Dictate");
        } else {
            apply_first(&mut t, P0, "Dictate");
        }
        damage(&mut t, giant, pl(P0), 3);
        assert_eq!(t.life(P0), life, "prevention first: {prevent_first}");
        assert_eq!(replacement_choosers(&t, from), vec![P0]);
    }
}

#[test]
fn ria_ivor_and_a_damage_doubler_in_the_damaged_players_order() {
    cr!("616.1", "616.1e", "615.5");
    ruling!(
        "Ria Ivor, Bane of Bladehold",
        "If multiple replacement or prevention effects try to modify damage that would be dealt to a player, the player being dealt damage chooses the order in which they apply."
    );
    supported("Ria Ivor, Bane of Bladehold");
    // Ria's shield on P0's Bears (2 power); Furnace of Rath doubles damage. P1 chooses:
    // prevention first prevents 2 (2 Mites); doubling first makes it 4 (4 Mites).
    for (prevent_first, mites) in [(true, 2), (false, 4)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Ria Ivor, Bane of Bladehold");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Furnace of Rath");
        t.answer_targets(P0, &[obj(bears)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        let from = t.asked().len();
        if prevent_first {
            apply_first(&mut t, P1, "Ria Ivor");
        } else {
            apply_first(&mut t, P1, "Furnace");
        }
        attack_with(&mut t, &at_p1(&[bears]));
        block_and_finish(&mut t, P1, &[]);
        assert_eq!(t.life(P1), 20);
        assert_eq!(
            crate::r_p108_common::tokens_with(&t, P0, "Mite"),
            mites,
            "prevention first: {prevent_first}"
        );
        assert!(replacement_choosers(&t, from).contains(&P1));
        assert!(!replacement_choosers(&t, from).contains(&P0));
    }
}
