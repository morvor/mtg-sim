//! "Reveal the top card of your library and put that card into your hand. You lose life
//! equal to its mana value." (Dark Confidant, Dark Tutelage).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn dark_confidant_reveals_and_loses_life_equal_to_mana_value() {
    cr!("701.20a");
    let c = card("Dark Confidant");
    assert!(c.unsupported_text().is_empty(), "{:?}", c.unsupported_text());
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    t.battlefield(P0, "Dark Confidant");
    // The draw step's card is under the Hill Giant (mana value 4).
    t.library_top(P0, "Forest");
    let giant = t.library_top(P0, "Hill Giant");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // It was revealed (shown to all players) on the way to the hand.
    assert!(t.g.turn_events.iter().any(|e| matches!(
        e,
        mtg_engine::events::Event::Custom { name, obj, player, .. }
            if name == mtg_engine::reveal::REVEALED && *obj == Some(giant) && *player == Some(P0)
    )));
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.life(P0), 16);
    // The card under it is drawn as usual.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(t.life(P0), 16);
}
