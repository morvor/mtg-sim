//! CR 701.42: meld, and melded permanents (CR 712.4).

use mtg_engine::events::MoveCause;
use mtg_engine::merge;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Graf Rats: "At the beginning of combat on your turn, if you both own and control this
/// creature and a creature named Midnight Scavengers, exile them, then meld them into
/// Chittering Host."
fn to_combat(t: &mut TestGame) {
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
}

#[test]
fn a_meld_pair_melds_into_one_permanent() {
    cr!("701.42a", "712.4a");
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Graf Rats");
    let scavengers = t.battlefield(P0, "Midnight Scavengers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t);
    assert!(!t.g.is_live(rats) && !t.g.is_live(scavengers));
    let host = t.named_on_battlefield("Chittering Host");
    assert_eq!(host.len(), 1);
    let host = host[0];
    assert_eq!(t.g.battlefield.len(), 2);
    assert_eq!(t.pt(host), (5, 6));
    assert_eq!(t.obj(host).controller, P0);
    // One object represented by the two cards; neither is in exile.
    assert_eq!(merge::physical_components(&t.g, host).len(), 2);
    assert!(t.g.exile.is_empty());
    // It entered the battlefield: "When this creature enters, other creatures you control
    // get +1/+0 and gain menace until end of turn."
    assert_eq!(t.pt(bears), (3, 2));
    // When it leaves the battlefield, both cards go to the graveyard.
    t.g.move_object(host, Zone::Graveyard(P0), MoveCause::Effect, None);
    assert!(t.in_graveyard(P0, "Graf Rats"));
    assert!(t.in_graveyard(P0, "Midnight Scavengers"));
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn only_the_two_cards_of_a_meld_pair_owned_and_controlled_by_you_meld() {
    cr!("701.42b");
    // Without Midnight Scavengers, nothing happens.
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Graf Rats");
    t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t);
    assert!(t.g.is_live(rats));
    assert!(t.named_on_battlefield("Chittering Host").is_empty());
    // With Midnight Scavengers owned by another player, nothing happens either.
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Graf Rats");
    let theirs = t.battlefield(P1, "Midnight Scavengers");
    t.g.objects[theirs.0 as usize].base_controller = P0;
    t.g.recompute();
    to_combat(&mut t);
    assert!(t.g.is_live(rats) && t.g.is_live(theirs));
    assert!(t.named_on_battlefield("Chittering Host").is_empty());
    // A token copy of Midnight Scavengers isn't a card of the meld pair.
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Graf Rats");
    let scav = t.exile(P0, "Midnight Scavengers");
    let host = CardDb::global().get("Chittering Host").unwrap();
    let rats_x =
        t.g.move_object(rats, Zone::Exile, MoveCause::Effect, None)
            .unwrap();
    t.g.objects[scav.0 as usize].kind = ObjKind::Token;
    assert!(merge::meld(&mut t.g, rats_x, scav, host, P0).is_none());
}

#[test]
fn cards_that_cant_be_melded_stay_where_they_are() {
    cr!("701.42c");
    let mut t = TestGame::new(2);
    let rats = t.exile(P0, "Graf Rats");
    let bears = t.exile(P0, "Grizzly Bears");
    let host = CardDb::global().get("Chittering Host").unwrap();
    assert!(merge::meld(&mut t.g, rats, bears, host, P0).is_none());
    assert!(t.g.is_live(rats) && t.g.is_live(bears));
    assert_eq!(t.g.exile.len(), 2);
}
