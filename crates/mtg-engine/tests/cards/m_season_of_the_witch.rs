//! Season of the Witch (hand-written, `src/cards/season_of_the_witch.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn destroys_creatures_that_could_have_attacked_but_didnt() {
    cr!("508.1a", "603.2");
    ruling!("Season of the Witch", "could have been declared as an attacker during that turn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Season of the Witch");
    let idle = t.battlefield(P0, "Grizzly Bears");
    let attacker = t.battlefield(P0, "Hill Giant");
    let sick = t.battlefield_sick(P0, "Craw Wurm");
    let wall = t.battlefield(P0, "Wall of Stone");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(attacker, Entity::Player(P1))], &[]);
    // The attacker untaps (it didn't attack? it did: it isn't destroyed either way).
    t.g.objects[attacker.0 as usize].tapped = false;
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(idle), "could attack and didn't");
    assert!(t.on_battlefield(attacker), "attacked");
    assert!(t.on_battlefield(sick), "summoning sick: couldn't attack");
    assert!(t.on_battlefield(wall), "defender: couldn't attack");
    assert!(t.on_battlefield(theirs), "not its controller's turn: couldn't attack");
}

#[test]
fn tapped_creatures_survive() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Season of the Witch");
    let idle = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PostcombatMain);
    t.g.objects[idle.0 as usize].tapped = true;
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(idle));
}

#[test]
fn creatures_that_arrived_after_attackers_were_declared_survive() {
    cr!("508.1a");
    ruling!("Season of the Witch", "A creature won’t be destroyed if it was unable to attack that turn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Season of the Witch");
    let idle = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PostcombatMain);
    let late = t.battlefield(P0, "Hill Giant");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(idle));
    assert!(t.on_battlefield(late));
}
