//! Rulings batch S35 — snow mana (CR 107.4h): snow is neither a color nor a type of mana,
//! so spending mana "as though it were mana of any type" (CR 118.14, 609.4b) doesn't let
//! mana from a non-snow source pay for {S}.

use crate::r_s01_common::supported;
use crate::r_s02_common::can_activate;
use crate::r_s05_common::run_from;
use mtg_engine::ability::{Duration, Effect, PlayerRef, Sel};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn mana_of_any_type_from_a_non_snow_source_cant_pay_for_snow() {
    cr!("107.4h", "118.14", "609.4b");
    ruling!(
        "Rimewood Falls",
        "Snow isn't a type of mana. If an effect says you may spend mana as though it were any type, you can't pay for {S} using mana that wasn't produced by a snow source."
    );
    ruling!(
        "Icebind Pillar",
        "Snow isn’t a type of mana. If an effect says you may spend mana as though it were any type, you can’t pay for {S} using mana that wasn’t produced by a snow source."
    );
    supported("Icehide Golem");
    supported("Rimewood Falls");
    supported("Icebind Pillar");
    // Icehide Golem costs {S}. An effect lets P0 spend mana of any type to cast it (as
    // Hostage Taker's does for the card it exiled).
    let mut t = TestGame::new(2);
    let golem = t.hand(P0, "Icehide Golem");
    run_from(
        &mut t,
        P0,
        None,
        Effect::SpendAnyTypeMana {
            who: PlayerRef::You,
            what: Sel::Target(0),
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(golem)],
    );
    let islands = t.lands(P0, "Island", 2);
    // (The same permission lets blue mana pay for {G}: Llanowar Elves is cast with an
    // Island's mana.)
    let elves = t.hand(P0, "Llanowar Elves");
    run_from(
        &mut t,
        P0,
        None,
        Effect::SpendAnyTypeMana {
            who: PlayerRef::You,
            what: Sel::Target(0),
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(elves)],
    );
    t.cast(P0, elves).go();
    t.resolve_all();
    assert!(t.obj_now(islands[0]).tapped);
    let islands = t.lands(P0, "Island", 2);
    assert!(t.cast(P0, golem).try_go().is_err());
    assert_eq!(t.zone(golem), Zone::Hand(P0));
    assert!(islands.iter().all(|l| !t.obj_now(*l).tapped));
    // Mana from Rimewood Falls, a snow land, pays it.
    let falls = t.battlefield(P0, "Rimewood Falls");
    t.cast(P0, golem).go();
    assert!(t.obj_now(falls).tapped);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Icehide Golem").len(), 1);
    // Icebind Pillar: "{S}, {T}: Tap target artifact or creature." An Island can't pay
    // the {S}; a snow land can.
    let mut t = TestGame::new(2);
    let pillar = t.battlefield(P0, "Icebind Pillar");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    assert!(!can_activate(&mut t, P0, pillar));
    t.battlefield(P0, "Rimewood Falls");
    assert!(can_activate(&mut t, P0, pillar));
}
