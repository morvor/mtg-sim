//! CR 601.3, 117.1a: a "next spell you cast this turn" effect that doesn't say the spell
//! can be cast as though it had flash doesn't change when it can be cast.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_next_creature_spell_effect_without_flash_doesnt_give_flash() {
    cr!("117.1a", "601.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Mountain", 1);
    // "The next creature spell you cast this turn can't be countered. Draw a card."
    let insist = t.hand(P0, "Insist");
    t.cast(P0, insist).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Insist"));
    // With Lightning Bolt on the stack, a creature spell can't be cast.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == bears)));
    assert!(t.cast(P0, bears).try_go().is_err());
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Once the stack is empty again in P0's main phase it can.
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.cast(P0, bears).go();
}
