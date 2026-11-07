//! The worked examples of CR 613 (interaction of continuous effects), played out as
//! tests. The CR 613.4d, 613.5, and the first three CR 613.6 examples, and the second
//! CR 613.9 example, are cited by tests in `r609_613_layers.rs` and
//! `r609_613_timestamps.rs`.

use crate::r600_examples_common::*;
use crate::r609_common::has_kw;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn svogthos_becomes_a_creature_with_a_graveyard_counting_ability_in_timestamp_order() {
    cr!("613.4b", "613.4c", "613.1d", "613.1e", "613.7a");
    cr_example!("613.6", "Svogthos, the Restless Tomb, is on the battlefield");
    supported("Svogthos, the Restless Tomb");
    supported("Animate Land");
    let mut t = TestGame::new(2);
    let sv = t.battlefield(P0, "Svogthos, the Restless Tomb");
    for _ in 0..10 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Island", 3);
    // "Until end of turn, target land becomes a 3/3 creature that's still a land" (layers
    // 4 and 7b).
    let animate = t.hand(P0, "Animate Land");
    t.cast(P0, animate).target(sv).go();
    t.resolve();
    assert!(t.obj_now(sv).is_creature());
    assert!(t.obj_now(sv).chars.card_types.contains(CardType::Land));
    assert_eq!(t.pt(sv), (3, 3));
    // "Target creature gets +1/+1 until end of turn" (layer 7c): a 4/4 land creature.
    let pump = t.custom(
        P0,
        custom_costed(
            "Small Boost",
            "Instant",
            "{G}",
            "Target creature gets +1/+1 until end of turn.",
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, pump).target(sv).go();
    t.resolve();
    assert_eq!(t.pt(sv), (4, 4));
    // Svogthos's own ability (layers 4, 5, and 7b) with ten creature cards in the
    // graveyard: an 11/11 black and green Plant Zombie land creature.
    t.activate(P0, sv, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(sv), (11, 11));
    let o = t.obj_now(sv);
    assert!(o.chars.has_subtype("Plant") && o.chars.has_subtype("Zombie"));
    assert!(o.chars.card_types.contains(CardType::Land));
    assert!(o.chars.colors.contains(Color::Black) && o.chars.colors.contains(Color::Green));
    // A creature card entering or leaving the graveyard modifies it accordingly.
    t.graveyard(P0, "Grizzly Bears");
    t.recompute();
    assert_eq!(t.pt(sv), (12, 12));
    for gone in t.g.find_in_zone(Zone::Graveyard(P0), "Grizzly Bears")[..2].to_vec() {
        t.g.move_object(gone, Zone::Exile, MoveCause::Effect, Some(P0));
    }
    t.recompute();
    assert_eq!(t.pt(sv), (10, 10));
    // The first effect applied again (a new timestamp): a 4/4 land creature again.
    let again = t.hand(P0, "Animate Land");
    t.lands(P0, "Forest", 1);
    t.cast(P0, again).target(sv).go();
    t.resolve();
    assert_eq!(t.pt(sv), (4, 4));
}

#[test]
fn rune_of_flights_granted_ability_stays_later_than_colossus_hammers_loses_flying() {
    cr!("613.7e", "613.9", "613.1f");
    cr_example!("613.7a", "Rune of Flight is an Aura that grants enchanted Equipment");
    supported("Rune of Flight");
    supported("Colossus Hammer");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Wind Drake");
    let hammer = t.battlefield(P0, "Colossus Hammer");
    t.lands(P0, "Island", 2);
    let rune = t.hand(P0, "Rune of Flight");
    t.cast(P0, rune).target(hammer).go();
    t.resolve();
    t.resolve_all(); // its "draw a card" trigger
    let rune = t.named_on_battlefield("Rune of Flight")[0];
    assert_eq!(t.obj_now(rune).attached_to, Some(Entity::Object(hammer)));
    // The Hammer now becomes attached to a creature: it gets a new timestamp (later than
    // Rune of Flight's), but the ability Rune of Flight grants it keeps its place after
    // "loses flying", so the equipped creature has flying.
    t.lands(P0, "Plains", 8);
    t.activate(P0, hammer, 0, &[Entity::Object(drake)]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(hammer).attached_to, Some(Entity::Object(drake)));
    assert!(t.obj_now(hammer).timestamp > t.obj_now(rune).timestamp);
    assert_eq!(t.pt(drake), (12, 12));
    assert!(has_kw(&t, drake, KeywordKind::Flying));
}

#[test]
fn flight_and_grounded_the_later_aura_wins() {
    cr!("613.7e", "613.1f");
    cr_example!("613.9", "Enchanted creature loses flying");
    supported("Flight");
    supported("Grounded");
    for flight_first in [true, false] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Island", 2);
        t.lands(P0, "Forest", 2);
        let order = if flight_first {
            ["Flight", "Grounded"]
        } else {
            ["Grounded", "Flight"]
        };
        for name in order {
            let aura = t.hand(P0, name);
            t.cast(P0, aura).target(bear).go();
            t.resolve();
        }
        // The one generated last wins.
        assert_eq!(has_kw(&t, bear, KeywordKind::Flying), !flight_first);
    }
    // The same result if the "loses flying" effect has a duration or comes from a
    // non-Aura source.
    for until_eot in [true, false] {
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Island", 1);
        t.lands(P0, "Swamp", 1);
        let flight = t.hand(P0, "Flight");
        t.cast(P0, flight).target(bear).go();
        t.resolve();
        assert!(has_kw(&t, bear, KeywordKind::Flying));
        if until_eot {
            let s = t.custom(
                P0,
                custom_costed(
                    "Clip Wings",
                    "Instant",
                    "{B}",
                    "Target creature loses flying until end of turn.",
                ),
                Zone::Hand(P0),
            );
            t.cast(P0, s).target(bear).go();
            t.resolve();
        } else {
            t.custom(
                P1,
                custom_costed(
                    "Grounding Field",
                    "Enchantment",
                    "{B}",
                    "All creatures lose flying.",
                ),
                Zone::Battlefield,
            );
            t.recompute();
        }
        assert!(!has_kw(&t, bear, KeywordKind::Flying));
    }
}
