//! A mana ability without {T} in its cost can be activated more than once while a cost is
//! paid (CR 601.2g, 605.3a): Ashnod's Altar ("Sacrifice a creature: Add {C}{C}") with two
//! creatures pays {4}; Diamond Kaleidoscope ("Sacrifice a Prism: Add one mana of any
//! color") with two Prisms makes two mana. Found by the Nov 20 every-card sweep (Inspired
//! Ultimatum, `docs/EVERY_CARD.md`). Open: the payment planner (`mana_abilities.rs`) lists
//! one `ManaSource` per ability and uses it at most once, so these casts aren't offered.

use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::*;

fn cast_offered(t: &mut TestGame, card: ObjectId) -> bool {
    t.g.turn.priority = Some(P0);
    t.g.legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

#[test]
#[ignore = "open: the payment planner uses a mana ability without {T} once per payment"]
fn ashnods_altar_sacrifices_two_creatures_for_one_cost() {
    cr!("601.2g", "605.3a");
    let mut t = TestGame::new(2);
    let tome = t.hand(P0, "Jayemdae Tome");
    t.battlefield(P0, "Ashnod's Altar");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    assert!(cast_offered(&mut t, tome));
    t.cast(P0, tome).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Jayemdae Tome").len(), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
#[ignore = "open: the payment planner uses a mana ability without {T} once per payment"]
fn diamond_kaleidoscope_sacrifices_two_prisms_for_one_cost() {
    cr!("601.2g", "605.3a");
    let mut t = TestGame::new(2);
    let ultimatum = t.hand(P0, "Inspired Ultimatum");
    let k = t.battlefield(P0, "Diamond Kaleidoscope");
    t.lands(P0, "Wastes", 6);
    for _ in 0..2 {
        let k = t.g.current(k);
        t.g.untap(k);
        t.activate(P0, k, 0, &[]).expect("make a Prism");
        t.resolve();
    }
    // {U}{U}{R}{R}{R}{W}{W} from five lands and two Prisms.
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 1);
    assert!(cast_offered(&mut t, ultimatum));
}
