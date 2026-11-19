//! Item `rulings-residue-counters` — damage and counters: replacement effects that add
//! damage for each counter (Fated Firepower) or raise it (Ojer Axonil) applied after the
//! damage is divided (CR 601.2d, 510.1c, 616.1), prevention that removes counters
//! (Protean Hydra, CR 615), a source dealing damage to itself twice (Tracker), and
//! Voodoo Doll's intervening "if" clause (CR 603.4).

use crate::r_residue_counters_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The amounts of damage dealt to `target` this turn, in order.
fn dealt_to(t: &TestGame, target: Entity) -> Vec<u32> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                target: x, amount, ..
            } if *x == target => Some(*amount),
            _ => None,
        })
        .collect()
}

/// P0's Fated Firepower with `n` fire counters.
fn firepower(t: &mut TestGame, n: u32) -> ObjectId {
    let f = t.battlefield(P0, "Fated Firepower");
    put(t, f, "fire", n);
    f
}

/// P1 casts Healing Salve's second mode on `target`: the next 3 damage that would be dealt
/// to it this turn is prevented.
fn salve(t: &mut TestGame, target: Entity) {
    t.lands(P1, "Plains", 1);
    let s = t.hand(P1, "Healing Salve");
    t.cast(P1, s).modes(&[1]).target(target).go();
    t.resolve();
}

// --- Fated Firepower ---------------------------------------------------------------------

#[test]
fn fated_firepower_and_a_prevention_shield_are_ordered_by_the_damaged_player() {
    cr!("616.1", "616.1e", "615.7");
    ruling!(
        "Fated Firepower",
        "the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Fated Firepower's effect no longer applies."
    );
    supported("Fated Firepower");
    supported("Healing Salve");
    for (first, life) in [("Healing Salve", 20), ("Fated Firepower", 18)] {
        let mut t = TestGame::new(2);
        firepower(&mut t, 2);
        salve(&mut t, Entity::Player(P1));
        prefer_replacement(&mut t, P1, first);
        let from = t.asked().len();
        cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
        let choices = replacement_choices(&t, from);
        assert!(!choices.is_empty(), "an order was chosen");
        assert!(choices.iter().all(|(p, _)| *p == P1), "by P1: {choices:?}");
        // Prevention first: all 3 is prevented and the fire counters add nothing. Fated
        // Firepower first: 5, of which 3 is prevented.
        assert_eq!(t.life(P1), life, "{first} first");
    }
}

#[test]
fn fated_firepower_adds_after_trample_damage_is_divided() {
    cr!("510.1c", "702.19b", "616.1");
    ruling!(
        "Fated Firepower",
        "divide the original amount before adding the number of fire counters on Fated Firepower"
    );
    supported("Colossal Dreadmaw");
    let mut t = TestGame::new(2);
    firepower(&mut t, 2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    // 6 trample damage: lethal 2 to the 2/2 blocker and 4 to P1, assigned from the original
    // 6; then each is raised by 2.
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 4]));
    t.attack(&[(maw, Entity::Player(P1))], &[(bears, maw)]);
    assert_eq!(dealt_to(&t, obj(bears)), vec![4]);
    assert_eq!(dealt_to(&t, Entity::Player(P1)), vec![6]);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn fated_firepower_extra_damage_is_dealt_by_the_same_source() {
    cr!("614.1a", "702.15b");
    ruling!(
        "Fated Firepower",
        "The additional damage is dealt by the same source as the original source of damage."
    );
    supported("Vampire Nighthawk");
    // A lifelink creature's controller gains the whole amount: the extra damage is its.
    let mut t = TestGame::new(2);
    firepower(&mut t, 3);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    t.attack(&[(hawk, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P0), 25);
    let sources: Vec<ObjectId> = t
        .g
        .turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target: Entity::Player(P1),
                ..
            } => Some(*source),
            _ => None,
        })
        .collect();
    assert_eq!(sources, vec![hawk], "one event, from the Nighthawk");
}

// --- Ojer Axonil -------------------------------------------------------------------------

#[test]
fn ojer_axonil_raises_divided_damage_after_the_division() {
    cr!("601.2d", "614.1a");
    ruling!(
        "Ojer Axonil, Deepest Might // Temple of Power",
        "you must divide the unmodified damage"
    );
    supported("Chandra's Pyrohelix");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ojer Axonil, Deepest Might // Temple of Power");
    let giant = t.battlefield(P1, "Hill Giant");
    // Pyrohelix's 2 damage is divided 1 and 1; only the 1 to the opponent becomes 4.
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    cast_slots(&mut t, P0, "Chandra's Pyrohelix", &[&[Entity::Player(P1), obj(giant)]]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(marked(&t, giant), 1);
}

// --- Protean Hydra -----------------------------------------------------------------------

#[test]
fn protean_hydra_prevents_all_damage_even_with_too_few_counters() {
    cr!("615.1a", "122.1");
    ruling!(
        "Protean Hydra",
        "All damage that would be dealt to Protean Hydra is prevented, even if it doesn’t have that many +1/+1 counters on it."
    );
    supported("Protean Hydra");
    supported("Glorious Anthem");
    // Two counters, 3 damage (a 3/3 thanks to Glorious Anthem): all of it is prevented and
    // both counters are removed — a 1/1 with no damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let hydra = with_counters(&mut t, P0, "Protean Hydra", 2);
    cast_resolve(&mut t, P1, "Lightning Bolt", &[obj(hydra)]);
    assert!(t.on_battlefield(hydra));
    assert_eq!(marked(&t, hydra), 0);
    assert_eq!(plus1(&t, hydra), 0);
    assert_eq!(t.pt(hydra), (1, 1));
    // No counters at all (a 1/1 thanks to Glorious Anthem): still all prevented.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let hydra = t.battlefield(P0, "Protean Hydra");
    assert_eq!(t.pt(hydra), (1, 1));
    cast_resolve(&mut t, P1, "Shock", &[obj(hydra)]);
    assert!(t.on_battlefield(hydra));
    assert_eq!(marked(&t, hydra), 0);
}

#[test]
fn protean_hydra_still_loses_counters_to_unpreventable_damage() {
    cr!("615.12", "609.3");
    ruling!(
        "Protean Hydra",
        "If unpreventable damage is dealt to Protean Hydra, the Hydra’s second ability will try to prevent it and fail"
    );
    supported("Flaring Pain");
    let mut t = TestGame::new(2);
    let hydra = with_counters(&mut t, P0, "Protean Hydra", 5);
    cast_resolve(&mut t, P1, "Flaring Pain", &[]);
    cast_resolve(&mut t, P1, "Shock", &[obj(hydra)]);
    // The 2 damage is dealt (5/5 with 2 damage) and two counters are removed (3/3 with 2
    // damage: it survives).
    assert!(t.on_battlefield(hydra));
    assert_eq!(marked(&t, hydra), 2);
    assert_eq!(plus1(&t, hydra), 3);
}

// --- Tracker -----------------------------------------------------------------------------

#[test]
fn tracker_targeting_itself_deals_damage_to_itself_twice() {
    cr!("120.3e", "608.2c");
    ruling!(
        "Tracker",
        "Tracker will deal damage to itself equal to its power, then immediately do it again"
    );
    supported("Tracker");
    supported("Giant Growth");
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P0, "Tracker");
    // Giant Growth in response makes it 5/5: it deals 5 damage to itself, then 5 again.
    t.lands(P0, "Forest", 2);
    t.activate(P0, tracker, 0, &[obj(tracker)]).unwrap();
    giant_growth(&mut t, tracker);
    t.resolve_all();
    assert_eq!(dealt_to(&t, obj(tracker)), vec![5, 5]);
    assert!(t.in_graveyard(P0, "Tracker"));
}

// --- Voodoo Doll -------------------------------------------------------------------------

/// P0's Voodoo Doll with `n` pin counters.
fn doll(t: &mut TestGame, n: u32) -> ObjectId {
    let d = t.battlefield(P0, "Voodoo Doll");
    put(t, d, "pin", n);
    d
}

#[test]
fn voodoo_doll_is_destroyed_even_if_its_damage_is_prevented() {
    cr!("608.2c", "615.7");
    ruling!(
        "Voodoo Doll",
        "Voodoo Doll is destroyed even if all the damage is prevented or redirected."
    );
    supported("Voodoo Doll");
    let mut t = TestGame::new(2);
    let d = doll(&mut t, 2);
    t.set_step(P0, Step::PrecombatMain);
    // P0's own Healing Salve shield: P1 casts it on P0 (any target).
    salve(&mut t, Entity::Player(P0));
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "all prevented");
    assert!(!t.on_battlefield(d));
    assert!(t.in_graveyard(P0, "Voodoo Doll"));
}

#[test]
fn voodoo_doll_end_step_ability_checks_untapped_twice_and_by_last_known_information() {
    cr!("603.4", "608.2h");
    ruling!(
        "Voodoo Doll",
        "Voodoo Doll's second ability has an \"intervening 'if' clause.\""
    );
    // Tapped at the beginning of the end step: no trigger.
    let mut t = TestGame::new(2);
    let d = doll(&mut t, 1);
    t.g.tap(d);
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "untapped"), 0);
    t.resolve_all();
    assert!(t.on_battlefield(d));
    assert_eq!(t.life(P0), 20);
    // Untapped as it triggers, tapped by the time it resolves: nothing happens.
    let mut t = TestGame::new(2);
    let d = doll(&mut t, 1);
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "untapped"), 1);
    t.g.tap(d);
    t.resolve_all();
    assert!(t.on_battlefield(d));
    assert_eq!(t.life(P0), 20);
    // Gone (untapped) by then: its last existence was untapped, so it deals its damage.
    let mut t = TestGame::new(2);
    let d = doll(&mut t, 3);
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "untapped"), 1);
    destroy(&mut t, d);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}
