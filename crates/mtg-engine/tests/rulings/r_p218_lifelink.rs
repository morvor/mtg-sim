//! Rulings batch P218 — lifelink (CR 702.15): spells with lifelink, last known
//! information, one life-gain event per source, Two-Headed Giant, redundancy, and effects
//! that don't deal damage.

use crate::r_p210_common::deal;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use crate::r_s06_common::activate_containing;
use crate::r_s11_common::triggered_from;
use crate::r_s13_common::add;
use crate::r_s28_common::energy;
use crate::r_s33_common::{life_gains_since, two_headed_giant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn a_spell_with_lifelink_gains_life_only_for_damage_it_deals() {
    cr!("702.15b", "702.15e", "120.3f");
    ruling!(
        "Soulfire Grand Master",
        "An instant or sorcery spell with lifelink causes its controller to gain life only if it’s the source of any damage that’s dealt. An instant or sorcery spell with lifelink that causes another source to deal damage won’t cause its controller to gain life."
    );
    // Only its last ability doesn't compile; the lifelink grant does.
    crate::r_p190_mana_costs::only_unsupported("Soulfire Grand Master", "The next time you cast");
    // "Instant and sorcery spells you control have lifelink." Lightning Bolt deals the
    // damage itself: P0 gains 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soulfire Grand Master");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
    // Prey Upon makes two creatures fight: the creatures deal the damage, not the spell.
    supported("Prey Upon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soulfire Grand Master");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let prey = t.hand(P0, "Prey Upon");
    t.cast(P0, prey).target(giant).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn piru_gains_life_for_its_dies_trigger_damage() {
    cr!("702.15b", "603.10a", "113.7a");
    ruling!(
        "Piru, the Volatile",
        "Because Piru had lifelink when it was on the battlefield, you will gain life equal to the damage it deals as its last triggered ability resolves."
    );
    supported("Piru, the Volatile");
    let mut t = TestGame::new(2);
    let piru = t.battlefield(P0, "Piru, the Volatile");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    destroy(&mut t, piru);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.life(P0), 34);
}

/// P0 controls the real card `name` (with lifelink). Returns how many times `name`'s
/// ability triggered (a) when it and Vampire Nighthawk (lifelink) attack unblocked, and
/// (b) when only Wurmcoil Engine (lifelink, deathtouch) attacks and is blocked by two
/// creatures, dealing combat damage to each of them.
fn lifelink_events(name: &str, targets_creature: bool) -> (usize, usize) {
    supported(name);
    let mut both = 0;
    for single in [false, true] {
        let mut t = TestGame::new(2);
        let card = t.battlefield(P0, name);
        let hawk = t.battlefield(P0, "Vampire Nighthawk");
        if targets_creature {
            for _ in 0..2 {
                t.answer_targets(P0, &[Entity::Object(card)]);
            }
        }
        if single {
            // Wurmcoil Engine (6/6 deathtouch, lifelink) is blocked by two creatures.
            let wurm = t.battlefield(P0, "Wurmcoil Engine");
            let b1 = t.battlefield(P1, "Grizzly Bears");
            let b2 = t.battlefield(P1, "Grizzly Bears");
            t.attack(&[(wurm, Entity::Player(P1))], &[(b1, wurm), (b2, wurm)]);
            assert!(!t.on_battlefield(b1) && !t.on_battlefield(b2));
            assert_eq!(t.life(P0), 26);
            return (both, triggered_from(&t, card));
        }
        t.attack(
            &[(card, Entity::Player(P1)), (hawk, Entity::Player(P1))],
            &[],
        );
        both = triggered_from(&t, card);
    }
    unreachable!()
}

#[test]
fn each_lifelink_creature_causes_a_separate_life_gain_event() {
    cr!("702.15b", "119.9", "510.2", "603.2c");
    ruling!(
        "Aerith Gainsborough",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Aerith Gainsborough's second ability will trigger twice."
    );
    ruling!(
        "Drogskol Reaver",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Drogskol Reaver's last ability will trigger twice."
    );
    ruling!(
        "Leonardo, Cutting Edge",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Leonardo's last ability will trigger twice."
    );
    ruling!(
        "Minwu, White Mage",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Minwu's last ability will trigger twice."
    );
    ruling!(
        "The Destined White Mage",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, The Destined White Mage's third ability will trigger twice."
    );
    supported("Vampire Nighthawk");
    supported("Wurmcoil Engine");
    // (card, its trigger targets a creature, triggers when it and the Nighthawk deal
    // combat damage: Drogskol Reaver has double strike, so it deals first-strike damage
    // alone, then regular damage together with the Nighthawk)
    for (name, targets, both) in [
        ("Aerith Gainsborough", false, 2),
        ("Drogskol Reaver", false, 3),
        ("Leonardo, Cutting Edge", false, 2),
        ("Minwu, White Mage", false, 2),
        ("The Destined White Mage", true, 2),
    ] {
        assert_eq!(lifelink_events(name, targets), (both, 1), "{name}");
    }
}

#[test]
fn sphinx_of_the_revelation_two_lifelink_creatures_two_triggers() {
    cr!("702.15b", "119.9", "510.2");
    ruling!(
        "Sphinx of the Revelation",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Sphinx of the Revelation's triggered ability will trigger twice."
    );
    supported("Sphinx of the Revelation");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Sphinx of the Revelation");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    t.attack(
        &[(sphinx, Entity::Player(P1)), (hawk, Entity::Player(P1))],
        &[],
    );
    assert_eq!(triggered_from(&t, sphinx), 2);
    assert_eq!(t.life(P0), 26);
    assert_eq!(energy(&t, P0), 6);
}

#[test]
fn brion_stoutarm_gone_still_has_lifelink_by_last_known_information() {
    cr!("702.15b", "113.7a", "608.2h");
    ruling!(
        "Brion Stoutarm",
        "If Brion Stoutarm leaves the battlefield after its ability has been activated but before it resolves, the game uses its last known information to determine that it had lifelink and you'll gain life for the damage it deals."
    );
    supported("Brion Stoutarm");
    let mut t = TestGame::new(2);
    let brion = t.battlefield(P0, "Brion Stoutarm");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, brion, "Sacrifice another creature").expect("activated");
    assert!(t.in_graveyard(P0, "Hill Giant"));
    destroy(&mut t, brion);
    assert!(t.in_graveyard(P0, "Brion Stoutarm"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn lone_rider_life_gained_by_a_teammate_doesnt_count() {
    cr!("810.9", "119.9", "702.15b");
    ruling!(
        "Lone Rider // It That Rides as One",
        "In a Two-Headed Giant game, events that cause a player to gain life affect each player separately, even though the result affects the team's life total."
    );
    supported("Lone Rider // It That Rides as One");
    // P0 and P1 are a team. P1's lifelink creature deals 5 damage: P1 gained the life.
    let mut t = two_headed_giant();
    let rider = t.battlefield(P0, "Lone Rider // It That Rides as One");
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    let from = t.g.turn_events.len();
    deal(&mut t, hawk, 5, P2);
    assert_eq!(life_gains_since(&t, from, P1), vec![5]);
    assert!(life_gains_since(&t, from, P0).is_empty());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.obj_now(rider).chars.name, "Lone Rider");
    // P0's own lifelink damage (3) does transform it.
    let mut t = two_headed_giant();
    let rider = t.battlefield(P0, "Lone Rider // It That Rides as One");
    deal(&mut t, rider, 3, P2);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.obj_now(rider).chars.name, "It That Rides as One");
}

#[test]
fn jerren_giving_lifelink_twice_is_redundant() {
    cr!("702.15f", "702.15b");
    ruling!(
        "Jerren, Corrupted Bishop // Ormendahl, the Corrupter",
        "Multiple instances of lifelink are redundant, so giving lifelink to a Human that already has it gives no additional benefit."
    );
    supported("Jerren, Corrupted Bishop // Ormendahl, the Corrupter");
    // Lone Rider is a Human Knight with lifelink; Jerren gives it lifelink twice more.
    let mut t = TestGame::new(2);
    let jerren = t.battlefield(P0, "Jerren, Corrupted Bishop // Ormendahl, the Corrupter");
    let rider = t.battlefield(P0, "Lone Rider // It That Rides as One");
    t.lands(P0, "Swamp", 4);
    for _ in 0..2 {
        t.answer_targets(P0, &[Entity::Object(rider)]);
        activate_containing(&mut t, P0, jerren, "gains lifelink").expect("activated");
        t.resolve_all();
    }
    deal(&mut t, rider, 1, P1);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn ocelot_pride_counts_life_gained_before_it_entered() {
    cr!("603.4", "119.9");
    ruling!(
        "Ocelot Pride",
        "Ocelot Pride doesn't need to have been on the battlefield when you gained life."
    );
    supported("Ocelot Pride");
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    deal(&mut t, hawk, 2, P1);
    assert_eq!(t.life(P0), 22);
    t.set_step(P0, Step::PostcombatMain);
    enter(&mut t, P0, "Ocelot Pride");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Cat").len(),
        1
    );
    // Without life gained this turn, no token.
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Ocelot Pride");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Cat").is_empty());
}

#[test]
fn vish_kal_minus_ability_deals_no_damage_and_gains_no_life() {
    cr!("702.15b", "120.1");
    ruling!(
        "Vish Kal, Blood Arbiter",
        "Vish Kal's last ability doesn't deal damage. It doesn't cause you to gain any life from its lifelink ability."
    );
    supported("Vish Kal, Blood Arbiter");
    let mut t = TestGame::new(2);
    let vish = t.battlefield(P0, "Vish Kal, Blood Arbiter");
    add(&mut t, vish, counters::PLUS1, 3);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, vish, "Remove all").expect("activated");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(vish).counter(counters::PLUS1), 0);
}
