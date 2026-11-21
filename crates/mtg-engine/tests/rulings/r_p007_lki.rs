//! Rulings batch P007 — sources that leave the battlefield before their damage resolves
//! use last known information (CR 608.2h, 113.7a), values counted as the spell resolves
//! (CR 608.2h), leaves-the-battlefield triggers that look back in time (CR 603.10a), and
//! Chandra, Fire Artisan's loyalty-removal trigger.

use crate::r_p007_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s06_common::activate_containing;
use crate::r_s07_common::damage_on;
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn aria_of_flame_trigger_resolves_first_and_uses_last_known_counters() {
    cr!("603.3", "608.2h", "113.7a");
    ruling!(
        "Aria of Flame",
        "Aria of Flame’s last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Aria of Flame",
        "If Aria of Flame leaves the battlefield while its last ability is on the stack, use the number of verse counters it had on it immediately before it left the battlefield to determine how much damage it deals."
    );
    supported("Aria of Flame");
    // The trigger is above the spell; the spell is countered and the trigger still resolves.
    let mut t = TestGame::new(2);
    let aria = t.battlefield(P0, "Aria of Flame");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let shock = cast_new(&mut t, P0, "Shock", &[obj(wurm)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], shock, "the spell is below the trigger");
    t.g.counter(shock, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.counters(aria, "verse"), 1);
    assert_eq!(dmg(&t, wurm), 0, "Shock was countered");
    // Aria leaves with the trigger on the stack: its last known verse counters count.
    let mut t = TestGame::new(2);
    let aria = t.battlefield(P0, "Aria of Flame");
    t.g.objects[aria.0 as usize]
        .counters
        .insert("verse".into(), 2);
    t.answer_targets(P0, &[pl(P1)]);
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.settle();
    destroy(&mut t, aria);
    t.resolve_all();
    // 2 verse counters from the trigger + 2 from Shock.
    assert_eq!(t.life(P1), 16);
}

#[test]
fn sources_that_left_deal_damage_by_last_known_information() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Goblin Razerunners",
        "If Goblin Razerunners leaves the battlefield after its ability triggers but before it resolves, the amount of damage it deals is equal to the number of +1/+1 counters that were on it as it last existed on the battlefield."
    );
    ruling!(
        "Leafkin Avenger",
        "If Leafkin Avenger leaves the battlefield while its last ability is on the stack, use its power as it last existed on the battlefield to determine how much damage it deals."
    );
    supported("Goblin Razerunners");
    supported("Leafkin Avenger");
    // Goblin Razerunners with two +1/+1 counters, destroyed with its end step trigger on
    // the stack.
    let mut t = TestGame::new(2);
    let gob = t.battlefield(P0, "Goblin Razerunners");
    t.g.objects[gob.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, gob);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Leafkin Avenger (4/3, +1/+1 counter: 5/4) destroyed in response to its ability.
    let mut t = TestGame::new(2);
    let leaf = t.battlefield(P0, "Leafkin Avenger");
    t.g.objects[leaf.0 as usize]
        .counters
        .insert("+1/+1".into(), 1);
    t.lands(P0, "Mountain", 8);
    t.answer_targets(P0, &[pl(P1)]);
    activate_containing(&mut t, P0, leaf, "{7}{R}").expect("activate");
    destroy(&mut t, leaf);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn values_are_counted_as_the_spell_resolves() {
    cr!("608.2h");
    ruling!("Blossoming Wreath", "Creatures are counted on resolution.");
    ruling!("Mob Justice", "Creatures are counted on resolution.");
    ruling!(
        "Immolating Gyre",
        "Because Immolating Gyre is still on the stack while it’s resolving, it doesn’t count towards the value of X."
    );
    ruling!(
        "Blitz of the Thunder-Raptor",
        "Blitz of the Thunder-Raptor is still on the stack while you determine how much damage is dealt. It isn’t in your graveyard to contribute to that amount of damage."
    );
    // Blossoming Wreath: a creature card put into the graveyard in response counts.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Blossoming Wreath", &[]);
    t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Mob Justice: a creature entering in response counts; one leaving doesn't.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Mob Justice", &[pl(P1)]);
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Hill Giant");
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Immolating Gyre: two sorceries in the graveyard, the Gyre itself not counted.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lava Spike");
    t.graveyard(P0, "Lava Spike");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Immolating Gyre", &[]);
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 2);
    // Blitz: one instant in the graveyard, Blitz itself not counted.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Shock");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Blitz of the Thunder-Raptor", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 1);
}

#[test]
fn charge_of_the_forever_beast_uses_the_revealed_cards_power() {
    cr!("608.2h", "601.2b");
    ruling!(
        "Charge of the Forever-Beast",
        "As Charge of the Forever-Beast resolves, it deals damage based on the revealed card’s power if it’s still in your hand. If that card has left your hand, Charge of the Forever-Beast deals damage based on the card’s power immediately before it left your hand."
    );
    for discard in [false, true] {
        let mut t = TestGame::new(2);
        let wurm = t.hand(P0, "Craw Wurm");
        t.hand(P0, "Grizzly Bears");
        let maw = t.battlefield(P1, "Colossal Dreadmaw");
        t.answer_choose(P0, &[obj(wurm)]);
        cast_new(&mut t, P0, "Charge of the Forever-Beast", &[obj(maw)]);
        assert_eq!(t.zone(wurm), mtg_engine::object::Zone::Hand(P0));
        if discard {
            t.g.discard(P0, wurm, None);
            t.g.flush_events();
        }
        t.resolve_all();
        assert!(!t.on_battlefield(maw), "discard {discard}: 6 damage");
    }
}

#[test]
fn cragganwick_cremator_uses_the_power_in_the_graveyard() {
    cr!("608.2h", "604.3");
    ruling!(
        "Cragganwick Cremator",
        "Cragganwick Cremator deals damage equal to the discarded creature card's power as it exists in your graveyard."
    );
    supported("Cragganwick Cremator");
    // Tarmogoyf: 0/1 in hand with empty graveyards, 1/2 in the graveyard (a creature card
    // is there now).
    let mut t = TestGame::new(2);
    t.hand(P0, "Tarmogoyf");
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Cragganwick Cremator");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Tarmogoyf"));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn dies_triggers_see_creatures_dying_at_the_same_time() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Hissing Iguanar",
        "If Hissing Iguanar dies at the same time as another creature, Hissing Iguanar's ability triggers."
    );
    ruling!(
        "Rage Thrower",
        "If Rage Thrower dies at the same time as another creature, its ability will trigger."
    );
    ruling!(
        "Spiteful Prankster",
        "If another creature dies at the same time as Spiteful Prankster, Spiteful Prankster’s last ability triggers for that creature."
    );
    for (name, n) in [
        ("Hissing Iguanar", 1),
        ("Rage Thrower", 2),
        ("Spiteful Prankster", 1),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[pl(P1)]);
        cast_new(&mut t, P0, "Day of Judgment", &[]);
        t.resolve();
        assert_eq!(triggers_on_stack(&t, "dies"), 1, "{name}");
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - n, "{name}");
    }
}

#[test]
fn chandra_fire_artisan_trigger_on_loyalty_removal() {
    cr!("603.2", "603.10a", "306.8", "606.4");
    ruling!(
        "Chandra, Fire Artisan",
        "Chandra's first ability triggers if she loses loyalty from being dealt damage, from her last loyalty ability being activated, or because an effect removes loyalty counters from her. If an effect simply causes Chandra to leave the battlefield, no counters were removed and so her ability doesn't trigger."
    );
    ruling!(
        "Chandra, Fire Artisan",
        "If all of Chandra's loyalty counters are removed, her first ability has her deal damage even though she's left the battlefield."
    );
    supported("Chandra, Fire Artisan");
    // Dealt 2 damage: 2 to the opponent.
    let mut t = TestGame::new(2);
    let c = pw(&mut t, P0, "Chandra, Fire Artisan", 4);
    let bolt = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    t.g.deal_damage(bolt, obj(c), 2, false);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.counters(c, "loyalty"), 2);
    // Her −7: seven counters are removed as a cost: 7 damage.
    let mut t = TestGame::new(2);
    let c = pw(&mut t, P0, "Chandra, Fire Artisan", 8);
    t.answer_targets(P0, &[pl(P1)]);
    loyalty(&mut t, P0, c, 1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
    // An effect removes counters: 3 damage.
    let mut t = TestGame::new(2);
    let c = pw(&mut t, P0, "Chandra, Fire Artisan", 4);
    t.answer_targets(P0, &[pl(P1)]);
    t.g.remove_counters(obj(c), "loyalty", 3);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Destroyed: no counters were removed, no trigger.
    let mut t = TestGame::new(2);
    let c = pw(&mut t, P0, "Chandra, Fire Artisan", 4);
    t.answer_targets(P0, &[pl(P1)]);
    destroy(&mut t, c);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // All four counters removed by damage: she's put into the graveyard and still deals 4.
    let mut t = TestGame::new(2);
    let c = pw(&mut t, P0, "Chandra, Fire Artisan", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    t.g.deal_damage(bears, obj(c), 5, false);
    t.g.flush_events();
    t.settle();
    assert!(!t.on_battlefield(c));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}
