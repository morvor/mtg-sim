//! Rulings batch P042 — damage-reducing effects among themselves: "prevent all but 1" and
//! "prevent N" effects ordered by the affected permanent's controller (CR 616.1), damage
//! replacements that aren't prevention (CR 614.1a vs 615.12), +1/+1 counters put on as the
//! damage is prevented (Test of Faith, Temper), Rem Karolus's spell damage, and combat
//! damage prevention tied to a permanent or counter (Goldbug, Woolly Razorback, Pack
//! Leader, Swans of Bryn Argoll, Uncle Istvan, Wall of Putrid Flesh).

use crate::r_p042_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Counters for prevented damage ------------------------------------------------------------

#[test]
fn test_of_faith_counters_arrive_with_the_damage() {
    cr!("615.5", "704.5g");
    ruling!(
        "Test of Faith",
        "The +1/+1 counters are put onto the creature at the same time the damage is prevented. If a 1/1 creature would be dealt 6 damage, 3 damage is prevented and three +1/+1 counters are put on the creature. The creature ends up as a 4/4 creature with 3 damage marked on it."
    );
    supported("Test of Faith");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Test of Faith", &[obj(elves)]);
    damage(&mut t, giant, obj(elves), 6);
    assert!(t.on_battlefield(elves));
    assert_eq!(t.pt(elves), (4, 4));
    assert_eq!(damage_marked(&t, elves), 3);
}

#[test]
fn temper_counters_are_there_before_lethal_damage_is_checked_and_count_only_prevented_damage() {
    cr!("615.5", "704.5g");
    ruling!(
        "Temper",
        "The +1/+1 counters do show up at the same time as any unprevented damage, which means they are there before you check for lethal damage."
    );
    ruling!(
        "Temper",
        "You can make X be larger than the amount of damage, but you only get one counter for each actual point of damage prevented."
    );
    supported("Temper");
    /// P0 casts Temper with X = `x` on P0's Grizzly Bears.
    fn temper(t: &mut TestGame, x: i64) -> ObjectId {
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Plains", 2 + x as usize);
        let card = t.hand(P0, "Temper");
        t.cast(P0, card).target(obj(bears)).x(x).go();
        t.resolve_all();
        bears
    }
    // X = 1, 3 damage to the 2/2: 1 prevented, it's a 3/3 with 2 damage and survives.
    let mut t = TestGame::new(2);
    let bears = temper(&mut t, 1);
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, obj(bears), 3);
    assert!(t.on_battlefield(bears));
    assert_eq!((t.pt(bears), damage_marked(&t, bears)), ((3, 3), 2));
    // X = 5, 1 damage: one counter.
    let mut t = TestGame::new(2);
    let bears = temper(&mut t, 5);
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, obj(bears), 1);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(damage_marked(&t, bears), 0);
    // The rest of the shield (4) is still there: 2 more damage, 2 more counters.
    damage(&mut t, giant, obj(bears), 2);
    assert_eq!(t.counters(bears, "+1/+1"), 3);
}

// --- Replacements that aren't prevention ---------------------------------------------------

#[test]
fn forethought_amulet_doesnt_change_damage_of_2_or_less() {
    cr!("614.1a");
    ruling!(
        "Forethought Amulet",
        "This does not reduce damage which is already 2 or 1."
    );
    supported("Forethought Amulet");
    supported("Lightning Bolt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forethought Amulet");
    t.set_step(P1, Step::PrecombatMain);
    let shock = cast_resolve(&mut t, P1, "Shock", &[pl(P0)]);
    assert_eq!(damage_events(&t), vec![(shock, pl(P0), 2)]);
    let bolt = cast_resolve(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    assert_eq!(damage_events(&t)[1], (bolt, pl(P0), 2), "3 becomes 2");
    assert_eq!(t.life(P0), 16);
}

#[test]
fn divine_presence_and_ghosts_reduce_damage_that_cant_be_prevented() {
    cr!("614.1a", "615.12");
    ruling!(
        "Divine Presence",
        "This is not preventing damage since it does not use the word “prevent”. It is a replacement."
    );
    ruling!(
        "Ghosts of the Innocent",
        "This isn’t a damage prevention effect. If Excruciator (“Damage that would be dealt by Excruciator can’t be prevented”) would deal 7 damage to a permanent or player, it deals 3 damage instead."
    );
    supported("Divine Presence");
    supported("Excruciator");
    for (card, dealt) in [("Divine Presence", 3), ("Ghosts of the Innocent", 3)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        let ex = t.battlefield(P1, "Excruciator");
        damage(&mut t, ex, pl(P0), 7);
        assert_eq!(damage_events(&t), vec![(ex, pl(P0), dealt)], "{card}");
        assert!(prevented_events(&t).is_empty(), "{card}");
        assert_eq!(t.life(P0), 17, "{card}");
        // A prevention effect doesn't apply to Excruciator's damage.
        let mut t = TestGame::new(2);
        let ex = t.battlefield(P1, "Excruciator");
        t.answer_choose(P0, &[obj(ex)]);
        cast_resolve(&mut t, P0, "Pay No Heed", &[]);
        damage(&mut t, ex, pl(P0), 7);
        assert_eq!(t.life(P0), 13);
    }
}

// --- Ordering prevention and other damage replacements ----------------------------------------

#[test]
fn temple_altisaur_and_another_prevention_effect_in_the_controllers_order() {
    cr!("616.1", "616.1e", "615.7");
    ruling!(
        "Temple Altisaur",
        "If multiple replacement and/or prevention effects could apply to a Dinosaur you control, you choose the order in which to apply those effects. For example, if another effect will prevent 1 damage that would be dealt to a Dinosaur, you may apply Temple Altisaur's effect to prevent all but 1 of that damage and then apply the other effect to prevent that 1 damage."
    );
    supported("Temple Altisaur");
    supported("Benevolent Ancestor");
    // 5 damage to P0's Relentless Raptor (3/3 Dinosaur) with a 1-damage shield on it.
    for (altisaur_first, dealt) in [(true, 0), (false, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Temple Altisaur");
        let raptor = t.battlefield(P0, "Relentless Raptor");
        let ancestor = t.battlefield(P0, "Benevolent Ancestor");
        activate_resolve(&mut t, P0, ancestor, 0, &[obj(raptor)]);
        let giant = t.battlefield(P1, "Hill Giant");
        let from = t.asked().len();
        if altisaur_first {
            apply_first(&mut t, P0, "Temple Altisaur");
        } else {
            apply_first_not(&mut t, P0, "Temple Altisaur");
        }
        damage(&mut t, giant, obj(raptor), 5);
        assert_eq!(damage_marked(&t, raptor), dealt, "Altisaur first: {altisaur_first}");
        assert_eq!(replacement_choosers(&t, from), vec![P0]);
    }
}

#[test]
fn temple_altisaur_trample_damage_to_a_blocking_dinosaur() {
    cr!("702.19b", "615.1a");
    ruling!(
        "Temple Altisaur",
        "Combat damage can be assigned to other Dinosaurs you control as normal, but most of that damage will be prevented. For example, if a 3/3 Dinosaur you control blocks a 5/5 creature with trample, the attacking player may assign 2 of that creature's combat damage to the player or planeswalker it's attacking and Temple Altisaur will prevent 2 of the 3 damage assigned to the defending Dinosaur."
    );
    supported("Colossal Dreadmaw");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Temple Altisaur");
    let raptor = t.battlefield(P0, "Relentless Raptor");
    // Colossal Dreadmaw: 6/6 trample.
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    to_beginning_of_combat(&mut t, P1);
    t.attack(&[(maw, pl(P0))], &[(raptor, maw)]);
    assert_eq!(combat_damage(&t, maw, pl(P0)), 3, "lethal (3) assigned to the Raptor");
    assert_eq!(prevented_events(&t), vec![(maw, obj(raptor), 2)]);
    assert_eq!(damage_marked(&t, raptor), 1);
    assert!(t.on_battlefield(raptor));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn absorb_and_shield_of_the_realm_ordered_by_the_damaged_creatures_controller() {
    cr!("616.1", "616.1e", "702.64a");
    ruling!(
        "Lymph Sliver",
        "If multiple replacement effects apply to a creature being dealt damage, the controller of the creature chooses the order to apply them, not the controller of the source of damage."
    );
    ruling!(
        "Shield of the Realm",
        "If multiple replacement effects would modify how damage would be dealt, the controller of the permanent being dealt damage chooses the order in which to apply those effects."
    );
    supported("Furnace of Rath");
    // P1's Furnace of Rath doubles; P1's Hill Giant deals 3 to P0's creature.
    // Lymph Sliver (absorb 1): prevent first 3-1=2, x2 = 4; double first 6-1 = 5.
    // Shield of the Realm (prevent 2): prevent first (3-2)x2 = 2; double first 6-2 = 4.
    for (card, furnace_first_dealt, prevention_first_dealt) in
        [("Lymph Sliver", 5u32, 4u32), ("Shield of the Realm", 4, 2)]
    {
        for furnace_first in [true, false] {
            let mut t = TestGame::new(2);
            let victim = if card == "Lymph Sliver" {
                t.battlefield(P0, "Lymph Sliver")
            } else {
                let wurm = t.battlefield(P0, "Craw Wurm");
                let shield = t.battlefield(P0, "Shield of the Realm");
                assert!(t.g.attach(shield, obj(wurm)));
                t.g.recompute();
                wurm
            };
            // Enough toughness to survive and read the damage marked.
            cast_resolve(&mut t, P0, "Giant Growth", &[obj(victim)]);
            t.battlefield(P1, "Furnace of Rath");
            let giant = t.battlefield(P1, "Hill Giant");
            let from = t.asked().len();
            if furnace_first {
                apply_first(&mut t, P0, "Furnace");
            } else {
                apply_first_not(&mut t, P0, "Furnace");
            }
            damage(&mut t, giant, obj(victim), 3);
            let want = if furnace_first {
                furnace_first_dealt
            } else {
                prevention_first_dealt
            };
            assert_eq!(
                damage_marked(&t, victim),
                want,
                "{card}, Furnace first: {furnace_first}"
            );
            assert_eq!(replacement_choosers(&t, from), vec![P0], "{card}");
        }
    }
}

#[test]
fn hedron_field_purists_applies_to_each_recipient_separately() {
    cr!("615.1a");
    ruling!(
        "Hedron-Field Purists",
        "If a source would deal damage to multiple applicable recipients, the prevention effects apply to each of those recipients separately. For example, if Hedron-Field Purists is level 1 and you cast Earthquake (“Earthquake deals X damage to each creature without flying and each player”) with X equal to 3, that Earthquake will deal 2 damage to you, 2 damage to each creature without flying you control, 3 damage to each other player, and 3 damage to each creature without flying other players control."
    );
    supported("Hedron-Field Purists");
    supported("Earthquake");
    let mut t = TestGame::new(2);
    let purists = t.battlefield(P0, "Hedron-Field Purists");
    crate::r_s29_common::put_counters(&mut t, purists, "level", 1);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let theirs = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 4);
    let card = t.hand(P0, "Earthquake");
    t.cast(P0, card).x(3).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(damage_marked(&t, wurm), 2);
    assert_eq!(damage_marked(&t, purists), 2);
    assert_eq!(t.life(P1), 17);
    assert_eq!(damage_marked(&t, theirs), 3);
}

// --- Rem Karolus ---------------------------------------------------------------------------------

#[test]
fn rem_karolus_extra_damage_is_dealt_by_the_original_source() {
    cr!("614.1a");
    ruling!(
        "Rem Karolus, Stalwart Slayer",
        "The additional 1 damage is dealt by the original source of damage. The additional damage isn't dealt by Rem Karolus."
    );
    supported("Rem Karolus, Stalwart Slayer");
    let mut t = TestGame::new(2);
    let rem = t.battlefield(P0, "Rem Karolus, Stalwart Slayer");
    let shock = cast_resolve(&mut t, P0, "Shock", &[pl(P1)]);
    assert_eq!(damage_events(&t), vec![(shock, pl(P1), 3)]);
    assert!(damage_by(&t, rem).is_empty());
}

#[test]
fn rem_karolus_doesnt_apply_to_fight_damage() {
    cr!("701.14a", "614.1a", "615.1a");
    ruling!(
        "Rem Karolus, Stalwart Slayer",
        "Some spells cause other objects to deal damage, such as spells that cause two creatures to fight. Rem Karolus, Stalwart Slayer's last two abilities don't apply to damage from such effects."
    );
    supported("Prey Upon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rem Karolus, Stalwart Slayer");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    // P0's Prey Upon: the Wurm (6/4) fights P1's Giant (3/3).
    cast_resolve(&mut t, P0, "Prey Upon", &[obj(wurm), obj(giant)]);
    assert_eq!(damage_marked(&t, wurm), 3, "not prevented");
    assert!(prevented_events(&t).is_empty());
    assert_eq!(
        damage_events(&t)
            .iter()
            .find(|e| e.1 == obj(giant))
            .map(|e| e.2),
        Some(6),
        "not 7"
    );
}

// --- Desperate Gambit --------------------------------------------------------------------------

#[test]
fn desperate_gambit_flips_as_it_resolves() {
    cr!("705.2", "608.2c");
    ruling!(
        "Desperate Gambit",
        "The coin is flipped during resolution of this spell, so you know in advance what will happen when the source deals damage the next time."
    );
    supported("Desperate Gambit");
    for win in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        t.g.dice.loaded_coins.push_back(win);
        t.answer_choose(P0, &[obj(giant)]);
        cast_new(&mut t, P0, "Desperate Gambit", &[]);
        assert_eq!(t.g.dice.loaded_coins.len(), 1, "not flipped while casting");
        t.resolve_all();
        assert!(t.g.dice.loaded_coins.is_empty(), "flipped as it resolved");
        damage(&mut t, giant, pl(P1), 3);
        assert_eq!(t.life(P1), if win { 14 } else { 20 }, "won: {win}");
    }
}

// --- Prevention tied to a permanent or its counters -----------------------------------------

#[test]
fn pack_leader_leaving_can_make_marked_damage_lethal() {
    cr!("704.5g", "611.3a", "514.2");
    ruling!(
        "Pack Leader",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Dog you control may become lethal if Pack Leader leaves the battlefield during that turn."
    );
    supported("Pack Leader");
    let mut t = TestGame::new(2);
    let leader = t.battlefield(P0, "Pack Leader");
    let dogs = t.battlefield(P0, "Guard Dogs");
    assert_eq!(t.pt(dogs), (3, 3));
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, obj(dogs), 2);
    assert!(t.on_battlefield(dogs));
    exile_now(&mut t, leader);
    assert!(!t.on_battlefield(dogs), "2 damage on a 2/2");
}

#[test]
fn goldbug_dying_to_first_strike_stops_preventing_regular_combat_damage() {
    cr!("510.4", "611.3a");
    ruling!(
        "Goldbug, Humanity's Ally // Goldbug, Scrappy Scout",
        "If Goldbug, Humanity's Ally dies before combat damage is dealt to attacking Humans, the damage won't be prevented. For example, this can occur if Goldbug is blocked by a creature with first strike."
    );
    supported("Goldbug, Humanity's Ally // Goldbug, Scrappy Scout");
    supported("Youthful Knight");
    // Goldbug (3/3) blocked by a 3/3 first striker or by a Hill Giant; the attacking Elite
    // Vanguard (2/1 Human) blocked by Grizzly Bears.
    for first_strike in [false, true] {
        let mut t = TestGame::new(2);
        let goldbug = t.battlefield(P0, "Goldbug, Humanity's Ally // Goldbug, Scrappy Scout");
        let human = t.battlefield(P0, "Elite Vanguard");
        let blocker = if first_strike {
            let k = t.battlefield(P1, "Youthful Knight");
            // Youthful Knight is 2/1 first strike: make it lethal to Goldbug.
            cast_resolve(&mut t, P0, "Giant Growth", &[obj(k)]);
            k
        } else {
            t.battlefield(P1, "Hill Giant")
        };
        let bears = t.battlefield(P1, "Grizzly Bears");
        to_beginning_of_combat(&mut t, P0);
        t.attack(
            &[(goldbug, pl(P1)), (human, pl(P1))],
            &[(blocker, goldbug), (bears, human)],
        );
        assert!(!t.on_battlefield(goldbug), "first strike: {first_strike}");
        assert_eq!(
            t.on_battlefield(human),
            !first_strike,
            "first strike: {first_strike}"
        );
    }
}

#[test]
fn woolly_razorback_deals_damage_once_its_last_ice_counter_is_removed() {
    cr!("510.2", "603.2");
    ruling!(
        "Woolly Razorback",
        "The third time Woolly Razorback blocks, its last ice counter will be removed, meaning the combat damage it deals that combat won't be prevented."
    );
    supported("Woolly Razorback");
    let mut t = TestGame::new(2);
    let boar = t.enter(P0, "Woolly Razorback");
    t.resolve_all();
    assert_eq!(t.counters(boar, "ice"), 3);
    for block in 1..=3 {
        let giant = t.battlefield(P1, "Craw Wurm");
        to_beginning_of_combat(&mut t, P1);
        t.attack(&[(giant, pl(P0))], &[(boar, giant)]);
        assert_eq!(t.counters(boar, "ice"), 3 - block);
        let dealt = combat_damage(&t, boar, obj(giant));
        if block < 3 {
            assert_eq!(dealt, 0, "block {block}: prevented");
            assert!(t.on_battlefield(giant));
            exile_now(&mut t, giant);
        } else {
            assert_eq!(dealt, 7, "the third block");
            assert!(!t.on_battlefield(giant));
        }
        // Clear the turn's events and damage.
        t.advance_to(P0, Step::Upkeep);
    }
}

#[test]
fn swans_of_bryn_argoll_combat_damage_source_is_the_creature() {
    cr!("510.2", "615.5");
    ruling!(
        "Swans of Bryn Argoll",
        "The source of combat damage is the creature that deals it."
    );
    supported("Swans of Bryn Argoll");
    let mut t = TestGame::new(2);
    let swans = t.battlefield(P0, "Swans of Bryn Argoll");
    let giant = t.battlefield(P1, "Hill Giant");
    let hand = t.hand_size(P1);
    to_beginning_of_combat(&mut t, P1);
    t.attack(&[(giant, pl(P0))], &[(swans, giant)]);
    assert_eq!(prevented_events(&t), vec![(giant, obj(swans), 3)]);
    assert_eq!(t.hand_size(P1), hand + 3, "the Giant's controller draws");
}

#[test]
fn uncle_istvan_prevents_damage_from_creatures_activated_abilities() {
    cr!("615.1a", "609.7");
    ruling!(
        "Uncle Istvan",
        "Works even on damage from activated abilities of creatures such as Prodigal Sorcerer."
    );
    let mut t = TestGame::new(2);
    let istvan = t.battlefield(P0, "Uncle Istvan");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.set_step(P1, Step::PrecombatMain);
    activate_resolve(&mut t, P1, sorcerer, 0, &[obj(istvan)]);
    assert_eq!(prevented_events(&t), vec![(sorcerer, obj(istvan), 1)]);
    assert_eq!(damage_marked(&t, istvan), 0);
    // A Shock (not a creature) still damages it.
    cast_resolve(&mut t, P1, "Shock", &[obj(istvan)]);
    assert_eq!(damage_marked(&t, istvan), 2);
}

#[test]
fn wall_of_putrid_flesh_enchanted_creatures_are_those_with_an_aura() {
    cr!("303.4", "615.1a");
    ruling!(
        "Wall of Putrid Flesh",
        "The term “enchanted creatures” means “creatures with an Aura on them”."
    );
    supported("Wall of Putrid Flesh");
    for aura in [false, true] {
        let mut t = TestGame::new(2);
        let wall = t.battlefield(P0, "Wall of Putrid Flesh");
        let giant = t.battlefield(P1, "Hill Giant");
        if aura {
            // P0's own Aura on P1's creature.
            let s = t.battlefield(P0, "Unholy Strength");
            assert!(t.g.attach(s, obj(giant)));
            t.g.recompute();
        }
        damage(&mut t, giant, obj(wall), 2);
        assert_eq!(damage_marked(&t, wall), if aura { 0 } else { 2 }, "aura: {aura}");
    }
}
