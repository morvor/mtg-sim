//! Values chosen as an effect resolves ("the color of your choice"), locked in for the
//! effect's duration (CR 608.2h).

use mtg_engine::testing::*;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn mother_of_runes_grants_protection_from_the_chosen_color() {
    cr!("702.16b", "608.2h", "611.2a");
    assert_supported("Mother of Runes");
    let mut t = TestGame::new(2);
    let mother = t.battlefield(P0, "Mother of Runes");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let merfolk = t.battlefield(P1, "Coral Merfolk");
    choose_color(&mut t, P0, Color::Red);
    t.activate(P0, mother, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert!(t.g.protected_from(t.g.current(bears), goblin));
    assert!(!t.g.protected_from(t.g.current(bears), merfolk));
    // A later choice for another activation doesn't change the first effect.
    let elves = t.battlefield(P0, "Llanowar Elves");
    let m = t.g.current(mother);
    t.g.objects[m.0 as usize].tapped = false;
    choose_color(&mut t, P0, Color::Blue);
    t.activate(P0, mother, 0, &[Entity::Object(elves)]).unwrap();
    t.resolve();
    assert!(t.g.protected_from(t.g.current(bears), goblin));
    assert!(!t.g.protected_from(t.g.current(bears), merfolk));
    assert!(t.g.protected_from(t.g.current(elves), merfolk));
    // Until end of turn.
    t.advance_to(P1, turn::Step::Upkeep);
    assert!(!t.g.protected_from(t.g.current(bears), goblin));
}

#[test]
fn gods_willing_protection_counters_a_red_removal_spell() {
    cr!("702.16b", "608.2b");
    assert_supported("Gods Willing");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, turn::Step::PrecombatMain);
    t.cast(P1, bolt).target(bears).go();
    // In response, protection from red: the Bolt's only target becomes illegal.
    t.lands(P0, "Plains", 1);
    let gw = t.hand(P0, "Gods Willing");
    choose_color(&mut t, P0, Color::Red);
    t.cast(P0, gw).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn becomes_the_color_of_your_choice() {
    cr!("105.3", "613.1e", "608.2h");
    assert_supported("Rainbow Crow");
    let mut t = TestGame::new(2);
    let crow = t.battlefield(P0, "Rainbow Crow");
    t.lands(P0, "Island", 1);
    choose_color(&mut t, P0, Color::Green);
    t.activate(P0, crow, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(crow).chars.colors, ColorSet::single(Color::Green));
    t.advance_to(P1, turn::Step::Upkeep);
    assert_eq!(t.obj_now(crow).chars.colors, ColorSet::single(Color::Blue));
}
