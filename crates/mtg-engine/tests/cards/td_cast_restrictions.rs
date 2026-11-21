//! "Until your next turn, your opponents can't cast spells from anywhere other than their
//! hands." (Avatar's Wrath): a casting prohibition for a duration (CR 601.3, 611.2b).

use crate::combat2_common::*;
use mtg_engine::kwa::bending::AIRBEND_METHOD;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const AIRBEND: CastMethod = CastMethod::Alternative(AIRBEND_METHOD);

#[test]
fn avatars_wrath_stops_opponents_casting_from_exile_until_your_next_turn() {
    cr!("601.3", "611.2b");
    supported("Avatar's Wrath");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Avatar's Wrath");
    t.cast(P0, wrath).go();
    t.resolve_all();
    let bears = t.g.current(bears);
    let elves = t.g.current(elves);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(elves), Zone::Exile);
    assert!(t.in_exile("Avatar's Wrath"));
    // You can still cast your airbent card.
    t.lands(P0, "Forest", 2);
    assert!(t.cast(P0, elves).method(AIRBEND).try_go().is_ok());
    t.resolve_all();
    // The opponent, on their turn: not from exile; from their hand, yes.
    t.advance_to(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 4);
    assert!(t.cast(P1, bears).method(AIRBEND).try_go().is_err());
    let other = t.hand(P1, "Grizzly Bears");
    assert!(t.cast(P1, other).try_go().is_ok());
    t.resolve_all();
    // From your next turn on, they can.
    t.advance_to(P0, Step::Upkeep);
    t.lands(P1, "Island", 2);
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P1, bears).method(AIRBEND).try_go().is_ok());
}
