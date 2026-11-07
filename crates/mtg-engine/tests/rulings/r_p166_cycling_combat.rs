//! Rulings batch P166 — cycling payoffs (CR 702.29), "nonland creatures", counting as an
//! ability resolves, damage assigned by toughness (CR 510.1a), and "enters with X
//! counters" (CR 614.1c).

use crate::r_p125_common::{at_p1, fight_it_out};
use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn planar_outburst_spares_land_creatures() {
    cr!("205.4", "701.8a");
    ruling!(
        "Planar Outburst",
        "A “nonland creature” is a creature that isn't also a land."
    );
    supported("Planar Outburst");
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P0, "Dryad Arbor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = in_hand_with_mana(&mut t, P0, "Planar Outburst");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(arbor));
}

#[test]
fn curse_of_thirst_counts_curses_as_it_resolves() {
    cr!("608.2h", "303.4");
    ruling!(
        "Curse of Thirst",
        "The number of Curses attached to the player is counted when the ability resolves."
    );
    supported("Curse of Thirst");
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Curse of Thirst", Entity::Player(P1));
    next_upkeep(&mut t, P1);
    assert_eq!(triggers_on_stack(&t, "Curses attached"), 1);
    // A second Curse is attached with the trigger on the stack.
    attach_new(&mut t, P0, "Curse of the Pierced Heart", Entity::Player(P1));
    let life = t.life(P1);
    t.resolve();
    assert_eq!(t.life(P1), life - 2);
}

#[test]
fn sacred_excavation_can_target_cycling_variants() {
    cr!("702.29e", "702.29f", "115.1");
    ruling!(
        "Sacred Excavation",
        "Certain older cards have variants of cycling, such as basic landcycling or Wizardcycling. Sacred Excavation can target these cards in your graveyard."
    );
    supported("Sacred Excavation");
    let mut t = TestGame::new(2);
    let barrens = t.graveyard(P0, "Ash Barrens");
    let abom = t.graveyard(P0, "Twisted Abomination");
    let mage = t.graveyard(P0, "Vedalken Aethermage");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let targets = spell_targets(&mut t, P0, "Sacred Excavation");
    for c in [barrens, abom, mage] {
        assert!(targets.contains(&Entity::Object(c)));
    }
    assert!(!targets.contains(&Entity::Object(bears)));
    // And returns them.
    let card = in_hand_with_mana(&mut t, P0, "Sacred Excavation");
    t.cast(P0, card)
        .targets(&[Entity::Object(barrens), Entity::Object(mage)])
        .go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Ash Barrens"));
    assert!(t.in_hand(P0, "Vedalken Aethermage"));
}

#[test]
fn cycling_and_cycle_triggers_arent_spells() {
    cr!("702.29a", "113.3b", "112.1", "115.1");
    ruling!(
        "Drannith Healer",
        "Triggered abilities from cycling a card and the cycling ability itself aren’t spells. Effects that interact with spells (such as that of Cancel) won’t affect them."
    );
    supported("Drannith Healer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drannith Healer");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    let spell = t.cast(P0, opt).go();
    t.lands(P0, "Wastes", 1);
    let stinger = t.hand(P0, "Drannith Stinger");
    cycle(&mut t, P0, stinger, 0).unwrap();
    t.settle();
    // Opt, the cycling ability and the Healer's trigger.
    assert_eq!(t.stack_len(), 3);
    let targets = spell_targets(&mut t, P1, "Cancel");
    assert_eq!(targets, vec![Entity::Object(spell)]);
    // The abilities resolve: draw a card and gain 1 life.
    let life = t.life(P0);
    t.resolve();
    assert_eq!(t.life(P0), life + 1);
}

#[test]
fn lightning_rift_targets_on_trigger_and_pays_on_resolution() {
    cr!("603.3d", "608.2c", "117.1");
    ruling!(
        "Lightning Rift",
        "You choose the target when Lightning Rift’s ability is put on the stack, but you choose whether to pay the mana as the ability resolves."
    );
    supported("Lightning Rift");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lightning Rift");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 3);
    let stinger = t.hand(P0, "Drannith Stinger");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(giant)]);
    cycle(&mut t, P0, stinger, 0).unwrap();
    t.settle();
    assert_eq!(
        target_candidates(&t, P0, from).len(),
        1,
        "targeted on trigger"
    );
    // Nothing was asked about paying yet.
    let paid_q = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .filter(|(p, d)| {
                *p == P0 && matches!(d, Decision::YesNo { .. } | Decision::OptionalCost { .. })
            })
            .count()
    };
    assert_eq!(paid_q(&t, from), 0);
    let lands = untapped_lands(&t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(paid_q(&t, from), 1, "asked as it resolves");
    assert_eq!(untapped_lands(&t, P0), lands - 1);
    assert_eq!(t.obj_now(giant).damage, 2);
}

/// Wall of Stone (0/8, defender) and Wall of Swords (3/5, flying, defender).
fn walls(t: &mut TestGame) -> (ObjectId, ObjectId) {
    (
        t.battlefield(P0, "Wall of Stone"),
        t.battlefield(P0, "Wall of Swords"),
    )
}

#[test]
fn arcades_changes_only_the_combat_damage_assigned() {
    cr!("510.1a", "208.1");
    ruling!(
        "Arcades, the Strategist",
        "Arcades's last ability doesn't actually change any creature's power. It changes only the amount of combat damage it assigns. All other rules and effects that check power or toughness use the real values. For example, Rabid Bite won't cause a creature with defender to deal damage equal to its toughness."
    );
    supported("Arcades, the Strategist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arcades, the Strategist");
    let (stone, _) = walls(&mut t);
    assert_eq!(t.pt(stone), (0, 8));
    let giant = t.battlefield(P1, "Hill Giant");
    // Rabid Bite: the Wall deals damage equal to its power (0).
    let bite = in_hand_with_mana(&mut t, P0, "Rabid Bite");
    t.cast(P0, bite)
        .target(Entity::Object(stone))
        .target(Entity::Object(giant))
        .go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0);
    // In combat it assigns 8.
    fight_it_out(&mut t, &[stone], &[]);
    assert_eq!(t.life(P1), 12);
}

#[test]
fn high_alert_doesnt_make_creatures_fight_with_toughness() {
    cr!("510.1a", "701.14a");
    ruling!(
        "High Alert",
        "High Alert’s first ability doesn’t actually change any creature’s power. It changes only the amount of combat damage it assigns. All other rules and effects that check power or toughness use the real values. For example, Titanic Brawl won’t cause a creature to fight with its toughness under High Alert."
    );
    supported("High Alert");
    supported("Titanic Brawl");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "High Alert");
    let (stone, _) = walls(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    let brawl = in_hand_with_mana(&mut t, P0, "Titanic Brawl");
    t.cast(P0, brawl)
        .target(Entity::Object(stone))
        .target(Entity::Object(giant))
        .go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.obj_now(stone).damage, 3);
    // Combat damage is assigned by toughness: 8.
    fight_it_out(&mut t, &[stone], &[]);
    assert_eq!(t.life(P1), 12);
}

#[test]
fn a_defender_stays_attacking_if_arcades_leaves() {
    cr!("506.4", "510.1a");
    ruling!(
        "Arcades, the Strategist",
        "If Arcades leaves the battlefield after a creature with defender has attacked, that creature remains an attacking creature, although it will assign damage equal to its power."
    );
    let mut t = TestGame::new(2);
    let arcades = t.battlefield(P0, "Arcades, the Strategist");
    let (_, swords) = walls(&mut t);
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &at_p1(&[swords]));
    t.resolve_all();
    move_to(&mut t, arcades, Zone::Exile);
    assert!(t.g.is_attacking(t.g.current(swords)));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn towering_titan_ignores_creatures_entering_with_it() {
    cr!("614.1c", "614.12");
    ruling!(
        "Towering Titan",
        "If another creature is entering the battlefield at the same time as Towering Titan, its toughness won’t contribute to the +1/+1 counters on Towering Titan."
    );
    supported("Towering Titan");
    supported("Living Death");
    // Living Death returns Towering Titan and Hill Giant together: no counters, and the
    // 0/0 Titan dies.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Towering Titan");
    t.graveyard(P0, "Hill Giant");
    let ld = in_hand_with_mana(&mut t, P0, "Living Death");
    t.cast(P0, ld).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(t.named_on_battlefield("Towering Titan").is_empty());
    assert!(t.in_graveyard(P0, "Towering Titan"));
}

#[test]
fn towering_titan_keeps_its_counters() {
    cr!("614.1c", "122.1");
    ruling!(
        "Towering Titan",
        "Once Towering Titan has entered the battlefield, it doesn’t gain or lose counters as the total toughness of other creatures you control changes."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    let titan = t.enter(P0, "Towering Titan");
    t.settle();
    assert_eq!(t.counters(titan, counters::PLUS1), 3);
    t.battlefield(P0, "Wall of Stone");
    move_to(&mut t, giant, Zone::Exile);
    t.settle();
    assert_eq!(t.counters(titan, counters::PLUS1), 3);
    assert_eq!(t.pt(titan), (3, 3));
}
