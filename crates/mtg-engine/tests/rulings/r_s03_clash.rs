//! Rulings batch S03 — clash (CR 701.30): "Clash with an opponent. If you win, return
//! [this spell] to its owner's hand."

use crate::r_s01_common::*;
use crate::r_s03_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether any player clashed (was asked where to put a revealed card) since decision
/// `from`.
fn clashed_since(t: &TestGame, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("Clash")))
}

/// Whether `card` (an object id of the spell) was put into a graveyard.
fn went_to_graveyard(t: &TestGame, card: ObjectId) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, Event::ZoneChange { old, to: Zone::Graveyard(_), .. } if *old == card))
}

#[test]
fn a_spell_that_wins_its_clash_goes_from_the_stack_to_its_owners_hand() {
    cr!("701.30a", "701.30b", "701.30d", "608.2n");
    ruling!(
        "Release the Ants",
        "If you win the clash, the spell moves from the stack to your hand as part of its resolution. It never hits the graveyard. If you don't win the clash, the spell is put into the graveyard from the stack as normal."
    );
    supported("Release the Ants");
    // "Release the Ants deals 1 damage to any target. Clash with an opponent. If you win,
    // return Release the Ants to its owner's hand."
    // P0 reveals Colossal Dreadmaw (mana value 6), P1 a card with mana value 0: P0 wins.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
    let c = in_hand_with_mana(&mut t, P0, "Release the Ants");
    let spell = t.cast(P0, c).target(P1).go();
    let from = t.asked().len();
    t.resolve();
    assert!(clashed_since(&t, from));
    assert_eq!(t.life(P1), 19);
    assert!(t.in_hand(P0, "Release the Ants"));
    assert!(!went_to_graveyard(&t, spell));
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.stack_len(), 0);

    // P1 reveals the card with the greater mana value: the spell goes to the graveyard.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P1, &["Colossal Dreadmaw"]);
    let c = in_hand_with_mana(&mut t, P0, "Release the Ants");
    let spell = t.cast(P0, c).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert!(!t.in_hand(P0, "Release the Ants"));
    assert!(t.in_graveyard(P0, "Release the Ants"));
    assert!(went_to_graveyard(&t, spell));

    // A tie isn't a win either.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
    stack_library(&mut t, P1, &["Colossal Dreadmaw"]);
    let c = in_hand_with_mana(&mut t, P0, "Release the Ants");
    t.cast(P0, c).target(P1).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Release the Ants"));
}

#[test]
fn a_clash_spell_that_doesnt_resolve_doesnt_clash_or_return() {
    cr!("608.2b", "701.6a", "701.30a");
    ruling!(
        "Release the Ants",
        "If the spell is countered or doesn't resolve for any reason (for example, if all its targets become illegal), none of its effects happen. There is no clash, and the spell card won't be returned to your hand."
    );
    // Its only target became illegal.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = in_hand_with_mana(&mut t, P0, "Release the Ants");
    t.cast(P0, c).target(bears).go();
    let bears_now = t.g.current(bears);
    t.g.destroy(bears_now, None);
    let from = t.asked().len();
    t.resolve_all();
    assert!(!clashed_since(&t, from));
    assert!(t.in_graveyard(P0, "Release the Ants"));
    assert!(!t.in_hand(P0, "Release the Ants"));
    // The revealed card would have been the Dreadmaw: it's still on top.
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(t.obj(top).chars.name, "Colossal Dreadmaw");
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Custom { name, .. } if name == "clash")));

    // It's countered.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Colossal Dreadmaw"]);
    let c = in_hand_with_mana(&mut t, P0, "Release the Ants");
    let spell = t.cast(P0, c).target(P1).go();
    let counter = in_hand_with_mana(&mut t, P1, "Counterspell");
    t.cast(P1, counter).target(spell).go();
    let from = t.asked().len();
    t.resolve_all();
    assert!(!clashed_since(&t, from));
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Release the Ants"));
    assert!(!t.in_hand(P0, "Release the Ants"));
}
