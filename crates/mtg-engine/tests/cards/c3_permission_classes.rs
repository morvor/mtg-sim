//! Qualities of the spells a permission allows (cards3-cast-zones): "a noncreature or
//! Robot spell" (Vision, Spectral Synthezoid), "a Vehicle or artifact creature spell"
//! (Sanwell, Avenger Ace), "spells with exactly three colors" (Meeting of the Five), and a
//! keyword given to such spells: "Each spell you cast that's exactly three colors has
//! replicate {3}." (Threefold Signal).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
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

fn try_cast_as(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    let c = t.g.current(card);
    t.g.turn.priority = Some(p);
    let ok = t.g.cast_spell(p, c, method).is_ok();
    t.g.flush_events();
    if ok {
        t.resolve_all();
    } else {
        t.clear_answers();
    }
    ok
}

#[test]
fn vision_casts_one_noncreature_or_robot_spell_free_each_of_your_turns() {
    cr!("601.3", "118.9");
    assert_supported("Vision, Spectral Synthezoid");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Vision, Spectral Synthezoid");
    let bears = t.hand(P0, "Grizzly Bears");
    let div = t.hand(P0, "Divination");
    let div2 = t.hand(P0, "Divination");
    let robot = t.hand(P0, "Ornithopter");
    // A creature that isn't a Robot: not for free.
    assert!(!try_cast_as(&mut t, P0, bears, CastMethod::Free));
    // A noncreature spell: once this turn.
    assert!(try_cast_as(&mut t, P0, div, CastMethod::Free));
    assert!(t.in_graveyard(P0, "Divination"));
    assert!(!try_cast_as(&mut t, P0, div2, CastMethod::Free));
    // Not during an opponent's turn.
    let opt = t.hand(P0, "Opt");
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!try_cast_as(&mut t, P0, opt, CastMethod::Free));
    t.advance_to(P0, Step::PrecombatMain);
    // A Robot creature spell (Ornithopter is a Thopter, not a Robot: no).
    assert!(!try_cast_as(&mut t, P0, robot, CastMethod::Free));
    assert!(try_cast_as(&mut t, P0, div2, CastMethod::Free));
}

#[test]
fn vision_casts_a_robot_creature_spell_free() {
    cr!("601.3", "118.9");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Vision, Spectral Synthezoid");
    let robot = t.hand(P0, "Vision, Spectral Synthezoid");
    // The second Vision (a legendary Robot creature card) is cast for free.
    assert!(try_cast_as(&mut t, P0, robot, CastMethod::Free));
}

#[test]
fn sanwell_casts_a_vehicle_or_an_artifact_creature_spell_from_among_them() {
    cr!("608.2g", "601.3");
    ruling!(
        "Sanwell, Avenger Ace",
        "You must pay all costs for a spell cast using Sanwell's last ability."
    );
    ruling!(
        "Sanwell, Avenger Ace",
        "If you choose not to cast a spell this way, it goes to the bottom of the library with the other exiled cards."
    );
    assert_supported("Sanwell, Avenger Ace");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let sanwell = t.battlefield(P0, "Sanwell, Avenger Ace");
    t.lands(P0, "Plains", 2);
    t.library_top(P0, "Mountain");
    let bears = t.library_top(P0, "Grizzly Bears");
    let copter = t.library_top(P0, "Smuggler's Copter");
    let thopter = t.library_top(P0, "Ornithopter");
    t.library_top(P0, "Divination");
    t.library_top(P0, "Darksteel Relic");
    t.answer_choose(P0, &[Entity::Object(copter)]);
    t.g.tap(sanwell);
    t.resolve_all();
    // The choice offered: the Vehicle and the artifact creature, not the Bears (a
    // creature that isn't an artifact) or the Relic (an artifact that isn't a creature).
    let offered: Vec<Entity> = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("a choice among the exiled cards");
    let mut names: Vec<String> = offered
        .iter()
        .map(|e| match e {
            Entity::Object(o) => t.g.obj(*o).chars.name.to_string(),
            _ => String::new(),
        })
        .collect();
    names.sort();
    assert_eq!(names, vec!["Ornithopter", "Smuggler's Copter"]);
    assert_eq!(t.named_on_battlefield("Smuggler's Copter").len(), 1);
    // The rest went to the bottom of the library.
    assert_eq!(t.zone(t.g.current(bears)), Zone::Library(P0));
    assert_eq!(t.zone(t.g.current(thopter)), Zone::Library(P0));
}

#[test]
fn meeting_of_the_five_casts_three_color_spells_from_among_them() {
    cr!("601.3", "106.6", "105.2");
    ruling!(
        "Meeting of the Five",
        "You may cast any spells that have exactly three colors from among the cards exiled this way"
    );
    assert_supported("Meeting of the Five");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    for land in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, land, 1);
    }
    t.lands(P0, "Wastes", 3);
    // Two more lands, for the Helix.
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    for _ in 0..8 {
        t.library_top(P0, "Island");
    }
    let helix = t.library_top(P0, "Lightning Helix");
    let charm = t.library_top(P0, "Esper Charm");
    let meeting = t.hand(P0, "Meeting of the Five");
    t.cast(P0, meeting).go();
    t.resolve_all();
    let charm_x = t.g.current(charm);
    let helix_x = t.g.current(helix);
    assert_eq!(t.zone(charm_x), Zone::Exile);
    // A two-color spell: no.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(!try_cast_as(&mut t, P0, helix_x, CastMethod::Normal));
    // The three-color one, with the mana Meeting of the Five added: draw two cards.
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    assert!(try_cast_as(&mut t, P0, charm_x, CastMethod::Normal));
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_exile("Lightning Helix"));
}

#[test]
fn threefold_signal_gives_three_color_spells_replicate() {
    cr!("702.56a", "105.2");
    assert_supported("Threefold Signal");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Threefold Signal");
    for land in ["Plains", "Island", "Swamp"] {
        t.lands(P0, land, 1);
    }
    t.lands(P0, "Wastes", 3);
    let charm = t.hand(P0, "Esper Charm");
    let hand = t.hand_size(P0);
    // Draw two cards, replicated once: four cards.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.cast(P0, charm).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 4, "{}", t.dump_log());
}

#[test]
fn threefold_signal_doesnt_give_two_color_spells_replicate() {
    cr!("702.56a", "105.2");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Threefold Signal");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    let helix = t.hand(P0, "Lightning Helix");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.cast(P0, helix).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::OptionalCost { .. })));
}
