//! "Return all/each [creature] card(s) exiled with ~ to the battlefield under ..." (CR
//! 607.2a): Cold Storage, Synod Sanctum, Endless Sands, Helvault.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn activate(t: &mut TestGame, src: ObjectId, i: usize, targets: &[Entity]) {
    t.activate(P0, src, i, targets).expect("activation");
    t.resolve_all();
}

#[test]
fn cold_storage_returns_only_creature_cards_it_exiled_under_your_control() {
    cr!("607.2a", "111.7");
    assert_supported("Cold Storage");
    let mut t = TestGame::new(2);
    let storage = t.battlefield(P0, "Cold Storage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 6);
    activate(&mut t, storage, 0, &[Entity::Object(bears)]);
    activate(&mut t, storage, 0, &[Entity::Object(giant)]);
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    // A card exiled some other way stays in exile.
    t.exile(P0, "Llanowar Elves");
    activate(&mut t, storage, 1, &[]);
    assert!(!t.on_battlefield(storage));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(t.in_exile("Llanowar Elves"));
}

#[test]
fn endless_sands_returns_creatures_under_their_owners_control() {
    cr!("607.2a");
    assert_supported("Endless Sands");
    let mut t = TestGame::new(2);
    let sands = t.battlefield(P0, "Endless Sands");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    activate(&mut t, sands, 1, &[Entity::Object(bears)]);
    assert!(t.in_exile("Grizzly Bears"));
    // Untap it for the third ability.
    let sands = t.g.current(sands);
    t.g.untap(sands);
    t.lands(P0, "Wastes", 4);
    activate(&mut t, sands, 2, &[]);
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).controller, P0);
}

#[test]
fn synod_sanctum_returns_all_cards_it_exiled() {
    cr!("607.2a");
    assert_supported("Synod Sanctum");
    let mut t = TestGame::new(2);
    let sanctum = t.battlefield(P0, "Synod Sanctum");
    let ring = t.battlefield(P0, "Sol Ring");
    t.lands(P0, "Wastes", 4);
    activate(&mut t, sanctum, 0, &[Entity::Object(ring)]);
    assert!(t.in_exile("Sol Ring"));
    activate(&mut t, sanctum, 1, &[]);
    assert_eq!(t.named_on_battlefield("Sol Ring").len(), 1);
}

#[test]
fn helvault_returns_what_it_exiled_when_it_goes_to_the_graveyard() {
    cr!("607.2a", "603.6c");
    assert_supported("Helvault");
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Helvault");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Wastes", 7);
    activate(&mut t, vault, 1, &[Entity::Object(theirs)]);
    assert!(t.in_exile("Hill Giant"));
    t.g.destroy(t.g.current(vault), None);
    t.resolve_all();
    let back = t.named_on_battlefield("Hill Giant");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).controller, P1);
}
