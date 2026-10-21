//! One object of each type (`oracle/patterns/choice_grammar_each_type.rs`): Sundering
//! Titan, Planar Overlay, Liliana, Dreadhorde General, Tragic Arrogance.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn sundering_titan_destroys_a_land_of_each_basic_land_type() {
    cr!("205.3i");
    ruling!(
        "Sundering Titan",
        "If a land has more than one basic land type, it can be chosen more than once"
    );
    ruling!(
        "Sundering Titan",
        "If one of the basic land types isn't present, it isn't chosen"
    );
    compiles("Sundering Titan");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let plains = t.battlefield(P0, "Plains");
    let island = t.battlefield(P1, "Island");
    let tundra = t.battlefield(P1, "Tundra");
    let mountain = t.battlefield(P1, "Mountain");
    t.answer_choose(P0, &[Entity::Object(tundra)]); // Plains
    t.answer_choose(P0, &[Entity::Object(tundra)]); // Island
    t.answer_choose(P0, &[Entity::Object(mountain)]); // Mountain
    t.enter(P0, "Sundering Titan");
    t.resolve_all();
    assert!(!t.on_battlefield(tundra) && !t.on_battlefield(mountain));
    assert!(t.on_battlefield(plains) && t.on_battlefield(island));
}

#[test]
fn planar_overlay_each_player_returns_a_land_of_each_basic_type() {
    cr!("101.4");
    ruling!(
        "Planar Overlay",
        "a dual land could be chosen as two of your land types"
    );
    compiles("Planar Overlay");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let tundra = t.battlefield(P0, "Tundra");
    let plains = t.battlefield(P0, "Plains");
    let forest = t.battlefield(P1, "Forest");
    let spell = t.hand(P0, "Planar Overlay");
    t.advance_to(P0, Step::PrecombatMain);
    // P0's Plains choice is the Tundra, and so is its Island choice.
    t.answer_choose(P0, &[Entity::Object(tundra)]);
    t.answer_choose(P0, &[Entity::Object(tundra)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Tundra") && t.in_hand(P1, "Forest"));
    assert!(t.on_battlefield(plains), "not chosen");
    let _ = forest;
}

#[test]
fn liliana_dreadhorde_general_each_opponent_keeps_one_of_each_permanent_type() {
    cr!("110.4", "101.4");
    ruling!(
        "Liliana, Dreadhorde General",
        "if a permanent has more than one permanent type, it can count for any of them"
    );
    compiles("Liliana, Dreadhorde General");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let lili = t.battlefield(P0, "Liliana, Dreadhorde General");
    t.g.objects[lili.0 as usize]
        .counters
        .insert("loyalty".into(), 9);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    let thopter = t.battlefield(P1, "Ornithopter"); // artifact creature
    let forest = t.battlefield(P1, "Forest");
    // The artifact: the Ornithopter; the creature: the Lions.
    t.answer_choose(P1, &[Entity::Object(thopter)]);
    t.answer_choose(P1, &[Entity::Object(lions)]);
    t.activate(P0, lili, 2, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(thopter) && t.on_battlefield(lions) && t.on_battlefield(forest));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn tragic_arrogance_you_choose_what_each_player_keeps() {
    cr!("101.4");
    ruling!(
        "Tragic Arrogance",
        "None of the chosen permanents are targets of Tragic Arrogance"
    );
    compiles("Tragic Arrogance");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    let forest = t.battlefield(P1, "Forest");
    let spell = t.hand(P0, "Tragic Arrogance");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.answer_choose(P0, &[Entity::Object(lions)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(mine) && t.on_battlefield(lions) && t.on_battlefield(forest));
    assert!(!t.on_battlefield(bears), "P0 chose the Lions for P1");
    let p1_choices = t
        .asked()
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }));
    assert!(!p1_choices, "you choose for each player");
}
