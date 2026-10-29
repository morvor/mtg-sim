//! Revealing cards from the top of your library until you reveal a card with a quality
//! (CR 701.20), then putting "that card" somewhere and the other revealed cards somewhere
//! else.

use mtg_engine::ability::AbilityKind;
use mtg_engine::card::card;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Stacks named cards on top of P0's library; the last one named ends up on top.
fn stack(t: &mut TestGame, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.library_top(P0, n)).collect()
}

/// Activates the first activated ability of `source`.
fn activate(t: &mut TestGame, p: PlayerId, source: ObjectId) {
    t.g.recompute();
    let uid = t
        .g
        .obj(source)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.uid)
        .expect("no activated ability");
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, source, uid).expect("activation");
    t.g.flush_events();
}

#[test]
fn clifftop_lookout_puts_the_land_onto_the_battlefield_tapped_and_the_rest_on_the_bottom() {
    cr!("701.20a");
    assert_supported("Clifftop Lookout");
    let mut t = TestGame::new(2);
    let deep = t.library_top(P0, "Hill Giant");
    let [forest, bolt, bears] = stack(&mut t, &["Forest", "Lightning Bolt", "Grizzly Bears"])[..]
    else {
        unreachable!()
    };
    t.enter(P0, "Clifftop Lookout");
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert!(t.obj_now(forest).tapped);
    // The two cards revealed before it are on the bottom; the Hill Giant, never revealed,
    // is on top.
    let lib = &t.g.player(P0).library;
    assert_eq!(*lib.last().unwrap(), deep);
    let mut bottom2 = lib[..2].to_vec();
    bottom2.sort();
    let mut expected = vec![bolt, bears];
    expected.sort();
    assert_eq!(bottom2, expected);
}

#[test]
fn hermit_druid_puts_the_basic_land_into_hand_and_the_others_into_the_graveyard() {
    cr!("701.20a");
    assert_supported("Hermit Druid");
    let mut t = TestGame::new(2);
    let [island, _, _] = stack(&mut t, &["Island", "Grizzly Bears", "Lightning Bolt"])[..] else {
        unreachable!()
    };
    let druid = t.battlefield(P0, "Hermit Druid");
    t.lands(P0, "Forest", 1);
    activate(&mut t, P0, druid);
    t.resolve_all();
    assert_eq!(t.zone(island), Zone::Hand(P0));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn with_no_card_found_all_revealed_cards_go_where_the_rest_go() {
    cr!("701.20a");
    // Hermit Druid with no basic land in the library: every card is revealed and put into
    // the graveyard.
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    stack(&mut t, &["Grizzly Bears", "Lightning Bolt"]);
    let druid = t.battlefield(P0, "Hermit Druid");
    t.lands(P0, "Forest", 1);
    activate(&mut t, P0, druid);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn sacred_guide_exiles_the_other_revealed_cards() {
    cr!("701.20a");
    assert_supported("Sacred Guide");
    let mut t = TestGame::new(2);
    let [angel, bears] = stack(&mut t, &["Serra Angel", "Grizzly Bears"])[..] else {
        unreachable!()
    };
    let guide = t.battlefield(P0, "Sacred Guide");
    t.lands(P0, "Plains", 2);
    activate(&mut t, P0, guide);
    t.resolve_all();
    assert_eq!(t.zone(angel), Zone::Hand(P0));
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn riptide_shapeshifter_finds_a_creature_of_the_chosen_type() {
    cr!("701.20a");
    assert_supported("Riptide Shapeshifter");
    let mut t = TestGame::new(2);
    let [giant, elves, bears] = stack(&mut t, &["Hill Giant", "Llanowar Elves", "Grizzly Bears"])[..]
    else {
        unreachable!()
    };
    let rs = t.battlefield(P0, "Riptide Shapeshifter");
    t.lands(P0, "Island", 4);
    let giant_type = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Giant")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(giant_type));
    activate(&mut t, P0, rs);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    // The Bears and the Elves (not Giants) were shuffled into the library.
    assert_eq!(t.zone(elves), Zone::Library(P0));
    assert_eq!(t.zone(bears), Zone::Library(P0));
}

#[test]
fn a_creature_put_onto_the_battlefield_tapped_and_attacking() {
    cr!("701.20a", "508.4");
    assert_supported("Raph & Mikey, Troublemakers");
    let mut t = TestGame::new(2);
    let [bears, bolt] = stack(&mut t, &["Grizzly Bears", "Lightning Bolt"])[..] else {
        unreachable!()
    };
    let rm = t.battlefield(P0, "Raph & Mikey, Troublemakers");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(rm, Entity::Player(P1))], &[]);
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).tapped);
    // Raph & Mikey 7 (trample) + the Bears 2.
    assert_eq!(t.life(P1), 11);
    assert_eq!(*t.g.player(P0).library.first().unwrap(), bolt);
}
