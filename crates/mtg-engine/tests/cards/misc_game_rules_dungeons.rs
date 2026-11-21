//! Completing dungeons (CR 309.7), compiled by `oracle/patterns/misc_game_rules_dungeons.rs`:
//! "as long as you've completed a dungeon", "activate only if you've completed a dungeon",
//! and "whenever you complete a dungeon".

use mtg_engine::ability::{Effect, KeywordAction, PlayerRef, Sel, Value};
use mtg_engine::object::Zone;
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

/// P0 ventures into the dungeon once (choosing option `choice` if asked).
fn venture(t: &mut TestGame, choice: Option<usize>) {
    if let Some(i) = choice {
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::KeywordAction {
            action: KeywordAction::Venture,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.resolve_all();
}

/// P0 owns Lost Mine of Phandelver and completes it: Cave Entrance (scry 1) → Mine
/// Tunnels (a Treasure) → Dark Pool (drain 1) → Temple of Dumathoin (draw a card). None
/// of its rooms touch creatures.
fn complete_dungeon(t: &mut TestGame) {
    t.custom(
        P0,
        (*card("Lost Mine of Phandelver")).clone(),
        Zone::Outside(P0),
    );
    venture(t, None);
    venture(t, Some(1));
    venture(t, Some(0));
    venture(t, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.player(P0).dungeons_completed, 1);
}

#[test]
fn as_long_as_youve_completed_a_dungeon() {
    cr!("309.7", "611.3a");
    compiles("Cloister Gargoyle");
    let mut t = TestGame::new(2);
    let gargoyle = t.battlefield(P0, "Cloister Gargoyle");
    t.settle();
    assert_eq!(t.pt(gargoyle), (0, 4));
    assert!(!t.obj_now(gargoyle).has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    complete_dungeon(&mut t);
    t.settle();
    assert_eq!(t.pt(gargoyle), (3, 4));
    assert!(t.obj_now(gargoyle).has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn the_permanent_neednt_have_been_there_when_the_dungeon_was_completed() {
    cr!("309.7", "611.3a");
    ruling!(
        "Cloister Gargoyle",
        "last ability works even if it wasn't on the battlefield when you completed a dungeon"
    );
    ruling!(
        "Nadaar, Selfless Paladin",
        "Nadaar doesn't need to have been on the battlefield when you completed a dungeon"
    );
    let mut t = TestGame::new(2);
    complete_dungeon(&mut t);
    let gargoyle = t.battlefield(P0, "Cloister Gargoyle");
    t.battlefield(P0, "Nadaar, Selfless Paladin");
    t.settle();
    // +3/+0 from its own ability, +1/+1 from Nadaar.
    assert_eq!(t.pt(gargoyle), (4, 5));
}

#[test]
fn an_opponents_completed_dungeon_doesnt_count() {
    cr!("309.7");
    compiles("Nadaar, Selfless Paladin");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Nadaar, Selfless Paladin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Nadaar, Selfless Paladin");
    let my_bears = t.battlefield(P0, "Grizzly Bears");
    complete_dungeon(&mut t);
    t.settle();
    // Only P0 has completed a dungeon: their other creatures get +1/+1.
    assert_eq!(t.pt(my_bears), (3, 3));
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn whenever_you_complete_a_dungeon() {
    cr!("309.7", "603.2");
    compiles("Varis, Silverymoon Ranger");
    ruling!(
        "Varis, Silverymoon Ranger",
        "Once you resolve the last room ability of a dungeon, that dungeon is now completed"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Varis, Silverymoon Ranger");
    complete_dungeon(&mut t);
    assert_eq!(t.named_on_battlefield("Wolf Token").len(), 1);
}

#[test]
fn an_aura_gets_a_bigger_penalty_once_youve_completed_a_dungeon() {
    cr!("309.7", "613.4c");
    compiles("Precipitous Drop");
    ruling!(
        "Precipitous Drop",
        "Precipitous Drop cares whether its controller has completed a dungeon, not whether the creature's controller"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    let drop = t.battlefield(P0, "Precipitous Drop");
    assert!(t.g.attach(drop, Entity::Object(giant)));
    t.settle();
    // Craw Wurm is 6/4.
    assert_eq!(t.pt(giant), (4, 2));
    complete_dungeon(&mut t);
    t.settle();
    // -5/-5 instead: a 1/-1 creature is put into its owner's graveyard.
    assert!(!t.on_battlefield(giant));
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}
