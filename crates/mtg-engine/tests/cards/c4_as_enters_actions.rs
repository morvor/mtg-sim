//! As-enters actions that aren't choices (round-4 card flow, CR 614.1c, 614.12): an
//! emblem created as a permanent enters (and "cards exiled with ~" on it), an Equipment
//! entering attached to a creature chosen as it enters, and an as-enters exile that sends
//! the permanent to the graveyard instead if it can't be done.

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn tibalt_gives_an_emblem_as_he_enters_for_the_cards_he_exiles() {
    cr!("614.12", "114.1", "607.2a");
    ruling!(
        "Valki, God of Lies // Tibalt, Cosmic Impostor",
        "even after that Tibalt leaves the battlefield"
    );

    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    t.lands(P0, "Swamp", 1);
    let tibalt = t.hand(P0, "Valki, God of Lies // Tibalt, Cosmic Impostor");
    t.cast(P0, tibalt).method(CastMethod::Half(1)).go();
    t.resolve();
    // The emblem exists as soon as Tibalt is on the battlefield: no trigger was used.
    let emblems: Vec<_> = t
        .g
        .command
        .iter()
        .filter(|o| t.g.obj(**o).kind == ObjKind::Emblem && t.g.obj(**o).controller == P0)
        .collect();
    assert_eq!(emblems.len(), 1);
    assert_eq!(t.stack_len(), 0);
    let tib = t.named_on_battlefield("Tibalt, Cosmic Impostor")[0];
    // +2: exile the top card of each player's library.
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.library_top(P0, "Grizzly Bears");
    t.activate(P0, tib, 0, &[]).unwrap();
    t.resolve();
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    // Tibalt leaves; the emblem still lets P0 cast the exiled Bolt, with any mana.
    let tib = t.g.current(tib);
    t.g.move_object(tib, Zone::Graveyard(P0), events::MoveCause::Effect, None);
    t.lands(P0, "Forest", 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn grifters_blade_enters_attached_to_a_chosen_creature() {
    cr!("614.12", "301.5b");
    ruling!("Grifter's Blade", "must enter attached to a creature you control, if possible");
    assert_supported("Grifter's Blade");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Plains", 3);
    let blade = t.hand(P0, "Grifter's Blade");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, blade).go();
    t.resolve();
    let blade = t.named_on_battlefield("Grifter's Blade")[0];
    assert_eq!(t.obj_now(blade).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (3, 3));
    // Only creatures P0 controls were offered.
    let offered = t.asked().into_iter().find_map(|(_, d)| match d {
        decision::Decision::ChooseEntities { candidates, .. } => Some(candidates),
        _ => None,
    });
    assert_eq!(offered.map(|c| c.len()), Some(2));
}

#[test]
fn grifters_blade_without_a_creature_enters_unattached() {
    cr!("614.12", "301.5e");
    ruling!("Grifter's Blade", "it simply enters unattached");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Plains", 3);
    let blade = t.hand(P0, "Grifter's Blade");
    t.cast(P0, blade).go();
    t.resolve();
    let blade = t.named_on_battlefield("Grifter's Blade")[0];
    assert_eq!(t.obj_now(blade).attached_to, None);
}

#[test]
fn frankensteins_monster_exiles_x_creature_cards_and_gets_chosen_counters() {
    cr!("614.12", "614.1c", "122.1");
    ruling!("Frankenstein's Monster", "does not have to match the total contents");
    assert_supported("Frankenstein's Monster");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Craw Wurm");
    t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Swamp", 4);
    let m = t.hand(P0, "Frankenstein's Monster");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    // A +2/+0 counter for the first card, a +0/+2 counter for the second.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.cast(P0, m).go();
    t.resolve();
    let m = t.named_on_battlefield("Frankenstein's Monster")[0];
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert_eq!(t.counters(m, "+2/+0"), 1);
    assert_eq!(t.counters(m, "+0/+2"), 1);
    assert_eq!(t.pt(m), (2, 3));
}

#[test]
fn frankensteins_monster_goes_to_the_graveyard_if_it_cant_exile_enough() {
    cr!("614.12", "614.1c");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Swamp", 4);
    let m = t.hand(P0, "Frankenstein's Monster");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.cast(P0, m).go();
    t.resolve();
    assert!(t.named_on_battlefield("Frankenstein's Monster").is_empty());
    assert!(t.in_graveyard(P0, "Frankenstein's Monster"));
    // Nothing was exiled.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}
