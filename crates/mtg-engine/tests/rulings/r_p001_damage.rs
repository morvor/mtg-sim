//! Rulings batch P001 — damage: prevention effects that deal damage of their own instead
//! of redirecting (CR 615.5, 609.7), prevention per spell and per recipient, damage a
//! creature deals (deathtouch, lifelink: CR 702.2b, 702.15b), damage from a spell on the
//! stack (CR 109.2, 208.3), "destroyed this way" (CR 701.8), and a damage trigger on lethal
//! damage (CR 603.2, 704.5g).

use crate::r_p001_common::*;
use crate::r_p006_common::in_graveyard;
use crate::r_s01_common::{supported, tokens};
use crate::r_s04_common::add_mana;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `source` deals `n` combat damage to `target`, then the game settles (triggers resolve).
fn combat_damage(t: &mut TestGame, source: ObjectId, target: Entity, n: u32) {
    t.g.deal_damage(source, target, n, true);
    t.settle();
    t.resolve_all();
}

/// Whether `id` was dealt noncombat damage this turn.
fn dealt_noncombat(t: &TestGame, id: ObjectId) -> bool {
    t.g.history.objects_dealt_noncombat_damage.contains(&id)
}

#[test]
fn channel_harm_deals_new_noncombat_damage_of_its_own() {
    cr!("615.5", "609.7a", "702.2b", "702.15b");
    ruling!(
        "Channel Harm",
        "Channel Harm’s effect is not a redirection effect. If it prevents damage, you may have Channel Harm (not the original source) deal damage to the creature as part of that prevention effect."
    );
    supported("Channel Harm");
    // Vampire Nighthawk (2/3, deathtouch, lifelink) deals combat damage to P0; Channel
    // Harm deals 2 damage to Hill Giant instead: no deathtouch, no lifelink, not combat.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Channel Harm");
    let card = t.hand(P0, "Channel Harm");
    t.cast(P0, card).target(giant).go();
    t.resolve();
    t.answer_yes(P0, true);
    combat_damage(&mut t, hawk, pl(P0), 2);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(giant).damage, 2);
    assert!(t.on_battlefield(giant), "no deathtouch");
    assert_eq!(t.life(P1), 20, "no lifelink");
    assert!(dealt_noncombat(&t, giant));
    assert!(!t
        .g
        .history
        .damage_by_source
        .iter()
        .any(|(s, o)| *s == hawk && *o == giant));
}

#[test]
fn channel_harms_target_is_chosen_as_its_cast() {
    cr!("601.2c", "608.2b", "115.1");
    ruling!(
        "Channel Harm",
        "Channel Harm’s only target is the creature it may deal damage to. You choose that target as you cast Channel Harm, not at the time it prevents damage."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Channel Harm");
    let card = t.hand(P0, "Channel Harm");
    let from = t.asked().len();
    t.cast(P0, card).target(a).go();
    let targets_asked = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count();
    assert_eq!(targets_asked, 1);
    t.resolve();
    // When it prevents damage, no target is chosen: the damage goes to the Giant.
    let from = t.asked().len();
    t.answer_yes(P0, true);
    combat_damage(&mut t, b, pl(P0), 2);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert_eq!(t.obj_now(a).damage, 2);
    assert_eq!(t.obj_now(b).damage, 0);
}

#[test]
fn comeuppance_is_the_source_of_the_new_damage() {
    cr!("615.5", "609.7a", "702.15b");
    ruling!(
        "Comeuppance",
        "Comeuppance's effect is not a redirection effect. If it prevents damage, Comeuppance (not the original source) deals damage to the permanent or player as part of that prevention effect."
    );
    supported("Comeuppance");
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    lands_for_cost(&mut t, P0, "Comeuppance");
    let card = t.hand(P0, "Comeuppance");
    t.cast(P0, card).go();
    t.resolve();
    // A creature source: Comeuppance deals that much damage to the creature (noncombat,
    // and the Nighthawk's lifelink doesn't apply: it isn't the source).
    combat_damage(&mut t, hawk, pl(P0), 2);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(hawk).damage, 2);
    assert!(dealt_noncombat(&t, hawk));
    assert_eq!(t.life(P1), 20);
    // A noncreature source: the damage is dealt to its controller.
    let bolt = t.hand(P1, "Lightning Bolt");
    add_mana(&mut t, P1, ManaType::R, 1);
    t.cast(P1, bolt).target(pl(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn deflecting_palm_chooses_its_source_as_it_resolves() {
    cr!("609.7a", "615.5", "115.1");
    ruling!(
        "Deflecting Palm",
        "Deflecting Palm doesn't target any permanent or player. You choose a source of damage as Deflecting Palm resolves."
    );
    supported("Deflecting Palm");
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Deflecting Palm");
    let card = t.hand(P0, "Deflecting Palm");
    let from = t.asked().len();
    t.cast(P0, card).go();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    // The Giant enters after it was cast: it can still be chosen as it resolves.
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve();
    let offered = crate::r_p006_common::choices_since(&t, P0, from);
    assert!(offered.iter().any(|(c, _, _)| c.contains(&obj(giant))));
    combat_damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn plated_pegasus_prevents_1_damage_to_each_recipient_of_a_spell() {
    cr!("615.1a", "120.2b");
    ruling!(
        "Plated Pegasus",
        "If a spell would deal damage to multiple creatures and/or players, Plated Pegasus will prevent 1 damage that the spell would deal to each of them."
    );
    supported("Plated Pegasus");
    supported("Earthquake");
    // Earthquake with X = 2: 2 damage to each creature without flying and each player.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Plated Pegasus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Earthquake");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, card).x(2).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(giant).damage, 1);
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 19);
    // A creature's damage isn't a spell's.
    combat_damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 16);
}

#[test]
fn wrack_with_madness_damage_has_the_creatures_deathtouch_and_lifelink() {
    cr!("702.2b", "702.15b", "120.3");
    ruling!(
        "Wrack with Madness",
        "Creature abilities like deathtouch or lifelink may affect the results of the damage dealt."
    );
    supported("Wrack with Madness");
    // Vampire Nighthawk (2/3) deals 2 damage to itself: deathtouch destroys it and P1
    // gains 2 life.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    lands_for_cost(&mut t, P0, "Wrack with Madness");
    let card = t.hand(P0, "Wrack with Madness");
    t.cast(P0, card).target(hawk).go();
    t.resolve_all();
    assert!(in_graveyard(&t, hawk));
    assert_eq!(t.life(P1), 22);
}

#[test]
fn deadly_tempest_counts_only_creatures_destroyed_this_way() {
    cr!("701.8a", "702.12b", "608.2c");
    ruling!(
        "Deadly Tempest",
        "Creatures that aren’t destroyed this way (perhaps because they regenerated or have indestructible) don’t count toward the life lost."
    );
    supported("Deadly Tempest");
    supported("Darksteel Myr");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let myr = t.battlefield(P1, "Darksteel Myr");
    lands_for_cost(&mut t, P0, "Deadly Tempest");
    let card = t.hand(P0, "Deadly Tempest");
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn essence_backlash_uses_the_creature_spells_own_power() {
    cr!("109.2", "208.1", "608.2h");
    ruling!(
        "Essence Backlash",
        "Abilities that change the power of a creature (such as the one of Collective Blessing) affect only creatures on the battlefield. They won’t change a creature spell’s power on the stack."
    );
    supported("Essence Backlash");
    // P1 controls Collective Blessing (+3/+3 to creatures P1 controls) and casts Grizzly
    // Bears: the spell's power is 2.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Collective Blessing");
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.hand(P1, "Grizzly Bears");
    add_mana(&mut t, P1, ManaType::G, 2);
    let spell = t.cast(P1, bears).go();
    assert_eq!(t.obj_now(spell).power(), 2);
    lands_for_cost(&mut t, P0, "Essence Backlash");
    let card = t.hand(P0, "Essence Backlash");
    t.cast(P0, card).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn essence_backlash_damages_even_if_the_spell_cant_be_countered() {
    cr!("101.2", "113.6g", "608.2c");
    ruling!(
        "Essence Backlash",
        "Essence Backlash can target a creature spell that can’t be countered. If Essence Backlash resolves, it won’t counter the spell but it will still deal damage."
    );
    supported("Carnage Tyrant");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let tyrant = t.hand(P1, "Carnage Tyrant");
    add_mana(&mut t, P1, ManaType::G, 6);
    let spell = t.cast(P1, tyrant).go();
    lands_for_cost(&mut t, P0, "Essence Backlash");
    let card = t.hand(P0, "Essence Backlash");
    t.cast(P0, card).target(spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    assert_eq!(t.life(P1), 13);
}

#[test]
fn backlash_does_nothing_if_its_target_became_tapped() {
    cr!("115.1", "608.2b");
    ruling!(
        "Backlash",
        "Being untapped is a targeting restriction. If the target is tapped before this spell resolves, then this spell doesn’t resolve."
    );
    supported("Backlash");
    for tapped_in_response in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        lands_for_cost(&mut t, P0, "Backlash");
        let card = t.hand(P0, "Backlash");
        t.cast(P0, card).target(giant).go();
        if tapped_in_response {
            t.g.tap(giant);
        }
        t.resolve_all();
        assert!(t.obj_now(giant).tapped);
        assert_eq!(t.life(P1), if tapped_in_response { 20 } else { 17 });
    }
    // A tapped creature can't be targeted at all.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    lands_for_cost(&mut t, P0, "Backlash");
    let card = t.hand(P0, "Backlash");
    assert!(!castable(&mut t, P0, card));
}

#[test]
fn brightmare_can_target_a_tapped_creature() {
    cr!("115.1", "701.26a");
    ruling!(
        "Brightmare",
        "Brightmare's ability can target a tapped creature. If the target creature is already tapped when the ability resolves, that creature just remains tapped and you gain life equal to its power."
    );
    supported("Brightmare");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    lands_for_cost(&mut t, P0, "Brightmare");
    let card = t.hand(P0, "Brightmare");
    t.answer_targets(P0, &[obj(giant)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn hornet_nest_triggers_on_lethal_damage() {
    cr!("603.2", "120.3", "704.5g");
    ruling!(
        "Hornet Nest",
        "Hornet Nest’s triggered ability will trigger even if it’s dealt lethal damage. For example, if it’s dealt 7 damage, its ability will trigger and you’ll put seven Insect creature tokens onto the battlefield."
    );
    supported("Hornet Nest");
    supported("Blaze");
    let mut t = TestGame::new(2);
    let nest = t.battlefield(P0, "Hornet Nest");
    let card = t.hand(P1, "Blaze");
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P1, ManaType::R, 8);
    t.cast(P1, card).x(7).target(nest).go();
    t.resolve_all();
    assert!(in_graveyard(&t, nest));
    assert_eq!(tokens(&t, P0).len(), 7);
}
