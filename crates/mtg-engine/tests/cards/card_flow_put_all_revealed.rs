//! "Reveal the top N cards of your library. Put all [kind] cards revealed this way into
//! your hand and the rest [on the bottom of your library in any order | into your
//! graveyard]." (CR 701.20a)

use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn goblin_ringleader_takes_every_goblin_and_bottoms_the_rest() {
    cr!("701.20a", "603.6a");
    compiles("Goblin Ringleader");
    let mut t = TestGame::new(2);
    // Top first: Goblin Guide, Forest, Mogg Fanatic, Hill Giant, then Island (not revealed).
    let island = t.library_top(P0, "Island");
    for name in ["Hill Giant", "Mogg Fanatic", "Forest", "Goblin Guide"] {
        t.library_top(P0, name);
    }
    let library = t.library_size(P0);
    let ringleader = t.hand(P0, "Goblin Ringleader");
    t.lands(P0, "Mountain", 4);
    t.cast(P0, ringleader).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Goblin Guide") && t.in_hand(P0, "Mogg Fanatic"));
    assert_eq!(t.library_size(P0), library - 2);
    // The Island is now on top; the Forest and Hill Giant are on the bottom.
    assert_eq!(t.g.library_top(P0), Some(island));
    let lib = &t.g.player(P0).library;
    let bottom: Vec<String> = lib[..2]
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    assert!(bottom.contains(&"Forest".to_string()) && bottom.contains(&"Hill Giant".to_string()));
}

#[test]
fn mulch_puts_the_lands_into_your_hand_and_the_rest_into_your_graveyard() {
    cr!("701.20a");
    compiles("Mulch");
    let mut t = TestGame::new(2);
    for name in ["Grizzly Bears", "Forest", "Hill Giant", "Swamp"] {
        t.library_top(P0, name);
    }
    let mulch = t.hand(P0, "Mulch");
    t.lands(P0, "Forest", 2);
    t.cast(P0, mulch).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Swamp") && t.in_hand(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Hill Giant") && t.in_graveyard(P0, "Grizzly Bears"));
}
